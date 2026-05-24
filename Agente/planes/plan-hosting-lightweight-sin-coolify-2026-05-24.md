# Plan: Hosting administrado ligero sin Coolify

> **Fecha:** 2026-05-24
> **Estado:** Activo — plan maestro propuesto para reemplazar el control-plane de Coolify en hostings WordPress y hostings normales
> **Base:** `plan-hosting-automation-2026-04-10.md`, `plan-seguridad-hosting-2026-04-16.md`, `plan-ssh-sftp-seguro-2026-04-16.md`, `plan-recursos-hosting-2026-05-22.md`, `plan-latencia-hostings-vps2-2026-05-23.md`

---

## Objetivo

Operar un servicio de hosting administrado de alta calidad sin depender de Coolify, manteniendo provisioning automatizado, panel, multi-VPS, backups, seguridad y despliegues repetibles, pero con un runtime mucho mas ligero y controlado.

El resultado buscado no es solo "que funcione". El objetivo es que cada hosting nuevo salga ya con:

- Nginx y TLS configurados correctamente
- acceso SFTP/SSH/FTP seguro segun perfil
- backups y restore definidos
- observabilidad minima real
- limites de recursos coherentes por plan
- WordPress optimizado por defecto
- politica de admision que rechace VPS lentas aunque tengan RAM libre

---

## Por que cambiar

La evidencia operativa reciente en VPS2 ya dejo dos conclusiones utiles:

1. El control-plane de Coolify fue una parte material del load del host y su eliminacion bajo inmediatamente la carga.
2. Incluso sin Coolify ni workloads Docker pesados, la VPS sigue mostrando presion de I/O, asi que la calidad del hosting no se resuelve solo cambiando el panel: hace falta controlar el runtime y vetar nodos lentos.

Ademas, hoy el producto de hosting depende de generacion de compose y provisioning acoplado a Coolify en `src/services/coolify.rs`. A nivel funcional ya existen dos productos distintos:

- hosting WordPress administrado
- hosting normal administrado basado en Nginx + archivos editables

La migracion correcta es desacoplar la capa de negocio del proveedor de runtime y reemplazar Coolify por un proveedor propio mas ligero dentro de `coolify-manager-rs`.

---

## Glosario operativo

### Landscape

`landscape` no es "Ubuntu entero". Son utilidades y jobs de Canonical para inventario, MOTD, mantenimiento y telemetria. Si no aportan valor al servicio, se pueden desactivar para reducir ruido de CPU y tareas programadas.

### dockerd

`dockerd` es el daemon de Docker. Aunque no haya sitios activos, sigue consumiendo algo de CPU y RAM porque mantiene redes, imagenes, capas, logs y lifecycle de contenedores. Si el hosting nuevo usa Docker, ese coste base existe y hay que medirlo.

### storage

`storage` es el rendimiento real del disco del VPS. Si el proveedor entrega I/O pobre, WordPress ira lento aunque el software este bien afinado. Por eso este plan incluye benchmarks obligatorios del nodo antes de meter clientes.

### FTP seguro

`FTP seguro` no debe significar FTP plano. La politica propuesta es:

- **Default:** SFTP
- **Compatibilidad opcional:** FTPS explicito
- **Prohibido:** FTP sin cifrado

---

## Decisiones base

1. `coolify-manager-rs` se mantiene como orquestador principal y capa portable de operaciones.
2. El runtime de hosting deja de depender de Coolify API y pasa a un proveedor propio `lightweight`.
3. La infraestructura de cada VPS debe quedarse en un baseline pequeño:
   - Docker Engine o runtime equivalente
   - reverse proxy compartido
   - servicio de certificados TLS
   - backups, monitoreo y health checks
   - firewall, fail2ban y hardening SSH
4. El acceso de archivos no se resolvera con un sidecar SSH por cada hosting como default.
5. El acceso base sera compartido y seguro:
   - SFTP para todos los planes
   - FTPS solo cuando haga falta por compatibilidad
   - SSH shell solo para planes o cuentas que lo requieran y despues de hardening adicional
6. El hosting WordPress premium no debe nacer con `wordpress:apache` como baseline final. Debe migrar a Nginx + PHP-FPM + OPcache + caché real.
7. El hosting normal seguira siendo, por ahora, un producto Nginx administrado para HTML/CSS/JS y sitios estaticos o frontends exportados. Si se quiere PHP generico o Node generico, eso se tratara como otro producto.
8. Ninguna VPS entra a rotacion comercial si no pasa criterios minimos de CPU, memoria e I/O.

## Decisiones ya cerradas para la primera iteracion

Para evitar una Fase 0 eterna, estas decisiones quedan cerradas salvo que una prueba real las invalide:

1. **Ingress compartido:** `Caddy`.
   - Motivo: TLS automatico, config simple y menos moving parts para este caso.
2. **Runtime base:** `Docker Engine + Compose`.
   - Motivo: ya existe experiencia operativa y sigue siendo portable.
3. **Acceso seguro base:** `OpenSSH del host`, no sidecar SSH por sitio.
   - Motivo: menos contenedores, menos puertos y mejor integracion con fail2ban y auditoria.
4. **Acceso por defecto del cliente:** `SFTP sin shell`.
   - El shell queda como extra premium.
5. **Estructura fisica del hosting:** bind mounts bajo `/srv/hosting/{site}`.
6. **Base de datos WordPress:** MariaDB compartida por VPS, aislada por base y usuario por sitio.
7. **Cache WordPress:** Redis compartida por VPS con prefijos por sitio cuando el plan lo incluya.
8. **Puertos expuestos por defecto:** solo `80/443/22` a nivel host.
   - No se abriran puertos SSH o FTP por sitio en la primera iteracion.

## No alcance de la primera iteracion

Para no inflar el proyecto y perder calidad, esto queda fuera del primer corte:

- hosting PHP generico multi-framework
- hosting Node generico con procesos arbitrarios del cliente
- panel autoservicio total de VPS para clientes finales
- shell habilitado para todos los planes
- panel tipo cPanel/Plesk clonado
- correo IMAP/POP por hosting
- replicacion o HA compleja de base de datos

---

## Arquitectura objetivo

## Capa de control

- Backend de negocio y panel siguen gestionando suscripciones, billing, estados y recursos.
- `coolify-manager-rs` pasa a exponer comandos y librerias para el nuevo runtime ligero.
- El backend deja de depender de identificadores nativos de Coolify y pasa a depender de un `deployment_id` propio.

## Capa por VPS

Cada VPS de hosting debe quedar con este baseline:

- `Caddy` o reverse proxy equivalente para TLS automatico, HTTP/2, HTTP/3 y routing central
- Docker Engine con configuracion minima y predecible
- MariaDB compartida por VPS para planes WordPress, con DB y usuario por sitio
- Redis compartido por VPS para object cache y colas ligeras si aplica
- OpenSSH del host o gateway SSH/SFTP compartido
- UFW + fail2ban + logs rotados
- estructura fija en disco, por ejemplo `/srv/hosting/{site}/...`
- backup agent y scripts de restore
- benchmark y health checks del nodo

## Capa por sitio

### Hosting normal

- contenedor `nginx` por sitio o receta equivalente
- web root bind-mounted desde `/srv/hosting/{site}/public`
- configuracion por plantilla con compresion, cache de estaticos, headers y logs
- acceso SFTP compartido al directorio del sitio

### Hosting WordPress premium

- Nginx por sitio
- PHP-FPM por sitio
- bind mounts para codigo, uploads y cache controlada
- base de datos compartida por VPS, aislada por DB/usuario
- Redis compartido por VPS con prefijos por sitio
- bootstrap via WP-CLI para dejar el sitio listo sin wizard manual

## Acceso seguro

Se proponen dos perfiles:

### Perfil estandar

- SFTP por puerto 22 compartido
- usuario enjaulado por sitio
- sin shell interactiva
- password temporal o key auth

### Perfil shell

- SSH shell habilitable solo por plan o permiso explicito
- entorno restringido, sin sudo
- auditoria completa
- WP-CLI, Composer, rsync y herramientas minimas permitidas
- no acceso al host global ni a otros sitios

## Matriz del producto objetivo

| Producto | Runtime | Acceso por defecto | Datos | Optimizacion base | Observaciones |
|---|---|---|---|---|---|
| Hosting normal | `nginx` por sitio | SFTP | solo archivos | cache estatico, compresion, TLS, headers | pensado para HTML/CSS/JS, sitios estaticos y frontends exportados |
| WordPress premium | `nginx + php-fpm` por sitio | SFTP | MariaDB compartida + Redis opcional | OPcache, page cache, cron real, bootstrap WP-CLI | no usa Apache ni MariaDB por contenedor como baseline final |
| Shell premium | mismo runtime del producto base | SSH shell restringido + SFTP | segun producto | mismas optimizaciones del producto base | addon con auditoria y hardening extra |

---

## Superficies que hay que adaptar

## Backend y dominio de hosting

Hoy el provisioning esta acoplado a Coolify en `src/services/coolify.rs`. Hay que introducir una abstraccion de proveedor, por ejemplo:

- `HostingRuntimeProvider`
- `CoolifyRuntimeProvider` como compatibilidad legacy
- `LightweightRuntimeProvider` como runtime nuevo

El backend de hosting debe dejar de asumir:

- `service_uuid` de Coolify como identificador principal
- estados provenientes de Coolify como unica verdad
- despliegues creados via `POST /api/v1/services`

## Panel e inventario

El panel de infraestructura ya tiene trabajo reusable multi-VPS. Hay que reapuntar sus fuentes desde Coolify a inventario propio del runtime ligero para:

- despliegues
- recursos por VPS
- metricas por sitio
- health
- backups
- suspension y restore

## Manager

`coolify-manager-rs` necesita nuevos comandos o equivalentes:

- `bootstrap-target-light`
- `uninstall-target-light`
- `benchmark-target`
- `provision-wordpress`
- `provision-static`
- `rotate-access`
- `issue-cert`
- `backup-site`
- `restore-site`
- `suspend-site`
- `resume-site`
- `delete-site`
- `inventory-light`
- `reconcile-light-runtime`

## Modelo de datos

Hay que agregar o adaptar tablas para guardar:

- `runtime_kind`
- `deployment_id`
- `target_name`
- `site_root`
- `public_root`
- `access_profile`
- `sftp_user`
- `ssh_enabled`
- `ftps_enabled`
- `database_name`
- `database_user`
- `redis_namespace`
- `certificate_state`
- `resource_profile`
- `performance_profile`
- `last_benchmark_result`
- `backup_policy`
- `restore_points`

## Mapa de impacto por repositorio y modulo

### En este proyecto (`glory-rust-template`)

- `src/services/coolify.rs`
   - separar la logica actual de provisioning hacia un provider abstracto
- `src/handlers/hosting/*`
   - dejar de asumir Coolify como backend unico de despliegue
- `src/repositories/infrastructure.rs`
   - adaptar inventario y estado al runtime ligero
- `src/models/hosting/*` y migraciones
   - agregar `runtime_kind`, `deployment_id` y metadatos del nuevo runtime
- `frontend/src/components/panel/*` y `frontend/src/hooks/*`
   - mostrar despliegues y recursos sin depender de UUIDs de Coolify

### En `coolify-manager-rs`

- `src/services/target_bootstrap_manager.rs`
   - nueva familia `bootstrap-target-light` / `uninstall-target-light`
- `src/cli/mod.rs` y `src/commands/*`
   - nuevos comandos de runtime ligero
- `src/services/*`
   - recipes de provisioning, backup, restore, benchmark, inventory y reconcile
- `config/templates/` o recipes equivalentes
   - recetas base de hosting normal y WordPress premium
- `README.md`
   - documentar comandos nuevos, target policies y flujo ligero

---

## Optimizacion obligatoria por defecto

## Hosting normal

Todo hosting normal nuevo debe nacer con:

- Nginx afinado para archivos estaticos
- compresion habilitada
- cache larga para assets versionados
- headers de seguridad
- logs por sitio
- indice inicial limpio y reemplazable
- health check HTTP
- certificado TLS automatico
- estructura de directorios consistente

## Hosting WordPress premium

Todo WordPress nuevo debe nacer con:

- Nginx + PHP-FPM, no Apache como baseline final
- OPcache activo y ajustado
- `DISALLOW_FILE_EDIT` activo
- cron del sistema en vez de depender de `wp-cron` web
- object cache con Redis cuando el plan lo soporte
- page cache real o equivalente desde la receta base
- `WP-CLI` disponible para bootstrap y mantenimiento
- limites PHP coherentes por plan
- caché y logs con rutas previsibles
- backup inicial y politica de retencion configurada desde el alta

## Nivel VPS

Cada nodo debe salir con:

- benchmark de CPU, memoria e I/O antes de aceptar cargas
- thresholds de admision
- alertas por `iowait`, `psi` y saturacion
- log rotation de Docker y servicios
- politica de prune segura
- `fstrim` y mantenimiento del host cuando aplique

## SLO y calidad minima del servicio

Estos objetivos no son marketing; son criterios para aceptar o bloquear nodos y recipes.

### Hosting normal

- TTFB warm objetivo: `< 300 ms` en nodo sano y dominio ya resuelto
- provisioning completo objetivo: `< 2 min`
- restore de archivos objetivo: `< 10 min` para sitios pequenos

### WordPress premium

- TTFB warm objetivo home sin plugins pesados: `< 800 ms` en nodo sano
- alta completa con bootstrap: `< 5 min`
- backup y restore completos: validados antes de vender en masa

### Nodo VPS

- host vacio: load razonable, sin `psi` alta sostenida
- escritura a disco dentro del rango aceptable frente al nodo de referencia
- baseline CPU sin clientes reales suficientemente bajo para no comerse el margen

Si una VPS no cumple, se bloquea para nuevas altas aunque siga encendida y con RAM libre.

## Dependencias entre fases

- **Fase 1** depende solo de las decisiones ya cerradas de este documento.
- **Fase 2** depende de Fase 1 para tener un target real que hablar, pero puede arrancar en paralelo en la capa de backend con un provider mock.
- **Fase 3** depende de Fase 1 + Fase 2.
- **Fase 4** depende de Fase 1 + Fase 2 y puede avanzar en paralelo con Fase 3 a nivel de recipes y benchmarks.
- **Fase 5** depende de Fase 1 y acompaña a Fase 3/Fase 4.
- **Fase 6** empieza desde Fase 1, pero su cierre real depende de que existan despliegues vivos de Fase 3 y Fase 4.
- **Fase 7** no debe arrancar hasta tener Fase 3, Fase 4, Fase 5 y Fase 6 validadas en al menos un target limpio.

---

## Fases del plan

## Fase 0 — Contrato y decisiones irreversibles

**Objetivo:** formalizar en código y documentación las decisiones ya cerradas, no volver a debatir la arquitectura base.

### Entregables

- fijar `Caddy` como ingress compartido del primer corte
- fijar `OpenSSH del host` como capa de acceso segura del primer corte
- congelar alcance comercial de los tres perfiles:
   - WordPress premium
   - hosting normal Nginx
   - shell premium opcional
- definir perfiles de recursos y rendimiento por plan
- definir criterios de admision del nodo
- definir el layout final en `/srv/hosting/{site}`
- definir contrato minimo del `deployment_id` interno

### Riesgos

- mezclar hosting normal con hosting PHP generico y complicar soporte
- reintroducir puertos por sitio para SSH/FTP y volver a colisiones

### Mitigacion

- congelar alcance comercial por producto antes de programar
- dejar shell como perfil aparte, no como default universal

### Definition of done

- este plan no deja decisiones criticas abiertas para la Fase 1
- existe una tabla clara producto → runtime → acceso → datos → optimizacion

---

## Fase 1 — Bootstrap ligero del target

**Objetivo:** reemplazar `install-coolify` por un bootstrap de nodo pensado para hosting.

### Trabajo

- crear `bootstrap-target-light` en `coolify-manager-rs`
- instalar y configurar runtime base del VPS
- preparar `/srv/hosting`
- instalar y configurar ingress compartido
- instalar MariaDB/Redis compartidos del nodo
- configurar UFW, fail2ban y SSH host-level
- registrar benchmark inicial del nodo

### Criterio de salida

- target listo para aceptar hostings sin Coolify
- health del ingress, DB y Redis compartidos
- benchmark guardado y visible

### Riesgos

- bootstrap no idempotente
- node baseline demasiado pesado

### Mitigacion

- scripts declarativos y reentrantes
- validar baseline vacio con medicion real antes de usarlo en produccion

### Validacion minima

- `bootstrap-target-light --dry-run` y `--apply`
- probe HTTP local al ingress
- login y query minima contra MariaDB/Redis compartidos
- benchmark guardado con umbrales evaluables

---

## Fase 2 — Abstraccion del proveedor de runtime

**Objetivo:** que el backend deje de depender de Coolify como contrato de negocio.

### Trabajo

- introducir `HostingRuntimeProvider`
- mover la logica actual de Coolify a una implementacion legacy
- crear `LightweightRuntimeProvider`
- adaptar altas, bajas, health, backups, suspension y metrics
- guardar `runtime_kind` por despliegue

### Criterio de salida

- el backend puede provisionar contra dos proveedores sin cambiar el producto
- panel y BD ya no dependen de `service_uuid` como unica identidad

### Riesgos

- divergencia de estados entre proveedor legacy y nuevo

### Mitigacion

- comando de reconciliacion e inventario periodico
- modelo interno de estado mas pequeno y propio

### Validacion minima

- tests de provider mock
- alta simulada contra `CoolifyRuntimeProvider` y `LightweightRuntimeProvider`
- panel mostrando `deployment_id` y `runtime_kind` sin asumir Coolify

---

## Fase 3 — Hosting normal sobre runtime ligero

**Objetivo:** recuperar primero el producto mas simple y barato de operar.

### Trabajo

- provisionar sitio Nginx con bind mount fijo
- emitir TLS automatico
- crear usuario SFTP del sitio
- health checks, logs, backup y delete seguro
- panel con datos reales del sitio

### Criterio de salida

- alta completa de hosting normal sin Coolify
- acceso SFTP seguro funcionando
- restore verificado

### Riesgos

- mal manejo de permisos del web root
- mal cacheado de assets o certificados incompletos

### Mitigacion

- recipe de permisos fija
- validacion post-provision obligatoria

### Validacion minima

- alta end-to-end desde checkout o flujo admin
- SFTP real con usuario del sitio
- `GET /` responde 200/3xx valido
- backup + restore de sitio de prueba

---

## Fase 4 — WordPress premium ultrafinado

**Objetivo:** sacar WordPress del camino `apache + contenedor DB por sitio` y moverlo a una receta premium.

### Trabajo

- imagen o receta base Nginx + PHP-FPM
- bootstrap via WP-CLI
- DB compartida por nodo con aislamiento por sitio
- Redis por prefijo o namespace
- cron del sistema
- tuning PHP, OPcache y page cache
- backup de archivos + dump SQL

### Criterio de salida

- TTFB estable y mas bajo que el baseline anterior en nodo sano
- sitio listo sin wizard manual
- metricas y health por sitio visibles en panel

### Riesgos

- shared MariaDB se vuelve noisy neighbor
- Redis compartido afecta a otros sitios

### Mitigacion

- limites de recursos por sitio
- split por VPS cuando el nodo supere umbrales
- aislamiento por credenciales y prefijos

### Validacion minima

- alta end-to-end de WordPress con bootstrap WP-CLI
- login `/wp-admin/` operativo
- TTFB warm medido varias veces en nodo sano
- backup y restore completos validados

---

## Fase 5 — SSH, SFTP y FTPS seguros

**Objetivo:** recuperar acceso seguro sin volver a sidecars y puertos publicos por hosting como default.

### Trabajo

- SFTP compartido sobre 22 para todos los sitios
- key auth y rotacion de password temporal
- auditoria de accesos
- fail2ban y limites de login
- FTPS explicito opcional para compatibilidad
- shell premium solo tras endurecimiento y auditoria

### Criterio de salida

- cliente puede subir archivos de forma segura sin exponer el host
- acceso shell, cuando exista, queda restringido y auditable

### Riesgos

- usuarios con shell ven demasiado del sistema
- FTPS complica puertos y firewall

### Mitigacion

- shell solo en perfil premium y no como default
- FTPS desactivado por defecto, activable solo si el cliente lo necesita

### Validacion minima

- SFTP con key auth operativo
- password temporal rota correctamente
- fail2ban detecta intentos fallidos repetidos
- shell premium validado sin visibilidad fuera del sitio

---

## Fase 6 — Observabilidad, calidad y protecciones automaticas

**Objetivo:** que el servicio sea defendible operativamente, no solo funcional.

### Trabajo

- synthetic checks por sitio
- metricas por despliegue y por VPS
- thresholds de `psi`, `iowait`, RAM, disco y cert expiry
- backups y restores probados
- watchdogs de drift de configuracion
- benchmark de admision y benchmark periodico del nodo

### Criterio de salida

- cada nodo tiene score de salud
- cada sitio tiene health real, no solo estado administrativo
- ningun sitio nuevo entra en un nodo degradado

### Riesgos

- seguir metiendo sitios en VPS lentas solo porque hay capacidad teorica

### Mitigacion

- usar benchmark como guardrail duro para placement

### Validacion minima

- dashboard o endpoint con score de nodo
- benchmark de admision bloqueando placement en nodo degradado
- synthetic checks y caducidad de certs visibles

---

## Fase 7 — Migracion desde Coolify

**Objetivo:** mover suscripciones activas al runtime ligero con rollback claro.

### Trabajo

- inventario de hostings actuales por tipo
- backup previo por sitio
- migracion por cohortes
- cutover de DNS o routing
- validacion post-migracion
- rollback automatizable por lote o por sitio

### Criterio de salida

- coexistencia temporal soportada
- migracion sitio a sitio sin perdida de datos

### Riesgos

- downtime por errores de permisos, DB o certificados

### Mitigacion

- staging previo o shadow deployment
- smoke tests antes de cambiar trafico

### Validacion minima

- al menos una cohorte piloto migrada con rollback probado
- inventario reconciliado entre panel, BD y runtime
- sin sitios huerfanos ni secretos perdidos

---

## Estrategia operativa de migracion

La migracion no debe ser `big bang`. Se ejecuta por cohortes:

1. **Cohorte 0:** host limpio sin clientes, solo recipes y benchmarks.
2. **Cohorte 1:** hostings normales internos o de bajo riesgo.
3. **Cohorte 2:** WordPress de prueba y clientes tolerantes a ventana corta.
4. **Cohorte 3:** WordPress premium productivos.

Cada cohorte exige:

- backup previo verificado
- smoke tests antes de mover trafico
- ventana de cambio definida
- checklist de rollback preparada
- verificacion post-cutover

## Runbook minimo de rollback

Todo movimiento al runtime ligero debe poder volver atras con estos pasos claros:

1. congelar escrituras del sitio o entrar en ventana corta
2. mantener backup inmediato previo al cutover
3. si falla health o smoke test, restaurar routing al entorno anterior
4. si hubo cambios de datos, restaurar dump y archivos del snapshot previo
5. dejar incidente documentado con causa y fix requerido antes de reintentar

## Checklist de listo para vender

El servicio no se considera listo comercialmente hasta que existan:

- un target ligero funcional y benchmarkeado
- alta real de hosting normal
- alta real de WordPress premium
- acceso SFTP seguro validado con usuario real
- backup + restore verificados en ambos productos
- panel mostrando estado, recursos y backups sin depender de Coolify
- runbook de migracion y rollback probado

---

## Riesgos maestros y mitigaciones

| Riesgo | Impacto | Mitigacion |
|---|---|---|
| La VPS tiene storage lento aunque tenga RAM libre | Alto | `benchmark-target` obligatorio y bloqueo de placement en nodos degradados |
| El nuevo runtime reintroduce deriva entre BD y host real | Alto | inventario periodico + `reconcile-light-runtime` + health y backups visibles |
| El acceso SSH por hosting expone demasiado del host | Critico | SFTP como default, shell solo premium, sin sudo, auditoria y fail2ban |
| FTPS obliga a abrir demasiados puertos | Medio | no habilitar FTPS globalmente; usarlo solo cuando el cliente lo exija |
| WordPress compartiendo MariaDB genera noisy neighbor | Alto | limites por sitio, perfiles por plan y split de nodo cuando haya saturacion |
| El panel sigue dependiendo de Coolify en rutas legacy | Alto | proveedor abstracto + migracion gradual de handlers y panel |
| El baseline del nodo sigue siendo demasiado pesado | Alto | medir host vacio y rechazar cualquier componente que no justifique su coste |
| Restaurar un sitio no devuelve el estado real | Alto | restore test obligatorio con checklist y validacion HTTP real |
| El servicio "normal" crece sin limite de alcance | Medio | congelar que el producto inicial es Nginx administrado, no hosting generico multi-runtime |

---

## Criterios de calidad del servicio

Un nodo no se considera apto para hosting premium si falla cualquiera de estos puntos:

- load e `psi` degradados en vacio o con carga liviana
- latencia de escritura claramente peor que nodos sanos
- health checks del ingress inestables
- baseline de CPU demasiado alto sin clientes reales
- backups o restore no verificados

Un hosting nuevo no se considera listo si no cumple:

- responde HTTP 200/3xx correctamente
- tiene certificado valido
- acceso SFTP operativo
- backup inicial disponible o programado
- health y metricas visibles en panel
- politicas de seguridad aplicadas

---

## Backlog ejecutable sugerido

### 245A-1 — Definir provider ligero y contrato de despliegue

- introducir `HostingRuntimeProvider`
- congelar `runtime_kind`
- separar identidad interna de despliegue vs proveedor

### 245A-2 — Bootstrap de VPS ligera para hosting

- crear `bootstrap-target-light`
- instalar ingress, DB, Redis, backups y hardening host-level
- agregar `benchmark-target`

### 245A-3 — Alta de hosting normal sin Coolify

- recipe Nginx + TLS + SFTP
- inventario y health reales

### 245A-4 — Alta de WordPress premium optimizado

- recipe Nginx + PHP-FPM + WP-CLI + Redis + backup

### 245A-5 — Acceso seguro compartido

- SFTP compartido
- key auth
- auditoria
- fail2ban

### 245A-6 — Migracion de hostings legacy

- inventario
- backups
- cutover por cohortes

---

## Cierre esperado

Este plan no busca reemplazar un panel por otro. Busca convertir el servicio de hosting en una plataforma propia y ligera, centrada en dos productos claros, con mejores defaults tecnicos, menos overhead por VPS y una operacion que siga siendo portable via `coolify-manager-rs` aunque Coolify desaparezca por completo del stack.