# Plan: Diagnostico y correccion de latencia en hostings de prueba VPS2

> **Fecha:** 2026-05-23
> **Origen:** Comparativa pedida entre dos hostings de prueba en VPS2 y referencias equivalentes de VPS1.
> **Estado:** **Actualizado 2026-05-24**. El fix del panel de disco ya fue desplegado en `studio`. El `504` del hosting estatico VPS2 dejo de reproducirse. El WordPress de VPS2 sigue lento, pero el cuello quedo acotado al host/stack interno, no a la red publica.

---

## Resumen ejecutivo

Los dos sitios lentos de VPS2 no terminaron en el mismo diagnostico:

- `site-hosting-3212defd.VPS2_IP.sslip.io` antes devolvia `504 Gateway Timeout` a los ~31s, pero durante la verificacion runtime posterior quedo respondiendo `200` de forma estable en `~0.43s - 1.18s`.
- `wordpress-hosting-de17015b.VPS2_IP.sslip.io` sigue respondiendo `200`, pero con TTFB muy irregular y muy por encima de referencias sanas de VPS1.

La separacion por stack sigue siendo importante porque por codigo no son equivalentes:

- `normal-basico` usa Nginx estatico + SFTP sobre `site-data:/usr/share/nginx/html` en [src/services/coolify.rs](src/services/coolify.rs#L685).
- `basico` usa WordPress + MariaDB en [src/services/coolify.rs](src/services/coolify.rs#L672).

---

## Medicion base

### Sitios bajo diagnostico

| URL                                                         | Stack           | VPS                  | HTTP  | Connect |     TTFB |    Total | Lectura                                                                            |
| ----------------------------------------------------------- | --------------- | -------------------- | ----- | ------: | -------: | -------: | ---------------------------------------------------------------------------------- |
| `http://site-hosting-3212defd.VPS2_IP.sslip.io/`      | `normal-basico` | VPS2 | `504` | `0.42s` | `31.67s` | `31.67s` | El proxy/ruta responde rapido, pero el upstream no entrega nada antes del timeout. |
| `http://wordpress-hosting-de17015b.VPS2_IP.sslip.io/` | `basico`        | VPS2 | `200` | `0.17s` |  `2.09s` |  `2.40s` | Funciona, pero el backend tarda bastante mas en renderizar que referencias sanas.  |

### Referencias VPS1

| URL                           | Stack aproximado | VPS                  | HTTP  | Connect |    TTFB |   Total |
| ----------------------------- | ---------------- | -------------------- | ----- | ------: | ------: | ------: |
| `https://cap.wandori.us/`     | `normal-basico`  | VPS1 | `302` | `0.08s` | `0.42s` | `0.42s` |
| `https://task.nakomi.studio/` | WordPress        | VPS1 | `200` | `0.09s` | `0.49s` | `0.57s` |

### Medicion repetida previa

- `site-hosting-3212defd` repitio `504` en ~30.57s, ~30.68s y ~30.79s.
- `wordpress-hosting-de17015b` bajo de ~4.44s a ~2.17s y luego ~1.00s en corridas sucesivas.

### Medicion actualizada tras diagnostico runtime

| URL                                                         | Contexto                | HTTP  | Connect |     TTFB |    Total | Lectura |
| ----------------------------------------------------------- | ----------------------- | ----- | ------: | -------: | -------: | ------- |
| `http://site-hosting-3212defd.VPS2_IP.sslip.io/`      | 5 corridas externas     | `200` | `0.19s` - `0.27s` | `0.43s` - `1.13s` | `0.43s` - `1.18s` | El `504` ya no fue reproducible; el sitio estatico quedo estable. |
| `http://wordpress-hosting-de17015b.VPS2_IP.sslip.io/` | 5 corridas externas     | `200` | `0.16s` - `0.25s` | `0.63s` - `26.76s` | `0.97s` - `27.14s` | La latencia sigue siendo muy variable y puede saltar a decenas de segundos. |
| `http://127.0.0.1/` dentro del contenedor WordPress         | 5 corridas internas     | `200` | n/a | `1.30s` - `6.18s` | `1.31s` - `6.21s` | La lentitud tambien existe dentro del contenedor; no es solo ubicacion o proxy publico. |
| `https://task.nakomi.studio/`                               | Referencia VPS1 externa | `200` | `0.09s` - `0.52s` | `0.44s` - `1.95s` | `0.54s` - `2.21s` | VPS1 sigue claramente mas estable. |

### Medicion comparativa directa de host desde ambos WordPress

Se tomo la misma prueba desde dentro del contenedor WordPress de `nakomi` en VPS1 y desde `hosting-de17015b` en VPS2:

- 3 ventanas de `busy/iowait/steal` desde `/proc/stat`
- 3 escrituras de `32 MB` con `dd ... conv=fdatasync` sobre `wp-content/uploads`

Resultado:

| Metrica | VPS1 `task.nakomi.studio` | VPS2 `wordpress-hosting-de17015b` | Lectura |
| ------- | ------------------------- | --------------------------------- | ------- |
| `cpu_count` | `4` | `4` | Mismo tamano nominal de VPS. |
| `loadavg` observado | `1.07 / 1.69 / 2.55` | `10.99 / 7.71 / 6.46` | VPS2 tenia mucha mas contencion runnable en ese momento. |
| `busy_pct` (3 muestras) | `12.12%`, `6.78%`, `8.55%` | `38.89%`, `34.33%`, `81.25%` | VPS2 estaba claramente mas ocupada por CPU/espera activa. |
| `iowait_pct` (3 muestras) | `0%`, `0%`, `0%` | `12.50%`, `0%`, `2.08%` | VPS2 si mostro espera de I/O; VPS1 no. |
| `steal_pct` (3 muestras) | `0%`, `0%`, `0%` | `0%`, `0%`, `0%` | No hubo evidencia de steal en esta toma puntual. |
| Escritura `32 MB fdatasync` | `299 ms`, `164 ms`, `95 ms` | `2724 ms`, `2428 ms`, `2862 ms` | El storage visible desde VPS2 fue aproximadamente `9x - 30x` mas lento. |

Conclusion directa de esta prueba:

- La diferencia ya no depende de la geografia publica ni del numero nominal de sitios.
- En esta toma, VPS2 estaba objetivamente peor en carga y en latencia de escritura a disco.
- Eso explica por que un WordPress vacio en VPS2 puede ir peor que un WordPress normal en VPS1.

Lectura operativa:

- En el sitio `normal-basico`, la red y el DNS no son el cuello de botella; el tiempo se pierde integro esperando al backend.
- En el WordPress, hay mejora fuerte entre corridas, asi que existe componente de cold start/cache/calientamiento.

---

## Hipotesis de trabajo

### H1. `hosting-3212defd` tenia un problema intermitente de runtime/upstream, no un problema estructural de peso de aplicacion

Estado actual:

- Durante el diagnostico posterior, el sitio dejo de reproducir el `504` y paso a responder `200` de forma consistente.
- Los access logs del contenedor Nginx mostraron la peticion externa llegando normalmente cuando se revalido.
- Conclusion operativa: el incidente del sitio estatico no quedo activo al cierre de esta fase. Puede haber sido un problema transitorio de routing/runtime, pero no quedo evidencia de lentitud persistente del stack estatico.

Fundamento historico:

- El plan `normal-basico` monta un Nginx estatico muy simple. No deberia acercarse a 30s salvo que el contenedor no este sirviendo, el router apunte mal o el proceso quede bloqueado en startup.
- El connect es rapido y el timeout cae al limite tipico de reverse proxy. Eso apunta a upstream no respondiendo, no a latencia de red.
- El uso de disco medido para ese deployment es minimo (`1 MB`), asi que no hay una carga de contenido que justifique espera.

Hipotesis concretas a verificar:

- El contenedor `site-{uuid}` no esta healthy o reinicia.
- Traefik/Coolify enruta al servicio correcto pero el proceso `nginx` no queda listo.
- El comando de arranque queda bloqueado en la manipulacion del volumen antes de ejecutar `nginx -g "daemon off;"`.
- El volumen `site-data` o el compose procesado en Coolify no coincide con lo esperado y deja al servicio sin ruta valida.

### H2. `hosting-de17015b` esta funcional pero el cuello no es la red publica ni MariaDB; queda dentro del host/stack WordPress de VPS2

Fundamento:

- El sitio responde `200` y mejora bastante al repetir la peticion.
- Sirve el WordPress default con Twenty Twenty-Five; no hay evidencia de plugins pesados ni de una capa de cache comparable a VPS1.
- Aun cuando la peticion se hace desde dentro del propio contenedor a `127.0.0.1`, el render sigue tardando varios segundos.
- La instrumentacion del render dio `~9023ms` totales con solo `~703ms` acumulados en SQL.
- La VPS2 expuso `4` vCPU y un `load average` sostenido de `~7.5 - 8.3`, con RAM libre amplia y sin swap.
- La comparativa directa WordPress vs WordPress mostro `iowait` real en VPS2 y escrituras `32 MB fdatasync` de `~2.4s - 2.9s`, frente a `~0.095s - 0.299s` en VPS1.

Hipotesis concretas a verificar:

- El host VPS2 tiene contencion real de CPU o tareas runnable/I/O wait suficientes para degradar PHP, aunque no este "lleno" de RAM o disco.
- El WordPress de prueba no tiene una capa de cache/pagina caliente comparable a la referencia de VPS1.
- La diferencia de ubicacion no explica el problema principal, porque la lentitud tambien aparece dentro del propio contenedor.

---

## Fases del plan

### F0. Inspeccion runtime de VPS2

Objetivo: convertir la hipotesis del `504` en evidencia directa.

Checks obligatorios:

- Estado real del stack de VPS2: `docker compose ps`, reinicios, health y si existe el contenedor `site-{uuid}`.
- Logs recientes del servicio `site` y del proxy/Traefik asociado.
- `curl` local desde el propio VPS contra el upstream del sitio para separar router externo de backend interno.
- Verificar compose procesado on-disk de Coolify y labels resultantes.
- Confirmar que el volumen `site-data` esta montado donde espera el compose.

Estado:

- F0 quedo sustancialmente completada via `coolify-manager-rs` con config temporal para los dos hostings de prueba.
- Se pudo leer logs del sitio estatico y del WordPress, ejecutar comandos dentro del contenedor WordPress y validar el contenedor estatico con un binario corregido del manager (`bash -> sh` fallback para Alpine).

### F1. Reparacion de `hosting-3212defd`

Estado actual:

- No se aplico reparacion porque el fallo ya no fue reproducible.
- Se deja como incidencia intermitente pendiente solo si vuelve a aparecer el `504` de ~30s.

### F2. Mejora de TTFB en `hosting-de17015b`

Objetivo: bajar el primer byte hacia el orden de `~0.5s - 1.0s` en peticion caliente.

Checks ya realizados:

- Comparados tiempos externos contra referencia VPS1.
- Medido `curl` interno a `127.0.0.1` dentro del contenedor WordPress.
- Medido costo DB con `mysqli`: `SELECT 1` en milisegundos y autoload options en `~116ms - 376ms`.
- Instrumentado render completo de WordPress para separar tiempo total vs SQL.
- Probado `DISABLE_WP_CRON`; no mejoro y se revirtio.

Siguiente paso correcto:

- Si el objetivo es que deje de ser lento, no alcanza con "esperar que caliente". Hay que mover ese hosting a una VPS sin esa contencion o reducir la carga real del host VPS2.

### F3. Cierre y regresion

Criterios de cierre actualizados:

- `site-hosting-3212defd`: cumplido provisionalmente; dejo de dar `504` y respondio `200` estable en corridas repetidas.
- `wordpress-hosting-de17015b`: pendiente; sigue con latencia irregular y muy por encima de VPS1.
- Panel de disco: cumplido; el fix fue desplegado en `studio` y `nakomi.studio/healthz` quedo `200` post-swap.

---

## Hallazgo paralelo ya corregido y desplegado

El panel perdia `disk_used_mb` porque las consultas tomaban la fila mas reciente del sampler aunque esa fila proviniera de un ciclo sin probe de storage. Eso dejaba `disk_limit_mb` poblado pero `disk_used_mb = NULL` en la UI.

Correccion aplicada en [src/repositories/infrastructure.rs](src/repositories/infrastructure.rs) y desplegada en `studio`:

- `latest_deployment_sample()` ahora recupera por separado el ultimo `disk_used_mb` no nulo y el ultimo `disk_limit_mb` no nulo.
- `resource_usage_report()` ahora usa el ultimo `disk_used_mb` no nulo en vez de la ultima fila a secas.

Validacion del criterio nuevo sobre la base real:

- `cap` -> `732 MB`
- `guillermo` -> `798 MB`
- `padel` -> `1243 MB`
- `glory-rest` -> `65 MB`
- `hosting-3212defd` -> `1 MB`
- `hosting-de17015b` -> `271 MB`

---

## Siguiente accion correcta

La siguiente accion ya no es seguir buscando "la query mala" o "la red mala" en ese WordPress de prueba. La evidencia actual dice que:

- la ubicacion publica no explica una diferencia de `5s - 10s`, porque el render tambien tarda dentro del contenedor;
- MariaDB no explica el grueso del tiempo;
- el host VPS2 muestra carga suficiente para degradar PHP aunque no este lleno de RAM o disco.

Si se quiere que `hosting-de17015b` deje de ser lento, la siguiente accion correcta es moverlo a un host menos cargado o bajar la carga real del VPS2 antes de seguir afinando WordPress.

---

## Plan de optimizacion automatizable del host

### Objetivo operativo

Pasar de diagnostico puntual a una mitigacion repetible desde `coolify-manager-rs`, sin depender de SSH manual cada vez que una VPS quede sin swap o con sintomas de contencion.

### F4. Mitigacion inmediata del host

1. Asegurar `swap` persistente en VPS2 para absorber picos de memoria y evitar OOM cuando la carga sube.
2. Persistir `vm.swappiness=10` y `vm.vfs_cache_pressure=50` para que Linux no abuse de swap pero tampoco se quede sin margen.
3. Repetir medicion de `load`, pressure y latencia app despues del cambio; swap no deberia "arreglar" CPU/I/O, pero si estabilizar el host bajo picos.

### F5. Foto de carga y optimizacion de workloads

El manager debe reportar en la misma operacion:

- top procesos por CPU;
- `docker stats` snapshot;
- estado de swap antes/despues;
- pressure CPU/I/O.

Con esa foto, las optimizaciones candidatas dejan de ser "genéricas" y pasan a ser accionables:

1. bajar concurrencia de workers/colas si uno o dos procesos concentran la CPU;
2. separar workloads ruidosos del mismo host si el problema se concentra en un stack concreto;
3. mover sitios sensibles fuera del nodo si la presion I/O del host sigue alta incluso con pocos contenedores.

### F6. Automatizacion en coolify-manager-rs

Se implementa un comando host-level nuevo, pensado para repetirse por target:

- `optimize-host --target standby-vps2 --swap-gb 4`
- `optimize-host --target standby-vps2 --dry-run`

El comando debe:

1. auditar el host;
2. asegurar swap persistente si falta;
3. persistir sysctl base de bajo riesgo;
4. devolver recomendaciones segun la carga real observada.

Estado actual:

- Implementado `optimize-host` para swap + sysctl + snapshot host-level.
- Implementado `audit-control-plane` para separar el plano de control de Coolify del workload alojado.
- Validado contra VPS2 y VPS1 con el binario release del manager.

Hallazgo nuevo:

- En VPS2, el load alto ya no apunta principalmente al WordPress de prueba sino al propio control-plane de Coolify.
- `audit-control-plane --target standby-vps2` mostro dominancia del stack interno de Coolify, con `coolify` llegando a `~170% - 275% CPU` segun la toma, mas contribucion adicional de `coolify-redis` y `coolify-realtime`.
- Los logs del contenedor `coolify` muestran `App\\Jobs\\ScheduledJobManager` ejecutandose repetidamente y `horizon:snapshot` tardando varios segundos (`4s - 8s`) en VPS2, mientras que en VPS1 el mismo tipo de actividad queda en valores mucho menores y con control-plane total alrededor de `~17% CPU`.
- Se amplió `audit-control-plane` con remediacion conservadora (`--repair`) para reciclar Horizon y limpiar estado del panel sin SSH manual.
- En VPS2, `horizon:terminate` si funciono y Horizon reinicio correctamente (`Horizon started successfully`). Tras ese recycle, el control-plane bajo de una foto de `~536% CPU` total (`coolify ~403%`, `coolify-redis ~126%`) a otra de `~339% CPU` total (`coolify ~259%`, `coolify-redis ~49%`), y `horizon:snapshot` bajo de `~14s` a `~8s`.
- En una ventana corta posterior (`--since 10m`), la mejora se sostuvo: `failed_jobs=0`, `queue_keys=0`, `horizon_keys=0` y el control-plane bajo a `~203% CPU` total (`coolify ~148%`, `coolify-db ~35%`, `coolify-redis ~15%`). `horizon:snapshot` siguio costando `~6s`, asi que hubo mejora clara, pero no resolucion completa.
- Aun con esa mejora parcial, VPS2 siguio con `load 13.86 / 11.69 / 9.83` y el control-plane siguio siendo el hotspot dominante. Con eso queda reforzado que el problema no era solo Horizon trabado ni falta de swap, sino una combinacion de host lento + stack interno de Coolify demasiado costoso para ese nodo.
- La version actual del panel en VPS2 no soporta `queue:flush --force` ni `horizon:clear-metrics`, asi que la remediacion automatizable de bajo riesgo hoy queda limitada a recycle de Horizon y limpieza conservadora del scheduler; para bajar mas la latencia estable ya no alcanza con tuning local del panel.
- Perfil host-level promediado en caliente (`optimize-host --dry-run --samples 5 --interval-seconds 2`) sobre VPS2: `load 8.05 / 8.25 / 7.31`, `cpu_some avg10=45.58`, `io_some avg10=1.30`, `io_full avg10=0.00`. Los mayores consumidores medios por proceso fueron `soketi-server ~20.66%`, `php ~18.38%`, `redis-server ~10.80%`, `dockerd ~6.10%`, `php-fpm ~5.90%`, `node ~5.02%`. Eso confirma que la CPU no la concentra un unico WordPress, sino una mezcla de realtime/control-plane y procesos PHP/Node del host.

Conclusion operativa actualizada:

- La lentitud de VPS2 no se explica solo por el hosting WordPress de prueba.
- El propio panel/control-plane de Coolify esta cargando de forma anormal esa VPS en comparacion con VPS1.
- Si no hay nada sensible en VPS2, la accion mas fuerte sigue siendo usarla para diagnostico agresivo o directamente sacar de ahi cualquier carga que necesite latencia estable.

### Criterio de decision despues de aplicar F4-F6

- Si baja la inestabilidad pero el `load` y `io_full` siguen altos, el problema real no era falta de swap sino contencion del nodo/workloads.
- Si la app sigue lenta con swap activa y sin presion de memoria, la decision correcta pasa a ser migracion o redistribucion de servicios, no mas tuning del CMS.
