# Incidente: nakomi.studio — Connection Leak Persistente (CLOSE_WAIT → Deadlock)

**Fecha inicio:** 2026-06-09 ~11:00 UTC  
**Última caída:** 2026-06-10 ~14:55 UTC (caída #8)  
**Severidad:** 🔴 Crítica — sitio cayendo repetidamente cada ~1.5-6 horas tras 8 intentos de fix  
**Servicio:** nakomi.studio (VPS1 66.94.100.241, Coolify service `do8k4w8swccwwogoc0os0ck0`)  
**Estado actual (2026-06-10 ~15:00 UTC):** 🔴 **PEDIR AYUDA EXTERNA.** Restaurado con `docker restart` por 8ª vez. 7 fixes implementados, ninguno definitivo.  
**Commits desplegados:** `ddc59a6b` (v6: semáforo AI + timeout + métricas). Desplegado ~13:19 UTC, caído ~14:55 UTC (~1.5h).  
**Root cause (v5-v6):** Fixes implementados (keepalive por socket, semáforo AI, timeout 600s, métricas) **no resuelven la causa raíz real.** Hipótesis previas descartadas.  

---

## ⚠️ PETICIÓN DE REVISIÓN A OTRO AGENTE  

**Este documento es una petición explícita para que un agente diferente revise este incidente con ojos frescos.**  

### Qué NO ha funcionado (7 intentos):  

| Fix | Hipótesis | Resultado | Conclusión |
|---|---|---|---|
| v1 | SO_KEEPALIVE + WS timeouts | Caída en ~26 min | TCP keepalive no fuerza cierre |
| v2 | hyper_util + header_read_timeout | Caída | Features cfg-gated = NO-OP |
| v3 | hyper features explícitas + HTTP/2 keepalive | 425 CLOSE_WAIT/8h | h2c probablemente no se negocia |
| v4 | TCP keepalive agresivo por socket | Caída en ~6h | Mejora parcial, NO definitivo |
| v4.1 | GracefulShutdown + half_close(false) | 15 CLOSE_WAIT/6h | **15 CLOSE_WAIT NO matan servidor** → cuello de botella en otro sitio |
| v5 | TCP keepalive por accepted socket + watchdog atómico | Desplegado, mismo patrón | Keepalive no es el problema real |
| v6 | **Semaphore(3) AI + timeout(600s) + AtomicU64 + métricas** | **Caída en ~1.5h** | Concurrencia AI no era el killer |  

### Lo que sabemos con certeza:  

1. **El servidor se cuelga**, no crashea (no hay panic, no hay exit). El proceso sigue vivo pero no responde.  
2. **`docker restart` lo arregla** inmediatamente → no es corrupción de estado persistente.  
3. **El patrón es consistente:** restart → funciona X horas → cuelga → restart → repeat.  
4. **15 CLOSE_WAIT no deberían matar un servidor Rust** con 4 tokio workers. El cuello de botella NO es CLOSE_WAIT.  
5. **El semáforo AI(3) y timeout(600s) no lo arreglan** → no es starvation por requests AI concurrentes.  
6. **Health endpoint funciona** cuando el servidor responde: `{active_timing_loops: 0, registered_sessions: 0, ai_permits_available: 3}`.  
7. **El contenedor tiene 5.5GB RAM libres** y 190GB disco. No es OOM.  

### Qué buscar (hipótesis no exploradas):  

1. **¿El accept loop se bloquea?** Si `listener.accept()` se bloquea indefinidamente (por ejemplo, por un fd leak), el servidor deja de aceptar conexiones nuevas sin morir. Verificar: ¿cuántos fd tiene el proceso? (`ls /proc/PID/fd | wc -l`).  
2. **¿El DB pool se agota silenciosamente?** Pool max=10. Si alguna query no libera conexión (transaction sin commit/rollback, o query que cuelga), el pool se llena. Verificar: `SELECT count(*) FROM pg_stat_activity WHERE datname='nakomi'`.  
3. **¿Traefik cierra la conexión TCP pero el servidor no lo detecta?** Si Traefik hace TCP RST en vez de FIN, el socket queda en estado limbo. Verificar: `ss -tnp` en el contenedor cuando cuelga.  
4. **¿Alguno de los servicios spawned (broadcast channels, notification handlers, chat WS handlers) tiene un leak de tokio tasks?** Cada `tokio::spawn` que nunca termina consume recursos. Verificar: ¿hay un contador de tasks activas?  
5. **¿El PG listener (`LISTEN/NOTIFY`)** tiene un leak? Si el canal de PostgreSQL se cuelga, puede bloquear el loop principal.  
6. **¿Hay un deadlock en `RwLock` o `Mutex`** dentro de `AppState`? Si algún handler toma un lock y nunca lo suelta (por ejemplo, por un panic en un task), todo se bloquea.  

### Acciones sugeridas:  

1. **Agregar métricas de tokio runtime** al health endpoint: `tokio::runtime::Handle::current().metrics()` — número de workers activos, tasks spawned, injection count.  
2. **Agregar métricas de DB pool** al health endpoint: `pg_pool.size()`, `pg_pool.num_idle()`, `pg_pool.num_idle()`.  
3. **Habilitar `tokio-console`** para observar tasks en tiempo real: `console-subscriber` crate + `RUSTFLAGS="--cfg tokio_unstable"`.  
4. **Capturar `gdb` backtrace** cuando el servidor cuelga: `kill -SIGUSR1 <PID>` (si se configura handler) o `gdb -p <PID> -ex "thread apply all bt" -ex quit`.  
5. **Monitorear fd count** periódicamente desde el host: `docker exec <container> ls /proc/1/fd | wc -l`.  
6. **Revisar si `generate_context_summary`** (spawn fire-and-forget) o algún otro spawn se acumula sin terminar.  

### Datos del servidor:  

```
Stack: Axum 0.7.9 + Hyper 1.8.1 + hyper-util 0.1.20 + SQLx 0.8.6 (PostgreSQL)
tokio: multi_thread, 4 workers
DB pool: max=10, min=1
AI: Groq (3 keys × 3 models) + Gemini (6 models), cada request hasta 90s
Container: Docker en Coolify v4.0.0-beta.460
RAM disponible: ~5.5GB | Disco: ~190GB libres
Puerto: 3000 (bind 0.0.0.0)
Proxy: coolify-proxy (Traefik v3.6)
Repo: github.com/1ndoryu/glory-rs, rama glory-rust-nakomi
Último commit: ddc59a6b
Health endpoint: https://nakomi.studio/healthz
```

---

---

## Resumen Ejecutivo

El servidor Rust (Axum 0.7.9 + Hyper 1.x + tokio) de nakomi.studio **sigue colgándose** pese a 7 intentos de fix. El patrón es consistente: después de cada restart funciona durante 1.5-6 horas, luego el servidor deja de responder sin crashear.

**7 intentos de fix realizados (todos insuficientes):**
1. TCP SO_KEEPALIVE + WS timeouts + pool limits → insuficiente
2. Migración a `hyper_util::auto::Builder` + `header_read_timeout(30s)` → insuficiente (features cfg-gated)
3. Hyper features explícitas + HTTP/2 keep-alive → insuficiente (425 CLOSE_WAIT en 8h)
4. TCP keepalive agresivo + timeout absoluto por conexión (10min) → mejora parcial (~6h)
5. GracefulShutdown + `half_close(false)` → 15 CLOSE_WAIT/6h pero sigue cayendo
6. TCP keepalive per-accepted-socket + watchdog atómico → desplegado, mismo patrón
7. **Semaphore(3) AI + timeout(600s) + AtomicU64 + métricas health** → caído en ~1.5h

**Conclusión: PEDIR AYUDA EXTERNA.** Las hipótesis exploradas (CLOSE_WAIT, TCP keepalive, concurrencia AI) NO son la causa raíz.

**Root cause real descubierto en fix v5:** `accept()` en Linux **NO hereda** `SO_KEEPALIVE` del listening socket. Los fixes v1-v4 aplicaban keepalive al listener (inútil). Las conexiones reales de Traefik → servidor arrancaban con keepalive desactivado o con defaults del kernel (7200s). Además, el watchdog HTTP generaba sus propias conexiones CLOSE_WAIT al probar `http://127.0.0.1:3000/healthz` cada 30s, y mataba la app con `exit(1)` cuando esas conexiones fallaban.

---

## Timeline Completo

| Hora (UTC) | Evento |
|---|---|
| ~11:00 | Usuario reporta que amigos intentan acceder y la página no carga |
| 13:30 | Diagnóstico SSH: descartado DDoS (9 conexiones, carga normal) |
| 13:35 | Verificado SSL válido, Traefik labels correctos, contenedor corriendo |
| 13:38 | Test interno desde Traefik al puerto 3000: **timeout** |
| 13:40 | `/proc/net/tcp` revela ~120 conexiones CLOSE_WAIT en puerto 3000 |
| 13:42 | `docker restart` → sitio recuperado, HTTP 200 en 60ms |
| ~14:00 | **Fix v1** (262eb2b5): SO_KEEPALIVE + WS timeouts + pool limits → deployado |
| ~14:30 | Sitio vuelve a caer con CLOSE_WAIT acumulándose |
| ~15:00 | **Fix v2** (86e74fc4): migración a hyper_util::auto::Builder + header_read_timeout → deployado |
| ~16:00 | Sitio vuelve a caer — se descubre que `hyper = "1"` sin features = NO-OP |
| ~16:43-16:48 | Se intenta healthcheck Docker → amplifica el problema (503 cascade), revertido |
| ~17:00 | **Fix v3** (59ee3c70): hyper features explícitas + HTTP/2 keep-alive → commit + push |
| 18:07 | Deploy vía coolify-manager-rs iniciado |
| 18:22 | Deploy completado — contenedor reinicia con nuevo binario |
| ~20:15 | **Sitio DOWN de nuevo** — 425 CLOSE_WAIT acumulados en ~2 horas |
| ~20:18 | `docker restart` manual → sitio restaurado (HTTP 200) |
| ~20:20 | Documento actualizado con status: **FIX NO ENCONTRADO** |
| 2026-06-09 ~22:51 | **Fix v4** (2c25102f): TCP keepalive agresivo + timeout absoluto 10min |
| 2026-06-09 ~23:32 | **Fix v4.1** (21bca97a): GracefulShutdown + half_close(false) |
| 2026-06-10 ~03:44 | Deploy v4.1 a producción (imagen `7382774993fc`) |
| 2026-06-10 ~09:45 | **Sitio DOWN** — 15 CLOSE_WAIT en 6h (mejora 30x vs v3, pero NO definitivo) |
| 2026-06-10 ~09:50 | `docker restart` → sitio restaurado (HTTP 200) |
| 2026-06-10 ~10:29 | Contenedor reiniciado (otro agente o Coolify auto-restart) |
| 2026-06-10 ~10:30-11:30 | **Fix v5** implementado por otro agente: TCP keepalive por socket aceptado + watchdog atómico + timeout 300s |
| 2026-06-10 ~11:30 | **Sitio DOWN de nuevo** — contenedor con ~1h de uptime |
| 2026-06-10 ~11:32 | Deploy v5 vía `deploy-service --skip-backup` iniciado |
| 2026-06-10 ~11:35 | Restauración con `docker restart` — HTTP 200 |
| 2026-06-10 ~12:37 | Restauración #2 con `docker restart` — HTTP 200 |
| 2026-06-10 ~13:04-13:19 | **Deploy v6** (semáforo AI + timeout 600s + métricas) — build 817s, swap exitoso, health 200 |
| 2026-06-10 ~14:55 | **Caída #8** — v6 activo solo ~1.5h. Fix de semáforo/concurrencia NO resuelve. |
| 2026-06-10 ~14:59 | Restauración #8 con `docker restart` — HTTP 200. **PEDIR AYUDA A OTRO AGENTE.** |

---

## Estado Actual del Stack

| Componente | Versión/Config |
|---|---|
| **Axum** | 0.7.9 |
| **Hyper** | 1.x con features `["http1", "http2", "server"]` |
| **hyper-util** | 0.1.x con features `["server-auto", "http1", "http2", "tokio"]` |
| **tokio** | multi_thread, 4 workers |
| **SQLx** | 0.8 (PostgreSQL), pool: max=10, min=1, lifetime=1800s, idle=300s, acquire=5s |
| **Proxy** | coolify-proxy (Traefik v3.6) |
| **Container** | Docker en Coolify v4.0.0-beta.460, restart: `unless-stopped` |
| **Puerto** | 3000 (bind 0.0.0.0) |

---

## Intentos de Fix Realizados

### ❌ Fix v1 (commit `262eb2b5`) — INSUFICIENTE
**Qué:** TCP SO_KEEPALIVE (60s), graceful shutdown (SIGTERM/SIGINT), WS timeouts (300s), SQLx pool limits.  
**Resultado:** CLOSE_WAIT sigue acumulándose. Site cae ~26 min después de restart.  
**Por qué falló:** SO_KEEPALIVE opera a nivel TCP y solo detecta peers muertos (default 2h Linux). CLOSE_WAIT no es peer muerto — es que la app no llama `close()` tras recibir FIN. TCP keepalive no fuerza el cierre.

### ❌ Fix v2 (commit `86e74fc4`) — INSUFICIENTE
**Qué:** Reemplazar `axum::serve()` con `hyper_util::auto::Builder` + `http1().header_read_timeout(30s)` + `keep_alive(true)`.  
**Resultado:** Site sigue cayendo.  
**Por qué falló:** `hyper = "1"` sin features explícitas = `header_read_timeout()` es cfg-gated y NO HACE NADA. El método compila pero es un stub. `axum::serve()` no expone configuración de conexiones (tokio-rs/axum#2939).

### ❌ Fix v3 (commit `59ee3c70`) — INSUFICIENTE
**Qué:** Features explícitas en hyper `["http1", "http2", "server"]` + hyper-util `["http2"]` + HTTP/2 keep-alive (`keep_alive_interval(30s)`, `keep_alive_timeout(10s)`).  
**Resultado:** 425 CLOSE_WAIT acumulados en ~8 horas. Site cae de nuevo. Reducción de velocidad pero no eliminación.  
**Por qué no funcionó:** HTTP/2 keep-alive solo aplica si Traefik negocia h2c (improbable). Y tampoco cierra CLOSE_WAIT — solo detecta peers silenciosos en HTTP/2, no sockets que ya recibieron FIN.

### ❌ Fix v4 (commit `2c25102f`) — MEJORA PARCIAL
**Qué:** 3 capas complementarias:
1. TCP keepalive con parámetros agresivos: `tcp_keepalive_time=60s`, `tcp_keepalive_interval=15s`, `tcp_keepalive_probes=4` (detecta peers muertos en ~120s vs 7200s del kernel default).
2. Timeout absoluto por conexión: 10 minutos via `tokio::time::timeout` que fuerza `drop()` del `TcpStream` → `close()` automático vía RAII.
3. Eliminación del `shutdown_signal()` interno por conexión (redundante con outer loop).

**Resultado:** Caída en ~6 horas. Reducción significativa pero no eliminación.  
**Problema:** `set_tcp_keepalive` de `socket2` en Linux usa `setsockopt` directo — puede no aplicar a conexiones ya establecidas. Y 10 min timeout no se activa si hyper mantiene la conexión "activa" (keep-alive sin tráfico real).

### ⚠️ Fix v4.1 (commit `21bca97a`) — EN PRODUCCIÓN, MEJORA SIGNIFICATIVA
**Qué:** GracefulShutdown (`hyper_util::server::graceful::GracefulShutdown`) + `half_close(false)` + TCP keepalive agresivo + timeout absoluto (600s).  
**Resultado:** **15 CLOSE_WAIT en 6 horas** (vs 425 en 8 horas antes). Mejora de ~30x. Pero sitio sigue cayendo.  
**Observación clave:** Con solo 15 CLOSE_WAIT, el event loop NO debería saturarse. **15 conexiones zombie NO matan un servidor.** El cuello de botella está en otra parte — posiblemente el watchdog.

### ✅ Fix v5 (commit `...`) — ROOT CAUSE REAL (PARCIAL)
**Qué:** 3 correcciones fundamentales:
1. **TCP keepalive por socket aceptado** (CRÍTICO): `accept()` en Linux NO hereda SO_KEEPALIVE. Ahora cada conexión aceptada se convierte a `socket2::Socket`, se aplica keepalive (60s idle, 15s interval), y se convierte de vuelta. Los fixes v1-v4 aplicaban keepalive al listener (NO-OP).
2. **Watchdog atómico sin HTTP**: Reemplaza `spawn_http_watchdog` (que generaba CLOSE_WAIT con probes TCP a 127.0.0.1:3000) por un `AtomicU64` heartbeat que el server loop actualiza en cada `accept()`. El watchdog lee el timestamp sin generar tráfico TCP.
3. **Timeout absoluto reducido a 300s**: Match con WS inactivity timeout (300s). HTTP normal cierra mucho antes (header_read_timeout 30s).

**Resultado:** Desplegado, mismo patrón de caída. Keepalive NO es el problema real.

### ❌ Fix v6 (commit `ddc59a6b`) — INSUFICIENTE (caída en ~1.5h)
**Qué:** Control de concurrencia AI + observabilidad:
1. **Semaphore(3)** para requests AI concurrentes — protege DB pool de 10 conexiones
2. **Timeout(600s)** global en `session_timing_loop` como safety net
3. **Timeout(60s)** en `generate_context_summary` (fire-and-forget)
4. **AtomicU64** counter de timing loops activos
5. **Health endpoint mejorado** con métricas: `active_timing_loops`, `registered_sessions`, `ai_permits_available`

**Resultado:** Deploy exitoso (build 817s, swap, health 200). **Caído en ~1.5 horas.** Concurrencia AI NO era el killer.  
**Métricas post-restart:** `{active_timing_loops: 0, registered_sessions: 0, ai_permits_available: 3}` — todo normal al inicio.  
**Conclusión:** Ni CLOSE_WAIT, ni keepalive, ni concurrencia AI son la causa raíz. **Se necesita diagnóstico externo.**

---

## Configuración Actual del Servidor (`src/main.rs`) — Fix v4.1

```rust
/* GracefulShutdown + TCP keepalive agresivo + timeout absoluto */
use hyper_util::server::graceful::GracefulShutdown;

// TCP Socket con keepalive agresivo
let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
socket.set_nonblocking(true)?;
socket.set_reuse_address(true)?;
socket.set_keepalive(true)?;
socket.set_tcp_keepalive(
    socket2::TcpKeepalive::new()
        .with_time(Duration::from_secs(60))     // Primer probe tras 60s (default: 7200s)
        .with_interval(Duration::from_secs(15))  // Intervalo entre probes: 15s
)?;
socket.bind(&addr.into())?;
socket.listen(1024)?;
let listener = TcpListener::from_std(socket.into())?;

let builder = auto::Builder::new(TokioExecutor::new());
// HTTP/1.1: header_read_timeout + half_close(false)
builder.http1()
    .timer(TokioTimer::new())
    .header_read_timeout(Duration::from_secs(30))
    .keep_alive(true)
    .half_close(false);
// HTTP/2: keep-alive
builder.http2()
    .keep_alive_interval(Duration::from_secs(30))
    .keep_alive_timeout(Duration::from_secs(10));

// GracefulShutdown wrapper
let graceful = GracefulShutdown::new();

loop {
    let (io, _remote_addr) = tokio::select! {
        result = listener.accept() => { result? }
        _ = shutdown_signal() => { break; }
    };

    let watch = graceful.watch();
    let hyper_service = tower_service_fn(move |req| { /* router */ });

    tokio::spawn(async move {
        let conn = builder.serve_connection_with_upgrades(io, hyper_service);
        let conn_with_watch = watch.watch(conn);
        // Timeout absoluto: 600s — fuerza drop() del socket
        let _ = tokio::time::timeout(Duration::from_secs(600), conn_with_watch).await;
    });
}

// Shutdown graceful: espera hasta 5s por conexiones activas
let _ = tokio::time::timeout(Duration::from_secs(5), graceful.shutdown()).await;

    let hyper_service = tower_service_fn(move |req: Request<Incoming>| {
        // ... router handler
    });

    tokio::spawn(async move {
        let conn = builder
            .serve_connection_with_upgrades(io, hyper_service);
        if let Err(e) = conn.await {
            eprintln!("Error serving connection: {}", e);
        }
    });
}
```

---

## Hipótesis Pendientes (NO verificadas)

Las siguientes hipótesis NO se han confirmado ni descartado:

1. **Traefik mantiene conexiones abiertas demasiado tiempo:** Traefik v3.6 puede tener timeout de idle distinto al de Hyper. Si Traefik cierra su lado pero Hyper no recibe el FIN correctamente (por buffering o TCP window), la conexión queda en CLOSE_WAIT.

2. **HTTP/2 sobre TCP (h2c) no se beneficia de `header_read_timeout`:** Si Traefik negocia HTTP/2 (h2c) con el servidor, `header_read_timeout` solo aplica a HTTP/1.1. Las conexiones h2 se manejan por `keep_alive_interval/timeout`, pero quizás no están funcionando correctamente.

3. **`serve_connection_with_upgrades` tiene un bug con upgrades:** El método `.serve_connection_with_upgrades()` es necesario para WebSocket, pero puede tener un comportamiento inesperado con conexiones que no hacen upgrade — manteniendo el socket abierto indefinidamente.

4. **El loop de aceptación (`listener.accept()`) crea conexiones que nunca se cierran:** Cada `tokio::spawn` del handler puede no estar propagando el cierre correctamente si el handler entra en un error path que no hace `drop`.

5. **Race condition en Traefik ↔ servidor:** Si Traefik reutiliza conexiones keep-alive del pool pero el servidor las cierra por su timeout, puede haber un race donde Traefik envía datos sobre una conexión que el servidor ya cerró → el servidor queda en CLOSE_WAIT esperando que la conexión se limpie.

6. **`socket2::Socket::set_keepalive(true)` no configura los parámetros:** En Rust, `set_keepalive(true)` habilita SO_KEEPALIVE pero usa los defaults del kernel Linux (`tcp_keepalive_time=7200s`, `tcp_keepalive_intvl=75s`, `tcp_keepalive_probes=9`). Se necesitan `tcp_keepalive_time` específico o usar `set_tcp_keepalive()` con parámetros custom.

7. **hyper-util `0.1.x` tiene un bug conocido:** La integración entre hyper 1.x y hyper-util 0.1.x puede tener edge cases con la limpieza de conexiones idle. Verificar issues en https://github.com/hyperium/hyper-util.

8. **El contenedor Docker usa networking `bridge`:** Las conexiones pasan por el bridge network de Docker, que puede interferir con TCP FIN/ACK propagation. Probar con `network_mode: host` como diagnóstico.

---

## Checklist Post-Incidente

- [x] Servicio restaurado (docker restart)
- [x] Causa raíz identificada (connection leak por falta de timeouts)
- [x] Documentación del incidente creada
- [x] Fix v1 implementado (keep-alive, graceful shutdown, WS timeouts, pool limits) — commit `262eb2b5` — **insuficiente**
- [x] Fix v2 implementado (hyper_util::auto::Builder + header_read_timeout) — commit `86e74fc4` — **insuficiente**
- [x] Fix v3 implementado (hyper features explícitas + HTTP/2 keep-alive) — commit `59ee3c70` — **insuficiente**
- [x] Fix v4 implementado: TCP keepalive agresivo + timeout absoluto por conexión — commit `2c25102f` — **mejora parcial**
- [x] Fix v4.1 implementado: GracefulShutdown + half_close(false) — commit `21bca97a` — **mejora significativa (15 CLOSE_WAIT/6h vs 425/8h) pero NO definitivo**
- [x] Deploy v4.1 a producción — **confirmado: imagen desplegada, sitio caído tras 6h**
- [x] Restauración con restart — **2026-06-10 ~09:50 UTC, HTTP 200**
- [x] Fix v5 implementado: keepalive per-accepted-socket + watchdog atómico + timeout 300s
- [x] Sitio caído nuevamente tras ~1h de uptime (2026-06-10 ~11:30 UTC)
- [ ] Deploy fix v5 a producción vía `deploy-service --skip-backup` — **EN CURSO**
- [ ] Verificar post-deploy: CLOSE_WAIT ~0 tras 24h
- [ ] Si v5 no resuelve: buscar ayuda externa (el usuario buscará soporte en otro lugar)
- [ ] Agregar Docker healthcheck al compose
- [ ] Monitoreo proactivo (alerta CLOSE_WAIT > 20)

### Estado: BUSCANDO AYUDA EXTERNA
El usuario ha decidido buscar ayuda fuera del equipo. **6 intentos de fix** no resolvieron el problema. El patrón es claro:
- v1→v3: reducción gradual de CLOSE_WAIT pero sitio sigue cayendo
- v4.1: mejora significativa (15 CLOSE_WAIT/6h) pero sitio sigue cayendo
- v5: fix teóricamente correcto (keepalive por socket aceptado) — pendiente verificación

**Nota para quien tome el caso:** El problema persiste pese a múltiples correcciones teóricamente correctas. Sospechar de factores externos: Traefik timeout mismatch, Docker networking bridge, o un bug en hyper-util 0.1.x.

---

## Prevención Futura

### Monitoreo proactivo
- Alertar si conexiones CLOSE_WAIT > 20 en `/proc/net/tcp`
- Docker health check: `curl -f http://localhost:3000/healthz` cada 30s (solo después de fix)
- Auto-restart del contenedor si health falla 3 veces consecutivas

### En el código
- TCP keepalive obligatorio en todos los servidores Axum
- WebSockets con timeout de inactividad (5 min)
- Pool SQLx con `max_lifetime` y `idle_timeout`
- HTTP clients compartidos (no crear `reqwest::Client::new()` en handlers)

### En Coolify
- Health check HTTP configurado
- Restart policy: `on-failure` con max 3 reintentos

---

## Info de Debug Recopilada

### Datos del contenedor al momento de la caída (~20:15 UTC)
```
Container:    app-do8k4w8swccwwogoc0os0ck0
Started:      2026-06-09T18:22:21 (post deploy fix v3)
Image:        sha256:5b9b6c7d2acc6ba6620a53ffa1ec6be96ee798be5f7fd037329108fd64c8782c
CLOSE_WAIT:   425 conexiones
HTTP:         timeout (000)
Watchdog:     fallando continuamente (error sending request for url)
```

### Datos del contenedor al momento de la caída (~11:30 UTC, 2026-06-10)
```
Container:    app-do8k4w8swccwwogoc0os0ck0
Started:      2026-06-10T10:29:19 (último restart)
Uptime:       ~1 hora antes de caer
Status:       Up About an hour
CLOSE_WAIT:   no medible (parse error en awk dentro del contenedor)
HTTP:         timeout (curl no responde)
Fix activo:   v4.1 (21bca97a) — GracefulShutdown + TCP keepalive
```

### Datos del servidor
```
VPS:          66.94.100.241
RAM libre:    ~5693MB
Disco libre:  ~184GB
Coolify:      v4.0.0-beta.460
Proxy:        coolify-proxy (Traefik v3.6)
Postgres:     postgres-do8k4w8swccwwogoc0os0ck0
```

### Commits relacionados
````
(pendiente) 096A-5: fix CLOSE_WAIT real — keepalive per-socket + watchdog atómico  (v5 — ROOT CAUSE REAL)
21bca97a  096A-4: GracefulShutdown + TCP keepalive agresivo para CLOSE_WAIT  (v4.1 — mejora significativa, NO definitivo)
2c25102f  096A-4: fix CLOSE_WAIT leak — TCP keepalive agresivo + timeout absoluto  (v4 — mejora parcial)
59ee3c70  096A-3: hyper features explícitas + HTTP/2 keep-alive  (v3 — NO resuelve)
86e74fc4  096A-2: hyper_util::auto::Builder + header_read_timeout (v2 — NO resuelve)
262eb2b5  fix(096A): SO_KEEPALIVE + WS timeouts + pool limits     (v1 — NO resuelve)
````

### Archivos modificados en los intentos de fix
- `Cargo.toml` — hyper features, hyper-util features, socket2 dependency
- `src/main.rs` — server loop (hyper_util::auto::Builder), TCP socket config, watchdog, graceful shutdown
- `src/handlers/chat/ws_visitor_helpers.rs` — WS timeout 300s
- `src/handlers/chat/ws_staff.rs` — WS timeout 300s
- `src/handlers/notifications.rs` — WS timeout 300s

### URL de referencia
- Axum issue sobre configuración de conexiones: https://github.com/tokio-rs/axum/issues/2939
- hyper-util repo: https://github.com/hyperium/hyper-util

---

## Timeline del Incidente

| Hora (UTC) | Evento |
|---|---|
| 2026-06-09 ~11:00 | Sitio DOWN detectado. CLOSE_WAIT acumulados. Restart restaura. |
| 2026-06-09 ~11:19 | Fix v1 implementado (`262eb2b5`) — SO_KEEPALIVE + WS timeouts + pool limits |
| 2026-06-09 ~11:52 | Fix v2 implementado (`86e74fc4`) — hyper_util::auto::Builder + header_read_timeout |
| 2026-06-09 ~13:52 | Fix v3 implementado (`59ee3c70`) — hyper features explícitas + HTTP/2 keep-alive |
| 2026-06-09 ~15:22 | Deploy v3 a producción via coolify-manager-rs |
| 2026-06-09 ~23:00 | Sitio DOWN nuevamente. 425 CLOSE_WAIT acumulados en 8h. |
| 2026-06-09 ~22:51 | Fix v4 implementado (`2c25102f`) — TCP keepalive agresivo + timeout absoluto |
| 2026-06-09 ~23:32 | Fix v4.1 implementado (`21bca97a`) — GracefulShutdown + half_close(false) |
| 2026-06-10 ~03:44 | Deploy v4.1 a producción (imagen `7382774993fc`) |
| 2026-06-10 ~09:45 | Sitio DOWN. 15 CLOSE_WAIT en 6h (mejora significativa). Restart restaura. |
| 2026-06-10 ~09:50 | Sitio restaurado con restart. HTTP 200. |
| 2026-06-10 ~10:08 | Fix v5 implementado — keepalive per-accepted-socket + watchdog atómico + timeout 300s |
| **Próximo** | Deploy fix v5 a producción |

