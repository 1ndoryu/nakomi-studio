# Plan de revisión — Chatbot identidad, roles y capacidades

**Fecha:** 2026-05-09
**Tarea origen:** 095A-17
**Cierre:** 095A-20
**Estado:** Completado

## Objetivo

Auditar de forma extensa cómo el chatbot reconoce visitantes, usuarios logueados, clientes, empleados y administradores, y cerrar los huecos para que pueda asistir con pedidos, pagos, hosting, reportes y consultas según permisos reales.

## Diagnóstico inicial

- El widget envía JWT por WebSocket cuando el frontend tiene sesión iniciada.
- El backend detectaba `user_id` desde el token, pero no propagaba rol real/effective_role a tools.
- Antes de 095A-16, la sesión `chat_sessions` y el `visitor_profile` no quedaban vinculados de forma persistente al usuario autenticado.
- El cierre normal del WebSocket cerraba la sesión al recargar, lo que rompía recuperación de historial y continuidad.
- El prompt base seguía dando mucha prioridad a capturar nombre/email; para usuarios registrados debe prevalecer el contexto de cuenta.

## Resultado

### Fase 1 — Continuidad básica

- Sesión activa al recargar/cerrar pestaña sin marcar cierre explícito.
- Historial visual persistido en `localStorage` por sesión.
- `/reset` limpia UI/storage de inmediato y deja que backend borre la sesión activa.
- Producción verificada con DeepSeek primario y `AI_RELEVANCE_ENABLED=false`.

### Fase 2 — Identidad autenticada

- El WebSocket visitor extrae claims completos del JWT y construye `ToolAuthContext`.
- `chat_sessions.user_id` y `visitor_profiles.user_id/email/display_name` se vinculan al usuario autenticado.
- El frontend separa conversaciones por owner local: anónimo, usuario, rol real, rol operativo e impersonación.
- Al cambiar de cuenta/rol efectivo, se rota `visitor_id`, `chat_session_id` y mensajes locales para evitar mezcla entre usuarios.

### Fase 3 — Roles y permisos

- `ToolAuthContext` incluye `user_id`, `role`, `effective_role` e `impersonator` firmados.
- Las tools no confían en texto del usuario para permisos.
- Cliente: solo datos propios.
- Empleado: pedidos/reportes/pagos asociados a pedidos asignados.
- Admin efectivo: consultas globales protegidas.
- Impersonación: se aplica el rol operativo del JWT; no se concede acceso global por prompt.

### Fase 4 — Capacidades por dominio

- Pedidos: `list_my_orders` devuelve estado, fases, entregables, empleado asignado y reportes abiertos visibles.
- Pagos: `list_my_payments` devuelve pagos/facturas visibles sin secretos de Stripe.
- Hosting: `list_my_hostings` exige login y suma eventos recientes.
- Reportes: `list_my_reports` consulta visibles y `create_order_report` valida acceso antes de crear.
- Admin: `admin_operational_summary` devuelve agregados operativos solo para admin efectivo.

### Fase 5 — Contratos de tools

- Se estandarizaron respuestas `ok`, `requires_login`, `forbidden`, `not_found` y `error`.
- `create_support_ticket` también exige login y evita loguear la descripción sensible.
- Se añadieron tests unitarios para tools protegidas y conteo/estructura de definitions.

### Fase 6 — UX y observabilidad

- Logs seguros para tool calls: sesión, nombre de tool, status, user_id/rol efectivo; sin argumentos.
- El prompt explica cómo reaccionar a `requires_login`/`forbidden` sin inventar datos.
- Documentación actualizada en `Agente/documentacion/chatbot/chatbot-deepseek-abuso-2026-05-09.md`.

## Validación

- `cargo fmt --check`
- `cargo check`
- `cargo clippy -- -D warnings`
- `cargo test` — 123 tests passed
- `npx tsc --noEmit`
- `npm run build`

## Gotchas

- Los permisos de cuenta deben vivir en backend; el prompt solo orienta estilo y flujo.
- `visitor_id` persistido sin owner key mezcla usuarios cuando hay logout/login en el mismo navegador.
- No registrar argumentos de tools: pueden contener descripción de problemas, email o datos de pago.