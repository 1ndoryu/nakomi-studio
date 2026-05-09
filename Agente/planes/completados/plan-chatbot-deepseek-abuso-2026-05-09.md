# Plan: DeepSeek primario y control de abuso del chatbot — 095A-10+095A-11

> Creado: 2026-05-09
> Estado: Completado
> Alcance: proveedor AI del chatbot, protección de coste y uso fuera de propósito.

## Diagnóstico

El chatbot ya tenía rate limiting por cantidad de mensajes y conexiones, pero el proveedor primario era Groq y varias llamadas auxiliares seguían pegadas a `config.api_url`. Esto dejaba tres riesgos: no usar DeepSeek como primera alternativa, gastar API en mensajes largos dentro del límite por minuto, y permitir uso generalista sin límites de coste por tokens.

## Decisiones

1. DeepSeek será proveedor primario con `deepseek-v4-flash` y endpoint OpenAI-compatible.
2. Groq queda como fallback; Gemini permanece como último respaldo si está configurado.
3. El filtro de relevancia queda disponible, pero desactivado por defecto por falsos positivos (`AI_RELEVANCE_ENABLED=false`).
4. El chatbot tendrá presupuesto horario en memoria por visitante y por IP, además del límite de mensajes.
5. No se guardan secretos en git; producción recibe `DEEPSEEK_API_KEY` vía Coolify env.

## Fases Completadas

### Fase 1 — Proveedor
- Añadidos `DEEPSEEK_API_KEY`, `DEEPSEEK_MODEL`, `DEEPSEEK_API_URL` a la configuración.
- `ai_providers` quedó como cadena DeepSeek → Groq → Gemini.
- Respuesta principal, resumen y llamadas terse pasan por el proveedor común.

### Fase 2 — Anti-abuso
- Presupuesto de tokens estimados por visitante/IP.
- Límite de texto combinado antes de llamar a IA.
- Relevancia opt-in; el control de abuso principal queda en rate limits, presupuesto horario y prompt de propósito.
- Refuerzo de prompt para no resolver tareas generalistas.

### Fase 3 — Producción
- Variables documentadas sin secretos.
- Producción verificada con `DEEPSEEK_API_KEY`, `DEEPSEEK_MODEL=deepseek-v4-flash`, `DEEPSEEK_API_URL=https://api.deepseek.com/chat/completions` y `AI_RELEVANCE_ENABLED=false`.
- `coolify-manager-rs sync-env` quedó corregido para usar `key/value` en env vars de servicio.

## Criterio de cierre

- DeepSeek se intenta antes que Groq.
- Si DeepSeek falla, Groq sigue operativo.
- Hay tests de configuración, proveedor, presupuesto anti-abuso y relevancia opt-in.
- Producción tiene las variables necesarias sin exponer la clave en git.