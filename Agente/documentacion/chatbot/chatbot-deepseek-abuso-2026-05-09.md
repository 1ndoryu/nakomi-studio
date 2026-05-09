# Chatbot: DeepSeek primario y control de abuso — 2026-05-09

## Proveedor AI

DeepSeek queda como primera alternativa del chatbot usando formato OpenAI-compatible:

- `DEEPSEEK_API_KEY`: secreto real, solo en entornos locales/producción.
- `DEEPSEEK_MODEL`: por defecto `deepseek-v4-flash`.
- `DEEPSEEK_API_URL`: por defecto `https://api.deepseek.com/chat/completions`.

La cadena de fallback queda así:

1. DeepSeek (`deepseek-v4-flash`).
2. Groq con la cadena de modelos existente y rotación de keys.
3. Gemini como último fallback si está configurado.

DeepSeek v4 flash soporta tool calls y JSON output según la documentación oficial, por eso puede ejecutar las tools actuales del chatbot.

## Superficies de abuso revisadas

- Mensajes WebSocket de visitante.
- Mensajes REST de cliente autenticado.
- Relevancia/off-topic como filtro opt-in.
- Resumen de sesión al desconectar.
- Resumen de órdenes.
- Tool-call loop de la respuesta principal.

## Guardrails activos

- 10 mensajes/minuto por visitante.
- 30 mensajes/minuto por IP.
- 10 conexiones WebSocket concurrentes por IP.
- 2000 caracteres máximos por mensaje entrante.
- 6000 caracteres máximos combinados antes de una respuesta AI.
- 24k tokens estimados por hora por visitante.
- 80k tokens estimados por hora por IP.
- Relevancia desactivada por defecto (`AI_RELEVANCE_ENABLED=false`) por falsos positivos; se mantiene como opt-in.
- Prompt reforzado para no resolver tareas generalistas ni cambios de rol.

## Política de fallo

Si DeepSeek devuelve error HTTP, rate limit o error de red, el backend intenta Groq. Si Groq tampoco responde y Gemini está configurado, intenta Gemini. Si todos fallan, el chat escala a atención humana.

## Secretos

No guardar claves en markdown, roadmap, commits ni logs. El valor real de `DEEPSEEK_API_KEY` se sube a producción con `coolify-manager-rs sync-env` usando un archivo temporal fuera del repo o `.env` ignorado por git.

En esta instancia de Coolify, la escritura por `sync-env` necesitó compatibilidad con el contrato real `key/value`; las variables quedaron verificadas en el servicio `studio` sin imprimir valores: `DEEPSEEK_API_KEY`, `DEEPSEEK_MODEL`, `DEEPSEEK_API_URL` y `AI_RELEVANCE_ENABLED=false`.

## Continuidad e identidad [095A-16, 095A-17]

- El widget persiste `visitor_id`, `chat_session_id` y hasta 100 mensajes por sesión en `localStorage` para evitar parpadeo vacío al recargar.
- Una desconexión normal del WebSocket ya no cierra la sesión en backend; solo marca offline y libera timing en memoria.
- `/reset` es el flujo explícito para borrar conversación: limpia storage, UI, sesión activa y deja que backend borre mensajes.
- Si el WebSocket trae JWT, `chat_sessions.user_id` y `visitor_profiles.user_id/email/display_name` quedan vinculados al usuario autenticado.
- El prompt distingue rol real y rol operativo para admin, employee y client. Un admin logueado ya no debe tratarse como lead anónimo.

## Identidad, roles y tools protegidas [095A-20]

El chatbot ya no decide permisos solo por prompt. Cada tool sensible recibe `ToolAuthContext` desde claims firmados del JWT:

- `user_id`: sujeto autenticado de la sesión.
- `role`: rol real del usuario.
- `effective_role`: rol operativo actual, incluyendo cambios de rol/impersonación.
- `impersonator`: admin origen si la sesión está impersonada.

El WebSocket visitor y el chat REST propagan ese contexto al prompt y al dispatcher de tools. El prompt solo explica el comportamiento esperado; la autorización real vive en backend.

Tools protegidas y alcance:

- `list_my_orders`: cliente ve pedidos propios; empleado ve pedidos asignados; admin efectivo ve pedidos recientes globales.
- `list_my_payments`: mismo alcance que pedidos, sin exponer secretos de Stripe.
- `list_my_reports`: cliente por sus pedidos; empleado por asignados; admin efectivo global.
- `create_order_report`: requiere sesión y valida que el pedido sea visible antes de crear el reporte.
- `admin_operational_summary`: solo admin efectivo; clientes/empleados reciben `forbidden`.
- `list_my_hostings` y `create_hosting_checkout`: requieren login; el checkout usa el `user_id` firmado, no texto del usuario.
- `create_support_ticket`: requiere login y registra metadata mínima de usuario/rol sin loguear la descripción sensible.

Respuestas estándar de tools:

- `ok`: operación o consulta completada.
- `requires_login`: falta sesión autenticada; el bot debe pedir login/registro.
- `forbidden`: sesión válida pero sin permisos; el bot debe explicar el límite y ofrecer escalar.
- `not_found`: recurso no existe o no es visible para esa cuenta.
- `error`: fallo operativo; se loguea sin argumentos sensibles.

El frontend separa `visitor_id`, `chat_session_id` y mensajes por owner local (`anonymous` o `user:{id}:role:{role}:effective:{effectiveRole}:{direct|impersonating}`). Al cambiar de cuenta, logout/login o rol efectivo, limpia el estado local y reconecta con JWT actual para evitar mezclar conversaciones entre usuarios en el mismo navegador. `/reset` sigue siendo el flujo explícito para rotar la conversación de la identidad actual.

Escenarios manuales mínimos:

- Visitante anónimo pregunta por pedidos: tool responde `requires_login` y el bot pide iniciar sesión.
- Cliente pregunta por pedidos/pagos/hosting: tools consultan solo recursos vinculados a su `user_id`.
- Cliente intenta consultar pedido ajeno por número: `create_order_report` o consultas deben responder `forbidden`/sin datos.
- Empleado pregunta por pedidos: solo ve asignados.
- Admin efectivo pregunta por operación global: `admin_operational_summary` responde agregados.
- Admin impersonando cliente: se aplica el rol operativo de la sesión; no se revelan datos globales salvo que el JWT mantenga admin efectivo.
