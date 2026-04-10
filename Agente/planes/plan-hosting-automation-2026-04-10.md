# Plan: Hosting Automation

> Creado: 2026-04-10
> Base: auditoría `Agente/documentacion/hosting/auditoria-flujo-hosting-2026-04-10.md`
> Estado: Activo

## Objetivo

Cerrar la brecha entre “hosting pagado en Stripe” y “hosting real operando en infraestructura”, incluyendo provisioning, ciclo de cobro, suspensión/cancelación real y datos de servidor consistentes en el panel.

## Fase 1 — Provisioning real post-checkout

1. Crear/adaptar un servicio backend que invoque provisioning en Coolify al cerrar `checkout.session.completed`
2. Guardar `coolify_site_name`, server metadata e identificadores operativos en la suscripción
3. Definir retries/idempotencia para evitar dobles despliegues

## Fase 2 — Sincronización operativa del ciclo de cobro

1. `invoice.payment_failed` debe notificar al cliente y dejar rastro visible en panel
2. `customer.subscription.deleted` debe sincronizar cancelación real con infraestructura
3. Definir política de escalada por impago prolongado

## Fase 3 — Datos reales en el panel

1. Devolver IP/VPS reales desde backend
2. Eliminar IP hardcodeada del detalle de hosting
3. Alinear estado BD vs estado real de la infraestructura

## Fase 4 — Dominios y DNS

1. Confirmar proveedor/API para registro y gestión DNS
2. Diseñar tablas y endpoints de dominio
3. Integrar asociación dominio-hosting al flujo de provisioning

## Riesgos

- Provisioning parcial tras pago exitoso
- Divergencia entre estado en Stripe, BD y Coolify
- Bloqueos por credenciales/API externas (Contabo, Coolify, registrar)

## Criterio de cierre

- Un hosting comprado queda provisionado de forma verificable o entra en un estado de error/reintento explícito
- Un fallo de pago o cancelación impacta estado real y comunica al cliente
- El panel muestra datos reales del servidor