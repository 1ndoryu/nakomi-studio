# Sin métricas CPU/RAM/disco en panel infraestructura

> **Fecha:** 2026-05-23
> **Contexto:** recuperación de métricas VPS1 para `studio` / `nakomi.studio`

## Síntoma

El panel de infraestructura (`/panel/?seccion=infraestructura`) muestra:
- **Despliegues:** Lista completa (10 VPS1 + 2 VPS2) — FUNCIONA
- **VPS:** Tarjetas de servidores visibles pero CPU/RAM/disco todo `—` (null)
- **Gráficos de métricas por deployment:** Sin datos

## Causas raíz (3 problemas independientes)

### 1. `upsert_configured_server` — duplicate key (VPS1)

**Log:**
```
WARN [infra-metrics] muestra parcial fallida:
Error de base de datos: duplicate key value violates unique constraint
"idx_infrastructure_servers_coolify_uuid"
```

**Por qué pasa:**
- `COOLIFY_VPS1_BASE_URL` cambió de `http://VPS1_IP:8000` → `http://coolify:8080`
- `upsert_configured_server` usa `ON CONFLICT (server_ip, coolify_base_url)`
- La tupla `(VPS1_IP, http://coolify:8080)` NO existe, pero `(VPS1_IP, http://VPS1_IP:8000)` SÍ
- El INSERT intenta crear nueva fila con mismo `coolify_server_uuid` → viola índice único
- **Fix implementado, pusheado y desplegado** en `repositories/infrastructure.rs`: two-step UPDATE por `coolify_server_uuid` primero, `INSERT ON CONFLICT` como fallback.
- Validación operativa: el build completo de producción finalizó y los logs recientes ya no muestran `duplicate key`.

### 2. `ssh=missing` en la imagen Docker

**Diagnóstico antes del rebuild:**
```
ssh=missing        # openssh-client no instalado
ip=missing         # iproute2 no instalado
free=missing       # procps no instalado
docker=missing     # docker-cli no instalado
```

**Por qué:**
- La imagen Docker de `studio` (Rust) no incluye estos binarios
- El sampler necesita `ssh` para conectarse a los servidores VPS
- También necesita `ip`, `free`, `df`, `docker stats` por SSH remoto (pero esos existen en los VPS)
- Solo `ssh` es necesario *dentro* del contenedor; el resto se ejecuta remoto

**Estado final:** el rebuild instaló `openssh-client`; `/usr/bin/ssh` existe dentro del contenedor.

### 3. Rutas SSH Windows en contenedor Linux

**Diagnóstico:**
```
COOLIFY_VPS1_SSH_KEY_PATH not_readable:ruta_windows_al_id_ed25519
COOLIFY_SSH_KEY_PATH empty
```

**Por qué:**
- Las claves SSH están en `~/.ssh/` en Windows (ruta local real)
- El contenedor Linux no puede leer rutas Windows
- `COOLIFY_SSH_KEY_PATH` ni siquiera está configurada
- El sampler detecta `ssh_key_path: None` y salta la muestra SSH

**Corrección permanente en manager:**
- `deploy-service` filtra cualquier env `*_SSH_KEY_PATH` leída desde Coolify para que una ruta Windows no sobrescriba el compose Linux.
- El compose efectivo monta `/root/studio-ssh` del host en `/home/appuser/.ssh`.
- El compose efectivo fija `COOLIFY_VPS1_SSH_KEY_PATH=/home/appuser/.ssh/id_ed25519`.
- Si existe `/root/.ssh/vps2_backup` en VPS1, `deploy-service` la copia a `/root/studio-ssh/vps2_backup` y fija `COOLIFY_SSH_KEY_PATH=/home/appuser/.ssh/vps2_backup`.
- El mount queda writable para que el entrypoint haga `chown/chmod` y `appuser` pueda leer la clave.

**Validación final:** `appuser` puede leer ambas claves y SSH responde `ssh_ok` para VPS1 y `vps2_ssh_ok` para VPS2.

## Impacto en cada tabla del panel

| Tab | Fuente de datos | Afectado |
|-----|----------------|----------|
| Despliegues | Coolify API directa (`list_services`) | **NO** — funciona ok |
| VPS | `configured_server_summaries` + `enrich_configured_metrics` (DB) | **SÍ** — servidores visibles, métricas null |
| Métricas CPU/RAM/disco | `infrastructure_resource_samples` via sampler SSH | **SÍ** — no hay samples |
| Gráfico por deployment | `deployment_metric_points` query | **SÍ** — sin samples |

## Fixes necesarios (por orden)

### A. Deployar fix duplicate key + build completo

**Archivos:**
- `src/repositories/infrastructure.rs` — `upsert_configured_server` two-step
- `src/commands/deploy_service.rs` + `src/commands/sync_env.rs` + `templates/rust-stack.yaml` — cambios de `coolify-manager-rs`

**Estado:** commit `d968702` está desplegado en producción. Se mantiene `--skip-compose-sync` mientras se resuelve el PATCH 422 de Coolify.

### B. Instalar openssh-client en Dockerfile

**Archivo:** `templates/Dockerfile.rust` en `coolify-manager-rs`

```dockerfile
RUN apt-get update && apt-get install -y --no-install-recommends \
    openssh-client \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
```

Sin esto, el sampler nunca podrá hacer SSH a ningún VPS.

### C. Materializar clave SSH por bind mount del host

La clave vive en VPS1 bajo `/root/studio-ssh/id_ed25519`. `deploy-service` sincroniza el mount y la env Linux en el compose efectivo, sin copiar secretos desde Windows ni guardarlos en env vars.

Para VPS2, la clave fuente vive en VPS1 bajo `/root/.ssh/vps2_backup`; el manager la copia al directorio montado de `studio` para que el contenedor la vea como `/home/appuser/.ssh/vps2_backup`.

### D. Configurar path SSH Linux

El path operativo es:
```
COOLIFY_VPS1_SSH_KEY_PATH=/home/appuser/.ssh/id_ed25519
COOLIFY_SSH_KEY_PATH=/home/appuser/.ssh/vps2_backup
```

No sincronizar rutas `C:/Users/...` hacia runtime; el manager las filtra.

## Archivos tocados en esta investigación

| Archivo | Línea | Qué reveló |
|---------|-------|------------|
| `src/handlers/hosting/deployments.rs` | 113-255 | `build_deployments` llama `list_services` por cada target Coolify |
| `src/handlers/hosting/infrastructure.rs` | 27-61 | Handler `list_infrastructure_servers` |
| `src/services/infrastructure_metrics.rs` | 73-128 | Sampler SSH: comando `fetch_server_snapshot` |
| `src/services/infrastructure_metrics.rs` | 460-499 | Loop de 10 min con `join_all` sobre targets |
| `src/repositories/infrastructure.rs` | 108-171 | `upsert_configured_server` — dupe key bug |
| `src/models/infrastructure.rs` | — | `InfrastructureServer`, `InfrastructureResourceSample` |

## Estado del fix duplicate key

**Código:** implementado, pusheado y desplegado en `upsert_configured_server` (two-step UPDATE/INSERT).
**Validación:** logs recientes sin `duplicate key`, sin `No such file or directory`, sin `Identity file ...` con ruta Windows y sin `Permission denied` para VPS2. Queda un warning transitorio posible si el primer sampler corre antes de que `deploy-service` reconecte el contenedor a la red `coolify`; la ruta Coolify interna ya responde 200 desde el runtime.
