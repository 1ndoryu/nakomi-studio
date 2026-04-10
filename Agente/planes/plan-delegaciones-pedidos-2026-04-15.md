# Plan: Delegaciones y Pedidos — Sistema Completo

> Creado: 2026-04-15
> Estado: **En ejecución**
> Roadmap task: 154A-15

## Objetivo

Implementar flujo completo tipo marketplace (Fiverr-like) para órdenes: wallet de saldo virtual, cancelación con solicitud, ventana exclusiva admin 48h, delegación manual, auditoría visible, IA intermediaria configurada correctamente.

## Estado actual del sistema

### Ya existe:
- `orders` table con `assigned_employee_id`, `auto_assign_deadline`, `status` enum completo
- `order_delegations` table con flujo delegate/help_request
- `auto_assign_loop` cada 60s en background (tokio::spawn)
- Endpoints REST: take, unassign, delegate, help, respond
- Frontend tabs: Asignados, Disponibles, Delegaciones
- `activity_log` table con índices
- `notifications` table + NotificationHub (push WS en tiempo real)
- Stripe integration para pagos por fase

### Falta:
1. **Wallet/balance** → no existe
2. **Cancelación con solicitud** → cancel es directo, sin aprobación del cliente
3. **Ventana exclusiva admin 48h** → deadline es 24h, auto-asigna a cualquier empleado
4. **Tab admin para pedidos sin delegar** → no hay
5. **Chat limitado a 2 personas** → sin restricción
6. **Auditoría visible al cliente** → activity_log existe pero sin endpoint/UI
7. **IA desactivada por defecto en órdenes** → default probablemente true
8. **Auto-desactivar IA cuando responsable responde** → no implementado
9. **Email confirmación al crear orden** → no implementado
10. **Notificación a empleados tras 48h** → auto_assign es silencioso

---

## Fases de implementación (ordenadas por dependencia)

### FASE 1 — Wallet y balance virtual (base para todo lo demás)
**Prioridad: ALTA — bloquea cancelaciones**

**1.1 Migración:**
```sql
CREATE TABLE user_wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id),
    balance_cents INT NOT NULL DEFAULT 0,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE wallet_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_id UUID NOT NULL REFERENCES user_wallets(id),
    user_id UUID NOT NULL REFERENCES users(id),
    amount_cents INT NOT NULL, -- positivo = ingreso, negativo = gasto
    transaction_type VARCHAR(30) NOT NULL,
    -- tipos: 'refund_credit', 'order_payout', 'withdrawal', 'order_payment', 'adjustment'
    reference_type VARCHAR(30), -- 'order', 'refund', 'withdrawal_request'
    reference_id UUID,
    description TEXT,
    balance_after_cents INT NOT NULL, -- snapshot del saldo post-transacción
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_wallet_user ON user_wallets(user_id);
CREATE INDEX idx_wallet_tx_user ON wallet_transactions(user_id, created_at DESC);
CREATE INDEX idx_wallet_tx_ref ON wallet_transactions(reference_type, reference_id);
```

**1.2 Backend:** modelo, repositorio, servicio, handler
- `GET /api/wallet` → saldo actual (crear wallet on-demand si no existe)
- `GET /api/wallet/transactions` → historial paginado
- `POST /api/wallet/withdraw` → solicitar retiro (futuro: Stripe Connect payout)
- Métodos internos: `credit(user_id, amount, type, ref)`, `debit(user_id, amount, type, ref)`
- Validación: balance no puede ser negativo (excepto adjustments admin)

**1.3 Frontend:**
- Badge en header del panel mostrando saldo: `$XX.XX USD`
- Página wallet con historial de transacciones en tabla
- Icono de wallet en la navegación del panel

### FASE 2 — Cancelación con solicitud y refund al wallet
**Prioridad: ALTA — core business logic**

**2.1 Nuevo flujo de cancelación:**
- Estado actual: `cancel_order` cancela directamente
- Nuevo flujo cuando empleado cancela:
  1. Empleado envía `POST /api/orders/{id}/cancel-request` con `{ reason: string }`
  2. Orden pasa a nuevo status: `cancellation_requested`
  3. Cliente recibe notificación + ve en historial
  4. Cliente acepta → orden se cancela, dinero va al wallet del cliente
  5. Cliente rechaza → orden vuelve a `awaiting_assignment` (sin employee), disponible para otros
  6. Admin siempre puede cancelar directamente (bypass solicitud)
  7. Cliente siempre puede cancelar sus propias órdenes directamente → dinero al wallet

**2.2 Migración:**
```sql
ALTER TYPE order_status ADD VALUE IF NOT EXISTS 'cancellation_requested';

CREATE TABLE cancellation_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(id),
    requested_by UUID NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    -- pending → accepted → rejected
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**2.3 Endpoints:**
- `POST /api/orders/{id}/cancel-request` → empleado solicita cancelación
- `POST /api/orders/{id}/cancel-request/{request_id}/respond` → cliente acepta/rechaza
- Modificar `cancel_order` existente: si admin → directo; si cliente → directo + wallet credit; si empleado → crear cancel-request

**2.4 Frontend:**
- Modal "Solicitar cancelación" en vista de orden (empleado)
- Banner "El responsable solicita cancelar" en la orden del cliente con botones Aceptar/Rechazar
- Si acepta: mensaje de éxito + saldo actualizado en wallet
- Si rechaza: orden vuelve a disponibles, empleado removido

### FASE 3 — Ventana exclusiva admin 48h + notificación
**Prioridad: ALTA — flujo de negocio principal**

**3.1 Cambios en auto_assign_loop:**
- Cambiar `auto_assign_deadline` de 24h a 48h al crear orden
- Durante las primeras 48h:
  - Orden SOLO visible para admin (no aparece en "Disponibles" para empleados)
  - Admin puede: tomar, asignar a empleado específico, o ignorar
- Después de 48h (auto_assign_deadline vencido):
  - NO auto-asignar automáticamente
  - En su lugar: marcar orden como "open_to_employees" (nuevo campo boolean)
  - Enviar notificación a TODOS los empleados disponibles: "Nueva orden disponible"
  - Orden ahora visible en tab "Disponibles" para empleados
  - Primer empleado que la tome se la queda

**3.2 Migración:**
```sql
ALTER TABLE orders ADD COLUMN IF NOT EXISTS open_to_employees BOOLEAN NOT NULL DEFAULT false;
```

**3.3 Cambios en código:**
- `auto_assign_loop`: en vez de asignar automáticamente, poner `open_to_employees = true` + notificar
- `list_unassigned`: filtrar por `open_to_employees = true` para empleados, sin filtro para admin
- Admin tab "Sin delegar": listar órdenes `awaiting_assignment AND NOT open_to_employees` (ventana 48h)
- `create_order`: cambiar deadline de 24h a 48h

**3.4 Frontend admin:**
- Nuevo tab o sección en "Todas las órdenes": "Pedidos nuevos (48h)" con contador
- Acciones: Tomar, Asignar a empleado (dropdown), Ignorar (esperar a que expire)

### FASE 4 — Confirmación de pedido al cliente
**Prioridad: MEDIA**

**4.1 Al crear orden:**
- Chat automático: "¡Felicidades! Tu pedido #{N} ha sido recibido. Será atendido dentro de las próximas 48 horas por nuestro equipo."
- Email al cliente con detalle del pedido (servicio, plan, precio, fechas estimadas)
- Notificación push

**4.2 Implementación:**
- En `create_order` handler, después de crear la orden:
  - Crear chat session si no existe
  - Enviar mensaje de sistema en el chat
  - Enviar email (necesita infraestructura de email — verificar si existe)
  - Enviar notification push

### FASE 5 — Auditoría visible al cliente
**Prioridad: MEDIA**

**5.1 Endpoint:**
- `GET /api/orders/{id}/activity` → historial de la orden desde `activity_log`
- Filtrar por `entity_type = 'order' AND entity_id = order_id`
- Retornar timeline ordenada cronológicamente

**5.2 Registrar eventos en activity_log:**
Verificar qué acciones ya registran y agregar las faltantes:
- `order_created` — cliente creó orden
- `employee_assigned` — admin/auto asignó empleado (with name)
- `payment_received` — fase pagada
- `phase_delivered` — empleado entregó fase
- `phase_approved` — cliente aprobó fase
- `revision_requested` — cliente pidió cambios
- `cancellation_requested` — empleado solicitó cancelar
- `cancellation_accepted` — cliente aceptó cancelación
- `cancellation_rejected` — cliente rechazó, orden reabierta
- `order_cancelled` — cancelación directa (admin/cliente)
- `employee_changed` — responsable cambió (from → to)
- `order_completed` — todas las fases aprobadas
- `delegation_created` — empleado delegó
- `delegation_resolved` — delegación aceptada/rechazada

**5.3 Frontend:**
- Componente `OrdenHistorial` — timeline visual dentro del detalle de orden
- Cada evento: icono, descripción legible, timestamp, actor

### FASE 6 — Chat: 2 personas + IA intermediaria correcta
**Prioridad: MEDIA**

**6.1 Chat limitado a 2 participantes:**
- En órdenes: solo cliente y empleado asignado pueden enviar mensajes
- Admin puede ver pero NO enviar (solo si es responsable)
- Si admin cambia responsable, nuevo responsable accede al historial

**6.2 IA intermediaria:**
- `ai_intermediary_enabled` default `false` en órdenes nuevas
- Cuando responsable envía primer mensaje → auto-desactivar IA
- Toggle manual sigue disponible para admin/employee

**6.3 Implementación:**
- Migración: `ALTER TABLE orders ALTER COLUMN ai_intermediary_enabled SET DEFAULT false;`
- En handler de chat `send_message`: si `sender_id == assigned_employee_id` → `UPDATE orders SET ai_intermediary_enabled = false`
- Validar que solo cliente/employee pueden enviar en chat de orden

### FASE 7 — Deploy y verificación
**Prioridad: FINAL**

- Deploy a producción
- Verificar flujo completo: crear orden → admin ve → admin asigna → trabajo → cancel request → wallet
- Health check
- Verificar que 154A-12 (badge) y hosting (Contabo) funcionan en producción

---

## Orden de ejecución

1. **FASE 1** (wallet) — Primero porque las cancelaciones dependen de poder creditear
2. **FASE 2** (cancelaciones) — Depende de wallet
3. **FASE 3** (ventana 48h) — Cambio en auto_assign_loop, independiente de 1-2
4. **FASE 4** (confirmación) — Independiente, puede ir en paralelo
5. **FASE 5** (auditoría) — Independiente, enriquece el historial
6. **FASE 6** (chat IA) — Fix rápido de defaults y auto-disable
7. **FASE 7** (deploy) — Al final de todo

## Sub-IDs de tareas

Cada fase será una sub-tarea:
- 154A-15a: Wallet system (Fase 1)
- 154A-15b: Cancelación con solicitud (Fase 2)
- 154A-15c: Ventana admin 48h (Fase 3)
- 154A-15d: Confirmación pedido + email (Fase 4)
- 154A-15e: Auditoría visible (Fase 5)
- 154A-15f: Chat restricciones + IA defaults (Fase 6)
- 154A-15g: Deploy + verificación (Fase 7)

## Notas

- El sistema de retiro de fondos (Stripe Connect payout) queda como mejora futura — requiere configuración de Stripe Connect que puede tardar días en aprobación
- La fusión de chats por pareja de usuarios (de 154A-14) queda como tarea futura
- Email sending: verificar si hay infraestructura existente o si hay que implementar (Resend, SES, SMTP)
