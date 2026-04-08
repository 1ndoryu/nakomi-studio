# Plan: Servicio de Hosting con Coolify Manager RS

**Fecha:** 2026-04-04  
**ID tarea:** 044A-29  
**Estado:** Planificación

---

## Resumen

Convertir Coolify Manager RS en la infraestructura operativa que permite a Nakomi Studio ofrecer hosting como servicio a clientes. El plan cubre: configuración de la infraestructura actual, automatización de operaciones, integración con el sitio web de Nakomi, panel de administración de hosting, y flujo completo de onboarding de clientes.

---

## Estado actual de Coolify Manager RS

### Lo que YA funciona (no hay que reimplementar)

- 29 comandos CLI + 26 herramientas MCP
- Backups externos a Google Drive con retención configurable
- Restore validado con rollback transaccional
- Migración VPS a VPS con preflight + dry-run
- Failover sin VPS origen (desde Google Drive)
- Health checks (HTTP + BD + logs + Docker)
- Deploy protegido con git rollback
- Auditoría de seguridad VPS + WordPress
- Configuración declarativa en `settings.json`

### Lo que está BLOQUEADO o INCOMPLETO

- **VPS2 sin Coolify operativo** — necesita `apiToken`, `serverUuid`, `projectUuid`
- **Contabo DNS API** — `apiPassword` no validada, conmutación DNS manual
- **Google Drive OAuth** — requiere `auth-drive` manual por primera vez
- **`backup --list`** — no funciona en modo Drive-only
- **GUI Tauri** — en evaluación, no iniciada
- **Tests de integración** — solo 61 unit tests, falta harness con mocks

---

## Fases de implementación

### Fase 1 — Infraestructura base (lo más difícil)

**Objetivo:** Tener la infraestructura operativa para alojar sitios de clientes con resilencia.

#### 1.1 Configurar VPS2 como standby

```bash
coolify-manager install-coolify --target standby-vps2
```

**Pasos:**

1. Verificar que VPS2 está accesible por SSH
2. Ejecutar `install-coolify` para instalar Coolify en VPS2
3. Obtener `apiToken`, `serverUuid`, `projectUuid` de la nueva instancia Coolify
4. Actualizar `config/settings.json` con los datos de VPS2
5. Verificar con `audit --target standby-vps2`

**Validación:** `health` devuelve OK para VPS2.

#### 1.2 Activar backups automáticos

**Google Drive:**

1. Ejecutar `auth-drive` para autorizar OAuth (una sola vez)
2. Verificar que el `rootFolderId` apunta a la carpeta correcta
3. Crear backup manual de prueba: `backup --name nakomi-rust --tier manual --label test`
4. Verificar que el backup se sube a Google Drive

**Programar backups:**

```bash
coolify-manager schedule-backup --name nakomi-rust
```

- Diarios: retener 2 copias
- Semanales: retener 2 copias

#### 1.3 Validar Contabo DNS API

1. Autenticar con Contabo API: probar `clientId` + `apiPassword`
2. Si la hipótesis "contraseña de usuario = API Password" es correcta, actualizar `.env`
3. Probar `switch-dns --name test --dry-run` para verificar la conmutación
4. Si Contabo API sigue bloqueada: documentar proceso manual de DNS como fallback

#### 1.4 Configurar failover automático

**Pipeline de failover (cuando VPS1 cae):**

```
1. Health check detecta fallo
2. failover --name {sitio} --target standby-vps2
3. Google Drive → Snapshot → Restore en VPS2
4. switch-dns manual (o Contabo API si funciona)
5. Health check en VPS2 confirma operatividad
```

**Automatización:** Windows Task Scheduler (o cron en VPS) ejecuta health checks cada 5 minutos. Si falla 3 veces consecutivas → trigger failover.

---

### Fase 2 — Flujo de onboarding de clientes

**Objetivo:** Proceso definido para dar de alta un nuevo sitio de cliente.

#### 2.1 Workflow de alta

```
1. Cliente contrata plan de hosting en nakomi.studio
2. Admin ejecuta: coolify-manager new --name {slug} --domain https://{dominio}
3. Coolify Manager crea:
   - Stack en Coolify (WordPress + MariaDB + Apache)
   - Tema Glory instalado
   - Certificado SSL automático (Let's Encrypt vía Coolify)
   - Entrada en settings.json
4. Admin ejecuta: coolify-manager backup --name {slug} --tier manual --label initial
5. Admin configura DNS del dominio del cliente → IP de VPS1
6. Health check confirma operatividad
7. Notificación al cliente con credenciales
```

#### 2.2 Templates por tipo de servicio

Coolify Manager ya soporta templates en `config/templates/`. Crear templates para los servicios de Nakomi:

| Template              | Contenido                                         | Para plan         |
| --------------------- | ------------------------------------------------- | ----------------- |
| `wordpress-basico`    | WP + MariaDB + Apache, sin cache                  | Hosting Básico    |
| `wordpress-pro`       | WP + MariaDB + Apache + Redis cache + CDN headers | Hosting Pro       |
| `wordpress-ecommerce` | WP + MariaDB + WooCommerce + Redis + SMTP         | E-commerce        |
| `rust-app`            | Rust + PostgreSQL + Nginx proxy                   | Aplicación custom |
| `static-site`         | Nginx + build artifacts                           | Landing pages     |

#### 2.3 Configuración por cliente en settings.json

```json
{
    "sitios": [
        {
            "name": "cliente-ejemplo",
            "domain": "https://ejemplo.com",
            "template": "wordpress-pro",
            "target": "produccion-vps1",
            "plan": "pro",
            "backupPolicy": {"daily": 2, "weekly": 2},
            "cliente": {
                "nombre": "Empresa Ejemplo",
                "email": "admin@ejemplo.com",
                "contratoInicio": "2026-04-01"
            }
        }
    ]
}
```

---

### Fase 3 — Integración con el sitio web de Nakomi

**Objetivo:** Los clientes pueden contratar hosting desde nakomi.studio y el sistema responde automáticamente.

#### 3.1 Planes de hosting en el frontend

Crear planes en el frontend (misma estructura que los planes existentes de otros servicios):

| Plan            | Precio     | Incluye                                                              |
| --------------- | ---------- | -------------------------------------------------------------------- |
| **Básico**      | $15/mes    | 1 sitio WP, 5GB, SSL, backup semanal, soporte email                  |
| **Profesional** | $35/mes    | 1 sitio WP, 20GB, SSL, backup diario, Redis, soporte prioritario     |
| **E-commerce**  | $60/mes    | 1 sitio WP + Woo, 50GB, SSL, backup diario, Redis, SMTP, soporte 24h |
| **Custom**      | Cotización | Apps Rust/React, PostgreSQL, WebSocket, infra dedicada               |

#### 3.2 Backend: endpoint de provisioning

**Nuevo endpoint:** `POST /api/hosting/provision`

```rust
/* Endpoint que recibe contratación de hosting y dispara el provisioning.
 * En Fase 3 es manual (notificación a admin), en Fase 5 será automático. */
#[derive(Deserialize)]
struct ProvisionRequest {
    plan: String,          // "basico" | "pro" | "ecommerce" | "custom"
    domain: String,        // dominio del cliente
    client_name: String,
    client_email: String,
}
```

**Flujo inicial (semi-automático):**

1. Cliente selecciona plan y paga (Stripe)
2. Backend recibe webhook de Stripe → crea registro en BD
3. Notificación a admin (email/panel) con datos del pedido
4. Admin ejecuta manualmente `coolify-manager new ...`
5. Admin confirma en el panel → cliente recibe email con credenciales

**Flujo futuro (Fase 5 — automático):**

1. Cliente paga → webhook Stripe
2. Backend llama a Coolify Manager RS vía subprocess o HTTP
3. Provisioning automático sin intervención humana
4. Health check confirma → email automático al cliente

#### 3.3 Modelo de datos

```sql
CREATE TABLE hosting_subscriptions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    client_name VARCHAR(200) NOT NULL,
    client_email VARCHAR(254) NOT NULL,
    plan VARCHAR(20) NOT NULL,           -- basico | pro | ecommerce | custom
    domain VARCHAR(253),
    coolify_site_name VARCHAR(100),       -- slug en coolify manager
    status VARCHAR(20) NOT NULL DEFAULT 'pending',  -- pending | provisioning | active | suspended | cancelled
    stripe_subscription_id VARCHAR(100),
    monthly_price_cents INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE hosting_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    subscription_id UUID NOT NULL REFERENCES hosting_subscriptions(id),
    event_type VARCHAR(50) NOT NULL,  -- provisioned | backup | restore | health_fail | payment | suspension
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

---

### Fase 4 — Panel de administración de hosting

**Objetivo:** Dashboard donde el admin gestiona todos los sitios alojados.

#### 4.1 Ruta: `/panel/hosting`

**Vista principal:**

```
┌─────────────────────────────────────────────────────────────┐
│  Hosting Dashboard                                          │
├──────────┬──────────┬──────────┬───────────┬───────────────┤
│  Sitio    │  Plan    │  Estado  │  Último    │  Acciones     │
│           │          │          │  backup    │               │
├──────────┼──────────┼──────────┼───────────┼───────────────┤
│  kamples  │  Custom  │  ● OK   │  hace 2h   │  [Health]     │
│           │          │          │            │  [Logs] [...]  │
├──────────┼──────────┼──────────┼───────────┼───────────────┤
│  cliente1 │  Pro     │  ● OK   │  hace 1d   │  [Health]     │
│           │          │          │            │  [Logs] [...]  │
├──────────┼──────────┼──────────┼───────────┼───────────────┤
│  cliente2 │  Básico  │  ● Warn │  hace 3d   │  [Health]     │
│           │          │          │            │  [Backup] [...] │
└──────────┴──────────┴──────────┴───────────┴───────────────┘
```

**Acciones por sitio:**

- Health check en tiempo real
- Ver logs (últimas 50 líneas)
- Trigger backup manual
- Restaurar desde backup
- Reiniciar servicios
- Ver eventos/historial
- Suspender/reactivar

#### 4.2 Backend: endpoints de gestión

| Método | Ruta                               | Descripción              |
| ------ | ---------------------------------- | ------------------------ |
| GET    | `/api/hosting/sites`               | Listar sitios con estado |
| GET    | `/api/hosting/sites/:name/health`  | Health check de un sitio |
| GET    | `/api/hosting/sites/:name/logs`    | Logs del contenedor      |
| POST   | `/api/hosting/sites/:name/backup`  | Trigger backup           |
| POST   | `/api/hosting/sites/:name/restore` | Restaurar backup         |
| POST   | `/api/hosting/sites/:name/restart` | Reiniciar servicios      |
| GET    | `/api/hosting/sites/:name/events`  | Historial de eventos     |
| PATCH  | `/api/hosting/sites/:name/status`  | Suspender/reactivar      |

**Integración con Coolify Manager RS:**
Los endpoints del backend Rust de Nakomi invocan `coolify-manager` como subprocess:

```rust
use std::process::Command;

let output = Command::new("coolify-manager")
    .args(["health", "--name", site_name, "--json"])
    .output()?;
```

Alternativa para mayor robustez: integrar `coolify-manager-rs` como library crate (`use coolify_manager::commands::health`).

---

### Fase 5 — Automatización completa

**Objetivo:** El sistema opera sin intervención humana para operaciones rutinarias.

#### 5.1 Provisioning automático

```
Stripe webhook → Backend Nakomi → Ejecuta coolify-manager new → Health check → Email a cliente
```

Requiere:

- Coolify Manager RS compilado y accesible en el servidor
- SSH keys configuradas automáticamente
- DNS wildcarding (\*.nakomi-hosting.com) o Contabo API operativa

#### 5.2 Health monitoring continuo

**Servicio en background (`src/services/hosting_monitor.rs`):**

```
Loop cada 5 minutos:
  Para cada sitio activo:
    1. Ejecutar health check
    2. Si falla → registrar evento
    3. Si falla 3 veces consecutivas:
       a. Intentar restart automático
       b. Si sigue fallando → notificar admin
       c. Si admin no responde en 30 min → trigger failover
```

#### 5.3 Renovación y suspensión automática

```
Stripe webhook (payment_succeeded) → Extender suscripción
Stripe webhook (payment_failed) → Marcar warning, notificar cliente
Stripe webhook (subscription_deleted) → Suspender sitio (no borrar datos)
```

Suspensión = el sitio se apaga pero los datos se preservan por 30 días. Después se archiva el backup y se elimina de la VPS.

---

## Orden de ejecución recomendado

| Prioridad | Fase                             | Dependencias               | Complejidad |
| --------- | -------------------------------- | -------------------------- | ----------- |
| 1         | Fase 1.1 — VPS2 standby          | Acceso SSH a VPS2          | Alta        |
| 2         | Fase 1.2 — Backups automáticos   | Google Drive OAuth         | Media       |
| 3         | Fase 2.1 — Workflow de alta      | Fase 1                     | Media       |
| 4         | Fase 3.1 — Planes en frontend    | Ninguna                    | Baja        |
| 5         | Fase 3.3 — Modelo de datos       | Ninguna                    | Baja        |
| 6         | Fase 3.2 — Endpoint provisioning | Fase 3.3 + Stripe          | Media       |
| 7         | Fase 4 — Panel admin             | Fase 3.3 + Coolify Manager | Alta        |
| 8         | Fase 1.3 — Contabo DNS           | Credenciales Contabo       | Media       |
| 9         | Fase 1.4 — Failover automático   | Fase 1.1 + 1.3             | Alta        |
| 10        | Fase 5 — Automatización          | Todo lo anterior           | Alta        |

---

## Dependencias externas

| Recurso                           | Estado                      | Acción requerida          |
| --------------------------------- | --------------------------- | ------------------------- |
| VPS2 SSH                          | Por verificar               | Confirmar acceso          |
| Google Drive OAuth                | Manual (una vez)            | Ejecutar `auth-drive`     |
| Contabo API password              | Bloqueado                   | Validar credenciales      |
| Stripe account                    | Existente (planes actuales) | Crear products de hosting |
| Coolify Manager RS binario en VPS | No desplegado               | Compilar y subir          |

---

## Consideraciones

- **Coolify Manager RS como library crate**: Si se va a invocar desde el backend de Nakomi directamente, es más eficiente importar como dependencia que usar subprocess. Evaluar si `lib.rs` expone la API necesaria.
- **Observabilidad**: Cada operación debe generar un evento en `hosting_events`. Esto es el log de auditoría para clientes y para el equipo.
- **Seguridad**: Los endpoints de `/api/hosting/*` requieren JWT con rol `admin`. Nunca exponer Coolify API tokens al frontend.
- **Escalabilidad**: Un VPS con 4-8GB RAM soporta ~10-15 sitios WordPress simultáneos. Para escalar, agregar más VPS targets en settings.json y distribuir sitios.
- **Pricing**: Los precios sugeridos ($15/$35/$60) son orientativos. Ajustar según costos reales de VPS + margen.
