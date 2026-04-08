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

(sin tareas pendientes)

3. Ya de soporte, para los servicios, tendran boton de soporte, esto abrira el chat del agente, el agente tiene que entender la intención del usuario, abrio el chat desde el boton de soporte del hosting tal o servicio tal, asi mismo tiene que identificarse de donde abrio el chat en todos los casos, no solo para los chat de soporte, y atender la solicitud del cliente, para esto el cliente necesitas capacidades y poder realizar acciones, esto requiere un plan. 

4. Hasta ahora hay muchas tareas, mucho contexto, el agente puede alucinar con tanta información, la organización del contento tiene que hacerse de manera precisa, muy probablmenente todo lo anterior dicho deben ser mcp resumidos que puedan brindar mas detalles, o tal vez lo mejor sea separar en agentes y crear subagentes, no lo se, estudiar y evaluar las tecnicas mas aficientes a aplicar, limitar el contexto a 64k tokens y que genere un resumen detallado cuando supere el limite en una conversacion. 

5. Algo que olvide mencionar es que todas las conversaciones entre la IA y los clientes tienen que guardarse ordenamente para ser evaluadas despues, para retroalimentanción. 

6. Tuve esta conversacion pero

yo: Estoy testeando tus funcionalidades, puedes crear una factura o crear un pedido? para ver como se ve

IA: 
Factura por $100.00 USD — Servicio de prueba

Aquí tienes una factura de prueba y una lista de nuestros servicios. ¿Necesitas algo más?

No veo ningun boton para pagar ni nada en el mensaje, esperaba un mensaje con un boton o algo y un resumen de la facturara para abrir en un modal.

- Hay que hacer auditoría de seguridad a todo el sistema de chat, y si todo es seguro subir los cambios al servidor vps nakomi.studio

## Detalles

useCarruselInfinito.ts:12 Warning: React has detected a change in the order of Hooks called by CarruselShowcase. This will lead to bugs and errors if not fixed. For more information, read the Rules of Hooks: https://reactjs.org/link/rules-of-hooks

   Previous render            Next render
   ------------------------------------------------------
1. useContext                 useContext
2. useContext                 useContext
3. useContext                 useContext
4. useEffect                  useEffect
5. useState                   useState
6. useCallback                useCallback
7. useSyncExternalStore       useSyncExternalStore
8. useEffect                  useEffect
9. useMemo                    useMemo
10. undefined                 useState
   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

    at CarruselShowcase (

chunk-RPCDYKBN.js?v=57755138:11678 Uncaught Error: Rendered more hooks than during the previous render.
    at useCarruselInfinito (useCarruselInfinito.ts:12:45)
    at CarruselShowcase (CarruselShowcase.tsx:42:65)

# La tarea final

Tienes que comunicarte con el chatbot para probarlo y testearlo, eres un modelo superior claramente tienes que evaluarlo en todos los escenarios, auditarlo, evaluarlo, ir anotando fallos, posible mejoras e ir configurando para maximar el mejor resultado posible. La simulación tiene que ser lo mas parecida a un escenario real y el modelo no se tiene que dar cuenta que esta hablando con otra IA, solo tu. Lo mejor para esto es hacer un plan detallado de una lista de pruebas y cosas que verificar y ver si las pasa todas.

Segunda tarea final: revisar que realmente se este usando la rotacion de api en cada solicitud para evitar rate limits, que se esten usando principalmente los modelos mas inteligentes, y que pase a un segundo modelo mas inteligente de groq y luego a un tercero y asi sucesivamente si falla por rate limits. En allowed_models veo que claramente no esta usando los modelos mas inteligente.


