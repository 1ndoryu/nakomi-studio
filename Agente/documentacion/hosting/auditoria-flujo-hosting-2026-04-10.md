# Auditoría del flujo de hosting

## Fecha
2026-04-10

## Resumen ejecutivo

El hosting ya cubre compra, Stripe Checkout, webhooks básicos y panel de gestión, pero todavía no cubre el tramo operativo crítico posterior al pago. Hoy el sistema vende la suscripción y la activa en base de datos, pero no provisiona automáticamente el sitio real en Coolify ni sincroniza suspensiones/cancelaciones con la infraestructura.

## Flujo implementado hoy

### Compra y activación

- Compra pública y self-service: `POST /api/hosting/subscribe`
- Crea `hosting_subscriptions` en estado `pending`
- Genera Stripe Checkout Session de suscripción mensual
- `checkout.session.completed` guarda `stripe_subscription_id` y cambia estado a `active`
- `invoice.paid` revalida/recupera estado `active`

### Gestión en panel

- Panel de hosting con listado, detalle, stats y facturación
- Upgrade/downgrade en BD
- Eventos de auditoría en `hosting_events`
- Tab admin de VPS vía proxy Contabo

## Huecos críticos encontrados

### 1. Provisioning real ausente

- El pago activa la suscripción en BD, pero no crea sitio real en Coolify
- `coolify_site_name` sigue siendo informativo y no se llena en el checkout
- Resultado actual: hosting “activo” puede existir sin sitio provisionado

### 2. Fallo de pago sin notificación ni acción real

- `invoice.payment_failed` cambia el estado a `suspended`
- No hay email, notificación interna ni flujo de recuperación para el cliente
- Tampoco se suspende el sitio real en Coolify, así que la BD y la infraestructura pueden divergir

### 3. Cancelación sin desprovisioning

- `customer.subscription.deleted` marca `cancelled`
- No existe stop/redeploy/remove en la infraestructura real

### 4. Datos de servidor incompletos

- La UI deriva acceso SSH desde una IP hardcodeada
- No hay contrato backend para devolver VPS/IP reales por suscripción

### 5. Dominios todavía fuera del producto

- Hoy solo se guarda `domain` como dato opcional en la suscripción
- No existe compra, renovación, gestión DNS ni asociación operativa automatizada
- El plan de dominios sigue bloqueado por proveedor/API y por integración pendiente con el hosting

## Estado del ciclo de cobro

### Implementado

- Compra inicial
- Renovación con `invoice.paid`
- Marcado de `payment_failed`
- Marcado de `customer.subscription.deleted`

### No implementado

- Escalada por impago prolongado
- Notificaciones al cliente en fallos o cancelación
- Suspensión/cancelación real en Coolify
- Reintentos operativos de provisioning
- Health checks del hosting suscrito

## Dominios: qué falta hoy

- Registrar dominios desde la plataforma
- Cobro anual y renovación automática
- Historial y eventos de dominio
- Gestión DNS visual y/o automática
- Vincular dominio comprado al provisioning del hosting
- Integrar proveedor real de registrar/DNS y definir pricing

## Recomendación

Cerrar la tarea de “revisar hosting” con esta auditoría y reemplazarla por tareas concretas:

- Provisioning automático en Coolify al completar checkout
- Sincronización real de suspensión/cancelación por cobro
- Notificaciones al cliente para payment failed/cancelación
- Contrato backend para servidor/IP reales en HostingDetalle
- Continuar la iniciativa de dominios con proveedor y modelo de datos

## Referencias principales

- `src/handlers/hosting.rs`
- `src/services/hosting_stripe.rs`
- `src/repositories/hosting.rs`
- `frontend/src/api/hosting.ts`
- `frontend/src/components/panel/SeccionHosting.tsx`
- `frontend/src/components/panel/HostingDetalle.tsx`
- `frontend/src/hooks/useHostingDetalle.ts`
- `Agente/planes/plan-dominios-2026-04-07.md`