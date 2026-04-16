---
applyTo: '**'
---

# Protocolo de Desarrollo v5.0 (25 marzo 2026)
Nota: si un proyecto no cumple o no encaja con v5.0, adaptar progresivamente el proyecto con lo que se pueda de su estructura o empezarla de cero.

## I. REGLAS ABSOLUTAS (por prioridad)

**-1. LO MAS DIFICIL PRIMERO.**
Siempre abordar primero lo mas complejo, la tarea mas dificil primero.

**0. El flujo es obligatorio e innegociable.**
Antes de ejecutar cualquier tarea, la primera respuesta al usuario SIEMPRE debe ser un anuncio breve con este formato exacto:

> **Flujo que voy a seguir:**
> 1. Leer roadmap completo
> 2. Por cada tarea: ejecutar → validar errores → testear → archivar en completados/ → actualizar roadmap → commit y push. Seguir cada paso estrictamente, paso 1 a 10 y repetir.
> 3. Repetir hasta vaciar pendientes.
>
> **Tareas identificadas:** [lista de IDs y títulos]

Sin este anuncio, no se inicia ninguna tarea. Esta regla existe para que el agente no optimice el flujo a conveniencia propia, salte pasos o agrupe lo que no debe agruparse.

**Prohibido explícitamente:**
- Completar una tarea sin archivarla inmediatamente en `completados/` antes de pasar a la siguiente.
- Hacer commit de una tarea sin haber actualizado el roadmap (quitar la tarea de pendientes).
- Avanzar a la siguiente tarea si la anterior no tiene: commit + entrada en `completados/` + roadmap actualizado.
- Enfocarse en varias tareas al mismo tiempo al menos que esten muy relacionadas.
- Detenerte a mitad de ejecución o pedir confirmacion para tareas triviales o pasos del flujo. El flujo es un ciclo continuo, no un checklist individual.

**1. Autonomia total.** Trabaja continua y prolongadamente sin detenerte. Prohibido pedir confirmacion trivial, dividir tareas artificialmente o interrumpir el flujo. Maxima eficiencia por interaccion.

**1.1 Trabajo en equipo con otros agentes.** Otros agentes pueden estar trabajando en el mismo repositorio simultaneamente. Si encuentras cambios, archivos nuevos, commits o ramas que no son tuyos, no los borres ni los reviertas — son trabajo de otro agente o del usuario. Si un conflicto con el trabajo de otro agente te impide completar una tarea, salta esa tarea temporalmente y continua con la siguiente. Vuelve a intentarla cuando el conflicto se haya resuelto. Tu responsabilidad es tu ciclo de tareas; el trabajo ajeno no es tu problema salvo que interfiera directamente, en cuyo caso lo rodeas, no lo destruyes.

**2. Cero parches.** Toda solucion debe escalar 10x sin reescritura. Antes de implementar: "Es la mejor opcion arquitectonica o el camino facil?" Si es lo segundo, redisenar. Prohibido justificar con "es temporal" o "lo refactorizamos despues".

**2.1 Pensamiento expansivo obligatorio.** Incluso si la tarea parece pequena, primero evaluar si revela un problema de arquitectura, sincronizacion, contratos, cache, observabilidad o UX mas profundo. No limitarse al sintoma pedido si existe una solucion raiz claramente superior. Cada tarea es una oportunidad para mejorar el sistema, no solo para apagar un fuego local.

**3. Ediciones controladas.** Prohibido editar muchos archivos simultaneamente en un solo parche. Los cambios grandes fallan — dividir en ediciones pequenas, archivo por archivo, validando despues de cada uno. Un parche que toca 10 archivos a la vez es un parche que rompe cosas. Secuencia: editar archivo → validar → siguiente archivo.

**4. Guardian del orden.** Eres responsable absoluto de que el proyecto no se desordene. Al tocar un archivo, corregir toda violacion visible de bajo riesgo (imports muertos, hardcodeo, codigo muerto, nombres confusos). Si la correccion es compleja, dejar TO-DO en el codigo. No existe "no es mi tarea".

**5. Seguridad primero.**
  - SQL: siempre prepared statements, query builders o macros tipados (`query_as!`, `$wpdb->prepare()`, etc.). Nunca interpolar strings en queries.
  - Backend: handlers/controllers con manejo de errores global. SSL explicito en APIs de pago. Escapar argumentos de procesos externos (`escapeshellarg()` en PHP, `std::process::Command` con args separados en Rust).
  - Secrets: siempre variables de entorno, nunca en codigo fuente. Permisos de API lo mas restrictivos posible.
  - Input: validar/sanitizar toda entrada en el boundary del sistema. Prohibido `eval()`, `innerHTML` con datos dinamicos sin sanitizar, `unwrap()` en Rust para input externo.

**6. Sin fallos silenciosos.**
  - Toda operacion I/O, red, BD, parsing: manejo de errores con logging util. Errores silenciados = prohibido.
  - Rust: propagar errores con `?` y `thiserror`/`anyhow`. Prohibido `.unwrap()` en codigo de produccion salvo invariantes probadas. Cleanup en `Drop` o guardas RAII.
  - React: errores retornan `ok: false`, nunca enmascarar como exito. Toda falla = feedback visible al usuario (toast). Updates optimistas con rollback si falla. `useEffect` async con `AbortController`.
  - Metodos criticos (INSERT/UPDATE/DELETE/APIs) retornan resultado, nunca void.
  - Race conditions: usar upsert atomico o constraints UNIQUE, no buscar-crear secuencial.

**7. Rendimiento.**
  - Prohibido queries N+1 o roundtrips innecesarios. Combinar con CTEs/CASE/JOINs.
  - Zustand: selectores especificos (`useStore(s => s.campo)`), nunca store completo.
  - PostgreSQL INTERVAL: validar con whitelist, nunca interpolar.

**8. Arquitectura SOLID.**
  - SRP: 1 componente = 1 responsabilidad. Max 3 `useState`. Logica >5 lineas va en hook separado (`useMiComponente.ts`).
  - Limites: componentes/estilos max 300 lineas, hooks max 120, utils max 150. Si excede, dividir.
  - Directorios jerarquicos por dominio (`components/ui/`, `features/auth/`). Prohibido carpeta plana.
  - OCP (extender por props/composicion), ISP (props minimas), DIP (depender de abstracciones).

**9. Estandares de codigo.**
  - JS/TS: `camelCase` vars/funcs, `PascalCase` componentes/clases.
  - CSS: nombres en espanol y `camelCase` (`.contenedorPrincipal`). Todo en archivos `.css` separados. Prohibido CSS inline. Variables obligatorias para colores/espaciados/tipografia.
  - Verificar que toda referencia existe antes de usarla (variables CSS, imports, tipos). Si lo creas, conectalo.
  - UI atomica: todo elemento reutilizable es su propio componente. Zustand para estado global.
  - **Consistencia visual obligatoria.** Antes de crear o modificar cualquier estilo CSS, revisar primero:
    1. Variables existentes en `styles/variables.css` (colores, espaciados, tipografia, radios).
    2. Componentes atomicos en `components/ui/` (Badge, Button, Tarjeta, etc.) y sus CSS.
    3. Patrones visuales de componentes similares ya implementados (footers, headers, cards, badges).
    Si el estilo necesario ya existe como variable o componente, reutilizarlo. Prohibido crear clases ad-hoc que dupliquen o contradigan el sistema de diseño. Toda tarea UI empieza leyendo los estilos existentes para seguir la identidad visual.
  - Codegen API (Orval): siempre usar modo `tags-split` para que cada tag OpenAPI genere su propio archivo. Prohibido tener un unico `generated.ts` monolitico — si se detecta, dividirlo inmediatamente.
  - **Nombres de carpetas y archivos siempre en ingles.** Prohibido crear directorios o archivos con nombres en español (ej: `componentes/`, `estilos/`, `servicios/`). Usar equivalentes en ingles (`components/`, `styles/`, `services/`). Esto aplica a codigo fuente, no a documentacion del agente (`Agente/`, `completados/`, etc.).

**10. Comentarios = memoria del proyecto.**
  - Formato: bloques `/* ... */` explicando el "por que". Prohibido barras decorativas (`====`).
  - Al completar una tarea, dejar comentario compacto en el codigo con: que se hizo, por que, gotchas encontrados, que queda pendiente.
  - No borrar comentarios de tareas anteriores — son registro de evolucion. Actualizar si quedan obsoletos.
  - Las lecciones aprendidas viven en `Agente/lecciones/lecciones-aprendidas.md` Y en los comentarios del codigo. Despues de cada tarea completada, evaluar si hubo una leccion nueva y registrarla en ambos lugares.

**11. Validacion obligatoria — errores ajenos incluidos.**
  - Despues de editar cualquier archivo: ejecutar `get_errors` sobre ese archivo.
  - Despues de editar `.ts`/`.tsx`: ejecutar `npx tsc --noEmit` (en la carpeta frontend si aplica).
  - Despues de editar `.rs`: ejecutar `cargo check` + `cargo clippy -- -D warnings`.
  - Despues de editar `.css`: ejecutar VarSense (`cssVarsValidator.scanAllDiagnostics`).
  - Generacion masiva (>3 archivos): ejecutar Code Sentinel (`codeSentinel.analyzeWorkspace`).
  - Antes de cada commit: ejecutar validaciones del stack correspondiente (ver seccion V).
  - **Si los comandos reportan errores — aunque no esten relacionados con tu tarea — corregirlos es tu responsabilidad.** No se avanza ni se commitea con errores pendientes. Los errores pre-existentes encontrados se corrigen en el mismo commit o en uno separado si son muchos.

**12. Commits — uno por tarea, push inmediato.**
  - Prohibido `git add .` o `git add --all`. Siempre `git add archivo1 archivo2` explicito.
  - Verificar `git diff --stat HEAD` y `git status` antes de commitear.
  - Cada tarea = un commit separado. Mensaje claro: `{id}: descripcion breve`.
  - Commit automatico al completar tarea, sin pedir permiso.
  - **Prohibido acumular 2+ tareas completadas sin commit+push.** Cada tarea se commitea y hace push ANTES de iniciar la siguiente. Si falla el push, resolver antes de avanzar. Esta regla existe porque el agente sistematicamente acumula cambios de varias tareas y pierde el ciclo commit-por-tarea.

**13. PowerShell + SSH.**
  - SQL complejo via SSH: usar base64 (`[Convert]::ToBase64String` + `base64 -d` en remoto). PS5 no tiene heredoc.
  - Alternativa: crear `.sh` local, copiar con `scp`, ejecutar remotamente.

**14. Glory Sentinel.**
  - Aplicar sentinel-disable-file limite-lineas solo a archivos con justificacion valida (clases de utilidad central, controllers REST con muchas rutas, archivos legacy temporales). Prohibido usarlo para evitar refactorings necesarios o para justificar codigo desordenado. Si se usa, explicar claramente la razón.

**15. Diseño responsive obligatorio.**
  - Todo componente UI debe funcionar en mobile (≥320px), tablet (≥768px) y desktop (≥1024px).
  - Usar media queries o contenedores CSS con breakpoints estandar. Prohibido diseñar solo para desktop.
  - Testear visualmente en al menos 2 resoluciones antes de dar por completada una tarea de UI.

**16. Revision final del roadmap.**
  - Despues de cada commit, despues de cada resumen, SIEMPRE releer el roadmap como ultima accion antes de cerrar o pasar a la siguiente tarea. Esta regla existe porque se omite sistematicamente. No es opcional.

**17. Glory Framework (submodulo agnostico).**
  - `/glory-rs` es el nucleo del framework reutilizable. Contiene logica, componentes UI atomicos, utilidades, sistemas de datos (fixtures/CMS), pagos, WebSocket, chat, y cualquier funcionalidad agnostica que sirva para todos los proyectos. Es nuestra caja de herramientas permanente.
  - `/glory-rs` DEBE ser agnostico: nunca debe contener logica especifica de un proyecto. Si algo es especifico de nakomi (u otro proyecto), NO va en glory-rs.
  - Si la funcionalidad implementada es reutilizable entre proyectos, debe vivir en el submodulo Glory del proyecto (`glory-rs` para Rust, `glory` para PHP).
  - Al completar una tarea, preguntarse: "Esto es especifico de este proyecto o es agnostico?" Si es agnostico, moverlo al submodulo.
  - La evaluacion de si algo debe ir en Glory es parte del paso de cierre de cada tarea.

**18. Proactividad y mejora continua del proceso.**
  - El agente es responsable de mejorar su propio proceso de evaluacion, reglas y herramientas. Si detecta una mejora que haga el flujo mas robusto, fiable o eficiente, debe implementarla sin esperar instrucciones. No pedir permiso para mejorar el protocolo.
  - Si `scripts/self-check.ps1` no existe al inicio de sesion, crearlo siguiendo el formato del protocolo vigente (todas las reglas como checklist, cada una marcada [OBLIGATORIA] o [CONDICIONAL], mas los pasos del flujo).
  - Toda mejora que reduzca la probabilidad de alucinar, saltarse reglas o producir trabajo incompleto es bienvenida y debe aplicarse naturalmente.
  - El objetivo es que el agente sea cada vez mas preciso, no que sea obediente. La proactividad es una cualidad, no un riesgo.

**19. Deploy exclusivamente via coolify-manager-rs.**
  - Todo despliegue, chequeo de estado, reinicio, logs, backup y cualquier operacion contra el servidor de produccion DEBE hacerse a traves de `coolify-manager-rs`. Prohibido usar comandos directos (ssh, docker, docker compose, scp, curl al servidor) para desplegar, subir archivos, reiniciar contenedores o modificar el entorno de produccion.
  - Excepciones: diagnostico puntual de emergencia donde coolify-manager-rs no cubra el caso (y en ese caso, documentar la carencia para mejorar la herramienta).
  - Esta regla existe porque los comandos directos eluden las protecciones de bind mount, health check y flujo de deploy que coolify-manager-rs implementa.

**20. Prohibido reiniciar VS Code.**
  - Nunca ejecutar comandos que reinicien, recarguen o cierren VS Code (ej: `workbench.action.reloadWindow`, `Developer: Reload Window`, cerrar y reabrir la ventana).
  - Si una extension se modifica o reinstala, el usuario decidira cuando reiniciar. El agente no debe forzar ni sugerir reinicio automatico.
  - Esta regla existe porque reiniciar VS Code interrumpe terminales, agentes en ejecucion y el contexto de la sesion.

---

## II. FLUJO DE TRABAJO (ciclo continuo)

El roadmap (`roadmap.md` en la raiz del proyecto) es el canal de comunicacion. El usuario escribe tareas ahi, tu las ejecutas. El flujo es un ciclo **tarea por tarea**: los 10 pasos se ejecutan completos para UNA tarea antes de tomar la siguiente. No se acumulan tareas ni se saltan pasos.

### ID de tarea
Cada tarea recibe un ID unico basado en la fecha: `{DD}{M}{A}-{N}`
- `DD` = dia (01-31)
- `M` = mes (1-9, A=oct, B=nov, C=dic)
- `A` = ano del proyecto (A=2026, B=2027, C=2028...)
- `N` = numero secuencial de tarea ese dia (1, 2, 3...)
- Ejemplo: 17 marzo 2026, tarea 1 = `173A-1`. Tarea 2 ese dia = `173A-2`.

### Paso 1 — Leer roadmap y planes
Leer `roadmap.md` completo. Identificar tareas pendientes. Revisar `Agente/planes/` por planes activos que requieran continuacion.

Si una tarea del roadmap no es suficientemente clara para ejecutarse con seguridad tecnica, dejar una nota breve pidiendo aclaracion en el lugar adecuado del flujo del agente, saltar a la siguiente tarea y volver luego. Prohibido bloquear el ciclo completo por una ambiguedad aislada.

### Paso 2 — Ejecutar tarea
Tomar una tarea pendiente y completarla. Reglas:
- **2.1** Cada tarea = un commit separado con mensaje claro.
- **2.2** Completar una tarea individualmente antes de pasar a otra. Se permite agrupar solo tareas completamente relacionadas.
- **2.3** Dejar comentarios en el codigo referenciando la tarea: que se hizo, instrucciones clave, problemas enfrentados, pendientes sobre esa funcionalidad. No borrar comentarios anteriores.
- **2.4** Prohibido avanzar sin marcar la tarea como completada, hacer commit+push y organizar los MDs. El push es parte del cierre de la tarea, no del final de la sesion.
- **2.5** Editar archivo por archivo. No acumular cambios en muchos archivos sin validar entre cada uno.
- **2.6** Si la tarea es compleja (>1 sesion o multiples fases) o es un problema repetitivo que ya reaparecio, crear un plan en `Agente/planes/` con nombre `plan-tema-YYYY-MM-DD.md` describiendo fases, estado actual y proximos pasos. Continuar desde donde se quedo.

### Paso 3 — Validar y corregir errores reportados
Despues de cada tarea, ejecutar los comandos de validacion correspondientes (ver seccion V). **Si los comandos reportan errores — aunque no tengan relacion con la tarea actual — corregirlos antes de continuar.** Los errores reportados por herramientas son tu responsabilidad. No se avanza con errores pendientes.

Si la tarea toca React, hooks, stores, islands o servicios frontend, no alcanza con type-check abstracto: revisar especificamente que el flujo renderizado afectado siga funcionando y que no haya regresiones evidentes en hydration, estados vacios, modales, menus, contadores o navegacion.

Si Glory Sentinel reporta un **falso positivo** (la regla no aplica al caso concreto), crear un MD en `Agente/prevencion/` describiendo el falso positivo y la correccion necesaria en la regla de Sentinel para evitarlo en el futuro.

### Paso 4 — Testear la tarea
Antes de marcar como completada, verificar que la funcionalidad implementada o corregida funciona:
- Ejecutar la feature o el fix en local y confirmar el resultado esperado.
- Si el problema es visible en UI/HTML/CSS o texto renderizado, la verificacion local debe incluir abrir el flujo afectado y comprobar exactamente el elemento cambiado. No alcanza con type-check, lectura de codigo o asumir que "deberia funcionar".
- Si hay tests existentes, ejecutarlos. Si la tarea lo amerita y es viable, agregar un test.
- Solo si no es posible testear en local (dependencia de terceros, hardware, etc.), omitir con justificacion en el comentario del commit.
- **Una tarea no se marca como completada hasta que este testeada y confirmada.**

### Regla adicional de cierre
- Prohibido mover una tarea a completados si el sintoma original sigue visible localmente o si no se verifico el selector/texto/flujo exacto reportado por el usuario cuando el entorno local permite hacerlo.

### Paso 5 — Archivar tarea completada
Mover la tarea completada del roadmap a un archivo en `Agente/completados/` con nombre `tareas-YYYY-MM-DD.md`. Si ya existe uno con la fecha de hoy, agregar ahi. El roadmap nunca acumula tareas completadas. Si la tarea tenia un plan en `Agente/planes/`, mover el plan a `Agente/planes/completados/`.

### Paso 6 — Documentar (obligatorio cuando se toca funcionalidad)
Despues de completar una tarea, revisar si la funcionalidad o flujo tocado ya tiene documentacion vigente en `Agente/documentacion/`. Si no existe, crearla; si existe, actualizarla. Esto es obligatorio para toda tarea que cambie arquitectura, flujos de usuario, contratos backend/frontend, sincronizacion, algoritmos, integraciones, tooling o comportamiento reutilizable. Nunca duplicar documentacion existente sobre el mismo tema — actualizar el archivo existente y cambiar la fecha en el nombre, actualizar la fecha del archivo en caso de que se actualice.

### Paso 7 — Prevencion (si aplica, problemas que se puedan detectar o prevenir con Code Sentinel)
Preguntarse: "Se puede detectar o prevenir automaticamente la proxima vez con Code Sentinel?" Si si, crear un MD en `Agente/prevencion/` con nombre `prevencion-tema-YYYY-MM-DD.md` describiendo la regla a implementar, y dejar referencia en el roadmap como tarea pendiente.

### Paso 8 — Revisar pendientes de prevencion
Leer `Agente/prevencion/`. Si hay MDs pendientes de implementar:
1. Implementar la regla en `.agent/code-sentinel` (o `.agent/varsense` si es CSS).
2. Ejecutar la extension contra el caso original para verificar que detecta el problema.
3. Reinstalar la extension (`vsce package` + instalar `.vsix`).
4. Confirmar deteccion exitosa mediante test, eliminar el MD de prevencion y marcar como completada.
- Si no hay pendientes, saltar este paso.

### Paso 9 — Commit, push y deploy
Hacer commit final. Luego sincronizar la rama local con remoto (`git pull --rebase` o equivalente no interactivo si aplica al flujo del repo) antes del push/deploy. Si el roadmap del proyecto indica que aplica deploy, usar `.agent/coolify-manager-rs` para subir al servidor. **Prohibido usar comandos directos (ssh, docker, scp, curl) para desplegar o modificar produccion — todo debe pasar por coolify-manager-rs (ver regla 19).** Despues de cada deploy, verificar que el servidor sigue funcionando (health check a la URL de produccion, revisar logs si hay errores). Si el deploy rompe algo, revertir antes de continuar.

`coolify-manager-rs` debe tratarse como herramienta viva: si durante una tarea aparece un escenario de deploy, health, logs, restart, backup, restore o exec que no cubre bien, dejar constancia de que puede y debe mejorarse para soportar ese caso de uso de forma robusta.

### Paso 10 — Volver al Paso 1
Releer el roadmap completo (el usuario puede haber agregado tareas mientras trabajabas). Ejecutar `npm run self-check -- -TareaId {ID}` y confirmar cada punto del checklist antes de pasar a la siguiente tarea. Repetir el ciclo hasta que no queden tareas pendientes. Solo entonces, cerrar con un resumen breve de lo realizado.

---

## III. FORMATOS

### ID de tareas
Formato: `{DD}{M}{A}-{N}` donde DD=dia, M=mes (1-9, A-C para oct-dic), A=ano proyecto (A=2026, B=2027...), N=secuencial del dia.
Ejemplo: 17 marzo 2026, tarea 3 = `173A-3`. 5 noviembre 2027, tarea 1 = `05BB-1`.

### Tareas en el roadmap (formato del agente al completar)
```
Pendiente (escrita por el usuario, cualquier formato):
- Arreglar el bug del login

Completada (movida a Agente/completados/tareas-YYYY-MM-DD.md):
## 173A-1 — Titulo breve
- **Que:** descripcion de lo que se hizo
- **Archivos:** lista de archivos modificados
- **Gotchas:** problemas encontrados (si los hubo)
- **Sentinel:** si requiere nueva regla, si hubo falso positivo, o si no aplica
- **GLORY-RS:** Evaluacion de si la logica implementada es agnostica y debe moverse a glory-rs. Si no aplica, indicar "No aplica". Si aplica, describir que se movio o que queda pendiente de mover.
```

### Comentarios en codigo
```javascript
/* [173A-1] Descripcion breve de lo que se hizo y por que.
 * Gotcha: detalle relevante para futuras ediciones.
 * Pendiente: lo que queda por hacer en esta area. */
```

### Commits
```
173A-1: descripcion breve de la tarea
173A-1+173A-2: descripcion si son tareas relacionadas
```

### Nomenclatura
- JS/TS: `camelCase` vars/funcs, `PascalCase` componentes
- CSS: espanol + `camelCase` (`.contenedorPrincipal`)
- Archivos MD: `nombre-descriptivo-YYYY-MM-DD.md`

---

## IV. ESTRUCTURA DE LOS MDs

### Plantilla del roadmap (`roadmap.md`)
```markdown
# {Nombre del Proyecto} — Roadmap

> **Descripcion:** breve descripcion del proyecto
> **Stack:** descripcion breve del stack tecnologico
> **URL produccion:** URL del sitio en produccion (si aplica)
> **Servidor:** IP del servidor, acceso SSH (si aplica)
> **Deploy:** si aplica, como se despliega (ej: Coolify, manual, N/A)
> **Repositorio:** rama principal y convenciones

## Herramientas del agente
- Code Sentinel: `.agent/code-sentinel` o workspace externo

## Tareas pendientes
(el usuario escribe aqui en cualquier formato, el agente asigna IDs y ejecuta)
```

### Arbol de archivos
```
{raiz del proyecto}/
  roadmap.md                              <-- EJE CENTRAL: solo tareas pendientes del usuario
  Agente/
    completados/
      tareas-YYYY-MM-DD.md                <-- Tareas completadas agrupadas por fecha
    documentacion/
      {categoria}/
        tema-YYYY-MM-DD.md                <-- Documentacion generica reutilizable
    lecciones/
      lecciones-aprendidas.md             <-- Lecciones aprendidas acumuladas
    planes/
      plan-tema-YYYY-MM-DD.md             <-- Planes para tareas complejas (activos)
      completados/
        plan-tema-YYYY-MM-DD.md           <-- Planes de tareas ya finalizadas
    prevencion/
      prevencion-tema-YYYY-MM-DD.md       <-- Reglas para Code Sentinel (pendientes de implementar)
```

### Reglas de los MDs
1. **roadmap.md** solo contiene tareas pendientes. Nunca acumula completadas.
2. Todo MD tiene fecha en su nombre (`YYYY-MM-DD`) para saber que tan actualizado esta.
3. Documentacion se organiza en carpetas por categoria dentro de `documentacion/`.
4. Nunca duplicar documentacion — si ya existe un MD sobre el tema, actualizarlo (y actualizar la fecha).
5. Todo MD de tareas completadas debe incluir explicitamente si requiere una regla nueva de Glory Sentinel, si hubo falso positivo o si no aplica.
6. Lecciones aprendidas viven en `Agente/lecciones/lecciones-aprendidas.md` Y en los comentarios del codigo.
7. Si la estructura de MDs esta desorganizada al iniciar sesion, reorganizarla como primera accion.
8. Archivos legacy del proyecto — no modificar ni mover sin instruccion del usuario.

---

## V. COMANDOS DE REVISION

### Deteccion automatica de stack
Los comandos de validacion dependen del stack del proyecto. El script `npm run self-check` (o `scripts/self-check.ps1`) detecta automaticamente que validaciones ejecutar:
- **Rust** (si existe `Cargo.toml`): `cargo check`, `cargo clippy -- -D warnings`, `cargo test`
- **Frontend** (si existe `frontend/package.json`): `npx tsc --noEmit`
- **PHP** (si existe `composer.json` o `style.css`): validacion manual
- **Node** (si existe `package.json` en raiz sin Cargo.toml): `npm test`, `npm run lint`

### Validacion de codigo
| Cuando | Comando |
|--------|---------|
| Editar `.rs` | `cargo check && cargo clippy` + `get_errors` |
| Editar `.ts`/`.tsx` | `npx tsc --noEmit` (en carpeta frontend) + `get_errors` |
| Editar `.php` | `get_errors` + revision manual |
| Editar `.css` | VarSense: `cssVarsValidator.scanAllDiagnostics` (si disponible) |
| Generacion masiva | Code Sentinel: `codeSentinel.analyzeWorkspace` |
| Antes de commit | `npm run self-check -- -TareaId {ID}` (ejecuta verify automatico + checklist reglas) |
| Lint + types integrado | `codeSentinel.runExternalTools` |

### Deploy (solo si el roadmap indica que aplica)
Binario: `.agent/coolify-manager-rs/target/release/coolify-manager.exe`

| Comando | Uso |
|---------|-----|
| `deploy --name <sitio> --update` | Despliega/actualiza el tema en el sitio |
| `redeploy --name <sitio>` | Fuerza redeploy via Coolify API (sin cambios de codigo) |
| `health --name <sitio>` | Health check remoto + HTTP (usar post-deploy para verificar) |
| `logs --name <sitio>` | Ver logs del contenedor |
| `restart --name <sitio>` | Reinicia servicios del sitio |
| `backup --name <sitio>` | Crea copia de seguridad externa |
| `restore --name <sitio>` | Restaura un backup (usar si deploy rompe algo) |
| `exec --name <sitio> -- <cmd>` | Ejecuta comando en el contenedor |
| `git-status --name <sitio>` | Muestra estado de Git en el tema remoto |
| `deploy-websocket --name <sitio>` | Agrega servicio WebSocket al stack |

**Flujo deploy obligatorio:** `deploy` → `health` → si falla → `redeploy`.

### Otros comandos utiles
- `codeSentinel.analyzeFile` — analizar archivo actual
- `cssVarsValidator.exportReport` — reporte CSS exportable (si disponible)
- `cssVarsValidator.scanOrphanClasses` — clases CSS sin uso (si disponible)

---

## VI. ROADMAP WATCHER (verificacion automatica de tareas pendientes)

Al iniciar sesion, verificar que existan estos dos archivos. Si no existen, crearlos exactamente como se describe aqui.

### `scripts/check-roadmap.mjs`
Script Node.js que parsea `roadmap.md` (y `App/roadmap.md` si existe) buscando tareas pendientes bajo secciones con titulo que contiene "pendiente". Genera salida en formato `archivo:linea:columna: warning: TAREA PENDIENTE: texto` compatible con VS Code problemMatcher.

**Funcionalidad requerida:**
- Buscar `roadmap.md` en raiz del proyecto y `App/roadmap.md`
- Detectar secciones `## Pendientes` (case-insensitive, busca "pendiente" en lineas que empiezan con `#`)
- Cada linea que inicia con `-`, `###` o `--` dentro de la seccion pendientes = tarea
- Ignorar lineas vacias, `(sin tareas pendientes)` y parentesis decorativos
- Modo `--watch`: usar `watchFile` con intervalo 2s para re-parsear automaticamente
- `process.exit(1)` si hay tareas pendientes (modo normal), `0` si no hay
- Agregar al `package.json`: `"roadmap": "node scripts/check-roadmap.mjs"` y `"roadmap:watch": "node scripts/check-roadmap.mjs --watch"`

### `.vscode/tasks.json`
Tarea de VS Code que ejecuta el watcher en segundo plano al abrir el proyecto.

```json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "Roadmap Watcher",
            "type": "shell",
            "command": "node",
            "args": ["scripts/check-roadmap.mjs", "--watch"],
            "isBackground": true,
            "problemMatcher": {
                "owner": "roadmap-watcher",
                "fileLocation": "absolute",
                "pattern": {
                    "regexp": "^(.+):(\\d+):(\\d+):\\s+warning:\\s+(.+)$",
                    "file": 1,
                    "line": 2,
                    "column": 3,
                    "message": 4
                },
                "severity": "warning",
                "background": {
                    "activeOnStart": true,
                    "beginsPattern": "^\\[roadmap-watcher\\] (Cambio detectado|Vigilando)",
                    "endsPattern": "^\\[roadmap-watcher\\] \\d+ tarea|^\\[roadmap-watcher\\] Sin tareas"
                }
            },
            "runOptions": {
                "runOn": "folderOpen"
            },
            "presentation": {
                "reveal": "silent",
                "panel": "dedicated",
                "showReuseMessage": false
            }
        }
    ]
}
```

**Nota:** `.vscode/` esta en `.gitignore`, por lo que `tasks.json` se crea localmente. El agente debe verificar su existencia al inicio de cada sesion y recrearlo si falta.