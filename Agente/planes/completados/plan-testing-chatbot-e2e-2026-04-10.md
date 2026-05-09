# Plan de Testing E2E del Chatbot — 084A-33

## Prioridad: pruebas manuales primero (smoke tests), luego unit tests

---

## Smoke Tests Manuales (máxima prioridad)

### 1. Visitante fresco (anónimo)
- [ ] Abrir widget → burbuja "Chat" visible
- [ ] Click burbuja → widget abre con historial vacío
- [ ] Escribir "Hola" → AI responde en 4-6s
- [ ] Refresh → historial persiste

### 2. Tool execution
- [ ] "Muéstrame los servicios" → AI llama list_services → responde con lista
- [ ] "¿Cuál es el precio de Diseño Web?" → AI llama show_service → service_card renderiza
- [ ] "Mi email es test@example.com, facturame $500 por diseño web" → capture_email + create_invoice → invoice card con "Pagar ahora"

### 3. Rich messages
- [ ] Service card: título, descripción, precio visibles
- [ ] Invoice card: monto, descripción, botón pago
- [ ] Botón "Pagar ahora" → abre Stripe hosted invoice

### 4. Multi-tab sync
- [ ] Abrir chat en 2 tabs del mismo browser
- [ ] Enviar mensaje en tab 1 → aparece en tab 2
- [ ] Cerrar tab 1 → tab 2 sigue funcionando

### 5. Escalación
- [ ] "Necesito hablar con un humano" → AI escala → notificación staff
- [ ] Staff entra a sesión → ve historial
- [ ] Staff responde → visitante recibe en tiempo real

### 6. Rate limiting
- [ ] Enviar 15 msgs en 30s → msg 11+ trigger warnings
- [ ] Nivel 2 → mute 30s

### 7. Visitante recurrente
- [ ] Guardar email via capture_email
- [ ] Refrescar página, reabrir widget
- [ ] AI no pide email de nuevo (prompt dice "ya capturado")

### 8. Fallback de modelos
- [ ] Verificar en logs la rotación de keys (key hints diferentes)
- [ ] Si rate limit, logs muestran "todas las keys con rate limit, probando siguiente modelo"

---

## Unit Tests Rust (cargo test)

### ai_chat.rs
- `sanitize_for_prompt`: elimina INSTRUCTION, SYSTEM, IGNORE; trunca a max_len; preserva newlines
- `estimate_tokens`: ~chars/4+1
- `build_context_messages`: budget 64k; truncamiento con tail-selection; mensajes recientes intactos
- `parse_escalation`: detecta [ESCALATE], lo elimina del texto
- `model_fallback_chain`: maverick primero, modelo primario al inicio, sin duplicados

### ai_tools.rs
- `tool_definitions`: retorna JSON válido con 6 herramientas
- Cada tool con name, parameters, description

### chat_timing.rs
- Rate limit: 10 msgs/min
- Cooldown escalation: warning → mute 30s
- Buffer: acumulación de mensajes durante WAITING
- Timing: IDLE → WAITING(4s) → LISTENING(8s) → RESPONDING

---

## Verificación en logs de producción (post-deploy)

- [ ] Logs muestran "AI OK: modelo=..., key=..., intento=..."
- [ ] Cada request usa key diferente (round-robin)
- [ ] Tool calls logeados: "AI tool call iteration X, Y tools executed"
- [ ] Errores de Stripe logeados con contexto
- [ ] Rate limit warnings logeados con visitor_id

---

## Estado: EN PROGRESO
Creado: 2026-04-10
