Objetivo: Nakomi Studio — sitio web de agencia creativa. Migrado de WordPress a Rust (Axum) + React SPA.
Rama: glory-rust-nakomi
Roadmap de tareas del proyecto: App/roadmap.md

## Estado: 044A-1 completada (migración SPA)

## Stack implementado

| Capa                 | Herramienta                                    |
| -------------------- | ---------------------------------------------- |
| Framework web        | Axum 0.7                                       |
| OpenAPI              | utoipa 4 + utoipa-swagger-ui 7                 |
| Serialización        | serde                                          |
| Base de datos        | SQLx 0.8 (PostgreSQL)                          |
| Migraciones          | SQLx migrate                                   |
| Validación           | validator 0.18                                 |
| Variables de entorno | dotenvy                                        |
| Logging              | tracing + tracing-subscriber                   |
| Errores              | thiserror 2                                    |
| Auth                 | jsonwebtoken + argon2                          |
| CORS                 | tower-http                                     |
| Linter               | clippy (deny all + warn pedantic)              |
| Frontend             | React 18 + TypeScript + Vite                   |
| State                | React Query + Zustand                          |
| Codegen              | Orval 8 (reemplaza openapi-typescript-codegen) |

# Nakomi Studio — Roadmap

## Contexto

Proyecto migrado de WordPress a Rust (Axum) + React SPA. El frontend React de App/React/ se integra en frontend/src/. El backend PHP se reemplaza por el template Rust.

---

## Pendientes (por prioridad — lo más difícil primero)

> Plan maestro: `Agente/planes/plan-marketplace-2026-04-04.md` (11 fases) — ✅ completado
> Plan de chat: `Agente/planes/plan-live-chat-2026-04-04.md` (5 fases) — ✅ completado (streaming IA = mejora futura)
> Plan de hosting: `Agente/planes/plan-hosting-coolify-2026-04-04.md` (5 fases) — Fases 3-4 ✅, Fases 1-2-5 bloqueadas por infraestructura externa (VPS2, DNS, Google Drive OAuth)
> Status hosting: `Agente/documentacion/hosting/status-hosting-administrado-2026-04-07.md`
> 084A-24: Contabo API + Stripe checkout + VPS panel admin + resource stats → ✅ completada (3 commits)

## Hosting — ✅ Plan v2 completado (094A-1 a 094A-11)

> Plan v2: `Agente/planes/plan-hosting-v2-2026-04-09.md` — ✅ 10 tareas + UI polish
> Auditoría de seguridad: `Agente/documentacion/hosting/auditoria-seguridad-hosting-2026-04-09.md`
> Completadas: `Agente/completados/tareas-2026-04-09.md`
>
> Resumen: Página detalle individual (6 tabs), self-service + Stripe checkout, dominios DNS,
> SSH/PuTTY, upgrade/downgrade, soporte, stats reales uptime, auditoría seguridad (6 fixes), 21 tests unitarios.

## Chatbot — Plan v2 en ejecución

> Plan maestro: `Agente/planes/plan-chatbot-v2-2026-04-10.md`
> MemPalace evaluado: no aplica (Python/ChromaDB, no justifica agregar al stack Rust)
>
> **Fase I** — Captación de clientes (front-facing): anti-spam, tool use, facturas, memoria, sync, archivos, escalación, branding
> **Fase II** — Clientes registrados: flujo autenticado, IA intermediaria en pedidos

## Pendientes 

- Necesito probar un hosting real, usar la segunda vps que maneja coolify para crear un hosting, emulando como si un usuario lo hubiera comprado, tiene que ser un hosting real, o sea un despliegue dentro de la vps.
- Al completar el checkout de hosting se debe provisionar el sitio real en Coolify y guardar `coolify_site_name` + datos reales del servidor.
- Cuando falle una renovación o se cancele la suscripción, hay que notificar al cliente y sincronizar suspensión/cancelación real en Coolify, no solo en base de datos.
- HostingDetalle debe dejar de usar IP hardcodeada y mostrar VPS/IP reales desde backend.
- Falta que el cliente pueda comprar y manejar dominios en nuestra plataforma. ¿Que es lo que falta?
- Ejecutar Hosting Automation
- El contador de mensajes nunca se actualiza, nunca se marcan las conversaciones como leida al abrir los mensajes.

## Tareas pendientes extraídas de planes activos (104A-26)

### Chatbot v2 (`plan-chatbot-v2-2026-04-10.md`)
- P-1: Refactorizar `chat.rs` (660 líneas → modular)
- P-2: Migración BD para archivos, perfiles, mensajes especiales
- Fase I (8 tareas): anti-spam, generación pedidos, memoria, sync, archivos, escalación, branding
- Fase II (3 tareas): clientes registrados, IA intermediaria en pedidos
- Testing e2e: 8 smoke tests + tests unitarios (`plan-testing-chatbot-e2e-2026-04-10.md`)

### Seed system (`plan-glory-rs-seed-system-2026-04-07.md`)
- Fase 5: Migrar órdenes, chat, reviews, activity log, notifications de Rust a TOML

### SEO (`plan-seo-completo-2026-04-04.md`)
- Fase 2 pendiente: conversión WebP + srcset + width/height explícitos en imágenes
- Fase 3: Pre-rendering para crawlers

### Hosting Automation (`plan-hosting-automation-2026-04-10.md`)
- Fase 1: Provisioning real post-checkout (endpoint + invocar Coolify)
- Fase 2: Sync cobro (webhooks Stripe + BD)
- Fase 3: Datos reales en panel (IP, VPS, estado)
- Fase 4: Dominios y DNS

## Bloqueado — requiere acción del usuario

- **Dominios** (`plan-dominios-2026-04-07.md`): ¿Qué proveedor DNS? (Cloudflare Registrar, Contabo DNS, Namecheap). Necesito API keys. RESPUESTA; PUES YA TENEMOS LA API DE CONTABO
- **VPS2 Coolify**: Falta `apiToken`, `serverUuid`, `projectUuid` para provisioning real. REPUESTA; NO ENTIENDO PORQUE NO PEUDES CONSEGUIR ESA INFORMACION CON LA API.
- **Contabo DNS API**: Credenciales no validadas. (LA DE LA VPS1 LAS HA PROBADO ANTES Y FUNCIONAN)
- **Google Drive OAuth**: Necesita `auth-drive` manual para backups. (LOS BACKUP AHORA SE HACEN LA VPS2)

## Ultima tarea

Sube los cambios a nakomi.studio