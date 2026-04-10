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
- Si la query nueva depende de una columna recién agregada, primero correr `cargo sqlx migrate run` contra la base local de `DATABASE_URL`; si no, `cargo sqlx prepare` falla aunque el código esté bien.

## PowerShell + cargo
- cargo escribe progreso en stderr. PowerShell interpreta stderr como error.
- `2>&1 | ForEach-Object { $_.ToString() }` y luego `$LASTEXITCODE` es el patrón correcto.

## Code Sentinel — sentinel-disable-file
- Al crear sentinel-disable-file comments, SIEMPRE usar el ID exacto de la regla, no un alias inventado.
- `button-nativo` ≠ `html-nativo-en-vez-de-componente`. El sentinel solo reconoce el ID registrado en ruleRegistry.ts.
- La función `tieneSentinelDisable()` solo verifica `sentinel-disable-next-line`, NO `sentinel-disable-file`.
- Cada regla del sentinel debe implementar su propia verificación de `sentinel-disable-file` explícitamente si necesita soporte file-level.
- [104A-4] Se añadió soporte sentinel-disable-file a: html-nativo-en-vez-de-componente, componente-sin-hook, usestate-excesivo.
- La suite de `code-sentinel` está forzada por `.mocharc.json` a cargar `out/test/suite/*.test.js`; para validar una regla aislada hay que usar `mocha --no-config --require out/test/registerMocks.js <archivo>`.

## Coolify — deploy vs restart
- `POST /api/v1/services/{uuid}/restart` solo reinicia containers existentes con la misma imagen.
- `GET /api/v1/deploy?uuid={uuid}&force=true` trigger un rebuild completo (git pull + docker build).
- Para cambios de código, SIEMPRE usar deploy, no restart.
- coolify-manager.exe `deploy --name` es para WordPress themes, no para apps Rust. Usar API directa.

## UI del panel — bases compartidas
- Si una variante visual ya es la buena (`hostingCardIcono` en este caso), promover ese estilo a la clase base compartida y dejar las variantes futuras como overrides mínimos con composición de clases, no como recetas duplicadas.

## CSS validator — variables resueltas
- Algunos diagnósticos de CSS siguen reportando “hardcodeado” aunque la regla ya use `var(--token)` y el build pase. Cuando ocurra, corroborar con `npm --prefix frontend run build` antes de perseguir falsos positivos del validador.

## Admin deletes — dependencias reales
- Si una entidad admin pide “eliminar” pero tiene FKs sin cascade repartidas en varias tablas, no implementar hard delete ciego. Primero exponer al panel un preflight de dependencias con mensaje explícito y usar suspensión como fallback operativo.

## CMS público — no maquillar vacíos con demo data
- Si una vista pública depende del CMS/API, distinguir entre “todavía no cargó” y “la API devolvió vacío”. Reutilizar fallback demo cuando el backend responde `[]` oculta desincronizaciones reales y hace que home/listados muestren contenido fantasma.

## Checkout de órdenes — IDs visuales no son contrato
- Si el catálogo frontend usa IDs compuestos para UI/traducciones (`web-basico`, `apps-medio`) pero el backend persiste slugs canónicos (`basico`, `medio`), normalizar en el cliente API antes del POST. Un `404` en creación puede ser un `NotFound` de dominio, no una ruta faltante.

## Empty states — no mezclar jerarquías en la misma sección
- Si una sección ya tiene un estado vacío completo con icono, título y texto, las tabs internas no deberían degradarse a un párrafo desnudo. Reutilizar un bloque común evita que el vacío “parcial” se vea como un render roto.

## Inputs del sistema — no reestilar por costumbre
- Si una sección usa `Input` base y el override local no cambia comportamiento ni semántica, eliminar la clase local en vez de mantener CSS duplicado. Cada wrapper visual extra encarece futuras limpiezas sin aportar contrato nuevo.
