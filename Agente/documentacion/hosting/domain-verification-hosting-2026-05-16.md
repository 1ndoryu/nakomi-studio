# Verificación de ownership para dominios de hosting

> **Fecha:** 2026-05-16
> **Tarea:** 165A-21

## Qué cambió

Los dominios custom del hosting ya no se activan en Coolify cuando el cliente solo guarda el valor en el panel. El flujo ahora es:

1. El usuario guarda el dominio en la suscripción.
2. El backend genera un token TXT único y deja el dominio en `pending_verification`.
3. El panel muestra el registro `_nakomi-verify.{dominio}` y permite lanzar la verificación.
4. `POST /api/hosting/subscriptions/{id}/verify-domain` consulta DNS TXT.
5. Si el TXT coincide, el backend marca el dominio `verified` y, si el hosting ya está provisionado, aplica el routing custom en Coolify y lo deja `active`.

## Estado de dominio

- `none`: no hay dominio custom configurado.
- `pending_verification`: el dominio fue guardado, pero todavía no existe ownership confirmado por TXT.
- `verified`: el TXT ya coincide, pero el routing custom aún no se aplicó al hosting.
- `active`: Coolify ya recibió el dominio custom y el panel puede tratarlo como URL principal.

## Decisiones clave

- La URL bootstrap temporal sigue siendo la fuente de verdad mientras el dominio no esté `active`.
- El endpoint de verificación devuelve un resultado explícito (`verified`, `applied`, `message`, `txt_records`) para que el panel diferencie entre TXT ausente, TXT incorrecto y dominio ya activado.
- Si el usuario cambia un dominio que ya estaba activo, el backend retira primero la ruta vieja para no dejar hosts huérfanos en Coolify.

## Validación aplicada

- `cargo check`
- `cargo check --all-targets --all-features`
- `cargo test domain_verification --all-features`
- `npm --prefix frontend run type-check`

## Gotchas

- `SQLX_OFFLINE=true` obliga a regenerar `.sqlx/` en el mismo bloque cuando se añaden columnas nuevas.
- Si la base local principal tiene checksums viejos de migraciones ya aplicadas, usar una base temporal limpia para `cargo sqlx migrate run` + `cargo sqlx prepare` evita tocar datos de desarrollo.