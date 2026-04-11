# Auditoría Completa del Sistema de Chat/Chatbot

> **Fecha:** 2026-04-16
> **Estado:** Auditoría completada, issues identificados
> **Cobertura:** Backend (Rust/Axum), Frontend (React), WebSocket, IA (Groq/Gemini), Stripe

---

## I. Archivos involucrados

### Backend (Rust)

| Archivo                       | Líneas | Responsabilidad                                            |
| ----------------------------- | ------ | ---------------------------------------------------------- |
| `handlers/chat/mod.rs`        | ~120   | Router + tipos compartidos                                 |
| `handlers/chat/rest.rs`       | ~750   | Endpoints REST (sesiones, mensajes, notas, upload, cierre) |
| `handlers/chat/ws_visitor.rs` | ~500   | WebSocket visitantes (anónimos/clientes) + rate limiting   |
| `handlers/chat/ws_staff.rs`   | ~200   | WebSocket staff (suscripción a sesiones)                   |
| `services/chat.rs`            | ~350   | ChatHub: broadcast, multi-conexión, enriquecimiento        |
| `services/chat_timing.rs`     | ~700   | Máquina de estados, rate limiting, relevance check         |
| `services/ai_chat.rs`         | ~900   | Respuesta IA, system prompts dinámicos, tool calls         |
| `services/ai_tools.rs`        | ~600   | Tool execution: create_invoice, capture_email, escalación  |
| `models/chat.rs`              | ~250   | Tipos: ChatSession, ChatMessage, VisitorProfile, etc.      |
| `repositories/chat.rs`        | ~500   | CRUD BD: sesiones, mensajes, perfiles, adjuntos, notas     |

### Frontend (React/TypeScript)

| Archivo                            | Responsabilidad                                            |
| ---------------------------------- | ---------------------------------------------------------- |
| `api/chat.ts`                      | API client (REST + WS URL builders)                        |
| `hooks/useChat.ts`                 | REST polling: sesiones (15s), mensajes (5s)                |
| `hooks/useChatWs.ts`               | WebSocket staff: init, join, typing                        |
| `hooks/useSeccionChat.ts`          | Orquestador panel chat + cierre                            |
| `hooks/useChatWidget.ts`           | Widget visitante: WS, BroadcastChannel, upload, reconexión |
| `components/chat/ChatWidget.tsx`   | UI widget: mensajes, input, upload, rich rendering         |
| `components/panel/SeccionChat.tsx` | Panel staff: lista sesiones + chat + info                  |
| `components/panel/ChatBell.tsx`    | Bell icon en header: dropdown, unread badge                |
| `stores/chatStore.ts`              | Zustand: `abierto`, `context` (para contexto contextual)   |

### Base de datos

- `chat_sessions` — sesiones activas (visitor_id, user_id, order_id, status, ai_enabled, assigned_staff_id)
- `chat_messages` — historial completo (sender_type, message_type, metadata JSON)
- `visitor_profiles` — memoria del visitante (email, preferences, context_summary)
- `chat_attachments` — archivos (file_name, mime_type, ai_description)
- `chat_session_notes` — notas internas staff

---

## II. Endpoints y autenticación

### REST API (`/api/chat/*`)

| Endpoint                               | Método   | Auth        | Roles                                 |
| -------------------------------------- | -------- | ----------- | ------------------------------------- |
| `/api/chat/sessions`                   | GET      | JWT         | Client (propias), Staff/Admin (todas) |
| `/api/chat/sessions`                   | POST     | JWT         | Client, Staff, Admin                  |
| `/api/chat/sessions/{id}/messages`     | GET      | JWT         | Participantes + Admin                 |
| `/api/chat/sessions/{id}/messages`     | POST     | JWT         | Participantes + Admin                 |
| `/api/chat/sessions/{id}/close`        | POST     | JWT         | Staff, Admin                          |
| `/api/chat/sessions/{id}/mark-viewed`  | PATCH    | JWT         | Staff, Admin                          |
| `/api/chat/sessions/{id}/notes`        | GET/POST | JWT         | Staff, Admin                          |
| `/api/chat/sessions/{id}/visitor-name` | PATCH    | JWT         | Staff, Admin                          |
| `/api/chat/sessions/{id}/upload`       | POST     | **Ninguno** | Visitante, Client                     |

### WebSocket

| Endpoint           | Auth            | Roles                       |
| ------------------ | --------------- | --------------------------- |
| `/ws/chat/visitor` | JWT opcional    | Visitante (anónimo), Client |
| `/ws/chat/staff`   | JWT obligatorio | Staff, Admin                |

---

## III. Comportamiento por tipo de usuario

### A. Visitante anónimo (widget flotante)

- **Identificación:** localStorage UUID (`visitor_id`), persistente
- **Primer contacto:** Widget flotante esquina inferior, saludo automático IA
- **System prompt:** Pre-venta (servicios, precios, horarios)
- **IA:** Activa por defecto, desactiva si staff responde
- **Rate limit:** 10 msgs/min per visitor_id, 30/min per IP, max 10 WS por IP
- **Capacidades:** Preguntar, recibir facturas Stripe, dar email, subir archivos
- **Reconexión:** Historial reenviado (50 msgs), BroadcastChannel multi-tab sync
- **Contexto:** Se pasa via `chatStore.abrir('service:slug')` o `'page:hosting'`

### B. Cliente registrado (panel)

- **Acceso:** Tab "Mensajes" en dashboard, no widget flotante
- **Sesiones:** Una por orden activa, REST polling
- **System prompt:** Soporte de orden (fase, entregables, fechas)
- **IA:** Activa, desactiva si empleado responde
- **Rate limit:** 10/min WS, **pero REST NO tiene límite (BUG)**
- **Capacidades:** Preguntar sobre orden, pedir cambios, pagar extras
- **Reportes:** Botón "Reportar problema" → abre chatbot con `context='problem:{order_id}'`

### C. Empleado (staff)

- **Acceso:** Tab "Mensajes" en panel, WebSocket staff
- **Sesiones visibles:** Asignadas + todas activas (si admin)
- **IA:** Desactiva automáticamente al escribir `→ staff_handling`
- **Rate limit:** Ninguno
- **Capacidades:** Responder, renombrar visitante, notas internas, cerrar sesión, ver metadata
- **Info panel:** IP, User-Agent, email, sesiones previas — **pero actualmente solo visible para admin (BUG)**

### D. Admin

- **Todo lo del staff +** ver TODAS las sesiones, acceso completo
- **Escalaciones:** Recibe notificaciones cuando IA detecta necesidad de human
- **Configuración chatbot:** No implementado aún (futuro)

---

## IV. Contexto del chat desde reportes

**Implementado en `OrdenDetalle.tsx` línea 151:**

- Cliente: botón "Reportar problema" → `chatStore.abrir('problem:' + order.id)` → abre widget con contexto
- Empleado: botón "Reportar problema" → abre modal directo (`setModalReportarAbierto`)
- El `context` se pasa al WS como query param → el system prompt del backend lo utiliza para contextualizar la conversación IA con datos de la orden
- **Gap:** No hay UI específica en el widget que indique que se abrió desde un reporte. El visitante solo ve el chat normal, no un banner tipo "Reportando problema con orden #X"

---

## V. Problema del "Chat general" (naming)

**Ubicación:** `SeccionChat.tsx` línea 27 y 114, `ChatBell.tsx` línea 125

```typescript
return s.visitor_name || 'Chat general';
```

**Problema:** Todo visitante sin nombre aparece como "Chat general". Si hay 15 visitantes activos, el staff ve 15 entradas idénticas "Chat general".

**Impacto:** Staff no puede distinguir entre sesiones de visitantes diferentes.

**Solución propuesta:** → **RESUELTO (164A-13)**: Fallback cambiado a `Visitante #${id.slice(-4).toUpperCase()}`

1. ~~Fallback a visitor_id parcial~~ → Implementado con últimos 4 chars del session ID
2. Mostrar email si se capturó (`visitor_profile.email`) — pendiente
3. Mostrar contexto: "Visitante (hosting)", "Visitante (diseño web)" según `session.context` — pendiente
4. Mostrar foto de perfil del otro participante en la lista (ya tienen avatar para usuarios logueados, falta para anónimos — usar iniciales o avatar genérico numerado)

---

## VI. Bugs y issues encontrados

### 🔴 Críticos

| #   | Problema                                                                                                        | Ubicación                         | Impacto                                                                                                           |
| --- | --------------------------------------------------------------------------------------------------------------- | --------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| 1   | ~~Rich messages no renderizan en panel staff~~ | `ChatBurbujaMessage.tsx` | **YA RESUELTO** (104A-32) |
| 2   | ~~Sin rate limit en REST~~ | `rest.rs` | **YA RESUELTO** (104A-36) |
| 3   | ~~Info panel solo visible para admin~~ | `SeccionChat.tsx` | **YA RESUELTO** (104A-36) |

### 🟡 Altos

| #   | Problema                                                                                                                                     | Ubicación                              | Impacto                                         |
| --- | -------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------- | ----------------------------------------------- |
| 4   | **Context summary no se reutiliza** — `visitor_profiles.context_summary` se genera al cerrar sesión pero NUNCA se carga en próximas sesiones | `ai_chat.rs` → `build_system_prompt()` | IA no recuerda visitantes recurrentes           |
| 5   | **"Chat general" naming** — todos los anónimos = "Chat general"                                                                              | `SeccionChat.tsx`, `ChatBell.tsx`      | Staff no puede distinguir sesiones              |
| 6   | **Upload file solo visitante** — SeccionChat no tiene botón de adjuntar                                                                      | `SeccionChat.tsx`                      | Staff no puede enviar archivos al cliente       |
| 7   | **Sin timeout de sesión inactiva** — sesión no se cierra automáticamente tras inactividad                                                    | No implementado                        | Sesiones zombie se acumulan                     |
| 8   | **Report context sin UI** — abrir chat desde "Reportar problema" no muestra banner indicativo                                                | `ChatWidget.tsx`                       | Cliente no sabe que está en contexto de reporte |

### 🟢 Bajos

| #   | Problema                                                                     | Ubicación        | Impacto                                               |
| --- | ---------------------------------------------------------------------------- | ---------------- | ----------------------------------------------------- |
| 9   | Typing indicator sin timeout (persiste si visitante desconecta mid-typing)   | `useChatWs.ts`   | Artefacto visual                                      |
| 10  | Contexto histórico limitado a 20 msgs (conversaciones largas pierden inicio) | `ai_chat.rs`     | Calidad de respuesta IA baja en conversaciones largas |
| 11  | Relevance check desactivado (falsos positivos)                               | `chat_timing.rs` | OK por ahora                                          |

---

## VII. Lo que funciona bien

- ✅ Conversación natural sobre servicios con contexto
- ✅ Generación de invoices Stripe + pago directo desde chat
- ✅ Captura de email y preferencias del visitante
- ✅ Escalación automática a human (IA detecta frustración/legal)
- ✅ WebSocket bidireccional visitor + staff
- ✅ Anti-bot per IP (rate limit WS, max conexiones)
- ✅ Multi-tab sync (BroadcastChannel)
- ✅ Reconexión automática con backoff exponencial
- ✅ System prompts distintos por tipo de sesión (pre-venta, orden, hosting)
- ✅ Tool use con Groq + Gemini fallback
- ✅ Máquina de estados timing (WAIT, LISTEN, RESPOND)
- ✅ Notas internas para staff
- ✅ Presencia real-time (online/offline dot)
- ✅ Contexto de página al abrir chat (service, hosting, problem)

---

## VIII. Recomendaciones priorizadas

### Inmediato (próxima sesión)

1. **Fix #1:** Implementar `renderMessageContent()` en SeccionChat que parsee `message_type` y `metadata`
2. **Fix #2:** Agregar `check_rate()` en REST `send_message()`
3. **Fix #3:** Cambiar `showInfoPanel` de `admin` a `isStaff`
4. **Fix #5:** Reemplazar "Chat general" por `Visitante #{id.substring(0,6)}` + contexto

### Corto plazo

5. **Fix #4:** Llamar `append_visitor_context()` en `build_system_prompt()` para reutilizar context_summary
6. **Fix #6:** Agregar botón upload en SeccionChat
7. **Fix #8:** Mostrar banner "Reportando problema con orden #X" en ChatWidget cuando context empieza con "problem:"
8. **Fix #7:** Implementar timeout de sesión inactiva (24h → auto-close)

### Medio plazo (chatbot v2)

9. Refactorizar chat.rs en módulos (ya planificado: P-1)
10. Panel admin de configuración chatbot
11. Analytics de chat (tiempos de respuesta, satisfacción, temas frecuentes)
12. Exportar conversaciones

---

## IX. Arquitectura (diagrama texto)

```
┌─────────────── FRONTEND ───────────────────────────────────────┐
│                                                                 │
│  Widget (visitante)           Panel (staff/admin)              │
│  ┌────────────────┐           ┌────────────────┐              │
│  │ ChatWidget     │           │ SeccionChat    │              │
│  │ useChatWidget  │←─WS──→   │ useChatWs     │              │
│  │ chatStore      │           │ useChat (REST) │              │
│  └────────────────┘           │ ChatBell       │              │
│         ↑                     └────────────────┘              │
│    chatStore.abrir('service:x')                                │
│    chatStore.abrir('problem:orderId')                          │
└────────────────────────────────────────────────────────────────┘
                    ↕ WebSocket + REST
┌─────────────── BACKEND (Rust/Axum) ───────────────────────────┐
│  handlers/chat/                                                │
│  ├ ws_visitor.rs → ChatTimingService → AiChatService          │
│  ├ ws_staff.rs → ChatHub (broadcast)                          │
│  └ rest.rs → ChatRepository                                   │
│                                                                │
│  services/                                                     │
│  ├ ChatHub (DashMap channels, staff_channel)                  │
│  ├ ChatTimingService (rate limit, state machine)              │
│  ├ AiChatService (Groq/Gemini, system prompts)               │
│  └ AiToolsService (Stripe, email, escalation)                │
└────────────────────────────────────────────────────────────────┘
                    ↕ SQLx
┌─────────────── PostgreSQL ─────────────────────────────────────┐
│  chat_sessions · chat_messages · visitor_profiles              │
│  chat_attachments · chat_session_notes                         │
└────────────────────────────────────────────────────────────────┘
                    ↕ HTTP
┌─────────── APIs Externas ──────────────────────────────────────┐
│  Groq (3 keys round-robin) · Gemini (fallback) · Stripe       │
└────────────────────────────────────────────────────────────────┘
```
