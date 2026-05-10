# Coolify Rust — Healthcheck por red Docker — 2026-05-10

## Contexto

En `studio` apareció un falso estado sano: el proceso podía responder dentro del contenedor por `localhost`, pero el acceso por IP de red Docker quedaba colgado y Traefik terminaba mostrando `no available server`.

## Causa raíz

- `localhost` dentro del contenedor solo prueba el loopback del namespace.
- Traefik no entra por loopback; entra por la IP del contenedor en la red Docker del stack.
- Si el proceso escucha en loopback, o si la red queda en un estado raro tras swap, Docker health puede quedar verde mientras el proxy no tiene backend funcional.

## Corrección

- El template Rust de Coolify cambió su `healthcheck` para resolver `hostname -i` y llamar `http://{container_ip}:3000/...`.
- `coolify-manager-rs health` ahora valida Rust con un probe host → IP del contenedor en la red del stack.
- `deploy-service` detecta `Rust network probe fallo` y recrea una sola vez `app` con `docker compose up -d --no-build --force-recreate --no-deps app`, verificando antes que `/data/uploads/{sitio}:/app/uploads` siga en el compose.
- `HOST` por defecto en los backends Rust pasó a `0.0.0.0`; `.env.example` mantiene `127.0.0.1` solo como configuración local explícita.

## Verificación

- `cargo test health_manager` y `cargo test network_recovery_tests` pasan en `coolify-manager-rs`.
- `cargo check` pasa en `coolify-manager-rs` y `glory-rust-template`.
- Producción se aplicó con `coolify-manager.exe deploy-service --name studio --skip-backup --skip-build`.
- `coolify-manager.exe health --name studio` devolvió `http_ok=true app_ok=true fatal_logs=false`.
- Verificación runtime: `HOST=0.0.0.0`, IP Docker `10.0.11.3`, `NETWORK_IP_HEALTH_OK`.
- Navegador: `https://nakomi.studio/` carga, `/api/health` responde 200 y `/api/public-config` responde 200.

## Pendiente observado

Durante el health global post-deploy, `kamples` devolvió 503. Es un sitio distinto y no se mezcló con este bloque.