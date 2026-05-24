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
- El bridge backend ↔ `coolify-manager-rs` ya hace reales las operaciones lightweight para hosting `normal-*`: inventario (`inventory-light`), control/borrado (`light-site`) y provisioning estático (`provision-static`).
- `update_deployment()` ya reconfigura sitios lightweight vía `light-site --action reconfigure`, de modo que refresh, rotación de credenciales y activación de dominio custom ya no dependen artificialmente de `CoolifyConfig`.
- El mismo bridge ya cubre backups remotos de lightweight: `light-backup` lista/crea snapshots remotos y `light-restore` restaura el sitio, rehidrata Caddy/SSH y devuelve la password SFTP regenerada para resincronizar la suscripción.
- Las compras nuevas ya no graban `runtime_kind` a partir del provider global. El alta fija el runtime por plan: `normal-*` usa `lightweight` solo cuando el manager tiene target configurado; WordPress sigue en `coolify` hasta que exista receta premium real en el runtime nuevo.

## Implicación operativa

Si en el futuro `HOSTING_RUNTIME_PROVIDER=lightweight`, los hostings legacy marcados como `coolify` siguen intentando operar contra Coolify en sus endpoints de control. Sin esta persistencia, el cambio global de provider habría roto start/stop/restart/delete sobre despliegues viejos.

En `studio`, el contenedor `app` no alcanzaba la API de Coolify por la URL pública `http://173.249.50.44:8000` aunque el alias interno `http://coolify:8080` sí respondía. Antes de abrir compras reales, el runtime debe usar el alias interno y dejar `GLORY_TEST_CHECKOUT_EMAILS` vacío para no mezclar bypass de prueba con checkout comercial.

## Pendiente siguiente

- Falta un smoke operativo sobre un target lightweight real para validar restore end-to-end fuera del entorno local de build/test.
- La receta WordPress premium del runtime lightweight sigue pendiente; este corte solo cubre hostings `normal-*`.
- La observabilidad, métricas de recursos y enforcement comercial del runtime lightweight siguen anclados en parte al camino legacy de Coolify.