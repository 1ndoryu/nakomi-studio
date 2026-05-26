# Studio Freeze Watchdog — 2026-05-25

> **Tarea:** 255A-1
> **Objetivo:** capturar evidencia del proximo freeze HTTP de `nakomi.studio` fuera del runtime Rust antes de cualquier reinicio.

## Problema

`nakomi.studio` no estaba cayendo por falta de contenedor. El patron real fue: contenedor `app` vivo, proceso Rust vivo, TCP aceptando conexiones y `/healthz` colgado sin responder bytes. El timer existente `cm-autoheal-studio` intentaba reparar, pero no guardaba evidencia util del estado zombi.

## Decision

Se reemplaza el autoheal ciego por un watchdog host-level basado en [scripts/host-runtime-watchdog.sh](scripts/host-runtime-watchdog.sh).

El script corre desde el host, asi que sigue pudiendo inspeccionar Docker y `/proc` aunque Tokio deje de poll()ear HTTP. Antes de tocar redes o reiniciar el servicio, guarda un snapshot persistente en:

- `/data/uploads/studio/diagnostics/runtime-freeze/<timestamp>-<reason>/`

## Que captura

- `docker inspect`, `docker top`, `docker logs --tail 400`, `docker stats --no-stream`
- `docker compose ps`
- probe publico a `https://nakomi.studio/healthz`
- probe interno por loopback dentro del contenedor
- probe host -> IP Docker del contenedor cuando existe
- `ss -tanp` del host
- `ps -T` del PID real del proceso dentro del host
- `/proc/<pid>/{status,limits,sched,cgroup,mountinfo,wchan,syscall,stack}`
- snapshot por hilo en `/proc/<pid>/task/*`

## Comportamiento operativo

1. Si el health publico responde 200, no hace nada.
2. Si el health publico falla pero el interno responde, captura snapshot y solo intenta reparar redes (`coolify-proxy` y `coolify`) sin recrear el contenedor.
3. Si tambien falla el probe interno, captura snapshot y luego hace `docker compose restart app` para recuperar el sitio sin perder la evidencia.
4. El cooldown evita bucles de snapshots cada minuto sobre el mismo incidente.

## Instalacion real en VPS1

Se deja montado sobre la unidad existente:

- `cm-autoheal-studio.service`
- `cm-autoheal-studio.timer`

La unidad pasa a ejecutar el watchdog nuevo con `TimeoutStartSec=180` y variables por `EnvironmentFile=/etc/default/cm-autoheal-studio`.

Valores usados para `studio`:

- `SITE=studio`
- `STACK_UUID=STUDIO_STACK_UUID`
- `SERVICE_DIR=/data/coolify/services/STUDIO_STACK_UUID`
- `COMPOSE_SERVICE=app`
- `PUBLIC_HEALTH_URL=https://nakomi.studio/healthz`
- `INTERNAL_HEALTH_PATH=/healthz`
- `SNAPSHOT_ROOT=/data/uploads/studio/diagnostics/runtime-freeze`

## Validacion

Smoke de instalacion sin reiniciar la app:

```bash
FORCE_SNAPSHOT=1 NO_RESTART=1 /usr/local/bin/cm-runtime-watchdog.sh
```

Criterio de exito:

- se crea un snapshot nuevo bajo `/data/uploads/studio/diagnostics/runtime-freeze/`
- aparecen `metadata.env`, `docker-inspect.txt`, `public-health.txt`, `ps-threads.txt`
- `systemctl status cm-autoheal-studio.timer` queda `active (waiting)`

## Pendiente estructural

`coolify-manager-rs` sigue sin un boundary para instalar o administrar units/timers del host. Por eso esta preparacion todavia requiere SSH directo y debe tratarse como gap de la herramienta, no como flujo deseado permanente.