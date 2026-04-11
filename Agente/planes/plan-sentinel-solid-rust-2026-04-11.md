# Plan: Glory Sentinel SOLID para Rust

**Fecha:** 2026-04-11
**Tarea:** 114A-6
**Estado:** En ejecución

## Problema

Code Sentinel solo tiene 3 reglas para Rust (2 sqlx + 1 todo-pendiente). No existe enforcement de principios SOLID, límites de líneas, detección de unwrap/panic en producción, ni validación arquitectónica (handler→repository pattern).

## Hallazgos del codebase actual

| Métrica                         | Valor                            | Severidad           |
| ------------------------------- | -------------------------------- | ------------------- |
| Archivo más grande              | handlers/hosting.rs (1,470 LoC)  | CRÍTICA             |
| Segundo más grande              | services/ai_chat.rs (~1,210 LoC) | CRÍTICA             |
| .unwrap() fuera de tests        | 7+ instancias                    | CRÍTICA             |
| panic!() fuera de tests         | 2 instancias                     | MEDIA               |
| sqlx::query directo en handlers | 29 llamadas                      | CRÍTICA (viola DIP) |
| Funciones > 100 líneas          | Varias en ai_chat.rs, hosting.rs | ALTA                |

## Fase 1 — Implementación (esta tarea)

### 1.1 Límites de líneas para .rs (`lineCounter.ts`)

- handlers/\*.rs: max 500 líneas
- services/\*.rs: max 500 líneas
- repositories/\*.rs: max 400 líneas
- models/\*.rs: max 300 líneas
- General .rs: max 500 líneas
- bin/\*.rs, migrations/, examples/: sin límite
- Bypass: `sentinel-disable-file limite-lineas`

### 1.2 Nuevo `rustAnalyzer.ts` con reglas context-aware

| ID                        | Nombre                    | Severidad | Descripción                                        |
| ------------------------- | ------------------------- | --------- | -------------------------------------------------- |
| `unwrap-produccion-rs`    | .unwrap() en producción   | warning   | Detecta .unwrap() fuera de bloques #[cfg(test)]    |
| `panic-produccion-rs`     | panic!() en producción    | warning   | Detecta panic!/todo!/unimplemented! fuera de tests |
| `handler-accede-bd-rs`    | Handler accede BD directo | warning   | sqlx::query en archivos bajo handlers/             |
| `funcion-larga-rs`        | Función excede 100 líneas | warning   | Funciones/métodos > 100 líneas efectivas           |
| `parametros-excesivos-rs` | Función con 6+ parámetros | hint      | Firmas con demasiados parámetros                   |

### 1.3 Nueva categoría `RustPatrones`

- Agrupar todas las reglas Rust-specific

### 1.4 Integración

- `types/index.ts`: Añadir 'rust' a TipoArchivo
- `diagnosticProvider.ts`: Llamar `analizarRust()` para archivos .rs
- `ruleRegistry.ts`: Registrar las 5 nuevas reglas
- `ruleCategories.ts`: Añadir metadata de RustPatrones

## Fase 2 — Mejoras futuras (no esta tarea)

- `struct-gigante-rs`: Structs con > 12 campos
- `impl-gigante-rs`: Bloques impl > 200 líneas
- `clone-innecesario-rs`: .clone() en hot paths
- `string-param-rs`: `String` en lugar de `&str` en parámetros
- Quick fixes automáticos para unwrap → ? conversion
- Integración con cargo clippy output parsing

## Notas técnicas

- El sistema regex per-line NO distingue contexto (test vs producción). Por eso unwrap/panic van en rustAnalyzer.ts con lógica de detección de bloques #[cfg(test)].
- La heurística de "dentro de test": línea `#[cfg(test)]` marca inicio de módulo test, se detecta por indentación/scope.
- handler-accede-bd-rs usa la ruta del archivo normalizada para determinar si está en handlers/.
- funcion-larga-rs cuenta llaves abiertas/cerradas para determinar el scope de la función.
