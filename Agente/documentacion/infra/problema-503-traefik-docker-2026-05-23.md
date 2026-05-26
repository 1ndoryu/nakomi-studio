# Problema: 503 Service Unavailable en nakomi.studio tras deploy

> **Última actualización:** 2026-05-23

## Síntoma

El endpoint público `https://nakomi.studio/` devuelve `503 Service Unavailable` después de un deploy (swap de contenedor `app`). El contenedor responde internamente (`localhost:3000/healthz → 200`) y está en las redes correctas, pero Traefik no logra enrutar.

## Causa raíz

**Traefik pierde el registro de rutas Docker** cuando el contenedor `app` es recreado (swap) mientras Traefik está corriendo. Es un race condition en el provider Docker de Traefik v3.6:

1. El pipeline de deploy recrea el contenedor `app-STUDIO_STACK_UUID`
2. Docker emite eventos `stop` + `start` para el contenedor nuevo
3. Traefik no procesa correctamente estos eventos y no registra las rutas del contenedor nuevo
4. El catchall `default_redirect_503.yaml` (priority -1000) captura la petición → devuelve 503

### Arquitectura actual

- **Proxy:** `coolify-proxy` (Traefik v3.6) en VPS1 (VPS1_IP)
- **App:** Contenedor `app-STUDIO_STACK_UUID` en 2 redes:
  - `coolify` → IP 10.0.1.7
  - `STUDIO_STACK_UUID` → IP 10.0.7.3
- **Traefik también en ambas redes:**
  - `coolify` → IP 10.0.1.6
  - `STUDIO_STACK_UUID` → IP 10.0.7.4
- **Labels del contenedor:** Correctas (Host nakomi.studio, port 3000, TLS letsencrypt)
- **Catchall:** `/traefik/dynamic/default_redirect_503.yaml` con `servers: { }` (sin backend)

## Fix temporal (inmediato)

```bash
# Reconnect a su propia red para forzar rediscovery de Traefik
docker network disconnect STUDIO_STACK_UUID app-STUDIO_STACK_UUID
sleep 1
docker network connect STUDIO_STACK_UUID app-STUDIO_STACK_UUID
```

Esto emite eventos Docker que Traefik procesa y registra las rutas. **No requiere reiniciar el contenedor ni Traefik.** El sitio vuelve a responder en segundos.

## Fix permanente

### 1. Etiqueta `traefik.docker.network` en el template compose

Agregar la etiqueta `traefik.docker.network=coolify` al contenedor `app` en el template `rust-stack.yaml` de `coolify-manager-rs`. Esto fuerza a Traefik a usar la IP de la red `coolify` para el backend, eliminando ambigüedad multi-red.

**Archivo:** `.agent/coolify-manager-rs/templates/rust-stack.yaml`

```yaml
services:
  app:
    labels:
      - traefik.docker.network=coolify
```

Las labels de Docker no se pueden agregar a un contenedor en ejecución. Se aplican al crear el contenedor (deploy con build o skip-build).

### 2. Post-deploy healthcheck en el manager

Agregar en `coolify-manager-rs` una verificación post-swap que confirme que la ruta Traefik está activa:

```bash
docker exec coolify-proxy wget -qO- --no-check-certificate \
  --header="Host: nakomi.studio" \
  https://127.0.0.1:443/healthz
```

Si devuelve 503 en vez de 200, ejecutar automáticamente el network reconnect (fix temporal).

### 3. Desactivar autoheal en servicios Rust (temporal hasta fix)

El autoheal `cm-autoheal-*.timer` ejecuta `docker compose up -d --no-build --force-recreate --no-deps` cada 60s. Esto recrea el contenedor y puede causar el race condition con Traefik.

Mientras no esté implementado el fix permanente #1, mantener el autoheal desactivado:

```bash
systemctl stop cm-autoheal-studio.timer
systemctl disable cm-autoheal-studio.timer
```

### 4. Cambiar URL de Coolify VPS1 a puerto interno

Coolify escucha en puerto **8080** internamente (mapeado a 8000 en host). La variable de entorno debe ser:

```
COOLIFY_VPS1_BASE_URL=http://coolify:8080
```

**No** `:8000`. El hostname `coolify` resuelve a 10.0.1.3 (red `coolify`), y desde ahí el puerto 8080 es accesible.

### 5. Materializar claves SSH dentro del contenedor

Las claves SSH (`COOLIFY_VPS1_SSH_KEY_PATH=ruta_windows_al_id_ed25519`) son rutas Windows. El contenedor Linux no puede leerlas.

**Solución aplicada:** `coolify-manager-rs deploy-service` filtra `*_SSH_KEY_PATH` proveniente de Coolify y repara el compose efectivo para montar `/root/studio-ssh` del host en `/home/appuser/.ssh`, con `COOLIFY_VPS1_SSH_KEY_PATH=/home/appuser/.ssh/id_ed25519`. El mount queda writable para que el entrypoint pueda ajustar permisos antes de ejecutar la app como `appuser`.

### 6. Instalar `openssh-client` en la imagen Docker

La imagen actual del contenedor `app` no incluye `ssh`. Agregar al `Dockerfile.rust`:

```dockerfile
RUN apt-get update && apt-get install -y --no-install-recommends openssh-client \
    && rm -rf /var/lib/apt/lists/*
```

## Resumen de acciones

| # | Acción | Archivo/Lugar | Prioridad |
|---|--------|---------------|-----------|
| 1 | Agregar `traefik.docker.network=coolify` al template | `templates/rust-stack.yaml` | Alta |
| 2 | Verificación post-swap + auto-reconnect | `deploy_service.rs` | Alta |
| 3 | Desactivar autoheal hasta fix #1 | SSH directo | Hecho |
| 4 | Cambiar URL Coolify a `coolify:8080` | `.env` | Hecho (pendiente deploy efectivo) |
| 5 | Materializar claves SSH en contenedor | `deploy_service.rs` + `volume_manager.rs` + entrypoint | Hecho, pendiente validación tras build |
| 6 | Instalar `openssh-client` en Dockerfile | `Dockerfile.rust` | En build completo |
