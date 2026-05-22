# Plan: Enforcement de recursos en hostings (ancho de banda, disco, capacidad)

> **Fecha:** 2026-05-22
> **Origen:** Auditoria de gaps en `src/services/storage_enforcement.rs`, `coolify.rs`, `docker_stats.rs`
> **Estado:** Validado contra codigo real — listo para implementar

---

## Resumen del problema

El sistema tiene enforcement solido para **disco en hosting administrado** (loop 6h, bloqueo SFTP, notificacion), pero **ancho de banda no se mide ni enforcea**, **VPS no tiene supervision**, y **no hay control de capacidad total del servidor** antes de provisionar.

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
- Por ahora: el label "Tráfico ilimitado" se mantiene, sin enforcement hasta que Contablo lo requiera

### 1.R Riesgos y mitigaciones (Fase 1)

| Riesgo | Prob. | Impacto | Mitigacion |
|--------|-------|---------|------------|
| **SSH overhead 24x** — 15min vs 6h actual de storage = paso de ~4 SSH calls/hora a ~96/hora. Cada una: TCP + key auth + exec. | Alta | Medio | (a) **Batch por server**: un solo SSH por server ejecuta `docker stats` filtrado por todas las subs de ese server, en vez de uno por sub. (b) **Considerar 1h** en vez de 15min — para limites de 50-500GB/mes, un abuse tarda horas en exceder; 1h de retraso es aceptable. (c) **Stagger start**: no arrancar todas las conexiones al mismo tick, distribuir en ventana de 60s. |
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
- Cada 30 min: consultar Contabo API (`GET /vps/instances/{instance_id}`)
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

## Tabla de esfuerzo estimado

| Tarea                                   | Archivos                                     | Esfuerzo | Depende de   |
| --------------------------------------- | -------------------------------------------- | -------- | ------------ |
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

**Total estimado:** 14-20h

---

## Orden sugerido de implementacion

1. **Fase 5** (rate limit global) — 0.5h, impacto inmediato, sin dependencias — solo reemplazar valores existentes
2. **Fase 6** (limite por usuario) — 0.5h, bajo esfuerzo, sin dependencias
3. **Fase 1** (bandwidth) — 6-7h, gap critico con `bandwidth_limit_gb` ya modelado en DB
   - 1.2 (copiar a subscription) y 1.1 (tracker + delta) pueden ir juntos
   - 1.3 (enforcement loop) y 1.4 (API) pueden ir juntos
4. **Fase 2** (capacidad servidor) — 3-4h, evita oversubscription
5. **Fase 3** (VPS monitor) — 3-4h, gap de visibilidad — simplificado (sin nuevo campo)
6. **Fase 4** (alertas proactivas) — 1h, polishing
