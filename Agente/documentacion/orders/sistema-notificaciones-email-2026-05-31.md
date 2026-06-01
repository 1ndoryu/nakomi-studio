# Sistema de Notificaciones Email — Análisis y Propuesta

**Fecha:** 2026-05-31
**Tipo:** Revisión arquitectónica + propuesta — sin modificaciones de código.
**Alcance:** Notificaciones al cliente, notificaciones al admin (andoryyu@gmail.com), trazabilidad de correos enviados.

---

## Índice

1. [Resumen Ejecutivo](#1-resumen-ejecutivo)
2. [Estado Actual del Sistema de Email](#2-estado-actual-del-sistema-de-email)
3. [Estado Actual del Sistema de Notificaciones In-App](#3-estado-actual-del-sistema-de-notificaciones-in-app)
4. [¿Funcionan los Descuentos?](#4-funcionan-los-descuentos)
5. [¿Recibe el Admin Notificación al Crearse un Pedido?](#5-recibe-el-admin-notificación-al-crearse-un-pedido)
6. [Gap Crítico: Sin Email al Admin en Nuevas Órdenes](#6-gap-crítico-sin-email-al-admin-en-nuevas-órdenes)
7. [Gap: Sin Trazabilidad de Correos Enviados](#7-gap-sin-trazabilidad-de-correos-enviados)
8. [Escenarios que Deberían Disparar Email al Admin](#8-escenarios-que-deberían-disparar-email-al-admin)
9. [Arquitectura Propuesta](#9-arquitectura-propuesta)
10. [Plan de Implementación por Fases](#10-plan-de-implementación-por-fases)
11. [Configuración SMTP Actual](#11-configuración-smtp-actual)

---

## 1. Resumen Ejecutivo

| Aspecto | Estado |
|---------|--------|
| SMTP configurado | ✅ Sí — Brevo (smtp-relay.brevo.com), puerto 587 |
| Email de confirmación al cliente al crear pedido | ✅ Sí |
| Email al admin cuando se crea un pedido | ❌ **NO** — solo notificación in-app (WebSocket) |
| Email al admin cuando se paga un pedido | ❌ **NO** |
| Email al admin por escalación de chat | ✅ Sí |
| Email al admin por VPS pendiente | ✅ Sí |
| Email al admin por factura de chat pagada | ✅ Sí |
| Trazabilidad de correos enviados (historial/log) | ❌ **NO** — no hay tabla ni log persistente |
| Los descuentos funcionan | ✅ Sí — Full 20%, HalfHalf 10%, Phased 0% |

**Problema principal:** Cuando un cliente crea un pedido de servicio, el admin **no recibe ningún email**. Solo una notificación in-app que desaparece si no tienes el panel abierto en ese momento. Esto significa que pedidos pueden pasar desapercibidos durante horas.

---

## 2. Estado Actual del Sistema de Email

### 2.1 Configuración SMTP

```rust
// src/services/email.rs
SMTP_HOST o GLORY_SMTP_HOST → smtp-relay.brevo.com
SMTP_PORT o GLORY_SMTP_PORT → 587
SMTP_USER o GLORY_SMTP_USER → andoryyu@gmail.com
SMTP_PASSWORD → Brevo API key
SMTP_FROM → fallback a SMTP_USER (andoryyu@gmail.com)
SMTP_FROM_NAME → "Nakomi Studio"
```

El sistema carga la configuración desde variables de entorno en `EmailConfig::from_env()`. Si faltan variables, `email_config` es `None` y los emails se omiten silenciosamente (non-fatal).

### 2.2 Emails implementados actualmente

| Tipo | ¿A quién? | ¿Cuándo? | ¿Implementado? |
|------|-----------|----------|----------------|
| `send_order_confirmation` | Cliente | Al crear pedido | ✅ |
| `send_escalation_emails` | Admins | Cuando IA detecta escalación | ✅ |
| `send_chat_invoice_paid_client` | Cliente | Factura de chat pagada | ✅ |
| `send_chat_invoice_paid_admin` | Admins | Factura de chat pagada | ✅ |
| `send_vps_pending_approval` | Admins | VPS pendiente de aprobación | ✅ |
| `send_vps_approved` | Cliente | VPS aprobado | ✅ |
| `send_vps_rejected` | Cliente | VPS rechazado | ✅ |
| `send_profile_email_changed_new_address` | Nuevo email | Cambio de email en perfil | ✅ |
| `send_profile_email_changed_old_address` | Email anterior | Cambio de email en perfil | ✅ |
| `send_profile_password_changed` | Usuario | Cambio de contraseña | ✅ |

### 2.3 Cómo se obtienen los emails de admin

```rust
// src/repositories/user.rs
pub async fn admin_emails(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query_scalar!(
        r#"SELECT email FROM users WHERE role = 'admin' AND status = 'active'"#
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
```

Esto significa que para que `andoryyu@gmail.com` reciba correos, debe existir un usuario en la BD con `role = 'admin'`, `status = 'active'`, y `email = 'andoryyu@gmail.com'`.

---

## 3. Estado Actual del Sistema de Notificaciones In-App

### 3.1 NotificationHub

El sistema tiene un sistema híbrido de notificaciones en `src/services/notification.rs`:

- **Persistencia:** Las notificaciones se guardan en BD (tabla `notifications`)
- **Tiempo real:** WebSocket por usuario (broadcast channel por `user_id`)
- **REST API:** Listar, contar no leídas, marcar como leídas

### 3.2 Notificaciones que recibe el admin vía in-app

| Evento | Constante | ¿Email también? |
|--------|-----------|----------------|
| Nueva orden | `NOTIF_NEW_ORDER` | ❌ Solo in-app |
| Pago recibido | `NOTIF_PAYMENT_RECEIVED` | ❌ Solo in-app |
| Orden completada | `NOTIF_ORDER_COMPLETED` | ❌ Solo in-app |
| Cancelación solicitada | `NOTIF_ORDER_CANCELLED` | ❌ Solo in-app |
| Problema reportado | `NOTIF_PROBLEM_REPORTED` | ❌ Solo in-app |
| Reembolso solicitado | `NOTIF_REFUND_REQUESTED` | ❌ Solo in-app |
| Retiro solicitado | `NOTIF_WITHDRAWAL_REQUESTED` | ❌ Solo in-app |
| Escalación de chat | `NOTIF_ESCALATION_NEEDED` | ✅ Email |
| VPS pendiente | `NOTIF_VPS_PENDING_APPROVAL` | ✅ Email |
| Chat invoice pagada | (notificación directa) | ✅ Email |

**Problema:** La mayoría de eventos críticos para el admin solo llegan como notificación in-app, que requiere tener el panel abierto en ese momento.

---

## 4. ¿Funcionan los Descuentos?

**Sí, funcionan correctamente.** Verificado en `src/services/order.rs`:

```rust
fn discount_for_mode(mode: PaymentMode) -> i32 {
    match mode {
        PaymentMode::Full => 20,     // 20% de descuento
        PaymentMode::HalfHalf => 10,  // 10% de descuento
        PaymentMode::Phased => 0,     // 0% de descuento
    }
}
```

El cálculo aplicado en `create_order`:

```rust
let base_price = plan.price_cents;
let discount = Self::discount_for_mode(payment_mode);
let final_price = base_price - (base_price * discount / 100);
```

**Flujo completo validado:**

1. Cliente selecciona servicio + plan + modo de pago
2. Backend calcula descuento → `discount_percent` y `final_price_cents` se guardan en la orden
3. Se crean las fases con precio prorrateado según `percentage_of_total`
4. En `Full`: primera fase en `PhaseStatus::Paid`; resto en `Locked`
5. En `HalfHalf`/`Phased`: primera fase en `PhaseStatus::PendingPayment`; resto en `Locked`
6. En `Phased`: cada fase se paga individualmente (requiere fases CMS definidas)

**Los descuentos están correctos y funcionales.**

---

## 5. ¿Recibe el Admin Notificación al Crearse un Pedido?

### 5.1 Flujo actual en `create_order` (src/handlers/orders.rs)

```rust
// 1. Notificación in-app a todos los admins (SÍ)
let admins = UserRepository::admin_ids(&state.pool).await.unwrap_or_default();
let _ = state.notification_hub.notify_many(&admins, &base).await;

// 2. Activity log (SÍ)
ActivityLogRepository::log(...).await;

// 3. Chat session + mensaje de bienvenida (SÍ)
chat_hub.get_or_create_order_session(...).await;

// 4. Email de confirmación al CLIENTE (SÍ)
if let Some(ref email_cfg) = state.email_config {
    EmailService::send_order_confirmation(&cfg, &email, &name, ...).await;
}

// 5. Email al ADMIN (NO — NO EXISTE)
```

**Conclusión:** El admin **no recibe ningún email** cuando se crea un pedido. Solo recibe una notificación in-app (WebSocket) que requiere tener el panel abierto.

### 5.2 Lo que NO existe pero debería

| Escenario | Notificación in-app | Email admin | Email cliente |
|-----------|:-------------------:|:-----------:|:-------------:|
| **Nuevo pedido creado** | ✅ | ❌ **FALTA** | ✅ |
| **Pago recibido** | ✅ | ❌ **FALTA** | ❌ **FALTA** |
| **Orden completada** | ✅ | ❌ **FALTA** | ❌ **FALTA** |
| **Problema reportado** | ✅ | ❌ **FALTA** | ❌ **FALTA** |
| **Reembolso solicitado** | ✅ | ❌ **FALTA** | ❌ **FALTA** |
| **Cancelación solicitada** | ✅ | ❌ **FALTA** | ❌ **FALTA** |

---

## 6. Gap Crítico: Sin Email al Admin en Nuevas Órdenes

### 6.1 Impacto

Si un cliente crea un pedido de servicio (ej: "Quiero una web corporativa"):

1. ✅ El cliente recibe email de confirmación
2. ✅ El admin ve notificación in-app **solo si tiene el panel abierto**
3. ❌ **El admin no recibe email** → puede pasar horas sin enterarse
4. ❌ No hay reenvío automático a `andoryyu@gmail.com`

### 6.2 Comparación con otros eventos

Los eventos que **sí** envían email al admin usan el patrón:

```rust
// Ejemplo: escalación de chat (funciona)
if let Some(ref email_cfg) = state.email_config {
    if let Ok(emails) = UserRepository::admin_emails(&state.pool).await {
        if !emails.is_empty() {
            EmailService::send_escalation_emails(email_cfg, &emails, ...).await;
        }
    }
}
```

Pero la creación de órdenes **no implementa este patrón** para notificar al admin por email.

---

## 7. Gap: Sin Trazabilidad de Correos Enviados

### 7.1 Situación actual

- No existe tabla `email_logs` ni `sent_emails` en la BD
- No hay registro persistente de qué correos se enviaron, a quién, cuándo, y si fallaron
- Los fallos de envío solo se loguean con `tracing::error!` (logs efímeros)
- No existe interfaz (ni admin ni API) para consultar el historial de correos

### 7.2 Lo que se pierde

- Auditoría: ¿se envió realmente el email de confirmación al cliente?
- Diagnóstico: ¿por qué falló un email? (solo está en logs rotativos)
- Cumplimiento: en disputas, no hay prueba de envío
- Visibilidad admin: no puedes ver "todos los correos enviados"

### 7.3 Opciones para resolverlo

| Opción | Descripción | Ventajas | Desventajas |
|--------|-------------|----------|-------------|
| **A. Tabla `email_logs` en BD** | Cada envío se registra en PostgreSQL | Control total, consultable desde app | Migración BD, algo de storage |
| **B. BCC a andoryyu@gmail.com** | Todo correo lleva BCC oculto al admin | Sin BD nueva, simple, ves todo | No hay metadata estructurada, solo bandeja |
| **C. Brevo Transactional Logs** | Usar logs nativos de Brevo (SMTP relay) | Sin implementación, ya existen | Acceso externo, no integrado en app |
| **D. Panel de logs en app** | Tabla + UI admin para consultar históricos | Solución completa | Más trabajo inicial |

---

## 8. Escenarios que Deberían Disparar Email al Admin

### 8.1 Prioridad crítica — Implementar ya

| Escenario | Destinatario | Disparador | Email actual |
|-----------|-------------|------------|:-----------:|
| Nueva orden creada | Admin (andoryyu@gmail.com) | `POST /api/orders` → handler `create_order` | ❌ No existe |
| Pago de orden recibido | Admin | Webhook `payment_intent.succeeded` | ❌ No existe |
| Problema reportado en orden | Admin | `POST /orders/:id/report-problem` | ❌ No existe |

### 8.2 Prioridad alta — Implementar pronto

| Escenario | Destinatario | Disparador | Email actual |
|-----------|-------------|------------|:-----------:|
| Reembolso solicitado | Admin | `POST /orders/:id/refund` | ❌ No existe |
| Cancelación solicitada | Admin | `POST /orders/:id/cancel-request` | ❌ No existe |
| Orden completada | Admin | Última fase aprobada | ❌ No existe |
| Orden sin asignar > 48h | Admin | Tarea cron `auto_assign_deadline` | ❌ No existe |

### 8.3 Prioridad media

| Escenario | Destinatario | Disparador |
|-----------|-------------|------------|
| Hosting contratado | Admin | `POST /hosting/subscribe` |
| Hosting cancelado | Admin | Webhook `subscription.deleted` |
| Nuevo usuario registrado | Admin | `POST /api/auth/register` |
| Retiro solicitado | Admin | `POST /api/wallet/withdraw` |

---

## 9. Arquitectura Propuesta

### 9.1 Patrón recomendado: NotificationService centralizado

En lugar de esparcir `UserRepository::admin_emails()` + `EmailService::send_*` en cada handler, crear un servicio centralizado que unifique notificaciones in-app + email:

```
┌─────────────┐     ┌──────────────────────┐     ┌──────────────────┐
│   Handler    │────▶│  NotificationService │────▶│ NotificationHub  │ (in-app WS)
│ (create_order)│    │  (centralizado)      │────▶│ EmailService     │ (SMTP)
└─────────────┘     └──────────────────────┘     └──────────────────┘
                            │
                            ▼
                     ┌──────────────┐
                     │  email_logs  │  (BD — persistencia)
                     └──────────────┘
```

### 9.2 Tabla `email_logs` propuesta

```sql
CREATE TABLE email_logs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    to_email    TEXT NOT NULL,
    subject     TEXT NOT NULL,
    template    TEXT NOT NULL,       -- ej: 'order_confirmation', 'new_order_admin'
    reference_type TEXT,             -- 'order', 'chat_session', 'vps', etc.
    reference_id   UUID,            -- ID de la entidad referenciada
    status      TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'sent', 'failed'
    error_msg   TEXT,                -- Mensaje de error si falló
    sent_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_email_logs_created ON email_logs(created_at DESC);
CREATE INDEX idx_email_logs_status ON email_logs(status);
CREATE INDEX idx_email_logs_reference ON email_logs(reference_type, reference_id);
```

### 9.3 BCC automático (solución complementaria)

Configurar `SMTP_FROM` y añadir un BCC fijo a `andoryyu@gmail.com` desde `EmailService::send()`:

```rust
// En EmailService::send(), añadir BCC oculto:
let email = Message::builder()
    .from(...)
    .to(to_email)
    .bcc("andoryyu@gmail.com".parse().unwrap())  // <-- AÑADIR
    .subject(subject)
    ...
```

Esto garantiza que **todo** correo enviado desde la plataforma llegue también a `andoryyu@gmail.com` sin necesidad de tocar cada handler. Es la solución más rápida para tener visibilidad inmediata.

---

## 10. Plan de Implementación por Fases

### Fase 0 — Inmediata (sin código, solo configuración)

1. **Verificar que `andoryyu@gmail.com` existe como admin activo en la BD:**
   ```sql
   SELECT id, email, role, status FROM users WHERE email = 'andoryyu@gmail.com';
   ```
   Si no existe, crearlo con `role = 'admin'` y `status = 'active'`.

2. **Verificar conectividad SMTP Brevo:**
   - Comprobar que el API key de Brevo sigue activa
   - Probar envío manual desde consola Rust o con curl

3. **(Opcional) Configurar reenvío en Brevo:**
   - En Brevo, configurar que todos los correos transaccionales se reenvíen a `andoryyu@gmail.com`
   - Esto ya capturaría los correos de confirmación que se envían a clientes

### Fase 1 — BCC global (solución rápida, < 1h)

Añadir `bcc("andoryyu@gmail.com")` en `EmailService::send()` para que **todo** correo enviado desde la plataforma tenga copia oculta al admin. Esto da visibilidad inmediata sin tocar cada handler.

### Fase 2 — Email al admin en nueva orden

Añadir en `src/handlers/orders.rs` (después de la notificación in-app existente):

```rust
// [NUEVO] Email a admins notificando nueva orden
if let Some(ref email_cfg) = state.email_config {
    if let Ok(emails) = UserRepository::admin_emails(&state.pool).await {
        if !emails.is_empty() {
            // Llamar nuevo método EmailService::send_new_order_admin(...)
            EmailService::send_new_order_admin(
                email_cfg, &emails, &order, &client_name
            ).await;
        }
    }
}
```

### Fase 3 — Tabla `email_logs` + historial

1. Crear migración para `email_logs`
2. Integrar logging en `EmailService::send()`
3. Exponer endpoint `GET /api/admin/email-logs` (solo admin)
4. Crear UI en panel admin para consultar/exportar

### Fase 4 — Email al admin en todos los eventos críticos

Completar los escenarios de las tablas 8.1 y 8.2: pago recibido, problema reportado, reembolso, cancelación, orden completada.

### Fase 5 — Dashboard de trazabilidad

- Panel admin con historial completo de correos
- Filtros por tipo, destinatario, fecha, estado
- Reintentar envíos fallidos desde el panel

---

## 11. Configuración SMTP Actual

### `.env` (producción)

```
# SMTP Brevo
GLORY_SMTP_HOST=smtp-relay.brevo.com
GLORY_SMTP_PORT=587
GLORY_SMTP_USER=andoryyu@gmail.com
GLORY_SMTP_PASSWORD=<redacted-sendinblue-key>
```

**Nota:** `SMTP_FROM` no está definido, por lo que `EmailConfig::from_env()` usará `andoryyu@gmail.com` como remitente (fallback a `SMTP_USER`). Esto es válido pero podría ser mejor usar `noreply@nakomi.studio` si está configurado en Brevo.

### `.env.example`

```
# SMTP_HOST=smtp.gmail.com
# SMTP_PORT=587
# SMTP_USER=
# SMTP_PASSWORD=
# SMTP_FROM=noreply@nakomi.studio
```

---

## Anexo A: Referencias en el Código

| Archivo | Propósito |
|---------|-----------|
| `src/services/email.rs` | Servicio de email: configuración, envío SMTP, todas las plantillas |
| `src/services/notification.rs` | NotificationHub: notificaciones in-app + WebSocket |
| `src/handlers/orders.rs` | Handler `create_order`: notifica admins (in-app) y cliente (email) |
| `src/handlers/chat/rest_messages.rs` | Escalación: notifica admins (in-app + email) |
| `src/handlers/vps.rs` | VPS approve/reject: notifica cliente (email) |
| `src/services/vps_stripe.rs` | VPS pending: notifica admins (email) |
| `src/repositories/user.rs` | `admin_ids()` y `admin_emails()` queries |
| `src/handlers/payments.rs` | Webhook Stripe: notificaciones de pago |
| `.env` | Configuración SMTP Brevo activa |

## Anexo B: Preguntas Pendientes para el Usuario

1. **¿Tienes una cuenta admin en la BD con email `andoryyu@gmail.com`?** Verificar con `SELECT * FROM users WHERE email = 'andoryyu@gmail.com'`.
2. **¿Quieres recibir un email por cada nuevo pedido, o solo un resumen periódico?**
3. **¿Prefieres BCC global (ver todos los correos) o solo eventos críticos?**
4. **¿Quieres tener un panel para ver el historial de correos enviados, o te basta con el reenvío a tu bandeja?**
5. **¿Qué otros eventos además de "nuevo pedido" consideras críticos?** (pago, problema, reembolso, etc.)

---

*Documento generado el 2026-05-31. Basado en análisis estático del código fuente. Sin modificaciones de código.*
