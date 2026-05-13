# Studio Health + Self-Heal — 2026-05-13

## Contexto
`studio` quedó en estado `unhealthy`: el proceso Rust seguía vivo, escuchaba en `0.0.0.0:3000` y aceptaba TCP, pero `curl http://127.0.0.1:3000/api/health` conectaba y agotaba timeout sin recibir bytes. Coolify/Traefik mostraban `no available server`.

## Cambios
- `/healthz` queda fuera de `/api` para healthchecks de Docker/Coolify.
- El template Rust usa `curl -fsS http://127.0.0.1:3000{{HEALTH_PATH}}` y elimina la dependencia de `awk`, ausente en `debian:bookworm-slim`.
- El template Rust declara labels Traefik explícitos al puerto 3000 para evitar FQDN/routing congelados por Coolify.
- `image_processing` mueve decode/resize/encode a `spawn_blocking` y limita concurrencia a 2 para que los `srcset` de páginas públicas no bloqueen workers async.
- `main.rs` lanza un watchdog local: tras 3 fallos consecutivos de `/healthz`, el proceso sale y Docker lo reinicia.

## Validación
- `cargo check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `coolify-manager-rs`: `cargo test`, `cargo clippy -- -D warnings`, `cargo build --release --target-dir target`

## Gotchas
- Un proceso que acepta TCP pero no responde HTTP no queda cubierto por probes que solo miran `docker ps` o memoria.
- En runtime slim, no asumir utilidades como `awk`; healthchecks deben depender de lo mínimo.
