# Alcance Sentinel/VarSense sobre coolify-manager-rs — 2026-05-10

## Decisión

`coolify-manager-rs` debe quedar dentro del workspace multi-root y dentro de los análisis de Glory Sentinel/VarSense. Es herramienta crítica del agente, así que sus cambios de Rust/React/CSS no pueden quedar fuera de los barridos por vivir bajo `.agent/`.

## Ajuste aplicado

- `codeSentinel.exclude` ya no excluye `**/.agent/**` completo.
- Se excluyen solo las herramientas de análisis para evitar autoanalizar Sentinel y VarSense: `**/.agent/code-sentinel/**` y `**/.agent/varsense/**`.
- `cssVarsValidator.excludePatterns` ya no excluye `**/coolify-manager-rs/**`.

## Uso operativo

- Para analizar la GUI/herramienta desde VS Code: ejecutar `codeSentinel.analyzeWorkspace` y `cssVarsValidator.scanAllDiagnostics` con el workspace multi-root abierto.
- Para prueba aislada de Sentinel contra `coolify-manager-rs`: usar `CODE_SENTINEL_TARGET_WORKSPACE=<ruta-coolify-manager-rs> npm test` desde `.agent/code-sentinel`.
- VarSense también puede validar por CLI si hace falta aislar el repo, pero en el editor debe ver `coolify-manager-rs` como carpeta del workspace.

## Gotcha

Excluir `**/.agent/**` parece prudente, pero oculta justo la herramienta que más necesita validación. La exclusión debe ser quirúrgica, no por carpeta padre.