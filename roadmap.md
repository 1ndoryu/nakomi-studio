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

## Pendientes

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

### Prerequisitos
- ~~P-1: Refactorizar handlers/chat.rs (660 líneas → módulo con 4 archivos)~~ ✅
- ~~P-2: Migración BD (visitor_profiles, chat_attachments, message_type/metadata, ai_intermediary)~~ ✅

### Fase I (por dificultad descendente)
- ~~T-1: Anti-spam + timing inteligente (rate limit WS, clasificador relevancia, máquina de estados para pausas)~~ ✅
- ~~T-2: Generación pedidos + facturas (tool use Groq, mensajes ricos, Stripe invoices, botones acción)~~ ✅
- ~~T-3: Memoria usuario + contexto (visitor_profiles, captura email, resúmenes, contexto por rol)~~ ✅
- ~~T-4: Sync cross-device/tab (BroadcastChannel, multi-conexión WS, sesión única por identidad)~~ ✅
- ~~T-5: Archivos en chat (upload multipart, Groq Vision imágenes, Whisper STT audio, PDF extraction)~~ ✅
- ~~T-6: Escalación humana (detección IA, notificación admin, flujo handoff)~~ ✅
- ~~T-7: Sin disclosure IA (system prompt, branding agente, UI sin indicadores IA)~~ ✅

### Fase II
- ~~T-9: Clientes registrados (detección JWT, contexto de servicios/pedidos/hosting, reportes)~~ ✅
- ~~T-10: IA intermediaria pedidos (toggle por orden, contexto completo, resúmenes automáticos)~~ ✅

### Mejoras adicionales (084A)
- ~~084A-25: Fix hooks CarruselShowcase (useCarruselInfinito antes de early return)~~ ✅
- ~~084A-26: Renderizado mensajes ricos (invoice, service_card, order_card en ChatWidget)~~ ✅
- ~~084A-27: Cadena fallback modelos Groq (70b→90b→mixtral→8b→gemma + rotación keys)~~ ✅
- ~~084A-28: Botones contextuales de soporte (hosting:uuid, service:slug, page:name)~~ ✅
- ~~084A-29: Gestión tokens contexto (truncamiento inteligente historial 16k budget)~~ ✅
- ~~084A-30: Auditoría seguridad (file ext mapping, prompt injection sanitization, XSS blog)~~ ✅

## Pendientes

- ~~084A-31: System prompt: prohibir simulación de herramientas~~ ✅
- ~~084A-32: Subir context budget a 64k tokens + resumen mejorado~~ ✅
- ~~084A-33: Plan de pruebas end-to-end del chatbot + 11 unit tests~~ ✅
- ~~084A-34: Verificar empíricamente rotación de API keys + fallback de modelos~~ ✅
- 084A-35: Deploy a nakomi.studio (subir cambios al VPS)


- llama-4-maverick-17b-128e-instruct esta deprecated!! se debe usar el gpt 
- hay que agregar un segundo modelo, gemma 4, tiene que documentarte bien para agregar este modelo. Hay una GOOGLE_GEMINI_API
