## [096A-7] Regla Sentinel: tokio::spawn sin timeout ni tracking

**Fecha:** 2026-06-10
**Estado:** Pendiente de implementar en code-sentinel
**Severidad:** warning
**Categoría:** RustPatrones

### Problema

`tokio::spawn` sin `tokio::time::timeout` wrapping crea tareas huérfanas que:
- Pueden vivir indefinidamente (90s+ reteniendo conexiones DB)
- No aparecen en CLOSE_WAIT ni métricas de red
- Saturan el pool de conexiones silenciosamente
- Son invisibles en tests y desarrollo (solo se manifiesta con tráfico real concurrente)

### Patrón incorrecto (detectar)

```rust
// ❌ Spawn sin timeout ni tracking
tokio::spawn(some_long_running_task(args));

// ❌ Spawn con await sin timeout
tokio::spawn(async move {
    let result = some_io_operation().await;  // puede colgarse
    process(result).await;
});
```

### Patrón correcto (no reportar)

```rust
// ✅ Spawn con timeout explícito
tokio::spawn(async move {
    let _ = tokio::time::timeout(Duration::from_secs(60), some_task()).await;
});

// ✅ Spawn con tracking + timeout
let counter = active_tasks.clone();
tokio::spawn(async move {
    counter.fetch_add(1, Ordering::Relaxed);
    let _ = tokio::time::timeout(Duration::from_secs(120), work()).await;
    counter.fetch_sub(1, Ordering::Relaxed);
});

// ✅ Spawn rápido sin I/O (no necesita timeout)
tokio::spawn(async move {
    let _ = tx.send(event).await;  // channel send, no retiene recursos
});
```

### Implementación en code-sentinel

**Archivos a modificar:**

1. `src/config/ruleRegistry.ts` — registrar:
```typescript
{ id: 'spawn-sin-timeout-rs', nombre: 'tokio::spawn sin timeout', severidadDefault: 'warning', categoria: CategoriaRegla.RustPatrones }
```

2. `src/analyzers/rustAnalyzer.ts` — agregar función `detectarSpawnSinTimeout`:
   - Regex para `tokio::spawn(`
   - Parsear bloque `{...}` siguiente
   - Buscar `tokio::time::timeout` dentro del bloque
   - Si no hay timeout Y hay `.await` con I/O (DB, HTTP, file) → reportar
   - Excluir tests, ejemplos, y líneas con `sentinel-disable-next-line spawn-sin-timeout-rs`

**Heurística anti-falsos-positivos:**
- Solo reportar si el bloque contiene `.await` (sin await, el spawn es sync y no se cuelga)
- Excluir spawns que solo hacen `tx.send()` o `drop()` (operaciones instantáneas)
- Excluir spawns dentro de `#[cfg(test)]`

### Supresión

```rust
// sentinel-disable-next-line spawn-sin-timeout-rs: razón
tokio::spawn(something_quick());
```

### Causa raíz del incidente 096A

El `session_timing_loop` se hacía con `tokio::spawn` sin timeout global. Cada visitor en chat creaba un loop que:
- Retenía 1 conexión DB del pool (max=10)
- Hacía requests HTTP a Groq/Gemini (hasta 90s cada una)
- No tenía límite de concurrencia

Con ~10 visitors concurrentes, el pool se agotaba y el servidor colgaba. Los 6 intentos de fix anteriores trataron CLOSE_WAIT como causa raíz, pero el problema real era starvation del pool DB por tareas huérfanas.
