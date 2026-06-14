# Prevención: broadcast::Mutex en tokio — Incidente 096A

**Fecha:** 2026-06-11
**Regla Sentinel:** `broadcast-mutex-riesgo-rs`
**Severidad:** error
**Estado:** ✅ Implementada

---

## Problema

`tokio::sync::broadcast::Sender::send()` usa `std::sync::Mutex` internamente para proteger el buffer compartido del canal. Bajo contención (múltiples sends concurrentes al mismo canal), el Mutex bloquea el OS thread del tokio worker en `futex_wait`.

Con N workers (típicamente 8), si todos se bloquean en el Mutex, el runtime tokio se congela completamente:
- Accept loop no despacha conexiones
- Heartbeat no se ejecuta
- Health checks fallan
- Watchdog detecta y reinicia

**Incidente 096A:** 15+ caídas en 3 días, 13 intentos de fix (v1-v13). El root cause fue `broadcast::Sender::send()` en el sistema de chat/notificaciones de nakomi.studio.

## Detección

Glory Sentinel detecta automáticamente:
- `use tokio::sync::broadcast` — import del módulo
- `broadcast::channel(` — creación de canal
- `broadcast::Sender` / `broadcast::Receiver` — anotaciones de tipo

## Solución

Reemplazar `broadcast` con `mpsc::unbounded_channel` por suscriptor:

```rust
// ❌ PROHIBIDO: broadcast usa Mutex interno
use tokio::sync::broadcast;
let (tx, _rx) = broadcast::channel::<Message>(100);
tx.send(msg)?; // bloquea con Mutex bajo contención

// ✅ CORRECTO: mpsc unbounded es lock-free
use tokio::sync::mpsc;
let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
tx.send(msg).ok(); // lock-free, nunca bloquea
```

Para múltiples suscriptores, usar un `Vec<mpsc::UnboundedSender>`:

```rust
type SessionSender = mpsc::UnboundedSender<Message>;

struct ChatService {
    channels: DashMap<Uuid, Vec<SessionSender>>,
}

impl ChatService {
    fn subscribe(&self, session_id: Uuid) -> mpsc::UnboundedReceiver<Message> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.channels.entry(session_id).or_default().push(tx);
        rx
    }

    fn broadcast(&self, session_id: Uuid, msg: Message) {
        if let Some(mut senders) = self.channels.get_mut(&session_id) {
            senders.retain(|tx| tx.send(msg.clone()).is_ok());
        }
    }
}
```

## Excepciones

Si se necesita `broadcast` por diseño (ej: fan-out a muchos suscriptores con backpressure), usar `sentinel-disable broadcast-mutex-riesgo-rs` con justificación documentada.

## Referencias

- [Documentación incidente 096A](../documentacion/incidents/nakomi-connection-leak-2026-06-09.md)
- [Tokio broadcast source](https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html) — nota sobre `std::sync::Mutex` interno
- Commit fix v13: `f2e20d7f`
