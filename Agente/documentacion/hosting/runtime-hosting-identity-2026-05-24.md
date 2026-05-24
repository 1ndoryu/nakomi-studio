# Runtime identity de hosting

> **Fecha:** 2026-05-24
> **Tarea relacionada:** 245A-6

## Resumen

Las suscripciones de hosting ya no dependen solo de `server_uuid` y `coolify_site_name` para saber qué despliegue controlar. `hosting_subscriptions` ahora persiste dos columnas explícitas:

- `runtime_kind`: runtime dueño de la suscripción (`coolify` hoy, `lightweight` después).
- `deployment_id`: identificador del despliegue dentro de ese runtime.

## Qué cambió

- Nueva migración: `20260524083000_hosting_runtime_identity`.
- Backfill local de `deployment_id <- server_uuid` para no perder vínculo con despliegues ya provisionados.
- `runtime_kind` se guarda al crear la suscripción y se reafirma al provisionar.
- Los handlers de control, refresh, rotación de credenciales, borrado y activación de dominio ya resuelven el runtime desde la suscripción persistida, no desde el provider global únicamente.
- `HostingSubscriptionResponse` expone `runtime_kind` y `deployment_id` desde la persistencia real; `server_uuid` queda como campo legacy de compatibilidad.
- El panel admin de despliegues y los loops legacy de observabilidad ya fuerzan Coolify por runtime explícito, aunque el provider global cambie a `lightweight`.
- El sampler y el storage enforcement ignoran suscripciones que ya no pertenezcan a Coolify, para no mezclar inventarios al convivir dos runtimes.

## Implicación operativa

Si en el futuro `HOSTING_RUNTIME_PROVIDER=lightweight`, los hostings legacy marcados como `coolify` siguen intentando operar contra Coolify en sus endpoints de control. Sin esta persistencia, el cambio global de provider habría roto start/stop/restart/delete sobre despliegues viejos.

## Pendiente siguiente

- El inventario y las métricas siguen teniendo acoplamientos a Coolify (`coolify_server_targets`, `coolify_site_name`, samplers legacy).
- El provider `lightweight` sigue declarado pero no implementado en `src/services/hosting_runtime.rs`.
- Falta introducir inventario propio para `lightweight`; en este bloque solo se blindó la convivencia con hostings legacy de Coolify.