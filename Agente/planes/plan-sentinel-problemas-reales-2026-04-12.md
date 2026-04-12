# Plan: Problemas Reales del Sentinel Report (sin ejecutar)

> Generado: 2026-04-12. Estos son los problemas REALES (no falsos positivos) del sentinel-report.
> Organizados por prioridad de impacto. El plan NO se ejecuta aún — queda como referencia para futuras tareas.

## Resumen: ~126 violaciones reales de las 531 originales (tras eliminar ~405 falsos positivos)

---

## R1 — handler-accede-bd-rs (ALTA PRIORIDAD — ~25 violaciones)

**Problema:** Queries SQL directas en handlers violan DIP (Dependency Inversion). Los handlers deben delegar a repositorios.

**Archivos afectados:**
- `src/handlers/orders.rs` — 10 queries directas (líneas 55, 73, 108, 117, 325, 465, 507, 521, 600, 756, 814)
- `src/handlers/wallet.rs` — 7 queries directas (170, 295, 329, 347, 375, 396, 514)
- `src/handlers/seo.rs` — 2 queries directas (41, 53)
- `src/handlers/chat/rest_messages.rs` — 2 queries directas (103, 146)
- `src/handlers/problems.rs` — 3 queries directas (81, 99, 288)
- `src/handlers/deliverables.rs` — 1 query directa (211)
- `src/handlers/payments.rs` — 2 queries directas (125, 188)
- `src/handlers/reviews.rs` — 1 query directa (124)
- `src/handlers/refunds.rs` — 1 query directa (89)
- `src/handlers/assignment.rs` — 1 query directa (61)
- `src/handlers/chat/rest.rs` — 1 query directa (88)
- `src/handlers/chat/mod.rs` — 1 query directa (79)

**Estrategia:** Mover queries a repositorios correspondientes. Crear métodos de repositorio que encapsulen cada query. El handler solo llama `repo::metodo(pool, params)`.

**Orden sugerido:** orders.rs (el más crítico por volumen), luego wallet.rs, luego el resto.

---

## R2 — sqlx-query-sin-macro / sqlx-query-as-sin-macro (~12 violaciones)

**Problema:** `sqlx::query()` y `sqlx::query_as()` sin macros de verificación en compilación.

**Archivos afectados:**
- `src/handlers/orders.rs` — 6 ocurrencias de `sqlx::query()` sin macro
- `src/handlers/assignment.rs` — 1
- `src/repositories/wallet.rs` — 1 `query_as()` sin macro
- `src/handlers/seo.rs` — 2 `query_as()` sin macro
- `src/handlers/chat/rest_messages.rs` — 1 `query_as()` sin macro
- `src/middleware/prerender.rs` — 2 `query_as()` sin macro

**Nota:** El proyecto usa `SQLX_OFFLINE=true`. Migrar a `query!`/`query_as!` requiere tener `.sqlx/` actualizado. Muchas de estas coinciden con R1 (se arreglan al mover a repos que ya usan macros).

---

## R3 — limite-lineas (~10 violaciones, archivos grandes)

**Archivos que realmente exceden límites:**
- `src/services/ai_chat.rs` — 991 líneas (límite 500)
- `src/handlers/hosting.rs` — 1190 líneas (límite 500)
- `src/services/chat_timing.rs` — 781 líneas (límite 500)
- `src/handlers/orders.rs` — 701 líneas (límite 500)
- `src/services/order.rs` — 624 líneas (límite 500)
- `src/services/ai_tools.rs` — 569 líneas (límite 500)
- `src/handlers/wallet.rs` — 566 líneas (límite 500)
- `src/repositories/order.rs` — 842 líneas (límite 400 para repo)
- `src/repositories/chat.rs` — 535 líneas (límite 400 para repo)
- `src/services/coolify.rs` — 508 líneas (límite 500)
- `src/repositories/wallet.rs` — 405 líneas (límite 400 para repo)
- `src/models/order.rs` — 312 líneas (límite 300 para model)
- `src/models/hosting.rs` — 332 líneas (límite 300 para model)

**Estrategia:** Dividir por subdominios. Ejemplo: `ai_chat.rs` → `ai_chat/mod.rs`, `ai_chat/generation.rs`, `ai_chat/tool_calls.rs`, etc. Empezar por los que más excedan el límite.

---

## R4 — parametros-excesivos-rs (~30 hints)

**Problema:** Funciones con >5 parámetros. Agrupar en structs.

**Los peores casos:**
- `chat_timing.rs` `generate_ai_response()` — 15 params
- `chat_timing.rs` `session_timing_loop()` — 14 params
- `chat/rest_upload.rs` `spawn_file_ai_processing()` — 10 params
- `deliverables.rs` `process_multipart_files()` — 9 params

**Estrategia:** Crear structs de contexto (`AiChatContext`, `SessionTimingContext`, etc.) que agrupen parámetros relacionados. Empezar por funciones con >8 params.

---

## R5 — funcion-larga-rs (~4 violaciones)

- `chat/rest_messages.rs` `send_message()` — 162 líneas (max 100)
- `services/hosting_stripe.rs` `on_checkout_completed()` — 136 líneas (max 100)
- `services/seed.rs` `create_seed_hosting_events()` — 167 líneas (max 100)
- `chat/ws_visitor.rs` `handle_visitor_ws()` — 121 líneas (max 100)

**Estrategia:** Extraer subfunciones auxiliares con nombres descriptivos.

---

## R6 — unwrap-produccion-rs (1 violación)

- `src/services/ai_chat.rs` línea 254 — `.unwrap()` en producción.
- **Fix:** Reemplazar con `?` o `.unwrap_or_default()` según contexto.

---

## R7 — componente-artesanal (2 violaciones)

- `SidebarPanel.tsx` — outside-click handler manual
- `IconPickerEnlace.tsx` — outside-click handler manual
- **Fix:** Migrar a `<MenuContextual>` si aplica, o extraer hook `useClickOutside`.

---

## R8 — html-nativo-en-vez-de-componente (3 violaciones)

- `UploadImage.tsx` línea 92 — `<button>` → `<Boton>`
- `ListaProyectos.tsx` línea 117 — `<button>` → `<Boton>`
- `ListaBlog.tsx` línea 115 — `<button>` → `<Boton>`
- **Fix:** Reemplazar `<button>` nativo con `<Boton variante="...">`.

---

## R9 — css-adhoc-button-style (1 violación)

- `IconPickerEnlace.css` línea 8 — CSS de botón fuera de Button.css.
- **Fix:** Evaluar si debe usar `<Boton>` o mover estilos a variante.

---

## R10 — objeto-mutable-exportado (1 hint)

- `IconPickerEnlace.tsx` línea 27 — `export const TIPOS_ENLACE` exporta array mutable.
- **Fix:** Agregar `as const` o `Object.freeze()`.

---

## Orden sugerido de ejecución

1. **R1+R2** (handler-accede-bd + queries sin macro) — se solapan, arreglar juntos
2. **R3** (archivos grandes) — dividir módulos
3. **R4** (params excesivos) — structs de contexto
4. **R5** (funciones largas) — extraer auxiliares
5. **R6** (unwrap) — fix trivial
6. **R7-R10** (UI minor) — fixes pequeños
