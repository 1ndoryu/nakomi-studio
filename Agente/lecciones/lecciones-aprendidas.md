# Lecciones Aprendidas

## Rust — Tests con env vars
- `std::env::set_var` / `remove_var` NO son thread-safe. Rust ejecuta tests en paralelo.
- Tests que modifican las mismas env vars compiten entre sí y fallan intermitentemente.
- **Solución:** `static ENV_LOCK: Mutex<()> = Mutex::new(());` y adquirir el guard al inicio de cada test.

## Rust — clippy too_many_lines
- Límite de 100 líneas por función. Extraer helpers agresivamente.
- Patrón efectivo: extraer loops internos, bloques de setup, y operaciones I/O a funciones separadas.
- Las funciones helper pueden ser `async fn` privadas en el mismo archivo.

## SQLx — query_as! vs query_as
- `query_as!` (macro) requiere TODOS los campos del struct en SELECT/RETURNING.
- `#[sqlx(default)]` solo funciona con `query_as` (runtime), no con la macro.
- Tras modificar queries con `sqlx::query!` o `query_as!`, SIEMPRE ejecutar `cargo sqlx prepare`.

## PowerShell + cargo
- cargo escribe progreso en stderr. PowerShell interpreta stderr como error.
- `2>&1 | ForEach-Object { $_.ToString() }` y luego `$LASTEXITCODE` es el patrón correcto.

## Code Sentinel — sentinel-disable-file
- Al crear sentinel-disable-file comments, SIEMPRE usar el ID exacto de la regla, no un alias inventado.
- `button-nativo` ≠ `html-nativo-en-vez-de-componente`. El sentinel solo reconoce el ID registrado en ruleRegistry.ts.
- La función `tieneSentinelDisable()` solo verifica `sentinel-disable-next-line`, NO `sentinel-disable-file`.
- Cada regla del sentinel debe implementar su propia verificación de `sentinel-disable-file` explícitamente si necesita soporte file-level.
- [104A-4] Se añadió soporte sentinel-disable-file a: html-nativo-en-vez-de-componente, componente-sin-hook, usestate-excesivo.

## Coolify — deploy vs restart
- `POST /api/v1/services/{uuid}/restart` solo reinicia containers existentes con la misma imagen.
- `GET /api/v1/deploy?uuid={uuid}&force=true` trigger un rebuild completo (git pull + docker build).
- Para cambios de código, SIEMPRE usar deploy, no restart.
- coolify-manager.exe `deploy --name` es para WordPress themes, no para apps Rust. Usar API directa.
