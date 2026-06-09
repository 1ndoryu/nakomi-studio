# Incidente: nakomi.studio DOWN por Connection Leak → Deadlock

**Fecha:** 2026-06-09  
**Duración estimada:** ~2 horas (detectado por usuario reportando "página no carga")  
**Severidad:** 🔴 Crítica — sitio completamente inaccesible  
**Servicio:** nakomi.studio (VPS1 66.94.100.241, Coolify service `do8k4w8swccwwogoc0os0ck0`)

---

## Resumen Ejecutivo

El servidor Rust (Axum) de nakomi.studio acumuló **~120 conexiones TCP en estado CLOSE_WAIT** hasta que el event loop se bloqueó, volviendo el sitio inaccesible. Traefik (coolify-proxy) no podía establecer nuevas conexiones al puerto 3000 del contenedor — el puerto escuchaba pero el proceso no respondía.

**Causa raíz:** El servidor Axum no configura timeouts de keep-alive HTTP, ni TCP keepalive, ni graceful shutdown. Las conexiones que los clientes (navegadores, Traefik) cerraban no se limpiaban del lado del servidor, acumulándose hasta el deadlock.

**Fix inmediato:** `docker restart app-do8k4w8swccwwogoc0os0ck0` → recuperado en ~5 segundos.

---

## Timeline

| Hora (UTC) | Evento |
|---|---|
| ~11:00 | Usuario reporta que amigos intentan acceder y la página no carga |
| 13:30 | Diagnóstico SSH: descartado DDoS (9 conexiones, carga normal) |
| 13:35 | Verificado SSL válido, Traefik labels correctos, contenedor corriendo |
| 13:38 | Test interno desde Traefik al puerto 3000: **timeout** |
| 13:40 | `/proc/net/tcp` revela ~120 conexiones CLOSE_WAIT en puerto 3000 |
| 13:42 | `docker restart` → sitio recuperado, HTTP 200 en 60ms |

---

## Diagnóstico Técnico

### Señales observadas

1. **Proxy:** coolify-proxy (Traefik v3.6) healthy, 5 días uptime
2. **Contenedor:** `nakomi-rust` corriendo, PID 1 sleeping (estado normal)
3. **Puerto 3000:** LISTEN en `0.0.0.0:3000` — el socket aceptaba conexiones
4. **Conexiones:** ~120 en CLOSE_WAIT (state `08` en `/proc/net/tcp`)
5. **Health check interno:** `wget http://127.0.0.1:3000/` → timeout
6. **Traefik → container:** timeout en todas las pruebas

### Interpretación

`CLOSE_WAIT` significa que el lado remoto (Traefik, navegador) cerró la conexión TCP, pero el servidor Rust no cerró su lado. Con ~120 conexiones zombie, el event loop de tokio se saturó procesando I/O en sockets muertos, impidiendo aceptar/procesar nuevas conexiones.

```
Cliente/Traefik          Servidor Rust (puerto 3000)
    │                         │
    │──── FIN ──────────────→ │  Cliente cierra
    │                         │  Estado: CLOSE_WAIT
    │                         │  (servidor debería cerrar, pero no lo hace)
    │                         │
    │  × timeout ×           │  Nuevas conexiones no se procesan
```

---

## Causas Encontradas en el Código

### 1. 🔴 Servidor Axum sin timeouts ni graceful shutdown (`src/main.rs:55-62`)

```rust
// ACTUAL — sin keep-alive, sin graceful shutdown
let listener = tokio::net::TcpListener::bind(&addr).await?;
axum::serve(
    listener,
    app.into_make_service_with_connect_info::<SocketAddr>(),
)
.await?;
```

**Problemas:**
- Sin `with_graceful_shutdown()` → SIGTERM corta conexiones abruptamente
- Sin TCP keepalive → conexiones inactivas nunca se detectan como muertas
- Hyper mantiene HTTP/1.1 keep-alive indefinidamente por defecto

### 2. 🔴 Pool SQLx sin límites de vida (`src/main.rs:35-40`)

```rust
// ACTUAL — sin max_lifetime, sin idle_timeout
let pool = sqlx::postgres::PgPoolOptions::new()
    .max_connections(10)
    .min_connections(2)
    .connect(&config.database_url)
    .await?;
```

### 3. 🔴 WebSockets sin timeout de inactividad

Los 3 handlers WS (`ws_visitor.rs`, `ws_staff.rs`, `notifications.rs`) usan:
```rust
while let Some(Ok(msg)) = receiver.next().await { ... }
```
Sin timeout — un cliente que abre WS y no envía nada mantiene la conexión indefinidamente.

### 4. 🟡 Clientes HTTP (reqwest) sin pool compartido

7+ instancias de `reqwest::Client::new()` en `ai_tools.rs` sin timeout ni pool management.

---

## Fix Aplicado

### Inmediato (este incidente)
- `docker restart` del contenedor → servicio restaurado

### Mitigaciones implementadas (prevención)

#### 1. TCP keepalive + graceful shutdown en `main.rs`
```rust
listener.set_keepalive(Some(Duration::from_secs(60)))?;
axum::serve(...)
    .with_graceful_shutdown(shutdown_signal())
    .await?;
```

#### 2. Pool SQLx con límites de vida
```rust
.max_lifetime(Duration::from_secs(1800))  // 30 min
.idle_timeout(Duration::from_secs(300))   // 5 min
.acquire_timeout(Duration::from_secs(5))
```

#### 3. WebSocket timeouts de inactividad (300s)
```rust
match tokio::time::timeout(Duration::from_secs(300), receiver.next()).await {
    Ok(Some(Ok(msg))) => { /* procesar */ }
    _ => break, // timeout o desconexión → cerrar limpiamente
}
```

#### 4. Docker health check en Coolify
Configurar health check HTTP en el compose de Coolify para auto-restart si el contenedor vuelve a colgar.

---

## Lecciones Aprendidas

1. **Axum/Hyper no configura keep-alive por defecto** — hay que hacerlo explícitamente
2. **Los WebSockets sin timeout son una bomba de tiempo** — cada conexión abierta consume un slot tokio
3. **El watchdog interno (`spawn_http_watchdog`) no fue suficiente** — hace probe a `/healthz` cada 30s, pero el proceso estaba vivo (solo colgado). El probe local (`127.0.0.1`) quizá se procesaba en un path diferente al de Traefik
4. **Coolify sin health check Docker** = sin auto-restart. El contenedor estaba "running" pero inútil
5. **CLOSE_WAIT es el síntoma clave** — si `/proc/net/tcp` muestra muchas conexiones CLOSE_WAIT, la app tiene un leak

---

## Checklist Post-Incidente

- [x] Servicio restaurado (docker restart)
- [x] Causa raíz identificada (connection leak por falta de timeouts)
- [x] Documentación del incidente creada
- [x] Fix de código implementado (keep-alive, graceful shutdown, WS timeouts, pool limits) — commit `262eb2b5`
- [x] Código pusheado a `glory-rust-nakomi`
- [x] Deploy del fix a producción (vía coolify-manager, 2026-06-09 17:03 CEST)
- [x] Docker health check configurado en Coolify (timeout 10s, interval 30s, retries 3, start_period 30s)
- [x] Verificar que el fix previene recurrencia — healthcheck pasando, container `healthy`

### Nota: Healthcheck intermedio fallido (16:43-16:48 UTC)

Se agregó healthcheck `curl http://localhost:3000/healthz` al compose con timeout de 5s. El servidor viejo (sin fixes) acumuló CLOSE_WAIT de nuevo y no respondió al healthcheck en 5s → Docker lo marcó `unhealthy` → Traefik devolvió **503**. Se quitó el healthcheck para restaurar el sitio. Se re-agregó después del deploy del código fixeado (con timeout ampliado a 10s) y funciona correctamente.

---

## Prevención Futura

### Monitoreo proactivo
- Alertar si conexiones CLOSE_WAIT > 20 en `/proc/net/tcp`
- Docker health check: `curl -f http://localhost:3000/healthz` cada 30s
- Auto-restart del contenedor si health falla 3 veces consecutivas

### En el código
- TCP keepalive obligatorio en todos los servidores Axum
- WebSockets con timeout de inactividad (5 min)
- Pool SQLx con `max_lifetime` y `idle_timeout`
- HTTP clients compartidos (no crear `reqwest::Client::new()` en handlers)

### En Coolify
- Health check HTTP configurado
- Restart policy: `on-failure` con max 3 reintentos
