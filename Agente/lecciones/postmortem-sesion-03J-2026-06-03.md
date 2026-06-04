# Postmortem — Sesión 03/Jun/2026

## Resumen

**Incidente**: Docker 27.0.3 SIGSEGV crash destruyó TODOS los contenedores WordPress y servicios del servidor.
**Duración**: ~8 horas (desde detección hasta recuperación completa de 7 de 8 sitios).
**Impacto**: Todos los sitios web excepto minecraft y kamples quedaron offline. Servicios afectados: padel, studio, guillermo, wandori, nakomi, cap, glory-rest, mail-nakomi.
**Causa raíz**: Bug conocido en Docker 27.0.3 con BuildKit que causa SIGSEGV en el daemon, destruyendo todos los contenedores.

---

## Errores Encontrados

### 1. Docker SIGSEGV Crash (causa raíz)
- **Qué**: Docker daemon 27.0.3 crashed con SIGSEGV, destruyendo todos los contenedores.
- **Cómo se detectó**: `docker ps -a` mostraba todos los contenedores como "Exited" o desaparecidos.
- **Lección**: Docker 27.0.3 tiene un bug conocido con BuildKit. Actualizar Docker es crítico.

### 2. Comandos SSH directos para deploy
- **Qué**: Se intentaron deploys directos por SSH en vez de usar coolify-manager-rs.
- **Impacto**: Sin historial, sin rollback, sin validación pre-deploy.
- **Lección**: NUNCA hacer deploy/management por SSH. Siempre usar coolify-manager-rs.

### 3. DB_PASSWORD vs SERVICE_PASSWORD_POSTGRES
- **Qué**: glory-rest usa `DB_PASSWORD` en su `.env`, pero `ensure_postgres_auth_and_hostname()` solo buscaba `SERVICE_PASSWORD_POSTGRES`.
- **Impacto**: El deploy de glory-rest fallaba en sincronizar credenciales de DB.
- **Fix**: Añadir fallback a `DB_PASSWORD` en la función, priorizando `SERVICE_PASSWORD_POSTGRES`.

### 4. Backticks en reglas Traefik
- **Qué**: `rewrite_compose_host_rules()` generaba `Host(domain)` sin backticks. Traefik espera `Host(\`domain\`)`.
- **Impacto**: Los dominios configurados no funcionaban correctamente después del redeploy.
- **Fix**: Añadir backticks en `deploy_service.rs` y `template_engine.rs`.

### 5. Validación de imagen detecta busybox:latest
- **Qué**: `ensure_compose_service_image_available()` detectaba `busybox:latest` (de servicio placeholder `rust-app`) en vez de `b8s0cks444o0sogo8kg8wcgw-app`.
- **Impacto**: El deploy abortaba después de un build exitoso de 578s.
- **Causa**: Fallback `docker compose config --images | grep -v postgres | head -1` captura la primera imagen no-postgres, que es `busybox:latest` del servicio placeholder.
- **Fix**: Fallback inteligente que busca imagen matching `-{service_name}$` antes de caer al genérico (excluyendo busybox).

### 6. Health check hard fail
- **Qué**: Algunos sitios fallaban health check después del deploy.
- **Lección**: El health check debe reintentar con backoff, no fallar inmediatamente.

---

## Mitigaciones Programáticas (coolify-manager-rs)

### M1: Pre-flight compose validation
- Validar que el compose generado es sintácticamente correcto antes de enviar a Coolify.
- Verificar que las reglas Traefik tienen backticks: `Host(\`domain\`)`.

### M2: Post-deploy health check con retry
- Después de cada deploy, hacer health check con 3 reintentos (5s, 10s, 20s).
- Si falla después de 3 reintentos, alertar pero no revertir automáticamente (el compose ya está en Coolify).

### M3: Watchdog automático
- Cada 5 minutos, health check de todos los sitios.
- Si un sitio falla 3 veces consecutivas, intentar redeploy automático.
- Notificar al usuario por el canal configurado.

### M4: Pre-write compose backup
- Antes de sobrescribir el compose en Coolify, guardar backup local del compose actual.
- Mantener los últimos 5 backups por sitio.

### M5: Centralized DB credentials
- Usar `SERVICE_PASSWORD_POSTGRES` como fuente de verdad, con fallback a `DB_PASSWORD`.
- Al sincronizar, intentar ambas variables y usar la que exista.

### M6: Validate compose labels in template engine
- Verificar que `Host()` labels tienen backticks después de generar el compose.
- Añadir test unitario que verifique el formato de las reglas Traefik.

---

## Plan de Implementación Priorizado

### P0 (Inmediato — esta sesión)
- [x] Fix backticks en `rewrite_compose_host_rules()` y `template_engine.rs`
- [x] Fix DB_PASSWORD fallback en `ensure_postgres_auth_and_hostname()`
- [x] Fix busybox:latest image detection en `ensure_compose_service_image_available()`
- [x] Fix bind mount insertado en servicio equivocado (Python rewrite)
- [x] Health check verification de todos los sitios después del deploy (7/7 OK)

### P1 (Próxima sesión)
- [ ] Watchdog automático (M3)
- [ ] Centralized DB credentials (M5)
- [ ] Post-deploy health check con retry (M2)

### P2 (Futuro)
- [ ] Pre-flight compose validation (M1)
- [ ] Pre-write compose backup (M4)
- [ ] Docker upgrade en servidor (27.0.3 → latest stable)
- [ ] Validate compose labels in template engine (M6)

---

## Errores Adicionales Descubiertos Durante Sesión

### E16: busybox:latest detectado como imagen del servicio
**Fecha**: 2026-06-03 17:23
**Archivo**: `deploy_service.rs` línea 1241-1270
**Error**: `Validacion: La imagen detectada 'busybox:latest' no existe localmente; abortando antes de recrear app. busybox:latest`
**Causa**: El compose de glory-rest tiene un servicio placeholder `rust-app` con `image: busybox:latest`. El fallback de detección de imagen usa `docker compose config --images | grep -v postgres | head -1` que captura `busybox:latest` en vez de `b8s0cks444o0sogo8kg8wcgw-app`.
**Fix**: Tres niveles de fallback:
1. `sed/awk` parse del `image:` field del servicio específico
2. `grep -E "\-${svc}$"` para buscar imagen que termine con el nombre del servicio
3. `grep -v postgres | grep -v busybox | head -1` como último recurso

### E17: Bind mount insertado en servicio postgres en vez de app
**Fecha**: 2026-06-03 17:46
**Archivo**: `volume_manager.rs` → `ensure_uploads_bind_mount()`
**Error**: El bind mount `/data/uploads/glory-rest:/app/uploads` se insertaba en el servicio `postgres` en vez de `app`. El contenedor de app quedaba sin mount y los uploads no persistían.
**Causa 1**: El awk anterior insertaba después del PRIMER `volumes:` encontrado en el compose, que era el de postgres.
**Causa 2**: Coolify tiene un worker async que reescribe el `docker-compose.yml` en disco desde su base de datos. Después de que nuestro script Python corregía el compose, Coolify lo sobrescribía con la versión de su API (que tenía el bind mount en postgres).
**Fix**: Reescritura completa con Python (base64-encoded, inyectado vía SSH) que:
1. Elimina TODAS las líneas `/app/uploads` existentes
2. Localiza el bloque del servicio destino por indentación YAML
3. Inserta `volumes:` + bind mount en el servicio correcto
4. El swap (`docker compose up`) lee el compose corregido inmediatamente después

**Nota**: El deploy de glory-rest falló 3 veces antes de funcionar:
1. Falla por DB_PASSWORD (E3)
2. Falla por busybox:latest (E16)
3. Falla por bind mount en servicio equivocado (E17)
Cada falla requería fix → recompile → redeploy (~15 min ciclo).

**Verificación final**: `docker inspect app-b8s0cks444o0sogo8kg8wcgw` confirmó `/app/uploads` como bind mount a `/data/uploads/glory-rest`.

---

## Acceso a Infraestructura

- **Servidor**: root@66.94.100.241
- **Docker**: 27.0.3 (BuildKit), Ubuntu 24.04.3 LTS, kernel 6.8.0-101-generic
- **Coolify API**: http://66.94.100.241:8000
- **Binary coolify-manager**: `C:\Users\Owner\OneDrive\Documentos\WP\app\public\wp-content\themes\glorytemplate\.agent\coolify-manager-rs\target\release\coolify-manager.exe`
- **Service UUIDs**:
  - padel: `zkcc040cc0scock4kcooowkc`
  - studio: `do8k4w8swccwwogoc0os0ck0`
  - guillermo: `owck8sww4ogk8gskgwcsk4w0`
  - wandori: `csoc88c0gw8kc4cwcwosc48s`
  - nakomi: `u00gc8ss4csc4cckkg4g00ks`
  - cap: `qgskgw8wwc08o444o08wko8o`
  - glory-rest: `b8s0cks444o0sogo8kg8wcgw`
  - mail-nakomi: `vk4c4oocow0sc844ocssgw4s`

---

## Timeline

| Hora (CEST) | Evento |
|---|---|
| ~09:00 | Docker SIGSEGV crash detectado. Todos los contenedores destruidos. |
| ~09:30 | Diagnóstico: Docker 27.0.3 bug con BuildKit. |
| ~10:00 | Inicio de recuperación con coolify-manager-rs. |
| ~11:00 | padel, studio, guillermo, wandori recuperados. |
| ~12:00 | nakomi, cap recuperados. |
| ~14:00 | glory-rest deploy falla por DB_PASSWORD mismatch. |
| ~15:00 | Fix DB_PASSWORD fallback coded + compilado. |
| ~16:00 | glory-rest redeploy iniciado (build toma ~10 min). |
| ~16:20 | Build completado (578s) pero falla por busybox:latest detection. |
| ~17:00 | Fix busybox image detection coded + compilado. |
| ~17:30 | glory-rest redeploy re-iniciado con todos los fixes. |
| ~17:40 | Build en progreso en servidor... |

