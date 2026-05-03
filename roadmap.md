Objetivo: Nakomi Studio — sitio web de agencia creativa. Migrado de WordPress a Rust (Axum) + React SPA.
Rama: glory-rust-nakomi

## Stack

| Capa | Herramienta |
|------|-------------|
| Framework web | Axum 0.7 |
| OpenAPI | utoipa 4 + utoipa-swagger-ui 7 |
| Base de datos | SQLx 0.8 (PostgreSQL) |
| Validación | validator 0.18 |
| Auth | jsonwebtoken + argon2 |
| Frontend | React 18 + TypeScript + Vite |
| State | React Query + Zustand |
| Codegen | Orval 8 |
| Deploy | coolify-manager-rs |

# Nakomi Studio — Roadmap

## Contexto

Proyecto migrado de WordPress a Rust (Axum) + React SPA. El frontend React se integra en frontend/src/. El backend Rust sirve API + SPA.

---

## Tareas pendientes

- Completar el barrido de bordes neutrales: `--border-default` ya es `#dcdcdc`, pero todavia quedan componentes usando `border: 1px solid var(--bg-item-active)` como borde generico cuando no representan un estado activo/seleccionado.
- Necesito datos de prueba para ver como se ve el historial de movimientos y solicitudes de retiro en la wallet (actualmente muestra "Sin movimientos aun" y "No has solicitado retiros aun").
- Revisar los planes activos en `Agente/planes/` — algunos pueden estar completados pero no movidos a `Agente/planes/completados/`. Evaluar estado real de cada uno.
- `POST /api/auth/switch-role` retorna 500 al hacer clic en el boton de cambio de rol en el panel. Necesita debug con servidor corriendo para ver el log del error (probablemente error de BD o usuario no encontrado con ese rol en local).