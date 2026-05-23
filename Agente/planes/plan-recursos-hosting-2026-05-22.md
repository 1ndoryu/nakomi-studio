# Plan: Enforcement de recursos en hostings (ancho de banda, disco, capacidad)

> **Fecha:** 2026-05-22
> **Origen:** Auditoria de gaps en `src/services/storage_enforcement.rs`, `coolify.rs`, `docker_stats.rs`, panel `/panel/?seccion=infraestructura`
> **Estado:** Implementado en 225A-4 — queda solo observacion post-deploy y ajustes finos segun datos reales

---

## Resumen del problema

El sistema tiene enforcement solido para **disco en hosting administrado** (loop 6h, bloqueo SFTP, notificacion), pero **ancho de banda no se mide ni enforcea**, **VPS no tiene supervision promediada**, **la vista de infraestructura mezcla resumenes poco utiles**, y **no hay control de capacidad total del servidor** antes de provisionar.

---

## Revision del plan y correcciones de alcance

El plan base tiene sentido para enforcement: ancho de banda, capacidad y limites por usuario son los frentes correctos. La correccion principal es separar **enforcement** de **observabilidad operativa**. El panel no necesita datos exactos en tiempo real; necesita promedios confiables para detectar anomalias sin meter carga al servidor.

Correcciones obligatorias antes de implementar:

- Incidente produccion 2026-05-22: en `/panel/?seccion=infraestructura`, produccion muestra despliegues de VPS2 pero no los de VPS1; local si muestra ambos porque local tiene `COOLIFY_VPS1_*` completo. La correccion no puede depender de Contabo: la tab **VPS** solo muestra `vmi3001645` (`66.94.100.241`) porque Contabo responde una instancia de esa cuenta, mientras VPS2 (`173.249.50.44`) existe como servidor Coolify configurado. El inventario debe unir Coolify configs + Contabo por IP, y los despliegues deben consultar todos los servidores configurados.
- `infraResumen` no aporta valor y debe eliminarse del panel. Mezcla una VPS elegida de forma arbitraria con conteos globales y puede ocultar que hay dos o mas VPS configuradas.
- Las tabs del panel `/panel/?seccion=infraestructura` deben llamarse simplemente **Despliegues** y **VPS**. Nada de `Despliegues VPS2`, `Contabo VPS` o nombres que aten la UI a una infraestructura temporal.
- Los nombres internos `Vps2DeploymentsPanel`, `useVps2DeploymentsPanel`, `apiListVps2Deployments` y comentarios equivalentes son deuda semantica. El endpoint ya intenta listar todas las VPS, asi que la implementacion debe renombrarse a `DeploymentsPanel`, `useDeploymentsPanel`, `apiListDeployments` o equivalente.
- El backend actual soporta como maximo dos Coolify configs (`coolify_config_vps1` y `coolify_config`). Eso no escala a futuras VPS. Hay que introducir un inventario iterable de servidores antes de seguir sumando excepciones.
- La pestaña **VPS** debe mostrar la union de VPS del proveedor (Contabo) y servidores Coolify configurados localmente. Si Contabo no devuelve una instancia por credenciales/cuenta, una VPS configurada para despliegues no debe desaparecer del panel.
- Las metricas de uso (CPU promedio, RAM usada, almacenamiento usado) no deben consultarse por SSH en cada render del dashboard. Deben salir de snapshots/rollups guardados por un sampler en background.
- La biblioteca recomendada para graficos es `uplot`: minimalista, rapida, sin dependencia React pesada y suficiente para series temporales simples. `recharts` seria mas comoda, pero demasiado grande para este caso.

### Aplicado 2026-05-22 — bloque infraestructura produccion

- Produccion: sincronizadas `COOLIFY_VPS1_BASE_URL`, `COOLIFY_VPS1_API_TOKEN`, `COOLIFY_VPS1_SERVER_UUID`, `COOLIFY_VPS1_PROJECT_UUID`, `COOLIFY_VPS1_SERVER_IP` via `coolify-manager-rs sync-env --name studio` y redeploy aplicado. Verificado en runtime: las cinco claves aparecen como `present`, health `http_ok=true app_ok=true fatal_logs=false`, logs muestran `Coolify VPS1 configurado`.
- Backend: creado helper `infrastructure` para generar targets Coolify deduplicados y fusionar inventario de VPS desde Coolify config + Contabo por IP.
- Backend: `/api/hosting/vps` ya no depende exclusivamente de Contabo; si Contabo solo devuelve VPS1, VPS2 sale desde la config Coolify.
- Backend: `/api/hosting/deployments` itera targets Coolify configurados en vez de ramas fijas VPS1/VPS2.
- Frontend: tabs renombradas a **Despliegues** y **VPS**; eliminado `infraResumen`; fila de despliegue separada en `DeploymentRow`, menú contextual migrado a `MenuContextual` y deuda semantica `Vps2DeploymentsPanel`/`useVps2DeploymentsPanel`/`apiListVps2Deployments` renombrada a `DeploymentsPanel`/`useDeploymentsPanel`/`apiListDeployments`.
- Tooling: `coolify-manager-rs` no inyectaba `COOLIFY_*` al compose runtime por seguridad; se corrigió para permitir solo claves `COOLIFY_VPSn_*` y mantener bloqueadas las `COOLIFY_*` planas de plataforma.
- Pendiente del plan: observacion post-deploy para confirmar muestras reales, overage mensual y estado Contabo con trafico productivo.

### Aplicado 2026-05-22 — bloque enforcement de recursos

- Migracion: agregadas `infrastructure_servers`, `infrastructure_resource_samples`, `bandwidth_snapshots`, `bandwidth_usage`, `server_capacity`, `vps_monitor_state`, `hosting_subscriptions.bandwidth_limit_gb`, `hosting_plan_configs.usage_alert_threshold_pct` y `user_profiles.max_active_subscriptions`.
- Backend: nuevo repositorio `InfrastructureRepository` con queries runtime preparadas para inventario, snapshots, bandwidth mensual, capacidad atomica, reporte admin y estado de monitor VPS.
- Backend: nuevo sampler `infrastructure_metrics_loop` cada 10 minutos, con una SSH por servidor configurado, CPU/RAM/disco VPS, `docker stats` por despliegue, `du` por volumen solo en ventana horaria y acumulado de bandwidth por deltas.
- Backend: `/api/infrastructure/servers`, `/api/infrastructure/deployments`, `/api/infrastructure/deployments/:uuid/metrics`, `/api/infrastructure/metrics/refresh` y `/api/infrastructure/resource-report`; endpoints legacy de hosting siguen como alias.
- Backend: `/api/hosting/deployments` ya no hace SSH en render; lee snapshots del sampler y deja guiones hasta tener datos.
- Backend: `/api/hosting/subscriptions/:id/stats` lee `bandwidth_limit_gb` por suscripcion, devuelve uso mensual real, remanente y fecha de reset.
- Backend: `bandwidth_enforcement_loop` suspende/restaura servicios Coolify cuando el uso mensual cruza el limite; `bandwidth_limit_gb = -1` queda como ilimitado.
- Backend: provisioning reserva capacidad de servidor de forma atomica cuando `server_capacity` tiene specs; si no hay specs conocidas, no bloquea falsamente.
- Backend: `vps_monitor_loop` consulta Contabo cada hora para estado proveedor, registra eventos y marca `provider_attention` tras estados no running.
- Frontend: instalado `uplot`, creado `ResourceUsageChart`, cargado bajo demanda al expandir despliegues y mostrados promedios CPU/RAM/disco en tab VPS.
- Seguridad/operacion: rate limit global ajustado segun Fase 5, manteniendo limites especificos de checkout/subscribe.

---

## Fase 0: Inventario multi-VPS y contratos del panel

### 0.1 Registro iterable de servidores

- Crear un modelo `infrastructure_servers` o equivalente con datos no sensibles:
  - `id UUID`, `label`, `provider`, `provider_instance_id`, `server_ip`, `coolify_base_url`, `coolify_server_uuid`, `is_active`, `created_at`, `updated_at`
  - Secrets fuera de DB: guardar solo `secret_ref`/nombre de variable de entorno para token Coolify o SSH key.
- Sembrar los dos servidores actuales:
  - VPS Principal (`coolify_config_vps1`)
  - VPS2 (`coolify_config`)
- Exponer helper backend `list_infrastructure_servers()` que devuelva un `Vec<InfrastructureServerConfig>` y haga fallback temporal a las env vars actuales mientras se migra.
- Reemplazar loops hardcodeados VPS1/VPS2 por iteracion sobre servidores activos.

### 0.2 API de infraestructura

- Mantener compatibilidad con `/api/hosting/deployments` y `/api/hosting/vps` si conviene, pero internamente nombrar el dominio como infraestructura.
- Agregar o adaptar endpoints admin:
  - `GET /api/infrastructure/servers`: lista todas las VPS conocidas con specs + ultimo promedio disponible.
  - `GET /api/infrastructure/deployments`: lista despliegues de todas las VPS configuradas.
  - `GET /api/infrastructure/deployments/:uuid/metrics?range=24h`: serie temporal agregada para el grafico al expandir/clickear un despliegue.
- Cada despliegue debe incluir `server_id`, `server_label`, `server_ip` o clave estable para agrupar por VPS.

### 0.3 Panel `/panel/?seccion=infraestructura`

- Renombrar tabs:
  - `Despliegues`
  - `VPS`
- Eliminar `infraResumen` y su CSS.
- En **Despliegues**: tabla compacta con todos los despliegues de todas las VPS actuales y futuras, badge de VPS origen, CPU/RAM/disco actuales o ultimo promedio.
- En **VPS**: deben salir las 2 VPS actuales y cualquier futura VPS integrada, con CPU promedio, RAM usada/total y almacenamiento usado/total.
- Al clickear un despliegue: expandir detalle y cargar grafico de uso promedio de recursos (CPU, RAM, disco) del despliegue.

### 0.R Riesgos y mitigaciones (Fase 0)

| Riesgo | Prob. | Impacto | Mitigacion |
|--------|-------|---------|------------|
| **Duplicar VPS entre Contabo y Coolify config** — una misma VPS aparece dos veces si se cruza por IP e instance_id parcial. | Media | Medio | Deduplicar por `provider_instance_id` cuando exista; fallback por `server_ip`; si ambos faltan, mostrar como servidor configurado sin proveedor vinculado. |
| **Guardar secretos en DB** | Baja | Alto | DB solo guarda `secret_ref`; tokens/SSH keys siguen en env o secret manager. |
| **Romper URLs actuales del frontend** | Media | Medio | Mantener endpoints legacy como alias durante la migracion y renombrar primero internamente. |
| **Futuras VPS requieren deploy de codigo** | Alta si no se corrige | Medio | Registro iterable + seed/admin config. Integrar nueva VPS debe ser dato/config, no nueva rama `if VPS3`. |

---

## Fase 0.5: Sampler de recursos y promedios de baja carga

### 0.5.1 Captura por servidor, no por render

- Crear `src/services/infrastructure_metrics.rs` con loop cada **10 minutos**.
- Ejecutar **una sola SSH call por servidor activo** para recoger:
  - CPU VPS desde `/proc/stat` por delta entre muestras.
  - RAM VPS desde `free -m`.
  - disco VPS desde `df -Pm /` o mount principal configurado.
  - CPU/RAM por despliegue desde `docker stats --no-stream --format`.
  - disco por despliegue con `du -sm` solo cada **1 hora** o reutilizando el dato de storage enforcement; no correr `du` pesado cada 10 minutos.
- Si una muestra falla, conservar el ultimo snapshot y registrar evento; no bloquear el panel.

### 0.5.2 Persistencia y rollups

- Tabla `infrastructure_resource_samples`:
  - `id UUID PK`
  - `entity_kind TEXT CHECK IN ('server', 'deployment')`
  - `server_id UUID NOT NULL`
  - `deployment_uuid TEXT NULL`
  - `sampled_at TIMESTAMPTZ NOT NULL`
  - `cpu_percent DOUBLE PRECISION NULL`
  - `ram_used_mb DOUBLE PRECISION NULL`
  - `ram_limit_mb DOUBLE PRECISION NULL`
  - `disk_used_mb DOUBLE PRECISION NULL`
  - `disk_limit_mb DOUBLE PRECISION NULL`
  - `source TEXT NOT NULL DEFAULT 'ssh_sampler'`
- Indices:
  - `(entity_kind, server_id, sampled_at DESC)`
  - `(deployment_uuid, sampled_at DESC) WHERE deployment_uuid IS NOT NULL`
- Retencion:
  - muestras de 10 min por 7 dias.
  - rollups horarios por 90 dias si se necesita historico.
- El dashboard lee promedios con ventanas simples (`last_1h`, `last_24h`) desde DB, nunca dispara SSH directo.

### 0.5.3 Semantica de promedios

- CPU promedio VPS: promedio de deltas `/proc/stat` validos en la ventana.
- CPU promedio despliegue: promedio de `docker stats` de los contenedores cuyo nombre contiene uuid/nombre del despliegue.
- RAM: ultimo valor y promedio de ventana; para alerta visual usar ultimo valor, para grafico usar promedio por bucket.
- Disco: valor lento; basta con ultima muestra horaria porque no cambia segundo a segundo.
- Si no hay suficientes muestras, mostrar `Sin datos todavia` y no inventar porcentajes.

### 0.5.4 Graficos frontend

- Instalar `uplot` en `frontend/`.
- Crear componente atomico `ResourceUsageChart` con CSS separado.
- Renderizar solo cuando el despliegue este expandido/clickeado para evitar multiples charts invisibles.
- Series minimas:
  - CPU promedio `%`
  - RAM usada `%` o MB
  - Disco usado `%` o MB
- El endpoint debe devolver buckets ya agregados para que el frontend no procese muestras crudas grandes.

### 0.5.R Riesgos y mitigaciones (Fase 0.5)

| Riesgo | Prob. | Impacto | Mitigacion |
|--------|-------|---------|------------|
| **Carga por SSH** — muchas VPS futuras pueden multiplicar conexiones. | Media | Medio | Intervalo 10 min, una SSH por servidor, jitter de 0-60s, timeout 10-15s y cache de ultimo snapshot. |
| **`du` caro en volumenes grandes** | Alta | Medio | Ejecutar disco por despliegue cada 1h, no cada 10 min; usar `df` para disco VPS general. |
| **Datos no exactos** | Alta | Bajo | Aceptado: el objetivo es detectar tendencias/anomalias, no facturacion exacta. Mostrar copy interno como promedio/estimado. |
| **Primer CPU sample sin delta** | Alta | Bajo | Guardar referencia y mostrar CPU como `null` hasta la segunda muestra. |
| **Charts pesados en tabla** | Media | Bajo | Cargar serie solo al expandir despliegue y limitar rango por defecto a 24h. |

---

## Fase 1: Ancho de banda — medicion + enforcement

### 1.1 Medir trafico real desde Docker stats

- **Archivo:** `src/services/docker_stats.rs`
- `ContainerStats` ya tiene `net_input_mb` y `net_output_mb` por contenedor
- **Issue critico:** Docker `--no-stream` da valores acumulados **desde que el contenedor arranco**, no desde inicio de mes. La primera muestra post-restart incluye todo el trafico historico del contenedor y debe descartarse.
- **Algoritmo de delta:**
  1. Guardar por subscription: `(ultimo_net_input_mb, ultimo_net_output_mb, ultimo_timestamp)` en memoria (HashMap con TTL) y opcionalmente en `bandwidth_snapshots` table
  2. En cada muestreo: leer `net_input`/`net_output` actuales
  3. Si es primera muestra (no hay anterior) o el contenedor se reinicio (delta negativo): descartar, solo guardar como referencia
  4. Si hay muestra anterior valida: `delta_rx = net_input - ultimo_net_input`, `delta_tx = net_output - ultimo_net_output`
  5. Acumular deltas en `bandwidth_usage`
- **Tabla nueva:** `bandwidth_usage (subscription_id UUID UNIQUE, month_start DATE, bytes_rx BIGINT NOT NULL DEFAULT 0, bytes_tx BIGINT NOT NULL DEFAULT 0, updated_at TIMESTAMPTZ)`
  - Una fila por subscription por mes, upsert acumulando
- Ejecutar muestreo cada **1 hora** (ver 1.R sobre intervalo vs. SSH overhead)

### 1.2 Copiar `bandwidth_limit_gb` a subscription al checkout

- **Archivo:** `stripe.rs` o `checkout.rs` (donde se copia `storage_limit_mb`)
- Similar a como `storage_limit_mb` se copia del plan `hosting_plan_configs` a `hosting_subscriptions.storage_limit_mb` durante checkout
- **Migration:** agregar `bandwidth_limit_gb INT NOT NULL DEFAULT 50` a `hosting_subscriptions`
- Copiar el valor del plan al crear/checkout, para que cambios posteriores en el plan no afecten subs existentes retroactivamente
- `GET /hosting/subscriptions/:id/stats`: leer de `sub.bandwidth_limit_gb` en vez de plan config

### 1.3 Background loop de enforcement

- **Archivo nuevo:** `src/services/bandwidth_enforcement.rs`
- Cada **1 hora**: sumar `bandwidth_usage` del mes actual por subscription_id
- Si `total_bytes > (bandwidth_limit_gb * 1_000_000_000)`:
  - Hosting admin: limitar velocidad del contenedor via Docker `--cpus` o detener temporalmente
  - VPS: notificar, no hay enforcement tecnico (depende de Contabo)
- Si vuelve a estar dentro del limite tras reinicio de ciclo mensual: restaurar
- `bandwidth_limit_gb = -1` = sin limite, saltar enforcement

### 1.4 Exponer en API

- `GET /hosting/subscriptions/:id/stats`: reemplazar `bandwidth_used_gb: None` con el valor real de la suma mensual
- Agregar `bandwidth_remaining_gb` y `bandwidth_reset_at` (inicio del mes siguiente)
- Leer `bandwidth_limit_gb` desde `sub` (per-subscription) en vez de plan config

### 1.5 Planes con bandwidth ilimitado

- `bandwidth_limit_gb = -1` = sin limite
- Saltar enforcement

### 1.6 VPS — adaptacion

- VPS usa Contabo API, no Docker stats
- Opcion A: confiar en Contabo API si expone trafico (no documentado)
- Opcion B: no medir, solo registrar el plan contratado y notificar si Contabo reporta uso
- Por ahora: el label "Tráfico ilimitado" se mantiene, sin enforcement hasta que Contabo lo requiera

### 1.R Riesgos y mitigaciones (Fase 1)

| Riesgo | Prob. | Impacto | Mitigacion |
|--------|-------|---------|------------|
| **SSH overhead 24x** — 15min vs 6h actual de storage = paso de ~4 SSH calls/hora a ~96/hora. Cada una: TCP + key auth + exec. | Alta | Medio | (a) **Batch por server**: un solo SSH por server ejecuta `docker stats` filtrado por todas las subs de ese server, en vez de uno por sub. (b) **Considerar 1h** en vez de 15min — para limites de 50-500GB/mes, un abuse tarda horas en exceder; 1h de retraso es aceptable. (c) **Stagger start**: no arrancar todas las conexiones al mismo tick, distribuir en ventana de 60s. (d) Reutilizar el sampler de Fase 0.5 para no duplicar SSH. |
| **Thundering herd SSH** — 50+ conexiones simultaneas al mismo server podrian saturar `max_startups` o `MaxSessions` del SSHD | Media | Alto | Stagger + `ControlMaster auto` en SSH config para reutilizar conexion existente al mismo server. Header `-o ControlMaster=auto -o ControlPath=...` |
| **Datos incorrectos al reiniciar contenedor** — delta negativo por reinicio | Media | Bajo | Descartar muestras con delta negativo. Perder 15min de datos post-reboot es aceptable. |
| **Contador en memoria volatil** — perdida de snapshot al reiniciar el servicio | Baja | Bajo | Peor caso: perder hasta 15min de delta. La ventana es pequena. Opcional: persistir snapshot en `bandwidth_snapshots` DB (1 write/sub cada 15 min). |
| **Lock contention en bandwidth_usage upsert** — 100 subs * 15min = 9600 writes/mes cada una, ~1 write/9s. Despreciable. | Baja | Nulo | Indice en `(subscription_id, month_start)`. No hay riesgo real. |

**Decision clave:** empezar con intervalo de **1 hora** en vez de 15 min. Reduce 24x a 4x vs storage. Si se necesita mas granularidad, bajar luego.

---

## Fase 2: Control de capacidad del servidor

### 2.1 Modelar capacidad del servidor

- **Tabla nueva o campo en `servers`:** `server_capacity (server_uuid PK, cpu_cores NUMERIC, ram_mb INT, disk_mb INT, cpu_allocated NUMERIC, ram_allocated INT, disk_allocated INT)`
- Seed inicial: leer specs reales del server (o insert manual)
- `cpu_allocated`, `ram_allocated`, `disk_allocated` = suma de todos los subscriptions activos en ese server

### 2.2 Verificacion pre-provisioning

- **Archivo:** `src/services/coolify.rs` o `provisioning.rs`
- Antes de `proceed_provisioning()`: calcular si el nuevo subscription cabe
- Si `cpu_used + nuevo_cpu > cpu_total` o `ram_used + nuevo_ram > ram_total` o `disk_used + nuevo_disk > disk_total`: rechazar con error claro
- Actualizar contadores al crear y al cancelar subscription

### 2.3 Recalculo periodico

- Trigger semanal que recalcula `allocated` desde los subscriptions activos (por si hubo desync)
- Log de alerta si `allocated > total` (oversubscription)

### 2.R Riesgos y mitigaciones (Fase 2)

| Riesgo | Prob. | Impacto | Mitigacion |
|--------|-------|---------|------------|
| **Race condition en provisioning** — dos provisioning simultaneos leen allocated antes de que ninguno actualice | Baja | Medio | Usar `SELECT ... FOR UPDATE` en la fila `server_capacity` dentro de la transaccion. O hacer update atomico: `UPDATE server_capacity SET cpu_allocated = cpu_allocated + $1 WHERE server_uuid = $2 AND cpu_allocated + $1 <= cpu_cores`, y verificar rows_affected == 1. |
| **Server_capacity desactualizado** — si un subscription se cancela fuera del flujo normal (admin, bug), allocated no se descuenta | Media | Bajo | El recalculo semanal corrige automaticamente. La ventana maxima de desfase es 7 dias. En ese lapso, el unico efecto es rechazar provisioning que cabrian (falso positivo), nunca aprobar de mas. |
| **Contadores incorrectos por migracion** — subs de servidores legacy sin `server_uuid` no se contabilizan | Media | Medio | Excluir subs con `server_uuid IS NULL` del calculo de allocated. No bloquean provisioning. Seed inicial debe contar solo servers conocidos. |
| **SELECT SUM en cada provisioning** — JOINS con hosting_subscriptions + plan_configs, tabla pequena, costo trivial | Baja | Nulo | Indice en `hosting_subscriptions(server_uuid, status)` para SUM rapido. Sin riesgo real. |

---

## Fase 3: VPS — monitoreo basico

### 3.1 Loop de health para VPS

- **Archivo nuevo:** `src/services/vps_monitor.rs`
- Cada 30-60 min: consultar Contabo API (`GET /compute/instances/{instance_id}`) solo para estado/proveedor, no para CPU/RAM/disco usados
- Comparar estado reportado vs. DB
- Si Contabo reporta `status != 'running'`: crear evento y notificar

### 3.2 Enforce de corte por falta de pago

- Reutilizar la logica de Stripe webhooks que ya existe
- **No crear campo nuevo.** `VpsSubscription.status` ya existe como `String` con default `pending_payment`
- Agregar nuevos valores de status existentes: `suspended_non_payment`, `payment_overdue`
- Si `payment_failed` y pasan 7 dias sin resolver: cambiar status a `suspended_non_payment` y llamar Contabo API para detener la instancia

### 3.R Riesgos y mitigaciones (Fase 3)

| Riesgo | Prob. | Impacto | Mitigacion |
|--------|-------|---------|------------|
| **Rate limit de Contabo API** — 48+ llamadas/dia por instancia. Si tienen rate limits no documentados, podrian bloquear temporalmente | Media | Medio | Cachear estado en DB y solo llamar si pasaron >=30 min desde ultima consulta. Si response es 429, respetar `Retry-After` header y extender intervalo. |
| **Falsos positivos por timeout de red** — Contabo API temporalmente caida, marcar instancias como "no running" incorrectamente | Media | Medio | No cambiar estado con una sola falla. Requerir 3 fallas consecutivas (45 min ventana) antes de marcar como inactiva y notificar. |
| **Carrera entre monitor y webhook Stripe** — webhook de pago fallido corre al mismo tiempo que el monitor, ambos cambian status | Baja | Bajo | Operaciones atomicas: `UPDATE vps_subscriptions SET status = $1 WHERE id = $2 AND status IN ('active', 'payment_overdue')`. Si la segunda UPDATE rows_affected == 0, ignorar. |
| **Contabo API costos** — si Contabo cobra por llamada API (no documentado) | Baja | Medio | Reducir intervalo a 1h tras periodo de observacion. Monitorear costo. Si es gratis, mantener 30min. |

**Nota:** el uso real de recursos VPS no debe depender de Contabo API porque la respuesta actual expone specs/estado, no promedios de CPU/RAM/disco usados. Para eso se usa el sampler SSH de Fase 0.5.

---

## Fase 4: Perfiles de uso y alertas proactivas

### 4.1 Umbrales de alerta

- Configurar `usage_alert_threshold_pct` en `hosting_plan_configs` (default: 80%)
- Cuando `used / limit > threshold` en disco o bandwidth: enviar notificacion
- Reutilizar sistema de notificaciones existente (`notifications.rs` o `events.rs`)

### 4.2 Reporte semanal de uso

- Endpoint `GET /admin/reports/resource-usage` que lista todos los subscriptions con su % de uso
- Para admin: ver quien esta cerca del limite o excedido

### 4.R Riesgos y mitigaciones (Fase 4)

| Riesgo | Prob. | Impacto | Mitigacion |
|--------|-------|---------|------------|
| **Notificaciones masivas** — si disco esta cerca del limite y el usuario recibe 4 notificaciones (cada 6h) antes de liberar espacio | Media | Bajo | Cooldown de notificacion: max 1 por subscription cada 24h. Usar `last_notified_at` en `hosting_events` o cache en memoria. |
| **Falsos positivos de alerta** — umbral 80% se dispara por un pico momentaneo que se resuelve solo | Baja | Medio | Exigir que el umbral se supere en 2 mediciones consecutivas antes de notificar. |
| **Reporte admin pesado** — `GET /admin/reports/resource-usage` JOINea hosting_subscriptions + stats + bandwidth_usage + eventos | Media | Bajo | Cache de 10 min con `CONCURRENTLY` refresh. O ejecutar en background y servir snapshot. |

---

## Fase 5: Rate limiting general de API

### 5.1 Ajustar rate limiter global existente

- **YA existe** en `src/handlers/mod.rs:615-625` con `per_second(1).burst_size(120)` — muy restrictivo para sostenido
- **Reemplazar** con `per_second(600).burst_size(60)` (~6 req/s sostenido)
- No mover a `middleware/` a menos que la logica crezca — el codigo actual en `mod.rs` es simple
- Endpoints sensibles (subscribe, checkout) mantienen sus limites mas restrictivos actuales
- Excepciones para webhooks de Stripe (IP whitelist) — verificar si el GovernorLayer ya soporta whitelist o toca custom

### 5.R Riesgos y mitigaciones (Fase 5)

| Riesgo | Prob. | Impacto | Mitigacion |
|--------|-------|---------|------------|
| **Bloqueo de webhooks Stripe** — Stripe envia callbacks a tasas variables; si el rate limiter los bloquea, pagos/checkouts fallan silenciosamente | Alta | Alto | Stripe publica sus IPs en un rango conocido (`/api/stripe/ips` o `stripe.com`). Excluir CIDR de Stripe del rate limiter, o aplicar antes de que llegue al GovernorLayer. |
| **Valores incorrectos** — `per_second(600).burst_size(60)` podria ser muy permisivo para algunos endpoints | Media | Medio | (a) Deploy gradual: observar rate de 429s en logs. (b) Si es necesario, reducir en vez de aumentar. (c) Los limites especificos de subscribe (3/h) y checkout (5/h) se preservan como capa adicional. |
| **Cliente legitimo atrapado en rate limit** — un usuario real haciendo requests rapidas (ej: carga de dashboard con muchos recursos) recibe 429 | Baja | Medio | Monitorear proporcion de 429s vs 200s. Si >1%, revisar. Agregar `Retry-After` header en respuesta 429. |
| **Panico en startup si config invalida** — `GovernorConfigBuilder::finish()` hace `expect()` que panic si los valores son invalidos | Baja | Alto | Usar `unwrap_or_default()` o manejo de error en vez de `expect()`. Mejor: validar valores en test. |
| **Race condition entre capas** — el rate limit global (GovernorLayer) y los especificos por endpoint se aplican en orden incorrecto | Baja | Medio | Verificar que el global se aplique como outer layer (despues de los especificos) para que no intercepte antes. En `mod.rs`, el orden de `.layer()` importa: el ultimo es el mas externo. |

---

## Fase 6: Limite de subscriptions por usuario

### 6.1 Modelo

- Agregar `max_active_subscriptions INT NOT NULL DEFAULT 5` en `user_profiles` o `subscription_settings`
- Admin puede sobrescribir por usuario

### 6.2 Check en provision/provisioning

- Antes de `proceed_provisioning()`: contar subscriptions activas del usuario
- Si `count >= max_active_subscriptions`: rechazar con mensaje de upgrade

### 6.R Riesgos y mitigaciones (Fase 6)

| Riesgo | Prob. | Impacto | Mitigacion |
|--------|-------|---------|------------|
| **SELECT COUNT lento sin indice compuesto** — `COUNT(*) FROM hosting_subscriptions WHERE user_id = $1 AND status IN ('active', 'provisioning')` | Baja | Bajo | Indice compuesto `(user_id, status)` ya existe parcial en `user_id IS NOT NULL`. Si el WHERE por status es lento, crear indice `(user_id, status) WHERE user_id IS NOT NULL`. |
| **Usuario con subs creadas por admin antes de existir el limite** — subs legacy cuentan contra su nuevo limite sin previo aviso | Media | Medio | Al hacer deploy de la columna `max_active_subscriptions`, inicializar con valor alto (ej: 20) para usuarios existentes. Solo aplicar default 5 a usuarios nuevos post-deploy. |
| **Admin con muchas subs** — el propio admin puede tener 50 servidores; el limite default 5 lo bloquearia | Baja | Alto | Rol admin exento del check, o con default 100. El limite solo aplica a usuarios normales. |

---

## Fase 7: Throttle anti-abuso por ancho de banda

> **Contexto:** Contabo no cobra por GB, solo port speed fijo (200-1000 Mbps segun plan VPS).
> Si un hosting consume mas de lo justo, afecta a los demas en el mismo VPS y arriesga
> que Contabo throttle el VPS completo. Necesitamos throttle reactivo por hosting.

### 7.1 Modelar port speed del VPS

- Agregar `port_speed_mbps INT NOT NULL` a `infrastructure_servers` (la tabla creada en Fase 0.1)
- Seed con valores Contabo: VPS1=200, VPS2=300, VPS3=600, VPS4=800, VPS5=1000, VPS6=1000

### 7.2 Funcion `fair_share()`

```sql
-- Cuantos Mbps le tocan a cada hosting en un VPS
port_speed_mbps / COUNT(hostings activos en ese VPS)
```

- Si un VPS3 (600 Mbps) tiene 6 hostings, fair share = **100 Mbps** cada uno
- Si tiene 3 hostings avanzados, fair share = **200 Mbps** cada uno

### 7.3 Implementar throttle via `tc` (SSH al servidor)

Nuevo modulo `src/services/tc_throttle.rs`:

- `find_veth(server_ip, site_name)` → SSH para encontrar el veth del contenedor
  ```bash
  CONTAINER=$(docker compose -p {site} ps -q {service} 2>/dev/null || echo "")
  PID=$(docker inspect -f '{{.State.Pid}}' "$CONTAINER")
  IFACE=$(nsenter -t "$PID" -n ip -o route show to default | awk '{print $5}')
  IDX=$(nsenter -t "$PID" -n cat /sys/class/net/$IFACE/iflink)
  ip link show | awk -F': ' "\$1 == ${IDX} {print \$2}" | cut -d@ -f1
  ```
- `set_rate_limit(server_ip, site_name, rate_mbps)` → aplica `tc qdisc replace`
  ```bash
  tc qdisc replace dev $VETH root tbf rate Xmbit burst 32kbit latency 400ms
  ```
- `remove_rate_limit(server_ip, site_name)` → `tc qdisc del dev $VETH root`
- `list_throttled(server_ip)` → lista veths con qdisc activo

### 7.4 Criterios de abuso (loop cada 5 min)

| Condicion | Accion |
|-----------|--------|
| `uso_mbps > fair_share * 0.50` por >10 min seguidos | Throttle a **10 Mbps** |
| `uso_mbps > fair_share * 1.00` por >5 min seguidos | Throttle a **5 Mbps** |
| `uso_mbps > port_speed * 0.70` (un hosting domina el VPS entero) | Throttle a **10 Mbps** |
| Proyeccion mensual >25 TB (riesgo Contabo) | Throttle preventivo a **20 Mbps** |

**Revertir:** cuando `uso_mbps < umbral_activacion * 0.30` por 5 min seguidos → restaurar velocidad.

**Histeresis:** umbral de activacion = 50%, umbral de desactivacion = 30% (diferencia 20pp evita oscilacion).

### 7.5 Enforcement loop

Refactorizar `bandwidth_enforcement.rs` (o crear `bandwidth_throttle.rs`):

```rust
pub async fn bandwidth_throttle_loop(pool, http_client, coolify_config) {
    loop {
        // 1. Leer bandwidth_usage del ultimo sample (uso_mbps por hosting)
        // 2. Calcular fair_share de cada VPS (port_speed / hostings_activos)
        // 3. Evaluar cada hosting contra los criterios
        // 4. Si aplica throttle y no esta throttled: tc::set_rate_limit(...)
        // 5. Si esta throttled y ya no aplica: tc::remove_rate_limit(...)
        // 6. Registrar evento bandwidth_throttled / bandwidth_restored
        tokio::time::sleep(Duration::from_secs(300)); // 5 min
    }
}
```

### 7.6 Visibilidad (solo admin)

- Sin notificaciones al cliente. El throttle es silencioso.
- Badge "Velocidad reducida" visible solo en panel admin
- Admin: lista de hostings actualmente throttled, historial de eventos, metrica de velocidad asignada

### 7.7 Sin corte mensual por GB

- **No hay limite mensual en GB** para ningun tipo de hosting.
- Fase 7 es el unico enforcement: **throttle reactivo por uso excesivo del port speed**.
- Nunca se corta el servicio completamente por trafico, solo se reduce velocidad temporalmente.
- El throttle se revierte automaticamente cuando el uso vuelve a niveles normales.

### 7.R Riesgos y mitigaciones (Fase 7)

| Riesgo | Prob. | Impacto | Mitigacion |
|--------|-------|---------|------------|
| `tc` no instalado en el servidor (falta `iproute2`) | Baja | Alto | Fallo silencioso + alerta admin. Agregar check en deploy del VPS. |
| veth name cambia al recrear contenedor | Alta | Medio | Buscar veth por PID cada vez que se aplica throttle, no cachear. |
| Falso positivo (cliente con pico momentaneo legitimo) | Media | Medio | Exigir 2 muestras consecutivas (10 min) antes de throttle. |
| Bucle throttle/restore si el cliente oscila en el umbral | Media | Medio | Histeresis 20pp (activa 50%, restaura 30%). |
| Throttle no persiste tras reboot del contenedor | Alta | Bajo | El loop cada 5 min lo re-aplica. |
| Contabo throttle a nosotros antes de que detectemos | Baja | Alto | Loop cada 5 min reacciona mucho mas rapido que Contabo (promedio 10 dias). |

---

## Tabla de esfuerzo estimado

| Tarea                                   | Archivos                                     | Esfuerzo | Depende de   |
| --------------------------------------- | -------------------------------------------- | -------- | ------------ |
| 0.1 Inventario multi-VPS                | migracion SQL, config/app state, repos       | 2-3h     | —            |
| 0.2 API infraestructura generica        | handlers/routes/modelos                      | 1-2h     | 0.1          |
| 0.3 Panel tabs + quitar infraResumen    | componentes/hooks/CSS panel                  | 1-2h     | 0.2          |
| 0.5 Sampler + snapshots + rollups       | `infrastructure_metrics.rs`, migracion SQL   | 3-5h     | 0.1          |
| 0.5 Charts de recursos con `uplot`      | frontend dep + componente chart              | 1-2h     | 0.5          |
| 1.1 Bandwidth tracker + delta + tabla   | `docker_stats.rs`, `bandwidth_snapshots`     | 2-3h     | —            |
| 1.2 Copiar bandwidth_limit_gb a sub     | migracion SQL, `stripe.rs` checkout          | 0.5h     | —            |
| 1.3 Background loop enforcement         | `bandwidth_enforcement.rs` (nuevo)           | 2-3h     | 1.1, 1.2     |
| 1.4 API stats bandwidth real            | `stats.rs`                                   | 0.5h     | 1.1, 1.2     |
| 2.1 Modelar capacidad server            | migracion SQL, modelo                        | 1-2h     | —            |
| 2.2 Check pre-provisioning              | `provisioning.rs` o `coolify.rs`             | 1-2h     | 2.1          |
| 3.1 Monitor VPS                         | `vps_monitor.rs` (nuevo)                     | 2-3h     | —            |
| 3.2 Corte VPS por impago                | `vps.rs` + webhooks                          | 1h       | 3.1          |
| 4.1 Alertas de umbral                   | `storage_enforcement.rs`, `notifications.rs` | 1h       | 1.1+1.3, F2  |
| 5.1 Ajustar rate limit global existente | `src/handlers/mod.rs`                        | 0.5h     | —            |
| 6.1-2 Limite por usuario                | migracion SQL (columna), provisioning check  | 0.5h     | —            |

**Total estimado:** 22-34h

---

## Orden sugerido de implementacion

1. **Fase 0** (inventario multi-VPS + contratos genericos) — desbloquea que salgan las 2 VPS actuales y futuras sin hardcodear VPS3/VPS4.
2. **Fase 0.5** (sampler + snapshots + UI de promedios) — resuelve la necesidad operativa inmediata: CPU promedio, RAM usada, almacenamiento y graficos por despliegue con baja carga.
3. **Fase 1** (bandwidth) — reutiliza el sampler/batch por servidor para no duplicar SSH.
   - 1.2 (copiar a subscription) y 1.1 (tracker + delta) pueden ir juntos.
   - 1.3 (enforcement loop) y 1.4 (API) pueden ir juntos.
4. **Fase 2** (capacidad servidor) — evita oversubscription antes de provisionar nuevos hostings.
5. **Fase 3** (VPS monitor proveedor) — estado Contabo y corte por impago, separado de metricas de uso.
6. **Fase 4** (alertas proactivas) — usar promedios/snapshots existentes.
7. **Fase 5** (rate limit global) y **Fase 6** (limite por usuario) — tareas pequeñas, aplicar cuando no interfieran con la base multi-VPS.
