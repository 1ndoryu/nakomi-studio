# Panel VPS de Contabo

## Fecha
2026-04-10

## Problema

El panel admin de Hosting consumía `/api/hosting/vps`, pero cualquier fallo de Contabo terminaba convertido en `500 Internal Server Error`. Eso ocultaba si el problema era autenticación (`invalid_grant`), formato inesperado de la respuesta o caída temporal del proveedor.

## Cambios aplicados

- `ContaboConfig::from_env()` ahora acepta nombres explícitos (`CONTABO_CLIENT_ID`, `CONTABO_CLIENT_SECRET`, `CONTABO_API_USER`, `CONTABO_API_PASSWORD`) y mantiene compatibilidad con los nombres legacy.
- El servicio `contabo.rs` conserva el detalle devuelto por Contabo en los errores upstream para que el handler pueda clasificarlo.
- `src/handlers/hosting.rs` mapea esos fallos a `ServiceUnavailable` con mensajes útiles en vez de `Internal` genérico.
- `VpsPanel` muestra el `message` real de la API, así el admin ve si falta el API password real o si Contabo está indisponible.
- `.env.example` documenta el contrato recomendado para la configuración opcional de Contabo.

## Estado operativo actual

El código ya no degrada como `500`, pero el listado real de VPS sigue dependiendo de que la cuenta tenga configurado un `CONTABO_API_PASSWORD` válido. Si Contabo responde `invalid_grant`, el panel mostrará ese bloqueo de configuración de forma explícita.

## Validación

- `cargo check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `npm --prefix frontend run type-check`
- `npm --prefix frontend run build`