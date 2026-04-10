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
- Las reglas CSS que miran selectores no pueden usar regex ciega sobre el bloque completo: primero hay que strippear comentarios y distinguir clases del sistema (`menuContextualBoton`, `botonBase`, etc.) o aparecen falsos positivos masivos.
- `inline-style-prohibido` debe aceptar `style={{ '--mi-var': valor }}` también cuando el objeto está en una sola línea; si no, barras de progreso y layouts con CSS vars vuelven a romper el reporte.
- Para validar una versión local de Sentinel contra otro repo sin reinstalar la extensión del editor, usar `CODE_SENTINEL_TARGET_WORKSPACE=<ruta> npm test` en `.agent/code-sentinel`; ese host de pruebas puede regenerar `.sentinel-report.md` de forma reproducible.

## Coolify — deploy vs restart
- `POST /api/v1/services/{uuid}/restart` solo reinicia containers existentes con la misma imagen.
- `GET /api/v1/deploy?uuid={uuid}&force=true` trigger un rebuild completo (git pull + docker build).
- Para cambios de código, SIEMPRE usar deploy, no restart.
- coolify-manager.exe `deploy --name` es para WordPress themes, no para apps Rust. Usar API directa.

## UI del panel — bases compartidas
- Si una variante visual ya es la buena (`hostingCardIcono` en este caso), promover ese estilo a la clase base compartida y dejar las variantes futuras como overrides mínimos con composición de clases, no como recetas duplicadas.

## Panel CMS — menus contextuales en cards
- Si una card clickeable del CMS contiene un `MenuContextual`, no puede usar `overflow: hidden` ni depender solo de `:hover` del card para mostrar el wrapper del menú. En cards bajas o con paneles que salen del contenedor, la acción destructiva queda inaccesible aunque el endpoint responda correctamente.

## CSS validator — variables resueltas
- Algunos diagnósticos de CSS siguen reportando “hardcodeado” aunque la regla ya use `var(--token)` y el build pase. Cuando ocurra, corroborar con `npm --prefix frontend run build` antes de perseguir falsos positivos del validador.

## Imágenes responsive — `sizes` omitido no puede caer siempre en `100vw`
- Si un `img` con `srcset` usa anchos descriptivos y el componente no recibe `sizes`, el navegador asume `100vw` y sobre-descarga variantes grandes en avatars, logos y cards pequeñas.
- Medir el ancho real renderizado con `ResizeObserver` y derivar un `sizes` en píxeles evita ese sesgo sin obligar a propagar `sizes` manual en todos los callers.
- Si el backend ya soporta más buckets que el frontend, el cuello de botella real está en la generación del `srcset`, no en el proxy.

## Admin deletes — dependencias reales
- Si una entidad admin pide “eliminar” pero tiene FKs sin cascade repartidas en varias tablas, no implementar hard delete ciego. Primero exponer al panel un preflight de dependencias con mensaje explícito y usar suspensión como fallback operativo.

## CMS público — no maquillar vacíos con demo data
- Si una vista pública depende del CMS/API, distinguir entre “todavía no cargó” y “la API devolvió vacío”. Reutilizar fallback demo cuando el backend responde `[]` oculta desincronizaciones reales y hace que home/listados muestren contenido fantasma.

## Checkout de órdenes — IDs visuales no son contrato
- Si el catálogo frontend usa IDs compuestos para UI/traducciones (`web-basico`, `apps-medio`) pero el backend persiste slugs canónicos (`basico`, `medio`), normalizar en el cliente API antes del POST. Un `404` en creación puede ser un `NotFound` de dominio, no una ruta faltante.

## Catálogo público — no vender servicios fantasma
- Si la compra depende del catálogo real del backend, el detalle/listado público no debe caer a datasets estáticos que incluyan servicios ya no publicados. Aunque `apiCreateOrder()` normalice slugs, seguir mostrando `ecommerce`/`seo`/`marketing-digital` cuando la API solo expone 4 servicios termina reproduciendo 404 de negocio igualmente.

## Empty states — no mezclar jerarquías en la misma sección
- Si una sección ya tiene un estado vacío completo con icono, título y texto, las tabs internas no deberían degradarse a un párrafo desnudo. Reutilizar un bloque común evita que el vacío “parcial” se vea como un render roto.

## Inputs del sistema — no reestilar por costumbre
- Si una sección usa `Input` base y el override local no cambia comportamiento ni semántica, eliminar la clase local en vez de mantener CSS duplicado. Cada wrapper visual extra encarece futuras limpiezas sin aportar contrato nuevo.

## Fixtures TOML — tracking no sustituye existencia real
- Si `_glory_fixtures` conserva `content_hash` y `db_id` pero la fila real ya no existe, el sync no puede hacer `skip` ciego. Primero debe verificar existencia física y reinsertar si falta.
- Cuando una migración agrega columnas `NOT NULL` a una tabla fixture-managed, actualizar ese mismo día todos los `content/*.toml` de la tabla. Un solo campo faltante (`users.username` en este caso) rompe en cascada todos los fixtures dependientes y termina pareciendo un bug del seed en vez de un drift del fixture.
