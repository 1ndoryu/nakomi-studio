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
- 084A-35: Deploy a nakomi.studio — **BLOQUEADO**: No existe Dockerfile ni docker-compose.yml. Requiere crear infraestructura Docker (Dockerfile multistage Rust + docker-compose con PostgreSQL) antes de poder desplegar. El código está pushed al repo y verificado con tests. Pendiente de preparación de infraestructura.


- ~~084A-36: Eliminar maverick deprecated, usar gpt-oss-120b como primario~~ ✅
- ~~084A-37: Agregar Google Gemini como segundo proveedor AI (GOOGLE_GEMINI_API)~~ ✅

## Pendientes nuevas (por prioridad)

- ~~084A-38: Factura duplicada texto plano — 3 bugs corregidos~~ ✅
- 084A-39: Deploy al servidor — Revisar Coolify por deploys anteriores. El usuario dice que ya se hizo un despliegue antes. Investigar.
- 084A-40: Comando /reset en chat — borrar todo el contexto de la conversación y usuario para empezar de cero al testear.
- 084A-41: Ampliar modelos Gemini — investigar gemini-3-flash, gemini-3.1, etc. y agregarlos al fallback chain.

## Notas del usuario (raw)

- ¿Como que no hay? Si antes se hizo un despliegue!!! Revisa bien coolify!!!

- ¿Porque es genera una factura pagada? No tiene sentido, estoy probando el chat. Ni siquiera sale un boton para pagar o algo directamente sale paga. 

Yo: Necesito que generes otra factura, estoy testeando el chatbot, por favor.

🧾
Factura
Pagada
$100.00 USD

Servicio de prueba para el cliente

Factura por $100.00 USD — Servicio de prueba para el cliente


Aquí tienes una factura de prueba por $100.00 USD. Si necesitas algo más, no dudes en preguntar.

"aparece sobre un chatWidgetMsgRich chatWidgetInvoiceCard" 

Lo volvi a intentar y

Diseño web.

🧾
Factura
Pagada
$100.00 USD

Diseño de sitio web

Factura por $100.00 USD — Diseño de sitio web


¡Genial! Acabo de generar una factura para el servicio de diseño web. El monto es de $100.00 USD. Para pagar, por favor haz clic en el siguiente enlace: https://invoice.stripe.com/i/acct_1M9uLoCdHJpmDkrr/live_YWNjdF8xTTl1TG9DZEhKcG1Ea3JyLF9VSWhNOUp4RHh3ME5LMkMxRnZab0plbWI2TGM3V0FLLDE2NjIzMzQxNQ0200bxIVTZ9L?s=ap

¿Necesitas algo más?

-- 

Me manda esto en texto plano

¡Genial! Acabo de generar una factura para el servicio de diseño web. El monto es de $100.00 USD. Para pagar, por favor haz clic en el siguiente enlace: https://invoice.stripe.com/i/acct_1M9uLoCdHJpmDkrr/live_YWNjdF8xTTl1TG9DZEhKcG1Ea3JyLF9VSWhNOUp4RHh3ME5LMkMxRnZab0plbWI2TGM3V0FLLDE2NjIzMzQxNQ0200bxIVTZ9L?s=ap

cuando voy al enlace dice 

0,00 MXN

Ver datos de la factura
Número de factura	XAZ4W5GW-0003
Fecha de pago	8 de abril de 2026

Descargar factura

Descargar recibo

--

- Agrega un comando para reiniciar el chat, asi puedo testear mejor, lo que hare este comando es borrar todo el contexto de la conversacion y usuario para empezar de cero el comando sería " /reset ", 

- gemini-2.5-flash no es el unico modelo, hay mas, hay gemini flash 3.0 y 3.5 creo o 3.1, investiga bien
