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
