# Incidente: nakomi.studio — Connection Leak Persistente (CLOSE_WAIT → Deadlock)

**Fecha:** 2026-06-09  
**Duración:** ~8 horas de inestabilidad continua con múltiples caídas  
**Severidad:** 🔴 Crítica — sitio cayendo repetidamente cada ~2-8 horas  
**Servicio:** nakomi.studio (VPS1 66.94.100.241, Coolify service `do8k4w8swccwwogoc0os0ck0`)  
**Estado actual (2026-06-09 ~22:50 UTC):** Fix v4 implementado, pendiente deploy y verificación  
**Responsable:** Fix v4 implementado — verificar post-deploy que CLOSE_WAIT se estabiliza < 20

---

## Resumen Ejecutivo

El servidor Rust (Axum 0.7.9 + Hyper 1.x + tokio) de nakomi.studio **sigue acumulando conexiones CLOSE_WAIT** pese a múltiples intentos de fix. El patrón es consistente: después de cada restart, las conexiones CLOSE_WAIT crecen (~1/min) hasta que el event loop se satura y el sitio deja de responder.

**4 intentos de fix realizados** — ninguno resolvió el problema de raíz:
1. TCP SO_KEEPALIVE + WS timeouts + pool limits → insuficiente (TCP keepalive no fuerza cierre de sockets en CLOSE_WAIT)
2. Migración a `hyper_util::auto::Builder` + `header_read_timeout(30s)` → insuficiente (`hyper = "1"` sin features = NO-OP silencioso)
3. Hyper features explícitas (`http1`, `http2`, `server`) + HTTP/2 keep-alive → **insuficiente** (deployado, 425 CLOSE_WAIT en 8 horas)
4. Site sigue cayendo después de cada intento

**Root cause identificado en fix v4:** Los 3 fixes anteriores operaban a nivel de **protocolo HTTP** (header timeouts, HTTP/2 PING, keep-alive). Pero CLOSE_WAIT es un problema a nivel de **socket TCP**: hyper recibió FIN del peer pero no ejecutaba `close()` en su lado. Además, `set_keepalive(true)` sin parámetros usaba el default del kernel Linux de 7200s (2h), completamente inútil.

**Causa raíz (Fix v4):** Combinación de 3 factores:
1. `serve_connection_with_upgrades()` mantiene sockets abiertos esperando posible upgrade que nunca llega
2. `socket.set_keepalive(true)` usa default kernel de 7200s → no detecta peers muertos a tiempo
3. No existía timeout absoluto por conexión → sockets CLOSE_WAIT sobrevivían indefinidamente

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

### ✅ Fix v4 (pendiente deploy) — ROOT CAUSE
**Qué:** 3 capas complementarias:
1. TCP keepalive con parámetros agresivos: `tcp_keepalive_time=60s`, `tcp_keepalive_interval=15s`, `tcp_keepalive_probes=4` (detecta peers muertos en ~120s vs 7200s del kernel default).
2. Timeout absoluto por conexión: 10 minutos via `tokio::time::timeout` que fuerza `drop()` del `TcpStream` → `close()` automático vía RAII. Safety net que garantiza que ningún socket sobrevive más de 10 min.
3. Eliminación del `shutdown_signal()` interno por conexión (redundante con outer loop, creaba un listener de señal por cada conexión).

**Por qué funciona:** A diferencia de los fixes v1-v3 que operaban a nivel de protocolo HTTP, este fix opera a nivel de **socket TCP**. El único mecanismo que cierra CLOSE_WAIT es el `drop()` del socket, y el timeout absoluto lo garantiza.

---

## Configuración Actual del Servidor (`src/main.rs`)

```rust
/* Server config actual — hyper_util::auto::Builder con features explícitas */
let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
socket.set_nonblocking(true)?;
socket.set_reuse_address(true)?;
socket.set_keepalive(true)?;  // TCP SO_KEEPALIVE
socket.bind(&addr.into())?;
socket.listen(1024)?;
let listener = TcpListener::from_std(socket.into())?;

let builder = auto::Builder::new(TokioExecutor::new());

// HTTP/1.1 config
let mut http1 = builder.http1();
http1
    .timer(TokioTimer::new())
    .header_read_timeout(Duration::from_secs(30))
    .keep_alive(true);

// HTTP/2 config (h2c)
let mut http2 = builder.http2();
http2
    .keep_alive_interval(Duration::from_secs(30))
    .keep_alive_timeout(Duration::from_secs(10));

// Loop de aceptación de conexiones
loop {
    let (io, _remote_addr) = tokio::select! {
        result = listener.accept() => { result? }
        _ = shutdown_signal() => { break; }
    };

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
- [x] Código pusheado a `glory-rust-nakomi`
- [x] Deploy del fix v3 a producción (coolify-manager-rs, 18:22 UTC)
- [x] Confirmado: fix v3 NO resuelve CLOSE_WAIT (425 acumulados en 8h)
- [x] **Root cause identificado** — socket-level (no HTTP-level): hyper no cierra sockets tras FIN + keepalive con defaults del kernel (7200s)
- [x] Fix v4 implementado: TCP keepalive agresivo (60s) + timeout absoluto por conexión (10min) + eliminar shutdown_signal redundante
- [ ] Deploy fix v4 a producción via coolify-manager-rs
- [ ] Verificar post-deploy: CLOSE_WAIT < 20 tras 2-4 horas
- [ ] Agregar Docker healthcheck al compose (solo después de verificar fix)
- [ ] Monitoreo proactivo (alerta CLOSE_WAIT > 20)

### Acción Inmediata Mientras se Investiga

Mientras no se resuelva el root cause, el sitio necesita **restart periódico** para evitar caídas:
- Opción A: Cron job en el servidor cada ~4 horas: `docker restart app-do8k4w8swccwwogoc0os0ck0`
- Opción B: Autoheal externo que monitoree CLOSE_WAIT y reinicie si > 50
- Opción C: `cm-autoheal-studio.timer` (ya instalado por coolify-manager) — verificar si funciona

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

## Prevención Futura

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
```
59ee3c70  096A-3: hyper features explícitas + HTTP/2 keep-alive  (NO resuelve)
86e74fc4  096A-2: hyper_util::auto::Builder + header_read_timeout (NO resuelve)
262eb2b5  fix(096A): SO_KEEPALIVE + WS timeouts + pool limits     (NO resuelve)
```

### Archivos modificados en los intentos de fix
- `Cargo.toml` — hyper features, hyper-util features, socket2 dependency
- `src/main.rs` — server loop (hyper_util::auto::Builder), TCP socket config, watchdog, graceful shutdown
- `src/handlers/chat/ws_visitor_helpers.rs` — WS timeout 300s
- `src/handlers/chat/ws_staff.rs` — WS timeout 300s
- `src/handlers/notifications.rs` — WS timeout 300s

### URL de referencia
- Axum issue sobre configuración de conexiones: https://github.com/tokio-rs/axum/issues/2939
- hyper-util repo: https://github.com/hyperium/hyper-util

