# Auditoría de Rendimiento — Code Sentinel + VarSense

**Fecha:** 2026-04-12  
**Tarea:** 124A-AUDIT1

## Resumen

Auditoría profunda de memory leaks y rendimiento en ambas extensiones VS Code.

## Issues Encontrados y Corregidos

### VarSense — 4 CRITICAL, 2 HIGH

| Issue | Archivo | Severidad | Fix |
|-------|---------|-----------|-----|
| Cache resolución ilimitado | variableResolver.ts | CRITICAL | LRU con MAX=200 |
| Token extraction explota RAM | classScanner.ts | CRITICAL | Solo class/className attrs + MAX_TOKENS=10000 |
| Regex recreadas por keystroke | diagnosticProvider.ts | CRITICAL | Constantes de módulo |
| Event storm onVariablesChange | diagnosticProvider.ts | MEDIUM | Debounce 1000ms |
| Promise.all sin límite | variableScanner.ts | HIGH | Lotes de 10 |
| Búsqueda similares O(n) → sort | hoverProvider.ts | MEDIUM | Top-3 con threshold dinámico |
| Campo muerto _mapaOffsets | cssParser.ts | LOW | Eliminado |

### Code Sentinel — 2 HIGH, 2 MEDIUM

| Issue | Archivo | Severidad | Fix |
|-------|---------|-----------|-----|
| editoresAnalizados crece sin límite | diagnosticProvider.ts | HIGH | .clear() en limpiarDiagnosticos |
| Webview panels duplicados | extension.ts | MEDIUM | Singleton + onDidDispose |
| Concatenación O(n²) en reporte | reportGenerator.ts | MEDIUM | array.join() |
| Regex imports por línea | staticCodeRules.ts | LOW-MEDIUM | Constantes módulo |

## Issues Identificados No Corregidos (bajo riesgo)

- **cacheService.ts**: No implementa cleanup TTL activo (solo LRU pasivo). Bajo impacto porque MAX_ENTRADAS=100.
- **externalToolsAnalyzer.ts**: maxBuffer de 5MB por proceso. Aceptable para 3 herramientas secuenciales.
- **Glory indexers**: Caches module-scoped sin cleanup explícito. Bajo impacto porque VS Code dispone subscriptions automáticamente.
- **completionProvider.ts**: CancellationToken no usado. VS Code maneja cancelación internamente para completions.
- **cssParser.ts**: Comment-strip en cada parse. No cache porque CssParser se crea por documento y el variableScanner cachea resultados.

## Lecciones

1. Regex con flag `/g` NO deben crearse dentro de loops — compilarlas una vez a nivel de módulo y resetear `.lastIndex = 0` antes de cada uso.
2. `Promise.all()` sin límite sobre arrays dinámicos (archivos encontrados) siempre necesita throttling.
3. Los Sets/Maps module-scoped son memory leaks si no se limpian en `deactivate()`.
4. Funciones que extraen "todos los tokens" de "todos los archivos" son bombas de RAM en proyectos medianos-grandes.
