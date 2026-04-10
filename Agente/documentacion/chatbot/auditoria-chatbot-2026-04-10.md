# Auditoría Completa del Sistema de Chatbot — 2026-04-10

## Arquitectura General

Sistema de chat integrado **Rust (Axum) + React + WebSocket + IA (Groq/Gemini)** que maneja:
- **Visitantes anónimos** (widget flotante) con pre-venta conversacional
- **Clientes autenticados** (panel) para soporte de órdenes
- **Empleados/Admins** (panel Mensajes) con gestión centralizada
- **Tool use (function calling)**: facturas Stripe, escalación, captura de datos

## Estructura de Archivos

### Backend (Rust)
| Archivo | Responsabilidad |
|---------|-----------------|
| `src/handlers/chat/mod.rs` | Router builder + tipos compartidos |
| `src/handlers/chat/ws_visitor.rs` | WebSocket visitante anónimo/cliente |
| `src/handlers/chat/ws_staff.rs` | WebSocket staff autenticado (JWT) |
| `src/handlers/chat/rest.rs` | CRUD REST: sesiones, mensajes, notas, upload |
| `src/services/ai_chat.rs` | Generación de respuestas IA con tool use |
| `src/services/ai_tools.rs` | Ejecución de herramientas (invoices, escalación) |
| `src/services/chat_timing.rs` | Rate limiting + typing simulado |
| `src/models/chat.rs` | Modelos: ChatSession, ChatMessage, VisitorProfile, ChatAttachment |
| `src/repositories/chat.rs` | CRUD PostgreSQL con prepared statements |

### Frontend (React/TypeScript)
| Archivo | Responsabilidad |
|---------|-----------------|
| `frontend/src/components/panel/SeccionChat.tsx` | Panel principal Mensajes (staff) |
| `frontend/src/components/panel/ChatBurbujas.css` | Estilos de burbujas de chat |
| `frontend/src/components/panel/ChatInfoPanel.tsx` | Meta visitante (solo admin) |
| `frontend/src/hooks/useSeccionChat.ts` | Orquestador del panel chat |
| `frontend/src/hooks/useChatWs.ts` | WebSocket real-time |
| `frontend/src/hooks/useChat.ts` | REST polling (5s mensajes, 15s sesiones) |
| `frontend/src/hooks/useOrderChat.ts` | Chat de orden específica |
| `frontend/src/hooks/useChatWidget.ts` | Widget flotante visitante anónimo |
| `frontend/src/api/chat.ts` | API client (sessions, messages, notes) |

## Endpoints

### REST
- `GET /api/chat/sessions` — lista sesiones del usuario
- `GET /api/chat/sessions/{id}/messages?limit=50&offset=0` — paginación
- `POST /api/chat/sessions` — crear sesión
- `POST /api/chat/sessions/{id}/messages` — enviar mensaje
- `POST /api/chat/sessions/{id}/notes` — agregar nota (admin)
- `PATCH /api/chat/sessions/{id}/visitor-name` — renombrar visitante

### WebSocket
- `/ws/chat/visitor?visitor_id=X&visitor_name=Y&token=JWT&context=hosting:uuid`
- `/ws/chat/staff?token=JWT`

## Integración IA

### Proveedores
- **Primario**: Groq (3 API keys en round-robin)
- **Fallback**: Google Gemini
- **Modelo default**: `openai/gpt-oss-120b`
- **Modelos whitelist**: 6 modelos (llama-4-scout, qwen3-32b, llama-3.3-70b, gpt-oss-20b, llama-3.1-8b)

### Prompt Dinámico
- **Pre-venta**: lista servicios con precios, contexto visitante
- **Soporte de orden**: contexto de orden, fase actual, historial (últimos 20 mensajes)
- **Seguridad**: `sanitize_for_prompt()` previene prompt injection

### Tools Disponibles
| Tool | Descripción | Output |
|------|-------------|--------|
| `create_invoice` | Factura Stripe con link de pago | RichMessage tipo `invoice` |
| `request_human_assistance` | Escalación a staff | Notificación real-time |
| `capture_email` | Guarda email en visitor_profiles | Persistencia BD |
| `save_client_info` | JSON en preferences (industria, presupuesto) | Contexto reutilizable |
| `create_support_ticket` | Ticket de soporte (parcial) | ticket_id |

### Tools Removidas (084A-51)
- `show_service`, `list_services` — antinatural en chat conversacional (recuperables de git)

## Base de Datos

### Tablas
- `chat_sessions` — sesiones con visitor_id, user_id, order_id, ai_enabled, status
- `chat_messages` — mensajes con sender_type, message_type (text/invoice/service_card/etc), metadata JSONB
- `visitor_profiles` — perfil persistente: email, preferences, context_summary, dispositivos
- `chat_attachments` — archivos (modelo existe, integración pendiente)
- `chat_session_notes` — notas internas staff

## Flujos de Datos

### Visitante Anónimo
1. Widget genera `visitor_id` en localStorage
2. WS → `/ws/chat/visitor?visitor_id=X&context=service:slug`
3. Backend crea sesión, broadcast a staff
4. Cada mensaje → guarda BD → envía a IA → guarda respuesta → broadcast WS

### Staff
1. Panel Mensajes → WS `/ws/chat/staff?token=JWT`
2. Recibe `init` con sesiones activas
3. `joinSession(id)` → REST fetch mensajes + suscribir WS
4. Hybrid: REST polling 5s + WS real-time

## Bugs y Limitaciones Identificados

### 🔴 Críticos

1. **Rich messages no se renderizan** — MessageBubble ignora `message_type` y `metadata`. Invoices generadas por IA no muestran botón de pago. Solo renderiza `content` como texto plano.
   - Ubicación: `SeccionChat.tsx` (componente de burbuja)
   - Impacto: Toda la funcionalidad de tool use visual está rota en frontend

2. **`create_invoice` parsing fragile** — Algunos modelos envían `amount_cents` como float (10000.0). Parcialmente corregido en 084A-52.

### 🟡 Altos

3. **Sin rate limiting en REST** — `POST /api/chat/sessions/{id}/messages` no tiene protección. ChatTimingService solo protege WS.
   - Fix: Aplicar `check_message_rate()` en rest.rs

4. **Cierre de sesión solo visual** — "Volver a la lista" limpia UI pero no cierra sesión en BD.
   - Fix: Implementar `handleCloseSession()` → `apiCloseSession()`

5. **Info panel solo admin** — Staff no ve IP, User-Agent, notas del visitante.
   - Fix: Cambiar condición a `isStaffOrAdmin`

6. **Sin persistencia de contexto entre sesiones** — VisitorProfile existe pero `context_summary` nunca se actualiza. System prompt no usa preferencias.

7. **Typing indicator sin timeout** — Si cliente desconecta mientras escribe, typingMap queda indefinidamente.
   - Fix: Timeout 5s en typingMap

### 🟢 Bajos

8. **Límite hard de 20 mensajes en contexto IA** — Conversaciones largas pierden contexto inicial.
9. **Upload de archivos no integrado** — Modelo ChatAttachment existe, POST multipart parcial.
10. **Typing indicator sin animación CSS real** — Dots estáticos.

## Capacidades Actuales vs Deseadas

### ✅ Funciona
- Conversación natural sobre servicios
- Preguntas sobre precios
- Escalación por frustración/legal
- Generación de invoices (flujo Stripe completo)
- Captura email y preferencias
- Contexto dinámico por orden/hosting/visitor
- Anti-bot per IP en WS
- Rate limiting en WS visitor

### ❌ No Funciona / No Implementado
- Renderizar rich messages (invoices, cards) en frontend
- Upload/procesamiento de archivos (Vision, Whisper)
- Resumen automático de conversaciones
- Personalización real por visitor_profile en prompt
- Atención contextual a clientes registrados
- Reportes/dashboard de soporte
- Configuración de chatbot desde panel admin
- Reportar problemas de órdenes desde chatbot

## Configuraciones Admin Propuestas

### Panel de Configuración del Chatbot (SeccionChatbotConfig)

1. **General**
   - Toggle IA activada/desactivada globalmente
   - Modelo IA seleccionable (de la whitelist)
   - Saludo inicial personalizable
   - Horario de atención (fuera de horario → solo IA)

2. **Prompt/Personalidad**
   - Nombre del asistente IA
   - Tono (formal, amigable, técnico)
   - Instrucciones adicionales (texto libre, se agrega al system prompt)
   - Lista de respuestas rápidas predefinidas para staff

3. **Herramientas**
   - Toggle por tool (create_invoice, capture_email, etc.)
   - Límite de monto máximo para invoices automáticas
   - Email de notificación para escalaciones

4. **Rate Limiting**
   - Máx mensajes/minuto por visitante
   - Máx conexiones WS simultáneas por IP
   - Cooldown entre mensajes

5. **Anti-Spam**
   - Palabras bloqueadas
   - Bloqueo por IP (manual)
   - Captcha después de N mensajes sin login

6. **Notificaciones Staff**
   - Toggle: notificar nueva sesión
   - Toggle: notificar escalación
   - Toggle: notificar inactividad prolongada
   - Sonido de notificación personalizable

7. **Branding**
   - Avatar del chatbot
   - Colores del widget (primario, secundario)
   - Posición del widget (esquina)
   - Texto del botón flotante

### Almacenamiento Propuesto
- Tabla `chatbot_config` (key-value JSON) o un registro único con columnas tipadas
- Cargar en memoria al iniciar servidor (cache)
- Endpoint admin: `GET/PUT /api/admin/chatbot/config`
- Frontend: sección en panel admin con formularios por categoría
