# Regeneración de seed por fixtures - 2026-04-10

## Sintoma

- `POST /api/admin/seed` devolvia "Reiniciar el servidor para que los fixtures de content/ se sincronicen primero" incluso despues de reiniciar.
- `GET /api/admin/users` mostraba `admin@admin.com` y `empleado2@test.com`, pero faltaban `cliente@test.com` y `empleado@test.com`.

## Causa raiz

1. `glory-rs/backend/src/fixtures/sync.rs` hacia `skip` de un record tracked si el `content_hash` no habia cambiado, sin comprobar que la fila siguiera existiendo en la tabla real. Si una fila fixture-managed era borrada manualmente pero `_glory_fixtures` conservaba el tracking, el sync no la reinsertaba.
2. `content/users.toml` habia quedado desactualizado frente a `migrations/20260412000000_user_username.up.sql`: `users.username` es `NOT NULL` y los registros fixture no declaraban `username`. Cuando el sync intento recrear `cliente@test.com`, PostgreSQL rechazo la insercion y rompio toda la cascada de fixtures dependientes (`hosting_subscriptions`, `orders`, `order_delegations`, `order_phases`, `order_payments`, `order_refunds`).

## Fix aplicado

- `glory-rs/backend/src/fixtures/sync.rs`: antes de hacer `skip` por hash, validar que `db_id` siga existiendo en la tabla real. Si falta, ejecutar upsert y refrescar tracking.
- `content/users.toml`: agregar usernames canonicos `admin`, `cliente`, `empleado`, `empleado2`.

## Verificacion

- `cargo check` y `cargo clippy -- -D warnings` en `glory-rs/backend`.
- `cargo check` y `cargo clippy -- -D warnings` en el backend principal.
- Arranque temporal del backend con `PORT=3001` y `CARGO_TARGET_DIR=target-temp-3001`.
- Log de arranque corregido: `[fixtures] inserted=0 updated=42 deleted=0 skipped=12 errors=0`.
- `GET /api/admin/users`: reaparecen `cliente@test.com` y `empleado@test.com`.
- `POST /api/admin/seed`: `Seed completado: 6 notificaciones + 1 reviews + 5 mensajes chat + 5 activity log + 5 hosting events`.

## Gotchas

- En Windows, si el backend principal esta corriendo, `cargo run` no puede sobrescribir `target/debug/glory-backend.exe`. Para validar en paralelo sin matar la instancia original, usar un `CARGO_TARGET_DIR` alternativo.