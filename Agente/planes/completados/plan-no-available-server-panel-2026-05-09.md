# Plan — No available server y pantalla blanca panel — 2026-05-09

## Tareas

- `095A-21` — Corregir pantalla blanca al entrar/salir del panel.
- `095A-22` — Resolver de raíz el “no available server” recurrente.
- `095A-23` — Sincronización segura de envs de producción.
- `105A-1` — Evitar falsos healthy cuando localhost responde pero la IP Docker no.

## Resultado

- `ChatWidget` ya no rompe el orden de hooks al ocultarse en `/panel`.
- `deploy-service` alinea Postgres con hostname único `postgres-{uuid}` y password real del rol `rust_app`.
- El bind mount `/data/uploads/studio:/app/uploads` se fuerza y verifica antes/después del swap.
- Stripe publishable key se sincroniza con `sync-env` y el frontend la obtiene por `/api/public-config` en runtime.
- Rust health ya no confía en `localhost`: valida la IP Docker del contenedor y el deploy recrea `app` una vez si detecta el fallo de red.

## Validación

- `coolify-manager-rs`: `cargo test health_manager`, `cargo test network_recovery_tests`, `cargo check`, `cargo build --release`.
- `glory-rust-template`: `cargo fmt --check`, `cargo check`.
- Producción: `deploy-service --name studio --skip-backup --skip-build`, `health --name studio`, navegador público, `/api/health`, `/api/public-config`.

## Estado

- Completado.