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

## Notas de infraestructura

- **nakomi.studio**: VPS1 (66.94.100.241), Coolify service `do8k4w8swccwwogoc0os0ck0`
- **VPS2 Coolify**: Configurado en settings.json
- **[164A-19] Panel admin de infraestructura**: `Despliegues VPS2` ahora lista servicios reales desde Coolify y `Contabo VPS` queda separado para mostrar solo la capa proveedor.
- **[164A-20] Carrusel del inicio**: las imágenes del showcase ahora salen por `/api/img/...?...w=1200&q=80` con ancho fijo de optimización, sin escalar a buckets mayores por DPR.
- **Deploy**: Siempre via coolify-manager-rs, nunca desde Coolify UI (ver doc de persistencia volúmenes)
- **Volúmenes**: Documentado en `Agente/documentacion/hosting/coolify-volumenes-persistencia-2026-04-12.md`
- **[164A-18] Fix aplicado**: Dockerfile.rust actualizado con `gosu` entrypoint para corregir permisos de volúmenes montados (`root:root` → `appuser`). Fix inmediato aplicado en producción via chown directo. Detalle en `Agente/completados/tareas-2026-04-16.md`.

## Contexto

Proyecto migrado de WordPress a Rust (Axum) + React SPA. El frontend React se integra en frontend/src/. El backend Rust sirve API + SPA.

---

## Tareas pendientes

- Completar el barrido de bordes neutrales: `--border-default` ya es `#dcdcdc`, pero todavia quedan componentes usando `border: 1px solid var(--bg-item-active)` como borde generico cuando no representan un estado activo/seleccionado.
- Necesito datos de prueba para ver como se ve el historial de movimientos y solicitudes de retiro en la wallet (actualmente muestra "Sin movimientos aun" y "No has solicitado retiros aun").
- `POST /api/auth/switch-role` retorna 500 al hacer clic en el boton de cambio de rol en el panel. En validacion directa por API el ciclo `admin -> employee -> client -> admin` responde OK; pendiente reproducirlo desde la sesion/UI real del panel para aislar si el fallo es de frontend, token persistido o estado local.

