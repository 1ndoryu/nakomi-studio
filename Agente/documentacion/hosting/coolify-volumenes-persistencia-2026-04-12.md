# Coolify: Persistencia de Volúmenes y Bind Mounts

> **Fecha:** 2026-04-12  
> **Aplica a:** Todos los deployments via Coolify + coolify-manager-rs

## Problema conocido: Coolify normaliza bind mounts a named volumes

Coolify procesa el `docker_compose_raw` y **convierte cualquier bind mount a named volume** en su versión procesada (`docker_compose`).

### Ejemplo:
```
RAW (lo que enviamos):    '/data/uploads/studio:/app/uploads'
PROCESSED (lo que Coolify genera): 'UUID_uploads-data:/app/uploads'
```

Cuando Coolify reescribe el compose a disco (restart desde UI, redeploy, auto-restart), usa su versión procesada. Esto causa que Docker cree un **named volume vacío**, perdiendo los archivos del bind mount.

## Solución implementada: `volume_manager::ensure_uploads_bind_mount()`

En `coolify-manager-rs`, cada operación que toca el compose (deploy, redeploy, restart) ejecuta un `sed` en el compose en disco para forzar el bind mount correcto:

```bash
sed -i "s|'[^']*:/app/uploads'|'/data/uploads/{site}:/app/uploads'|g" docker-compose.yml
```

### Archivos que ejecutan el fix:
- `deploy_service.rs` — después de sync_compose y antes del build
- `redeploy.rs` — después de que Coolify escribe compose (post start_service)
- `restart_site.rs` — después del restart API (solo para templates Rust)

## Env runtime en compose efectivo

Coolify puede tener una variable sincronizada en `/api/v1/services/{uuid}/envs` pero no reflejarla en el `docker-compose.yml` que queda en `/data/coolify/services/{uuid}`. En stacks Rust, `coolify-manager-rs deploy-service` no depende solo del raw de Coolify: hace el swap con `docker compose up` sobre ese archivo efectivo.

Desde `155A-11`, `deploy-service` también lee las envs runtime de Coolify e inserta en el `environment:` de `app` las claves faltantes antes de recrear el contenedor. Esto evita que `sync-env` reporte “sincronizado” mientras el runtime sigue sin ver variables como `GLORY_TEST_CHECKOUT_EMAILS`.

### Señal de diagnóstico

```bash
coolify-manager.exe sync-env --name studio --direction diff --only GLORY_TEST_CHECKOUT_EMAILS
coolify-manager.exe exec --name studio --target app --command 'printenv GLORY_TEST_CHECKOUT_EMAILS'
```

Si el primero dice sincronizado y el segundo falla, el problema está en el compose efectivo del servidor, no en el `.env` local.

### Qué NO cubre:
- **Restart desde Coolify UI** — Si alguien reinicia directamente desde la UI de Coolify, el compose se reescribirá con named volumes. El próximo deploy desde coolify-manager-rs lo corregirá automáticamente.
- **Recomendación:** SIEMPRE usar coolify-manager-rs para operaciones, nunca la UI de Coolify directamente.

## Persistencia de base de datos (PostgreSQL)

PostgreSQL usa **named volume** de Docker (`UUID_pg-data`). Esto es correcto porque:

1. Named volumes persisten a través de `docker compose up/down` (sin `-v`)
2. El nombre del volumen (`UUID_pg-data`) es consistente mientras el stack exista
3. Coolify NO cambia el tipo de mount para PostgreSQL (siempre named volume)

### Riesgos de pérdida de datos en BD:
| Acción | ¿Pierde datos? |
|--------|---------------|
| Deploy via coolify-manager-rs | NO — solo recrea app, no postgres |
| Restart via coolify-manager-rs | NO — restart no borra volúmenes |
| Restart desde Coolify UI | NO — restart no usa `-v` |
| `docker compose down -v` | **SÍ** — borra todos los named volumes |
| Eliminar servicio en Coolify | **SÍ** — borra el stack completo |
| `docker volume rm` manual | **SÍ** — borra el volumen explícitamente |

### Medida de protección:
El backup pre-deploy de coolify-manager-rs (`backup_manager`) crea una copia de la BD antes de cada deploy. También existe el comando `backup --name studio` para backups manuales.

## Estructura de volúmenes del stack nakomi.studio

| Servicio | Mount | Tipo | Persistencia |
|----------|-------|------|-------------|
| app | `/data/uploads/studio → /app/uploads` | bind mount (forzado por volume_manager) | Host filesystem |
| app | `UUID_app-data → /app/data` | named volume | Docker volume |
| postgres | `UUID_pg-data → /var/lib/postgresql/data` | named volume | Docker volume |
