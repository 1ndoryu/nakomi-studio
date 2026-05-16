# Panel Admin de Infraestructura de Hosting

## Fecha
2026-04-16

## Problema

La tarea original se había dado por completada mostrando instancias de Contabo en el panel admin, pero eso no cumplía el requerimiento real: ver los despliegues existentes dentro de la VPS2.

Contabo responde la capa de proveedor (qué VPS existen). Coolify responde la capa operativa (qué servicios/deployments están realmente corriendo en la VPS2). Mezclar ambas vistas ocultaba si el panel reflejaba o no la infraestructura real.

## Cambios aplicados

- El backend ahora expone `GET /api/hosting/deployments` para admin.
- Ese endpoint consulta `GET /api/v1/services` en Coolify con la configuración activa de VPS2.
- El listado se enriquece con el cruce contra `hosting_subscriptions` para mostrar si cada despliegue está vinculado o no a una suscripción del panel.
- El panel de Hosting ahora separa dos vistas admin:
  - `Despliegues VPS2`: servicios reales de Coolify en la VPS2.
  - `Contabo VPS`: instancias del proveedor, útiles para revisar la infraestructura base.
- El build del frontend se estabilizó eliminando un `<style>` embebido en `frontend/index.html` que activaba un fallo `vite:html-inline-proxy` durante `vite build`.

## Estado operativo actual

- El tab `Despliegues VPS2` muestra la infraestructura real visible por Coolify.
- Cada card indica si el despliegue está vinculado a una suscripción del panel o si quedó huérfano.
- La vista previa de Contabo sigue disponible, pero ya no sustituye la visibilidad de deployments reales.

## Actualización 2026-05-16

- Los despliegues huérfanos ahora exponen una acción explícita de limpieza real desde el panel admin.
- El flujo nuevo llama `DELETE /api/hosting/deployments/:deployment_uuid` y elimina el stack en Coolify con `delete_volumes=true`.
- El backend rechaza el borrado si detecta una suscripción vinculada por `server_uuid` o por `coolify_site_name`; esto cubre vínculos legacy que todavía no tienen `server_uuid` persistido.
- El panel pide confirmación antes de borrar y refresca el listado de huérfanos al completar la limpieza.

## Validación

- `cargo check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `npx tsc --noEmit` en `frontend/`
- `npm --prefix frontend run build`
- Prueba funcional local:
  - login `admin@admin.com` / `admin`
  - `GET /api/hosting/deployments` respondió `200`
  - devolvió un despliegue real (`hosting-0d1de66f`) vinculado a una suscripción `active`

## Nota operativa

Al arrancar el backend local siguen apareciendo errores preexistentes de fixtures (`orders`, `order_delegations`, `order_phases`, `order_payments`, `order_refunds`). No bloquearon esta tarea ni el endpoint nuevo, pero siguen siendo deuda real del entorno local.

La validación de esta mejora fue no destructiva en local: compilación Rust, `self-check` y build completo del frontend. No se ejecutó un borrado real contra producción durante esta tarea para evitar eliminar infraestructura existente solo como smoke test.