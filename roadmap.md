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

- Corregir ~126 problemas reales del sentinel-report → ver plan detallado en `Agente/planes/plan-sentinel-problemas-reales-2026-04-12.md`

### Delegaciones y pedidos — ✅ COMPLETADO

> Todo implementado: Wallet, retiros, cancelación con solicitud, ventana 48h, delegaciones, admin assignment UI, activity audit, wallet header, chat auto-greeting al crear orden, email de confirmación, restricción chat 2 personas (cliente+empleado), auto-disable IA cuando empleado responde. 
> Verificado en código: `rest_messages.rs` líneas 104-132, `orders.rs` líneas 88-107.

### Hosting / Infraestructura — bloqueados por dependencias externas

> Todos estos items requieren acceso a servidores remotos, APIs de terceros o credenciales que necesitan revisión manual.

- Hosting/SSH seguro Fase 1: verificación VFS disco en VPS2 (requiere ops en servidor).
- Seguridad hosting Fase 4.1: DNS ownership (requiere definir API provider).
- Contabo rechazó autenticación. Revisar CONTABO_API_PASSWORD y credenciales OAuth2.
- Hosting Automation Fase 4: Dominios y DNS — bloqueado por API DNS provider + Contabo auth.
- COOLIFY_PROJECT_UUID: verificar si el valor en prod coincide con el actual de Coolify (task de ops remota).

### Planes activos — ✅ TODOS COMPLETADOS

> Chatbot v2 (Fases I-II + testing), SEO (Fases 2-3), Seed system (Fase 5) — todo completado y verificado en código.

## Notas de infraestructura

- **nakomi.studio**: VPS1 (66.94.100.241), Coolify service `do8k4w8swccwwogoc0os0ck0`
- **VPS2 Coolify**: Configurado en settings.json
- **Deploy**: Siempre via coolify-manager-rs, nunca desde Coolify UI (ver doc de persistencia volúmenes)
- **Volúmenes**: Documentado en `Agente/documentacion/hosting/coolify-volumenes-persistencia-2026-04-12.md`