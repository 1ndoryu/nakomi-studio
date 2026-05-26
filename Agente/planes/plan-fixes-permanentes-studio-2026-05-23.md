# Plan: Fixes permanentes para nakomi.studio

> **Problemas raíz:**
> 1. Build Rust >40 min (timeout) — no se puede deployar fixes de código
> 2. Claves SSH no materializadas en el contenedor — sin métricas CPU/RAM/disco
> 3. El compose de Coolify usa `dockerfile: Dockerfile.rust` externo (no inline)
> 4. Post-deploy requiere reconectar red Traefik manualmente
>
> **Creado:** 2026-05-23
> **Actualizado:** 2026-05-23 (imagenes recuperadas, deploy completo validado, SSH VPS1/VPS2 runtime corregido)

---

## Fase 1: Diagnóstico del build lento — COMPLETADO

Objetivo: Entender por qué el build tarda >40 min.

### Resultados del diagnóstico (2026-05-23)

| Variable | Valor |
|----------|-------|
| RAM contenedor | **Sin límite** (0 = ilimitado) |
| CPU contenedor | **Sin límite** (0 = ilimitado) |
| RAM VPS1 total | 7.8 GiB |
| RAM VPS1 disponible | ~3.9 GiB |
| CPU VPS1 | 4 cores |
| SWAP | **0B — no hay swap** |
| Build cache Docker | 148 entries, 29.03 GB, **0 activos** (todos colgados) |
| Docker builder | `default` driver (efímero, no `docker-container`) |
| Coolify concurrent_builds | 2 |
| Coolify dynamic_timeout | 3600s (1 hora) |
| Coolify force_docker_cleanup | true (cleanup al 80% disco) |
| Dockerfile.rust | No encontrado localmente en el repo local `glory-rs` |

### Causa raíz

1. **Sin swap** — `cargo build --release` en un workspace como glory-rs puede disparar la RAM a 4-6 GiB durante el linking. Sin swap, el OOM killer mata el proceso o el sistema se congela.
2. **Build cache no reutilizado** — el driver `default` de Docker no persiste cache entre builds de Coolify. Las 148 entries (29 GB) están todas `ACTIVE: 0`. Cada build empieza desde cero.
3. **4 CPUs** — limitado para un workspace multi-crate; la paralelización es baja.

### Swap añadido (2026-05-23) ✅
- 4 GiB swapfile creado en VPS1 (`/swapfile`)
- `swapon --show` confirma activo
- Persistido en `/etc/fstab`
- Próximo build debería completar (ya no OOM en linking)

### Soluciones pendientes para build
- Migrar a builder `docker-container` con cache persistente (volume)
- Pre-build Docker fuera de Coolify: GitHub Actions build + push a registry

---

## Fase 2: Arreglar SSH keys en contenedor — COMPLETADO

Objetivo: Que el contenedor pueda conectar por SSH a VPS1 para métricas.

### Progreso (2026-05-23) ✅

1. **Key generada en VPS1:** `/root/studio-ssh/id_ed25519`
2. **Key pública agregada** a `authorized_keys` de VPS1 (loopback SSH)
3. **Verificado:** container puede `ssh root@VPS1_IP` exitosamente
4. **Bind mount en template `config/templates/rust-stack.yaml`:**
   ```yaml
   volumes:
       - /root/studio-ssh:/home/appuser/.ssh
   ```
5. **Env var en compose:**
   ```yaml
   COOLIFY_VPS1_SSH_KEY_PATH: /home/appuser/.ssh/id_ed25519
   ```
6. **Dockerfile.rust (`config/templates/Dockerfile.rust`):**
   - Añadido `openssh-client` al stage runtime
   - Entrypoint.sh actualizado para fix permisos SSH
7. **`sync_env.rs` modificado:**
   - `COOLIFY_VPS1_SSH_KEY_PATH` y `COOLIFY_SSH_KEY_PATH` removidos del allowlist de push
   - Evita que la ruta Windows del `.env` sobrescriba el path Linux del template
8. **Guardrail runtime en `deploy-service`:**
   - `COOLIFY_*_SSH_KEY_PATH` se filtra de envs runtime de Coolify.
   - El compose efectivo se repara en cada deploy para montar `/root/studio-ssh` y fijar `COOLIFY_VPS1_SSH_KEY_PATH=/home/appuser/.ssh/id_ed25519`.
   - El mount quedó sin `:ro` porque el entrypoint necesita ajustar owner/permisos para que `appuser` pueda leer la clave.

### Estado actual

- Build completo terminado y desplegado: `deploy-service --name studio --skip-compose-sync` completó en 1001s y dejó `healthz` 200.
- Imagen runtime validada: `/usr/bin/ssh` existe dentro del contenedor.
- VPS1 validado como `appuser`: clave `/home/appuser/.ssh/id_ed25519` legible y `ssh root@VPS1_IP echo ssh_ok` responde `ssh_ok`.
- VPS2 validado como `appuser`: `deploy-service` copia `/root/.ssh/vps2_backup` a `/root/studio-ssh/vps2_backup`, fija `COOLIFY_SSH_KEY_PATH=/home/appuser/.ssh/vps2_backup` y `ssh root@VPS2_IP echo vps2_ssh_ok` responde `vps2_ssh_ok`.
- Coolify VPS1 validado desde runtime: `GET $COOLIFY_VPS1_BASE_URL/api/v1/services` con token runtime responde 200 desde `appuser`.
- Los logs posteriores ya no muestran `ssh missing`, `Identity file C:/Users/...` ni `Permission denied` para VPS2.
- Queda un warning transitorio en el primer sampler de arranque si corre antes de que `deploy-service` reconecte el contenedor a la red `coolify`; la conectividad queda validada inmediatamente despues.

### Workaround eliminado
- Ya no se depende de copiar claves manualmente al contenedor; el compose efectivo lo repara `deploy-service`.

---

## Fase 2b: Bug 422 en sync compose de Coolify — PARCIALMENTE MITIGADO

### Síntoma
`PATCH /api/v1/services/do8k4...` devuelve 422:
```json
{"message":"Validation failed.","errors":{"docker_compose_raw":"The docker_compose_raw should be base64 encoded."}}
```

### Causas identificadas

1. **Backticks en labels Traefik + `build:`** = 500 Internal Server Error (bug en Coolify PHP)
   - Líneas del template: `Host(\`{{DOMAIN_CLEAN}}\`)`
   - Fix: removido backticks, dejado `Host({{DOMAIN_CLEAN}})`
   - Template actualizado: `config/templates/rust-stack.yaml` y `templates/rust-stack.yaml`

2. **`$VITE_STRIPE_PUBLISHABLE_KEY` en build.args** = 422
   - `$VAR` en `environment:` funciona; en `build.args:` rompe el parser de Coolify
   - Fix: removida la línea, documentada con comentario `[235A]`
   - Template actualizado

### Estado: INCONCLUSO, con mitigaciones adicionales

Incluso con ambos fixes, el compose renderizado por Rust (y por PowerShell con las mismas sustituciones) sigue dando 422. Un compose equivalente construido manualmente (sin backticks, sin $VITE) PASÓ exitosamente. La diferencia exacta no se identificó — investigación pausada.

Mitigaciones nuevas aplicadas al manager:
- Los backticks restantes en `EXTRA_DOMAIN_LABELS` de `template_engine.rs` fueron eliminados y cubiertos por tests.
- El deploy de producción se ejecuta con `--skip-compose-sync` hasta que el PATCH de Coolify quede confirmado.

### Hipótesis pendientes
- El compose renderizado desde template tiene alguna diferencia sutil (indentación, líneas vacías, orden de keys) que Coolify rechaza
- Posible corrupción del estado interno de Coolify tras el overwrite con busybox
- Probar: push del compose explícito funcional primero, luego el template renderizado

---

## Fase 3: Deployar fix `upsert_configured_server` — COMPLETADO

Objetivo: Aplicar el fix de duplicate key (repositories/infrastructure.rs).

**Estado:**
- Commit `d968702` ya está desplegado en producción.
- Backup correcto pre-deploy creado: `20260523_163630-pre_deploy_service`, con `files-app_uploads.tar.gz` de 183.2 MB.
- Build completo finalizó en 1001s.
- Swap posterior `--skip-build --skip-compose-sync --skip-backup` aplicó el mount SSH writable y la env VPS2 sin rebuild.
- Logs recientes no muestran `duplicate key`.

---

## Fase 4: Post-deploy auto-reconnect — APLICADO EN COMPOSE EFECTIVO

Objetivo: Eliminar la intervención manual de reconectar red Traefik tras swap.

### Progreso (2026-05-23) ✅

**Label `traefik.docker.network=coolify` agregado a ambas copias del template:**
- `config/templates/rust-stack.yaml` ✅
- `templates/rust-stack.yaml` ✅

### Estado
- `deploy-service` ya verifica/reconecta red `coolify` tras el swap.
- La label queda en templates; mientras `sync compose` siga bloqueado por 422, el deploy operativo usa reparación del compose en disco + verificación de red.
- El template fuente ahora declara `/data/uploads/{{SITE_NAME}}:/app/uploads`; el manager igualmente mantiene la reparación runtime porque Coolify puede normalizar binds a named volumes.
- `execute_long_running` ahora emite heartbeat cada 120s para que futuros builds largos no queden mudos.

---

## Fase 5: Validación — VALIDADO

Repetir health check post-deploy sin intervención manual:
- `https://nakomi.studio/healthz` → 200 confirmado tras el deploy completo y tras el swap runtime.
- `https://nakomi.studio/` → 200 confirmado.
- `https://nakomi.studio/panel` → 200 confirmado.
- Imágenes reales de `/uploads/content` → 200 `image/jpeg` y 200 `image/png`.
- Imágenes públicas recuperadas: `/app/uploads` usa bind `/data/uploads/studio`.
- Login: endpoint vivo; el usuario confirmó que ya funciona.
- Runtime SSH VPS1: `/usr/bin/ssh`, clave legible por `appuser`, `ssh_ok`.
- Runtime SSH VPS2: `COOLIFY_SSH_KEY_PATH=/home/appuser/.ssh/vps2_backup`, clave legible por `appuser`, `vps2_ssh_ok`.
- Coolify interno: `http://coolify:8080/api/v1/services` responde 200 con token runtime.

---

## Resumen de archivos modificados

| Archivo | Cambio |
|---------|--------|
| `config/templates/rust-stack.yaml` | backticks removidos, $VITE removido, label coolify, bind mount SSH, env SSH |
| `templates/rust-stack.yaml` | backticks removidos, label coolify |
| `config/templates/Dockerfile.rust` | openssh-client, entrypoint SSH setup |
| `src/commands/sync_env.rs` | SSH_KEY_PATH excluido de push |
| `src/services/volume_manager.rs` | fusión de uploads desde named volume, reparación de mount SSH, clave VPS2 runtime, reemplazo por target para evitar duplicados |
| `src/infra/template_engine.rs` | labels de dominios extra sin backticks |
| `src/infra/ssh_client.rs` | heartbeat en comandos largos |
| VPS1: `/etc/fstab` | swap añadido |

---

## Próximos pasos (prioridad)

1. **Resolver 422:** Encontrar la diferencia exacta entre compose explícito (funciona) y renderizado (falla).
2. **Sync compose** con todos los fixes aplicados cuando Coolify acepte el PATCH.
3. **Reducir build time:** builder persistente o imagen prebuild en registry; el deploy funcional ya completó, pero 1001s sigue siendo alto.
4. **Opcional:** retrasar el primer ciclo del sampler o reconectar `coolify` antes del arranque para eliminar el warning transitorio post-swap.

---

## Criterios de cierre

1. Build completo funcional: cumplido (1001s; optimización pendiente para <30 min)
2. Deploy via manager sin intervención manual SSH: cumplido
3. Métricas CPU/RAM/disco: runtime SSH y Coolify validados para VPS1/VPS2; esperar o disparar siguiente ciclo del sampler para poblar puntos nuevos
4. Contenedor healthy después de deploy: cumplido (`healthz` 200, red `coolify` reconectada por manager)
