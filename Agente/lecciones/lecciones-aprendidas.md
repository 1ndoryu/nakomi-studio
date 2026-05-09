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
- Si la base local principal tiene checksums viejos de migraciones ya aplicadas, no fuerces esa BD para regenerar `.sqlx/`: crea una base temporal limpia, corre ahí `cargo sqlx migrate run` y luego `cargo sqlx prepare`.

## PowerShell + cargo
- cargo escribe progreso en stderr. PowerShell interpreta stderr como error.
- `2>&1 | ForEach-Object { $_.ToString() }` y luego `$LASTEXITCODE` es el patrón correcto.
- Comandos largos o ambiguos no se deben "esperar" por intuicion: si no tienen criterio de fin claro, se ejecutan en background/async y se validan con una senal puntual (puerto, health, proceso, archivo generado, ultimas lineas del log).
- Si `npx` o una CLI puede pedir confirmacion, usar `-y` o modo no interactivo desde el inicio; si no, el flujo parece colgado aunque en realidad esta esperando input.

## Node/Vite — dependencias por rama
- Cambiar de rama no actualiza `frontend/node_modules`: Git cambia `package.json`/`package-lock.json`, pero el árbol instalado queda como estado local compartido.
- Vite puede detectar el lockfile nuevo y reoptimizar, pero no instala paquetes ausentes. El launcher compartido `glory-rs/scripts/dev.mjs` debe sincronizar una vez por huella de lockfile antes de arrancar Vite.
- Recalcular la huella después de `npm install`, porque npm puede ajustar `package-lock.json` y dejar un marker obsoleto si se guarda la huella previa.
- Si un script es agnóstico del flujo local (`dev`, limpieza de target, reparación de dependencias), debe vivir en `glory-rs/scripts/`; la raíz solo debe conservar wrappers o scripts específicos inevitables.

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
- `modal-estructura-no-canonica` no puede limitarse a `form/div` internos: también debe inspeccionar `className` sobre el propio `<Modal>`, porque clases como `usuariosModal` o `checkoutModal` redefinen el contenedor compartido y si no se miran ahí el reporte queda ciego justo en el punto de entrada.
- Para detectar especificaciones de diseño CSS, no depender de nombres `boton/button`: cruzar rol interactivo local (`Trigger`, `Lista`, `Opcion`, `__dropdown`) con varias propiedades visuales (`background`, `border`, tipografia, padding, transition/animation) y cubrirlo con fixture core vs CLI.

## VarSense — CLI editor-agnostico
- Si `tsc` emite en el mismo `dist/` que esbuild, puede sobrescribir el CLI bundlereado con una version que conserva aliases `@/`. Despues del type-check, regenerar el bundle o separar outDirs.
- El smoke valido de una CLI editor-agnostica no es importar funciones desde tests: hay que ejecutar `node dist/cli/index.js ...` contra un workspace temporal para confirmar que Node puro no carga `vscode` ni aliases sin resolver.
- Si el entrypoint ya tiene shebang, no agregar otro con `banner` de esbuild; el segundo shebang queda en linea 2 y rompe `require()`/ejecucion.
- Los snapshots de equivalencia deben normalizar rutas relativas y rangos 0-indexed; los codigos de salida se prueban aparte porque un fixture con errores debe devolver `1` aunque su JSON sea el esperado.
- Un LSP no queda validado por probar solo el mapper de diagnostics: hay que levantar el binario compilado por stdio, enviar `initialize` y `textDocument/didOpen`, y verificar `textDocument/publishDiagnostics`.
- El LSP no debe importar defaults desde el entrypoint CLI. En un bundle esbuild, `require.main === module` puede hacer que el codigo de CLI imprima `Uso:` por stdio y rompa cualquier cliente LSP. Mover defaults/config a `core/config` y cubrirlo con smoke real.
- Los guards de `src/core/**` deben permitir solo el adaptador boundary (`vscodeAdapter.ts`) y fallar sobre cualquier import directo de `vscode`; asi se protege la arquitectura editor-agnostica sin bloquear la compatibilidad VS Code existente.
- La integracion Zed debe ser un launcher fino: registrar `language_servers`, resolver `varsense-lsp` por entorno/PATH/dist local y no copiar reglas ni empaquetar el servidor dentro de la extension.
- En manifests Zed reales, la tabla estable observada es `language_servers`; algunos docs muestran `language-servers` en ejemplos multi-lenguaje, asi que validar contra ejemplos oficiales antes de copiar sintaxis.

## Coolify — deploy vs restart
- `POST /api/v1/services/{uuid}/restart` solo reinicia containers existentes con la misma imagen.
- `GET /api/v1/deploy?uuid={uuid}&force=true` trigger un rebuild completo (git pull + docker build).
- Para cambios de código, SIEMPRE usar deploy, no restart.
- coolify-manager.exe `deploy --name` es para WordPress themes, no para apps Rust. Usar API directa.
- coolify-manager.exe `restart` no siempre reinicia los contenedores de apps Docker Compose; `redeploy` (API) es más fiable para forzar recreación.
- En `studio` (nakomi.studio), los endpoints admin firmados por JWT usan el env runtime `SERVICE_PASSWORD_64_JWTSECRET`; `JWT_SECRET` listado en Coolify no autenticó `/api/admin/blog` durante la verificación real.

## Rust/Axum — Timeouts HTTP obligatorios para APIs externas
- **NUNCA crear `reqwest::Client::new()` sin `.timeout()` en código de producción.** Una API externa que se cuelga bloquea la tarea async indefinidamente. Si la tarea retiene una conexión del pool de BD (SQLx), agota el pool y congela toda la aplicación (deadlock de pool). Síntomas: proceso vivo pero 503, tcp backlog lleno, threads dormidos.
- Usar `reqwest::Client::builder().timeout(Duration::from_secs(30)).build()` como mínimo.
- Para cadenas de retry (Groq 3 keys × 3 modelos + Gemini 6 modelos = 24 intentos), agregar timeout global con `tokio::time::timeout(Duration::from_secs(90), ...)` además del per-request timeout.
- [124A-1] Este bug causó un 503 en producción ~1h después de deploy. Los logs no mostraron crash porque no hubo panic — el pool simplemente se agotó en silencio.

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

## Hero/carrusel above-the-fold — a veces hace falta ancho fijo
- Cuando el objetivo es una URL exacta de optimización (`w=1200&q=80`) para controlar peso en un bloque hero/carrusel, el cálculo responsive por ancho medido + DPR puede seguir sobredescargando.
- En esos casos conviene un modo explícito sin `srcSet` responsive y con ancho fijo de proxy, en vez de pelear contra buckets automáticos.

## Admin deletes — dependencias reales
- Si una entidad admin pide “eliminar” pero tiene FKs sin cascade repartidas en varias tablas, no implementar hard delete ciego. Primero exponer al panel un preflight de dependencias con mensaje explícito y usar suspensión como fallback operativo.

## CMS público — no maquillar vacíos con demo data
- Si una vista pública depende del CMS/API, distinguir entre “todavía no cargó” y “la API devolvió vacío”. Reutilizar fallback demo cuando el backend responde `[]` oculta desincronizaciones reales y hace que home/listados muestren contenido fantasma.

## Checkout de órdenes — IDs visuales no son contrato
- Si el catálogo frontend usa IDs compuestos para UI/traducciones (`web-basico`, `apps-medio`) pero el backend persiste slugs canónicos (`basico`, `medio`), normalizar en el cliente API antes del POST. Un `404` en creación puede ser un `NotFound` de dominio, no una ruta faltante.
- Los alias de `service_slug` también envejecen: si el backend vuelve a usar `diseno-web` como canónico y el cliente lo sigue remapeando a `diseno-de-sitios-web`, reaparece un `404` aunque el `plan_slug` ya esté bien normalizado.
- [095A-19] Si el CMS puede cambiar el slug público activo, la compatibilidad legacy debe vivir en backend y probar primero el slug recibido. El frontend no debe decidir el canónico porque se desactualiza con cada sync de contenido.

## Catálogo público — no vender servicios fantasma
- Si la compra depende del catálogo real del backend, el detalle/listado público no debe caer a datasets estáticos que incluyan servicios ya no publicados. Aunque `apiCreateOrder()` normalice slugs, seguir mostrando `ecommerce`/`seo`/`marketing-digital` cuando la API solo expone 4 servicios termina reproduciendo 404 de negocio igualmente.

## Catalogo CMS — no fusionar servicios reales para acomodar fallbacks
- Si el negocio distingue `ecommerce` y `marketing-digital`, el CMS debe modelarlos como servicios separados aunque el frontend heredado tenga un fallback incompleto. Fusionarlos para “encajar” con el dataset estatico termina rompiendo el catalogo real y obliga a deshacer la carga luego.
- La correccion segura es: arreglar primero la propuesta fuente y resincronizar el CMS; el fallback del frontend se corrige despues como tarea aparte, nunca al reves.

## Empty states — no mezclar jerarquías en la misma sección
- Si una sección ya tiene un estado vacío completo con icono, título y texto, las tabs internas no deberían degradarse a un párrafo desnudo. Reutilizar un bloque común evita que el vacío “parcial” se vea como un render roto.

## Inputs del sistema — no reestilar por costumbre
- Si una sección usa `Input` base y el override local no cambia comportamiento ni semántica, eliminar la clase local en vez de mantener CSS duplicado. Cada wrapper visual extra encarece futuras limpiezas sin aportar contrato nuevo.

## Fixtures TOML — tracking no sustituye existencia real
- Si `_glory_fixtures` conserva `content_hash` y `db_id` pero la fila real ya no existe, el sync no puede hacer `skip` ciego. Primero debe verificar existencia física y reinsertar si falta.
- Cuando una migración agrega columnas `NOT NULL` a una tabla fixture-managed, actualizar ese mismo día todos los `content/*.toml` de la tabla. Un solo campo faltante (`users.username` en este caso) rompe en cascada todos los fixtures dependientes y termina pareciendo un bug del seed en vez de un drift del fixture.

## Code Sentinel — sentinel-disable-next-line formato
- `sentinel-disable-next-line {rule-id}` DEBE estar en la línea inmediatamente anterior a la violación.
- En Rust, usar `// sentinel-disable-next-line {rule-id}` como comentario single-line.
- NUNCA ponerlo dentro de un comentario multilínea `/* ... */` que ocupe varias líneas, porque el checker compara `lineas[i-1]` y la línea anterior real sería el cierre `*/`, no el disable.
- Patrón correcto: explicación en `/* ... */` arriba, y `// sentinel-disable-next-line ...` en la línea justo antes del código.

## Stripe live mode - verificacion local de SetupIntent
- Si el entorno local apunta a llaves `pk_live` y `sk_live`, Stripe bloquea crear tarjetas de prueba por REST con numeros crudos aunque el flujo real con Stripe.js si sea valido.
- Para validar un flujo nuevo de tarjetas guardadas en ese contexto, separar: backend y contrato por API local, compilacion del modal con Stripe.js, y justificar que la confirmacion completa requiere navegador o llaves test.

## Extensiones VS Code — Memory leaks y rendimiento (AUDIT1)
- Regex con flag `/g` NO crear dentro de loops: compilar una vez a nivel de módulo, resetear `.lastIndex = 0` antes de reusar.
- `Promise.all()` sobre arrays dinámicos (archivos encontrados) siempre necesita throttling con lotes.
- Sets/Maps module-scoped son memory leaks si no se limpian en `deactivate()` o al cerrar documentos.
- Funciones que extraen "todos los tokens" de "todos los archivos" sin filtro son bombas de RAM. Siempre acotar semánticamente (ej: solo class/className, no todo identificador).
- WebviewPanels sin singleton ni `onDidDispose` crean múltiples instancias que nunca se liberan.
- Concatenación de strings en loop para reportes es O(n²) — usar `array.push()` + `.join()`.
- Event handlers que actualizan "todos los documentos abiertos" necesitan debounce cuando el emisor puede disparar muchas veces en ráfaga.

## Checkout publico - no crear dos PaymentIntent para la misma orden
- Si el flujo publico ya creo la orden y recibio `client_secret`, el checkout siguiente debe reutilizarlo. Volver a llamar `/api/orders/{id}/pay` desde la siguiente pantalla duplica intents y complica la conciliacion del pago cancelado.
- Cuando el panel persiste la tab activa en `sessionStorage`, cualquier flujo que redirija a `/panel` como fallback debe fijar primero la seccion correcta o el usuario puede aterrizar lejos de la orden recien creada.

## Hosting publico - no reutilizar el funnel de ordenes
- Si un producto ya tiene backend propio de suscripcion (`/api/hosting/subscribe`), la UI publica no debe seguir entrando por `/api/orders` aunque visualmente reuse cards o modales de compra.
- Un `404` en checkout puede venir de un contrato de dominio equivocado: en este caso `service_slug = hosting` no existia en el catalogo de ordenes, asi que la solucion correcta fue mover el flujo al endpoint real de hosting, no inventar aliases en orders.

## Infra admin - proveedor no equivale a deployment
- Si el panel pide “despliegues reales”, la fuente correcta es la capa de orquestacion (Coolify, Kubernetes, etc.), no la API del proveedor de VPS.
- Contabo responde “que servidores existen”; Coolify responde “que servicios estan desplegados”. Mezclar ambas capas permite cerrar tareas en falso y oculta orfandades reales entre deployment y suscripcion.

## Commit-por-tarea — no acumular cambios
- Si el protocolo dice "un commit por tarea", cumplirlo inmediatamente después de validar, no al "final de la sesión" ni "cuando haya tiempo". Acumular 3+ tareas sin commit significa que un solo error en git rompe todo el trabajo.

## Wallet demo local — seed antes que frontend
- Si `cliente@test.com` o `empleado@test.com` muestran wallet vacía pero `src/services/seed.rs` ya define movimientos y retiros, el problema suele ser de entorno local sin reseed reciente, no de UI. Reejecutar `POST /api/admin/seed` y verificar `/api/wallet`, `/api/wallet/transactions` y `/api/wallet/withdrawals` antes de modificar componentes.
- El push es parte del cierre de la tarea. Si no se hizo push, la tarea no está cerrada.
- Refuerzo agregado al protocolo (104A-19): prohibición explícita de acumular 2+ tareas sin commit+push.

## Consistencia visual — leer antes de crear
- Antes de crear o modificar CSS, leer primero `variables.css`, los componentes atómicos en `ui/` y los patrones de componentes similares. Cada clase ad-hoc que duplica un token existente es deuda visual que se acumula.
- Badge siempre en grises (sin color semántico) fue una decisión de diseño Nakomi. Footers, headers, cards deben compartir un patrón unificado.

## Modales — semantica compartida primero
- Si un modal necesita copy neutral, usar `.modalTexto` en `Modal.css` antes de inventar `.algoModalTexto` o `.algoModalDescripcion`. Las clases locales solo deben conservar layout o estado.
- En analyzers CSS por bloques, `sentinel-disable-next-line` debe anclarse a la linea real del selector y no al inicio de un comentario previo; si no, la supresion parece rota aunque el helper este bien.
- La estructura comun del modal tambien es contrato: cuerpo y campos deben salir de `.modalFormulario` / `.modalCampo` o de `ModalBody` / `ModalField`. Clases como `.algoFormCrear` o `.algoCampo` vuelven a introducir especificaciones de diseno prohibidas.

## Checkout publico — cortar roles invalidos antes del request
- Si el backend solo permite crear ordenes como `client` o `admin`, el modal publico no debe decidir solo por `logueado`. En local es frecuente quedar con sesion `employee` por pruebas del panel y eso reproduce `403` evitables.
- Para validar ese caso sin tocar backend, basta simular `auth_user` en `localStorage`, recargar la SPA y comprobar que el guard del frontend muestra el mensaje en vez de entrar al estado de procesamiento.

## Ordenes por fases — el CMS manda
- Si un plan ya define `service_plan_phases`, la creacion de la orden no debe reescribir esos titulos/descripciones con placeholders genericos. Hacerlo rompe el contrato con el CMS y da la falsa impresion de que el empleado debe “definir” las fases a mano.
- Si el checkout ofrece `payment_mode = phased`, un plan sin fases debe rechazarse en dos boundaries: al guardar el plan desde admin y al crear la orden, para que la inconsistencia no llegue a producción.

## CMS de servicios — editar plan no puede reciclar IDs
- Si una orden ya referencia `service_plans.id`, el CMS no puede hacer `DELETE + INSERT` para guardar planes. Hay que preservar el ID existente y tratar `service_plan_phases` como el hijo reemplazable.
- Para permitir cambios de slug/nombre sin romper órdenes históricas, el frontend admin debe enviar `id` opcional por plan y el backend usarlo como llave estable antes de caer al slug.

## Chat de órdenes — sender_type también es contrato
- Si el sistema persiste nuevos tipos semánticos de mensaje (`ai_intermediary`, por ejemplo), el esquema de `chat_messages.sender_type` debe crecer junto con el código. Dejarlo en `VARCHAR(10)` hace que la IA genere bien pero no pueda guardar la respuesta.
- En rutas de chat con `tokio::spawn`, ignorar el resultado de `send_message().await` convierte un fallo de persistencia en un "la IA no responde" imposible de diagnosticar desde la UI. Al menos hay que registrar el error explícitamente.

## Chatbot de cuenta — el prompt no autoriza
- Si el bot puede consultar pedidos, pagos, hosting o reportes, el prompt solo debe orientar la conversación. La frontera real es un contexto firmado (`user_id`, rol real, rol operativo, impersonator) pasado a cada tool y validado en backend.
- `visitor_id` persistido sin owner key mezcla conversaciones cuando alguien hace logout/login o impersona en el mismo navegador. Separar storage por identidad/rol efectivo evita fugas entre cuentas.
- Los logs de tools deben incluir nombre, sesión y status, pero nunca argumentos crudos: pueden contener email, reportes o detalles de pago.

## Panel interno — storage solo no alcanza para deep links
- Si una vista interna del panel debe sobrevivir recargas, compartirse por URL o abrirse desde una notificación, `localStorage`/`sessionStorage` y custom events no bastan. La URL debe ser una fuente observable de verdad y los hooks dueños del detalle deben hidratarse desde `location.search`.
- Al sincronizar estado profundo (`order`, `hostingId`, `chat`) a la URL, no borrar el query param en el primer render si ese mismo param todavía está intentando hidratar el estado. El orden correcto es: leer URL, seleccionar recurso, luego persistir el estado ya resuelto.

## Sesiones impersonadas — no degradar a 500
- Si un JWT con `impersonator` sobrevive a un reseed local y el admin original ya no existe, volver a `admin` no debe caer en `500`. El backend debe responder `401/403` con mensaje accionable y el frontend debe limpiar la sesión persistida para cortar el bucle.

## Tokens visuales — activo no equivale a neutro
- `--bg-item-active` no sirve como borde base. Los bordes genéricos del sistema deben usar `--border-default`; reservar el token activo evita que estados normales parezcan seleccionados y simplifica los barridos visuales.

## Upstreams opcionales - no esconderlos tras 500 internos
- Si una integración externa opcional falla (Contabo, por ejemplo), no devolver `Internal` genérico desde el handler. Clasificar y exponer un `message` accionable evita perseguir fantasmas de backend cuando el bloqueo real es `invalid_grant`, parseo o indisponibilidad del proveedor.
- Cuando una credencial legacy es ambigua (`PASSWORD_CONTABO`), documentar y soportar una variable explícita (`CONTABO_API_PASSWORD`) reduce drift entre proyectos y evita repetir el mismo diagnóstico en cada repo.

## Auditorías vagas - convertirlas en backlog ejecutable
- Si el roadmap trae una tarea tipo “hay que revisar” o “presiento que falta mucho”, cerrarla solo con lectura no alcanza. Hay que salir de la auditoría con un documento, un plan activo y subtareas concretas reinsertadas en el roadmap.
- En hosting, el valor real de la revisión estuvo en separar claramente: provisioning, ciclo de cobro/suspensión, datos reales del servidor y dominios. Sin ese corte, todo queda escondido en una sola tarea imposible de cerrar.

## CMS de servicios - update no puede delegar el contrato a SQLx
- Si el panel edita `services`, el handler de update debe llamar `validate()` igual que create y mapear conflictos/constraints frecuentes a `409/422`. Dejar que `sqlx::Error` suba crudo convierte errores previsibles del CMS en `500` opacos.
- Cuando el slice ya tiene una version mas robusta en otro dominio cercano, conviene copiar el patron completo. En este caso, `UpdateServiceParams<'_>` + `query_as::<_, ServiceRecord>(...).bind(...)` dejo services alineado con projects y evito pelear con una macro mas fragil para updates parciales.

## Catalogos publicos - la shell compartida va antes que dos CSS parecidas
- Si servicios y proyectos tienen la misma pagina a nivel de hero, contenedor y espaciado, ese layout debe vivir en un componente compartido. Mantener dos islands con wrappers casi iguales solo garantiza drift visual y correcciones duplicadas.
- Antes de “arreglar el padding” de una clase legacy, buscar todos sus consumidores. `serviciosContenedor` y `proyectosContenedor` ya se usaban fuera del catalogo publico, asi que la solucion segura fue crear clases nuevas `catalogPage*` y mover el layout comun a una shell dedicada.

## CMS de servicios - un 409 util no sirve si el frontend lo esconde
- Si el backend devuelve `ErrorResponse { message }`, los hooks del panel no deben quedarse con `err.message` de Axios. Hay que extraer `response.data.message`; si no, una mejora real del contrato HTTP termina viendose como otro error generico de red.
- Cuando el formulario ya tiene en memoria el conjunto completo de slugs, conviene hacer preflight local antes del submit. El servidor sigue siendo la verdad final, pero el usuario recibe feedback inmediato y se evita un roundtrip inutil para conflictos obvios.

## CMS de servicios - guardar servicio y planes en dos pasos exige preflight
- Si el editor primero guarda el servicio y despues hace `PUT /plans`, cualquier validacion tardia de planes produce sensacion de guardado parcial. Antes del primer request hay que validar los invariantes minimos del segundo paso con el mismo contrato del backend.
- En este slice, los invariantes minimos que no deben salir del cliente son: slug y nombre obligatorios por plan, slug unico dentro del servicio, al menos una fase por plan y titulo obligatorio por fase.

## CMS de servicios - no mezclar guardado de metadatos con persistencia de planes
- Si el usuario solo edita imagen, SEO o datos generales, el editor no debe volver a persistir `service_plans`. Un segundo request redundante puede fallar por deuda historica de planes y dejar la falsa impresion de que el campo principal “no guarda”.
- Cuando una pantalla guarda recursos relacionados en dos endpoints, hay que detectar qué slice cambió realmente y disparar solo ese endpoint. La comparación contra el snapshot cargado evita roundtrips innecesarios y reduce errores cruzados.

## Editor inline vs panel - un fix no existe si el control path real sigue duplicado
- Cuando una misma UI reutiliza el mismo modal pero lo monta desde dos owners distintos, no basta con corregir uno. Si el stack apunta a otro provider, hay que seguir ese call site exacto antes de dar el bug por cerrado.
- Si dos flows necesitan decidir lo mismo sobre persistencia (`guardar planes o no`), extraer un helper compartido es más seguro que copiar la condición. En este caso, el drift entre `SubTabServicios` y `AdminEditorProvider` mantuvo vivo el 422 aunque el panel ya estaba corregido.

## Servicios publicos - un fallback estatico puede ocultar deuda real del CMS
- Si la vista pública cae a un dataset estático cuando la API devuelve planes vacíos, el sitio puede aparentar estar “bien” mientras el CMS refleja la realidad de la BD. Esa divergencia confunde el diagnóstico y retrasa la corrección del origen de datos real.
- En este repo, `SeccionPlanesServicio` usa `obtenerPlanesServicio(slug)` como fallback. Mientras exista, cualquier auditoría de catálogo debe distinguir entre “planes reales del backend” y “planes heredados del frontend”.

## Sync admin de servicios - preferir el slug activo y no tocar media vacia
- Si un servicio tiene aliases legacy en la misma BD, el sync por API no puede elegir el primer slug que coincida. Debe priorizar el registro `is_active = true`; en Nakomi, `diseno-de-sitios-web` es el servicio público activo y `diseno-web` quedó como legacy inactivo.
- Cuando la propuesta no trae `image_url` o `gallery`, el sync debe omitir esos campos o preservar la media existente. Enviar media vacía desde el cargador pisa información del CMS justo en la parte que el usuario quiere seguir gestionando manualmente.

## Capabilities del CMS - mas texto, mismo estado de publicacion
- Si el usuario pide ampliar las Capabilities de servicios, el cambio real vive en `skills.descripcion` del catalogo CMS. La validacion mas fiable es medir por API que esas descripciones crecieron y no solo confiar en una lectura visual indirecta.
- Cuando un servicio ya fue archivado manualmente, cualquier sync de copy debe respetar ese estado en la fuente reusable (`status = archived`, `is_active = false`) o lo reactivara en el siguiente empuje aunque el texto sea el unico objetivo del cambio.
