# Plan Maestro: Chatbot v2 — Nakomi Studio
**Fecha:** 2026-04-10  
**Contexto:** Evolución del chat existente (plan-live-chat completado) hacia un sistema completo de captación/atención al cliente
**Prioridad:** Las tareas están ordenadas por dificultad descendente (lo más difícil primero)

---

## Estado actual del sistema

### Lo que ya funciona
- WebSocket bidireccional: visitante anónimo (widget flotante) + staff (panel Mensajes)
- IA con Groq (llama-3.3-70b-versatile), round-robin de 3 API keys
- System prompt dinámico (servicios + contexto de orden)
- Sesiones pre-venta (visitor_id) y vinculadas a órdenes (order_id)
- Typing indicators (throttled 200ms)
- Panel staff: lista sesiones, chat activo, toggle IA, cerrar sesión
- Info panel admin: IP, User-Agent, notas, renombrar visitante
- Chat de órdenes (useOrderChat, REST polling 5s)
- Persistencia: mensajes, sesiones, typing, notas en PostgreSQL
- Upload de archivos existente para deliverables (multipart, MIME whitelist, magic bytes)

### Lo que falta (del roadmap)
1. **Archivos en chat** — imágenes, PDF, audio con procesamiento IA multimodal
2. **Anti-spam + timing inteligente** — modelo de relevancia, rate limiting WS, pausas humanas
3. **Generación de pedidos/facturas** — tool use, mensajes ricos, botones de acción
4. **Memoria de usuario** — perfil por IP/dispositivo/cuenta, captura email, contexto persistente
5. **Contexto por rol** — visitante vs cliente vs admin/empleado, resúmenes de conversación
6. **Sync cross-device** — misma sesión en todas las ventanas/dispositivos
7. **Escalación humana** — detección inteligente + notificación en tiempo real
8. **Sin disclosure IA** — eliminar indicadores de que es IA
9. **Atención clientes registrados** — ver servicios activos, pedidos, hosting, reportes
10. **IA intermediaria en pedidos** — toggle por orden, contexto completo, resúmenes

### Evaluación de MemPalace
Investigado: es un sistema Python + ChromaDB para memoria de conversaciones de desarrollador (local, offline). **No aplica** para nuestro caso:
- Necesitamos memoria en PostgreSQL (ya tenemos la BD)
- El contexto del chatbot es por visitante/cliente, no por sesión de desarrollo
- Nuestro sistema de resúmenes y perfiles se implementará directamente en Rust
- No justifica agregar Python como dependencia al stack

---

## Pre-requisitos (antes de Fase I)

### P-1: Refactorizar handler de chat (OBLIGATORIO)
**Problema:** `src/handlers/chat.rs` tiene ~660 líneas y va a crecer significativamente con las nuevas features. Viola el límite de 300 líneas por archivo.

**Acción:**
```
src/handlers/chat.rs (~660 lines) →
  src/handlers/chat/mod.rs            — re-exports + route builder
  src/handlers/chat/ws_visitor.rs     — WS visitor handler + helpers
  src/handlers/chat/ws_staff.rs       — WS staff handler + helpers  
  src/handlers/chat/rest.rs           — REST endpoints (CRUD sessions, messages, notes)
  src/handlers/chat/types.rs          — Request/response types locales al handler
```

**Validación:** `cargo check && cargo clippy -- -D warnings` + mismo comportamiento funcional

### P-2: Migración BD para features nuevas
**Nuevas tablas/columnas:**
```sql
-- Perfil persistente del visitante (memoria)
CREATE TABLE visitor_profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    visitor_id VARCHAR(64) UNIQUE NOT NULL,      -- El mismo de localStorage
    email VARCHAR(255),                           -- Capturado progresivamente
    user_id UUID REFERENCES users(id),            -- Si se registra después
    display_name VARCHAR(100),
    context_summary TEXT,                          -- Resumen IA de conversaciones previas
    preferences JSONB DEFAULT '{}',               -- Datos extraídos por IA
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    total_sessions INTEGER NOT NULL DEFAULT 0,
    ip_addresses TEXT[] DEFAULT '{}',              -- Historial de IPs
    device_fingerprints TEXT[] DEFAULT '{}'        -- User-agents conocidos
);
CREATE INDEX idx_visitor_profiles_email ON visitor_profiles(email) WHERE email IS NOT NULL;
CREATE INDEX idx_visitor_profiles_user ON visitor_profiles(user_id) WHERE user_id IS NOT NULL;

-- Archivos adjuntos en mensajes de chat
CREATE TABLE chat_attachments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    message_id UUID NOT NULL REFERENCES chat_messages(id) ON DELETE CASCADE,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(512) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    file_size_bytes BIGINT NOT NULL,
    ai_description TEXT,                          -- Descripción generada por IA (vision/STT)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_chat_attachments_message ON chat_attachments(message_id);

-- Mensajes especiales (facturas, servicios, acciones)
ALTER TABLE chat_messages ADD COLUMN IF NOT EXISTS message_type VARCHAR(20) DEFAULT 'text';
-- Tipos: text | image | file | audio | invoice | service_card | order_card | action
ALTER TABLE chat_messages ADD COLUMN IF NOT EXISTS metadata JSONB;
-- metadata contiene datos estructurados según message_type:
-- invoice: { stripe_invoice_id, amount_cents, currency, status, payment_url }
-- service_card: { service_slug, title, base_price_cents }
-- order_card: { order_id, order_number, status, service_title }
-- action: { action_type, label, payload }
-- image/file/audio: { attachment_id, file_name, mime_type, ai_description }

-- Configuración de IA por pedido (para Fase II)
ALTER TABLE orders ADD COLUMN IF NOT EXISTS ai_intermediary_enabled BOOLEAN DEFAULT false;
ALTER TABLE orders ADD COLUMN IF NOT EXISTS ai_summary TEXT;
```

**Validación:** `cargo sqlx prepare` + tests de migración up/down

---

## Fase I — Captación de clientes (front-facing)

### Tarea 1: Sistema anti-spam + timing inteligente de respuestas
**Dificultad:** ★★★★★ (la más compleja — orquestación de múltiples señales en tiempo real)
**Dependencias:** P-1

**Componentes:**

**1a. Rate limiting WS por visitante**
- Max 10 mensajes/minuto por visitor_id (configurable)
- Cooldown progresivo: 1er exceso → warning, 2do → 30s mute, 3ro → sesión cerrada con mensaje
- Implementar como middleware en el handler WS del visitante, no como GovernorLayer global
- Contadores en DashMap<String, (u32, Instant)> dentro de ChatHub

**1b. Clasificador de relevancia (modelo pequeño)**
- Usar Groq con modelo pequeño: `llama-3.1-8b-instant` (rápido y barato)
- Prompt: "¿Este mensaje es relevante para una agencia de diseño web? Responde solo 'sí' o 'no'."
- Si "no" → respuesta genérica sin gastar tokens del modelo grande
- Si el visitante insiste con temas irrelevantes (3+ seguidos) → mensaje de cierre cortés
- Config: `AI_RELEVANCE_MODEL` en .env, desactivable con flag

**1c. Timing inteligente de respuestas (lo más difícil)**
- **Problema**: Un cliente puede enviar "hola" → "tengo un proyecto" → "es sobre una tienda online" → "quiero que tenga pagos con stripe" en 4 mensajes seguidos. La IA no debe responder a cada uno.
- **Solución — Máquina de estados por sesión:**
  ```
  Estado IDLE → Recibe mensaje → Estado WAITING (timer 4s)
  Estado WAITING → Timer expira sin nuevo mensaje → Estado RESPONDING → genera respuesta → IDLE
  Estado WAITING → Recibe otro mensaje → Resetear timer a 4s
  Estado WAITING → Typing indicator activo → Estado LISTENING (timer 8s desde último typing)
  Estado LISTENING → Typing stops + no mensaje en 3s → Estado RESPONDING
  Estado LISTENING → Nuevo mensaje → Agregar al buffer + reset timer
  Estado RESPONDING → En progreso, no acepta nuevos triggers
  ```
- Buffer de mensajes: acumular todos los mensajes del visitante desde que empezó a escribir, enviar todos al modelo como contexto
- Timeout máximo: 30s desde primer mensaje → forzar respuesta aunque siga escribiendo
- Implementar como `tokio::spawn` con `tokio::time::sleep` + `tokio::select!` en el session handler

**Test:** Enviar 3 mensajes rápidos via WS → verificar que la IA responde UNA vez con contexto de los 3. Enviar mensaje offtopic → verificar respuesta genérica sin llamada al modelo grande.

---

### Tarea 2: Generación de pedidos + facturas desde chat
**Dificultad:** ★★★★★ (integración Stripe + tool use + mensajes ricos)
**Dependencias:** P-1, P-2

**Componentes:**

**2a. Tool use / Function calling en Groq**
- Groq soporta tool use con Llama 4 Scout y llama-3.3-70b
- Definir tools disponibles para la IA:
  ```json
  {
    "tools": [
      {
        "type": "function",
        "function": {
          "name": "show_service",
          "description": "Muestra una tarjeta de servicio al cliente",
          "parameters": { "service_slug": "string" }
        }
      },
      {
        "type": "function",
        "function": {
          "name": "show_project",
          "description": "Muestra un proyecto de ejemplo al cliente",
          "parameters": { "project_slug": "string" }
        }
      },
      {
        "type": "function",
        "function": {
          "name": "create_order_draft",
          "description": "Crea un borrador de pedido con el servicio seleccionado",
          "parameters": { "service_slug": "string", "plan_id": "string", "client_notes": "string" }
        }
      },
      {
        "type": "function",
        "function": {
          "name": "create_invoice",
          "description": "Genera una factura de Stripe para el cliente",
          "parameters": { "amount_cents": "integer", "description": "string", "client_email": "string" }
        }
      },
      {
        "type": "function",
        "function": {
          "name": "request_human_assistance",
          "description": "Solicita intervención humana cuando no puede resolver",
          "parameters": { "reason": "string" }
        }
      }
    ]
  }
  ```
- Procesar tool_calls en la respuesta de Groq → ejecutar la función → enviar resultado como mensaje especial

**2b. Mensajes especiales (rich messages)**
- Nuevo enum `MessageType`: text | service_card | project_card | invoice | order_card | action
- Cada tipo tiene metadata JSONB con campos específicos
- Frontend renderiza componentes especiales según type:
  - `ServiceCardMessage`: nombre, descripción, precio, botón "Me interesa"
  - `InvoiceMessage`: monto, descripción, estado, botón "Pagar" (link Stripe)
  - `OrderCardMessage`: número de orden, estado, servicio, botón "Ver detalles"
  - `ActionMessage`: botones genéricos que el cliente puede clickear
- Los botones envían un WsClientMessage::Action { action_type, payload } que la IA procesa

**2c. Integración Stripe para facturas**
- Usar API de invoices de Stripe (ya tenemos stripe_secret_key)
- Crear customer con email capturado → crear invoice → agregar line item → finalizar → obtener payment link
- Guardar stripe_invoice_id en metadata del mensaje
- Webhook para actualizar estado del invoice (paid/failed)

**2d. Panel de control**
- En SeccionChat, mensajes especiales se renderizan con UI rica
- Staff puede ver: qué servicios mostró la IA, qué facturas generó, qué pedidos creó
- Indicadores visuales por tipo de mensaje en la lista de sesiones

**Test:** Interactuar con chatbot pidiendo "quiero una página web" → verificar que muestra service_card → clickear "Me interesa" → verificar que IA avanza hacia crear pedido. Verificar invoice creada en Stripe dashboard.

---

### Tarea 3: Memoria de usuario + contexto persistente
**Dificultad:** ★★★★☆ (diseño de datos + IA summarization)
**Dependencias:** P-2

**Componentes:**

**3a. Perfil de visitante (visitor_profiles)**
- Al conectar WS: buscar visitor_profile por visitor_id → crear si no existe
- Actualizar: last_seen_at, ip_addresses (append unique), device_fingerprints (append unique), total_sessions++
- Si hay email previo → precargar display_name
- Si visitor_profile tiene user_id → el visitante ya se registró, cargar datos de users

**3b. Captura progresiva de email (no bloqueante)**
- Widget: después de 2-3 intercambios, la IA pregunta natural: "¿Me compartes tu correo para enviarte la información?"
- Tool call: `capture_email(email)` → guarda en visitor_profiles
- Si el email ya existe como user → vincular automáticamente
- Si no existe → crear usuario con password NULL + email_verified=false
- Flujo posterior: al registrarse con ese email → vincular visitor_profile + migrar pedidos

**3c. Resumen de contexto**
- Al cerrar sesión o cada N mensajes: generar resumen de la conversación con modelo pequeño
- Guardar en visitor_profiles.context_summary (append, max 2000 tokens)
- Al abrir nueva sesión: cargar context_summary en el system prompt
- visitor_profiles.preferences: JSON con datos extraídos (industria del cliente, presupuesto mencionado, servicios de interés)

**3d. Contexto por rol (system prompt diferenciado)**
- **Visitante anónimo**: servicios, precios, proyectos de ejemplo, contexto de conversaciones previas (si tiene visitor_profile)
- **Cliente registrado**: + sus pedidos activos, servicios contratados, hosting, historial
- **Admin/Empleado**: contexto de gestión, métricas, pedidos pendientes, clientes
- Cada rol tiene su propio template de system prompt en `src/services/ai_chat.rs`

**Test:** Conversar con chatbot → cerrar → reabrir → verificar que la IA recuerda contexto previo. Dar email → verificar visitor_profile actualizado en BD. Registrarse con ese email → verificar vinculación.

---

### Tarea 4: Sync cross-device/tab
**Dificultad:** ★★★★☆ (cambio arquitectónico en sesiones)
**Dependencias:** Tarea 3 (visitor_profiles)

**Problema actual:** Cada pestaña/dispositivo crea su propia sesión WS. Un mismo visitante puede tener múltiples sesiones activas sin conexión entre ellas.

**Solución:**
- **Identificación**: visitor_id (localStorage UUID) es el identificador primario
- **Sesión única**: `get_or_create_visitor_session()` ya reutiliza sesión activa por visitor_id
- **Broadcast a todas las conexiones**: ChatHub mantiene `DashMap<Uuid, Vec<Sender>>` en vez de `Sender` único por sesión
- Cada nueva conexión WS del mismo visitor_id → se agrega al Vec de senders de la sesión existente
- Mensajes se envían a TODOS los senders de la sesión
- Al cerrar una conexión → remover del Vec, solo cerrar sesión si Vec queda vacío

**Frontend:**
- `useChatWidget`: al reconectar, verificar si hay sesión activa existente en servidor
- BroadcastChannel API (navegador): sincronizar estado entre pestañas del mismo origen
  ```typescript
  const bc = new BroadcastChannel('nakomi-chat');
  bc.onmessage = (e) => { /* sync state */ };
  // Al enviar mensaje → bc.postMessage({type: 'new_message', ...})
  ```
- Si otra pestaña ya tiene WS abierto → pestaña nueva usa BroadcastChannel en vez de abrir otro WS

**Para usuarios registrados (con user_id):**
- visitor_profile con user_id → buscar sesión activa por user_id (no solo visitor_id)
- Dispositivo diferente con mismo login → misma sesión

**Test:** Abrir chat en 2 pestañas → enviar mensaje en una → verificar que aparece en la otra. Abrir en móvil (mismo visitor_id) → verificar continuidad.

---

### Tarea 5: Archivos en chat (imágenes, PDFs, audio)
**Dificultad:** ★★★☆☆ (reutiliza infra de uploads existente)
**Dependencias:** P-2

**Componentes:**

**5a. Upload endpoint para chat**
```
POST /api/chat/sessions/{session_id}/upload
  Multipart: file (max 10MB)
  Response: { message_id, attachment_id, file_name, mime_type, ai_description? }
```
- Reutilizar lógica de `deliverables.rs`: MIME whitelist, magic bytes, sanitize filename
- MIME types permitidos: imágenes (jpeg, png, webp, gif), PDF, audio (mp3, ogg, wav, webm, m4a, flac), documentos (doc, docx)
- Guardar en `uploads/chat/{session_id}/{uuid}.{ext}`
- Crear chat_message con message_type='image'|'file'|'audio' + metadata con attachment info
- Crear chat_attachment vinculado al mensaje

**5b. Procesamiento IA de archivos**
- **Imágenes**: Cambiar modelo a Llama 4 Scout (`meta-llama/llama-4-scout-17b-16e-instruct`) que soporta multimodal
  - Enviar imagen como base64 en el mensaje (max 4MB) o URL si es accesible
  - La IA describe/analiza la imagen en contexto de la conversación
- **Audio**: Groq Whisper STT (`whisper-large-v3-turbo`) → transcribir → enviar transcripción como mensaje de texto + adjunto original
  - Endpoint: `POST https://api.groq.com/openai/v1/audio/transcriptions`
  - Max 25MB (free tier), formatos: flac, mp3, mp4, ogg, wav, webm
  - Guardar transcripción en chat_attachments.ai_description
- **PDFs**: Extraer texto con un crate Rust (pdf-extract o lopdf) → incluir en contexto IA
  - Límite: primeras 3000 palabras del PDF para no saturar context window

**5c. Frontend: UI de upload en ChatWidget**
- Botón de clip (📎) junto al input de texto
- Click → file picker (accept: image/*, audio/*, .pdf, .doc, .docx)
- Preview antes de enviar (imágenes → thumbnail, audio → icono + nombre, PDF → icono + nombre)
- Progress bar durante upload
- En mensajes: imágenes inline (clickeable para ampliar), audio con player embebido, PDF/doc como link descargable
- En SeccionChat (panel): mismos renders + badge de tipo en lista de sesiones

**Test:** Subir imagen JPG → verificar que IA la describe. Subir audio MP3 → verificar transcripción. Subir PDF → verificar que IA resume contenido. Verificar archivos en disco + BD.

---

### Tarea 6: Detección de escalación humana + notificación
**Dificultad:** ★★★☆☆
**Dependencias:** Tarea 1 (tool use)

**Componentes:**

**6a. Detección en IA**
- Tool call `request_human_assistance(reason)` ya definida en Tarea 2
- System prompt instruye a la IA cuándo usar este tool:
  - Presupuestos superiores a $X (configurable)
  - Problemas técnicos específicos que requieren diagnóstico
  - Cliente frustrado o insatisfecho (sentiment detection)
  - Solicitudes fuera del scope de servicios
  - Preguntas legales, contractuales, refunds
  - Después de 3+ intercambios sin resolución clara

**6b. Notificación en tiempo real**
- NotificationHub ya existe en AppState → usar para notificaciones de escalación
- Nuevo tipo de notificación: `ChatEscalation { session_id, visitor_name, reason }`
- En panel: badge rojo en tab "Mensajes" + sonido + toast notification
- Browser Notification API (si permiso concedido): push notification de escritorio
- Email al admin si no hay staff conectado en WS (verificar staff_channel subscribers)

**6c. Flujo de escalación**
- IA envía mensaje al visitante: "Entiendo, voy a contactar a un especialista para ayudarte mejor con esto."
- Internamente: tool call → crea nota en sesión con motivo → notifica staff → status cambia a 'escalated'
- Staff recibe notificación → abre sesión → puede responder inmediatamente o programar callback
- Si no hay staff en 5 min → IA retoma con: "Nuestro equipo recibirá tu consulta y te contactará. ¿Me dejas tu correo para avisarte?"

**Test:** Simular conversación frustrante → verificar que IA detecta y escala. Verificar notificación en panel. Verificar que staff puede tomar la sesión inmediatamente.

---

### Tarea 7: Sin disclosure de IA
**Dificultad:** ★☆☆☆☆
**Dependencias:** Ninguna (se puede hacer en paralelo)

**Cambios:**
- System prompt: agregar instrucción explícita "Nunca menciones que eres una IA, bot, o asistente virtual. Actúa como un miembro del equipo de Nakomi Studio. Tu nombre es [configurable]."
- Frontend ChatWidget: cambiar label "AI" por el nombre configurado
- `SENDER_LABELS` en chat.ts: 'ai' → nombre del agente
- Avatar del chatbot: foto realista o avatar personalizado (configurable en .env o BD)
- sender_type en mensajes sigue siendo 'ai' internamente (para el panel staff), pero el frontend público lo muestra como persona
- En el panel staff: mantener indicador claro de que es IA (icono robot) para que el staff sepa

**Test:** Conversar con chatbot → verificar que nunca se identifica como IA. Verificar que en panel staff sí se distingue.

---

### Tarea 8: No AI disclosure en UI + branding del agente
**Dificultad:** ★☆☆☆☆ (se combina realmente con Tarea 7)

> Esta tarea se fusiona con Tarea 7. No es separada.

---

## Fase II — Atención de clientes registrados

### Tarea 9: Flujo de clientes registrados
**Dificultad:** ★★★★☆
**Dependencias:** Tarea 3 (memoria), Tarea 4 (sync)

**Componentes:**

**9a. Detección de usuario autenticado en chat**
- Si el usuario tiene JWT válido → usar user_id en vez de visitor_id
- ChatWidget en /panel: oculto (ya existe SeccionChat)
- ChatWidget en sitio público: si hay token → pasar como param WS `?token=JWT&visitor_id=X`
- Backend: si JWT válido → vincular sesión a user_id + cargar context de usuario

**9b. Contexto de cliente registrado para IA**
- Consultar: pedidos activos, fases, pagos, servicios contratados, hosting
- System prompt incluye:
  ```
  Cliente: {display_name} ({email})
  Pedidos activos: #{order_number} - {service} ({status}) [x{n}]
  Hosting: {plan} - {domain} ({status})
  Último contacto: {fecha}
  Resumen previo: {context_summary}
  ```
- IA puede responder preguntas sobre estado de pedidos, hosting, facturas

**9c. Manejo de reportes/incidencias**
- Tool call: `create_support_ticket(category, description, priority)`
- Categorías: hosting_issue | order_issue | billing_issue | general
- Se crea como nota en la sesión + notificación al staff correspondiente
- Si es hosting → incluir datos del VPS (status, uptime) en contexto

**Test:** Login como cliente → abrir chat → preguntar "¿cuál es el estado de mi pedido?" → verificar respuesta correcta con datos reales. Reportar problema → verificar ticket creado.

---

### Tarea 10: IA intermediaria en chats de pedidos
**Dificultad:** ★★★★☆
**Dependencias:** Tarea 2, Tarea 9

**Problema:** Los chats de pedidos son directos entre cliente y empleado. Pero a veces el empleado quiere que la IA atienda al cliente (porque fastidia mucho, por ejemplo).

**Componentes:**

**10a. Toggle IA por pedido**
- Columna `orders.ai_intermediary_enabled` (ya en migración P-2)
- En panel de detalles del pedido: toggle "IA intermediaria" (solo admin/empleado asignado)
- En SeccionChat: indicador visual en sesiones de orden con IA intermediaria activa
- Configuración global: admin puede ver todos los pedidos con IA activa

**10b. Contexto completo del pedido para IA intermediaria**
- System prompt especial:
  ```
  Estás atendiendo como intermediario en el pedido #{order_number}.
  Servicio: {title} - Plan: {plan}
  Estado: {status} - Fase actual: {current_phase}/{total_phases}
  Notas del cliente: {client_notes}
  Notas internas: {internal_notes}
  Historial de fases: [fase 1: completada, fase 2: en progreso...]
  Empleado asignado: {employee_name}
  
  IMPORTANTE: Eres un intermediario. Tu rol es:
  1. Responder preguntas del cliente sobre el estado del pedido
  2. Recopilar solicitudes y cambios pedidos por el cliente
  3. Generar resúmenes para el equipo
  4. Escalar al empleado si es algo que requiere acción humana
  No tomes decisiones sobre el trabajo — solo comunica y documenta.
  ```

**10c. Resumen automático del pedido**
- Después de cada interacción significativa (>5 mensajes): generar resumen
- Guardar en `orders.ai_summary` — visible en detalles del pedido
- Resumen incluye: solicitudes del cliente, cambios pedidos, estado emocional, acciones pendientes
- Se actualiza (no acumula) con cada resumen nuevo

**10d. Distinción de mensajes en chat de pedido**
- El cliente DEBE poder distinguir entre mensajes del empleado y de la IA intermediaria
- sender_type: 'ai_intermediary' (nuevo) vs 'employee'
- Frontend: avatar diferente, label "Asistente de [NombreEmpleado]"
- El cliente sabe que está hablando con un asistente, no con el empleado directamente

**Test:** Crear pedido → activar IA intermediaria → conversar como cliente → verificar que IA tiene contexto del pedido. Verificar resumen generado. Verificar que cliente distingue mensajes IA vs empleado.

---

## Resumen de tareas por orden de ejecución

| # | Tarea | Dificultad | Dependencias | Test clave |
|---|-------|------------|--------------|------------|
| P-1 | Refactorizar handler chat | ★★☆☆☆ | Ninguna | cargo check + clippy clean |
| P-2 | Migración BD nuevas features | ★★☆☆☆ | Ninguna | sqlx prepare + tests |
| 1 | Anti-spam + timing inteligente | ★★★★★ | P-1 | 3 msgs rápidos → 1 respuesta IA |
| 2 | Generación pedidos + facturas | ★★★★★ | P-1, P-2 | Pedir servicio → invoice Stripe |
| 3 | Memoria usuario + contexto | ★★★★☆ | P-2 | Reabrir chat → IA recuerda |
| 4 | Sync cross-device | ★★★★☆ | T-3 | 2 pestañas → msgs sincronizados |
| 5 | Archivos en chat | ★★★☆☆ | P-2 | Upload imagen → IA describe |
| 6 | Escalación humana | ★★★☆☆ | T-1 (tools) | Conversación difícil → notificación |
| 7 | Sin disclosure IA | ★☆☆☆☆ | Ninguna | Chatbot nunca dice ser IA |
| 9 | Clientes registrados | ★★★★☆ | T-3, T-4 | Login → preguntar estado pedido |
| 10 | IA intermediaria pedidos | ★★★★☆ | T-2, T-9 | Toggle IA → contexto orden completo |

**Estimación total:** ~15 tareas de roadmap (P-1, P-2, T1a-c, T2a-d, T3a-d, T4, T5a-c, T6, T7, T9, T10)

---

## Notas técnicas

### Modelos Groq a usar
| Modelo | Uso | Costo |
|--------|-----|-------|
| `llama-3.3-70b-versatile` | Chat principal (texto) | Token-based |
| `meta-llama/llama-4-scout-17b-16e-instruct` | Multimodal (imágenes) | Token-based |
| `llama-3.1-8b-instant` | Clasificador relevancia (anti-spam) | Muy bajo |
| `whisper-large-v3-turbo` | STT audio → texto | $0.04/hr |

### Capacidades Groq confirmadas
- **Vision**: Llama 4 Scout soporta imágenes via URL (max 20MB) o base64 (max 4MB), max 5 imgs/request
- **STT**: Whisper soporta flac/mp3/mp4/ogg/wav/webm, max 25MB free tier
- **Tool Use**: Soportado en llama-3.3-70b y Llama 4 Scout
- **JSON Mode**: Soportado en todos los modelos
- **Content Moderation**: Disponible como feature core (meta-llama/llama-guard-4-12b)

### Archivos que se van a crear/modificar
**Backend (crear):**
- `src/handlers/chat/mod.rs`
- `src/handlers/chat/ws_visitor.rs`
- `src/handlers/chat/ws_staff.rs`
- `src/handlers/chat/rest.rs`
- `src/handlers/chat/types.rs`
- `src/services/ai_tools.rs` — tool execution engine
- `src/services/chat_spam.rs` — rate limiting + relevance check
- `src/services/chat_timing.rs` — intelligent response timing state machine
- `src/models/visitor_profile.rs`
- `src/models/chat_attachment.rs`
- `src/repositories/visitor_profile.rs`
- `migrations/YYYYMMDD_chat_v2.up.sql`

**Backend (modificar):**
- `src/handlers/chat.rs` → split into module
- `src/services/ai_chat.rs` — tool use, multimodal, role-specific prompts
- `src/services/chat.rs` — multi-connection sync, timing integration
- `src/models/chat.rs` — MessageType enum, metadata
- `src/repositories/chat.rs` — new queries for attachments, message types
- `src/lib.rs` — new config fields en AppState

**Frontend (crear):**
- `frontend/src/components/chat/messages/ServiceCardMessage.tsx`
- `frontend/src/components/chat/messages/InvoiceMessage.tsx`
- `frontend/src/components/chat/messages/OrderCardMessage.tsx`
- `frontend/src/components/chat/messages/FileMessage.tsx`
- `frontend/src/components/chat/messages/AudioMessage.tsx`
- `frontend/src/components/chat/ChatUploadButton.tsx`

**Frontend (modificar):**
- `frontend/src/api/chat.ts` — new types, upload endpoints
- `frontend/src/hooks/useChatWidget.ts` — file upload, sync
- `frontend/src/hooks/useChatWs.ts` — new message types
- `frontend/src/components/chat/ChatWidget.tsx` — upload UI, rich messages
- `frontend/src/components/panel/SeccionChat.tsx` — rich messages, escalation indicators
- `frontend/src/stores/chatStore.ts` — cross-tab sync

### Evaluación GLORY-RS
La mayoría de estas features son específicas de Nakomi (chatbot de agencia con contexto de servicios/pedidos). Sin embargo, algunos componentes son candidatos para glory-rs:
- **Anti-spam state machine** → abstracción genérica de rate limiting + timing en WS
- **Rich message rendering** → componentes de mensajes ricos reutilizables
- **File upload en chat** → utilidad genérica de chat + archivos
- Evaluar al completar cada tarea si mover a glory-rs.
