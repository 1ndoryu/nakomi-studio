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
- **Deploy**: Siempre via coolify-manager-rs, nunca desde Coolify UI (ver doc de persistencia volúmenes)
- **Volúmenes**: Documentado en `Agente/documentacion/hosting/coolify-volumenes-persistencia-2026-04-12.md`

## Contexto

Proyecto migrado de WordPress a Rust (Axum) + React SPA. El frontend React se integra en frontend/src/. El backend Rust sirve API + SPA.

---

## Tareas pendientes

- 105A-29 — Mover el selector global de VPS a la zona `logoSidebar` y usar un selector personalizado, no `<select>` nativo.
- 105A-30 — Agregar regla en Glory Sentinel para detectar `<select>` nativos en React/TSX y recomendar el componente personalizado de Nakomi.
- 105A-31 — Convertir `Agregar sitio` en modal funcional con validación, feedback visible y verificación real.
- 105A-32 — Retirar `rutaPagina` de la GUI porque no aporta valor operativo y limpiar la jerarquía visual.
- 105A-33 — Mostrar favicons online de los sitios en la tabla, con fallback seguro y sin ralentizar el listado.
- 105A-34 — Completar publicación de `vps.nakomi.studio` vía Coolify/coolify-manager-rs con dominio extra, health y verificación del subdominio.
- 105A-35 — Implementar login/admin seguro para el portal VPS sin hardcodear credenciales.
- 105A-36 — Endurecer seguridad operativa online: RBAC, rate limits, auditoría y límites de exposición de logs/secrets.
- 105A-37 — Planificar y migrar progresivamente compra/gestión de VPS desde Nakomi principal hacia `vps.nakomi.studio`, reutilizando Stripe/Contabo/Coolify existentes.
- Respecto a vps.nakomi.studio, la publicación y el acceso a controlar la API de Coolify deben estar protegidos solo para el usuario admin. Tiene que ser seguro, con página de inicio, login modal y opción de cerrar sesión en el panel.