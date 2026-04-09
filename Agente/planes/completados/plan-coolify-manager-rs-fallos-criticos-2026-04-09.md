# Plan: Fallos Críticos de coolify-manager-rs — 2026-04-09

## Estado: ✅ Implementación completada (11 de 12 fixes)

## Contexto
Sesión de emergencia: tras deployar nakomi-rust (sitio Rust), TODOS los sitios WordPress cayeron.
- guillermo (guillechatbots.es): se eliminó completamente de Coolify — sin contenedores, sin volúmenes, sin directorio
- nakomi-task (task.nakomi.studio): tema "glory" desapareció del contenedor
- kamples (kamples.com): tema "glorytemplate" desapareció, BD apuntaba a twentytwentyfive

## Fallos Encontrados

### F1 — Deploy de un sitio destruye otros sitios
- **Severidad:** CRÍTICA
- **Qué pasó:** Al deployar nakomi-rust, los demás sitios WordPress perdieron sus temas
- **Causa probable:** Coolify recreó contenedores WordPress sin persistir temas clonados en runtime
- **Root cause:** Los temas se clonan vía `git clone` dentro del contenedor pero NO están en volúmenes persistentes ni en la imagen Docker. Un restart/recreate los elimina
- **Fix necesario:** Los temas deben sobrevivir recreaciones de contenedor. Opciones:
  1. Montar volumen persistente para wp-content/themes/{tema}
  2. Bake themes into image via Dockerfile customizado
  3. Startup script que clone/actualice tema al iniciar contenedor
  4. coolify-manager debe verificar salud de TODOS los sitios después de cualquier deploy

### F2 — Guillermo eliminado completamente sin explicación
- **Severidad:** CRÍTICA
- **Qué pasó:** El servicio `jcsc84cgc4ggcso44800w0sk` desapareció de Coolify — no existe en API ni en filesystem
- **Causa probable:** Desconocida. Posible eliminación accidental via API, bug de Coolify, o efecto colateral de otro deploy
- **Fix necesario:**
  1. Antes de cualquier operación destructiva, coolify-manager debe hacer backup automático (DB + wp-content)
  2. Implementar `pre-deploy-safety-check` que verifique que todos los sitios en settings.json siguen existiendo
  3. Log de auditoría de todas las operaciones que coolify-manager ejecuta via API
  4. Nunca eliminar stacks — solo recrear servicios dentro del stack

### F3 — `deploy --update` salta npm install cuando node_modules no existe
- **Severidad:** ALTA
- **Qué pasó:** `package-lock.json sin cambios, saltando npm install` pero el directorio `node_modules/` no existía
- **Causa:** El manager compara hashes de lock files para decidir si instalar, pero NO verifica si node_modules/ existe
- **Fix necesario:** Antes de saltar npm install, verificar que `node_modules/.package-lock.json` existe. Si no existe, instalar siempre

### F4 — `deploy --update` salta composer install cuando vendor/ no existe
- **Severidad:** ALTA  
- **Qué pasó:** `composer.lock sin cambios, saltando composer install` pero `vendor/` no existía
- **Causa:** Misma lógica rota que F3 — solo compara hashes, no verifica existencia de vendor/
- **Fix necesario:** Verificar que `vendor/autoload.php` existe antes de saltar composer install

### F5 — `new` genera labels Traefik con sslip.io en vez del dominio real
- **Severidad:** ALTA
- **Qué pasó:** Al crear guillermo con `--domain "https://guillechatbots.es"`, Coolify generó labels con `.sslip.io`
- **Causa:** Coolify v4 auto-genera FQDN al crear servicios y NUNCA lo actualiza via API
- **Fix necesario:** `new` debe inyectar labels Traefik explícitos en el docker-compose con el dominio correcto (HTTPS + HTTP redirect + certresolver=letsencrypt). Estos coexisten con los auto-generados de Coolify

### F6 — `import` falla con "Argument list too long" para DBs grandes
- **Severidad:** MEDIA
- **Qué pasó:** Intentar importar un SQL de 119KB falló
- **Causa:** El manager pasa el SQL como argumento de comando en vez de usar stdin/pipe
- **Fix necesario:** Usar `docker exec -i container mysql < file.sql` pattern o copiar archivo al contenedor primero con `docker cp`

### F7 — No hay health check post-deploy
- **Severidad:** CRÍTICA
- **Qué pasó:** Después de deploy, no se verificó si los sitios seguían funcionando
- **Causa:** deploy no ejecuta health check automáticamente
- **Fix necesario:** 
  1. Después de CUALQUIER deploy, ejecutar health check del sitio deployado
  2. Ejecutar health check de TODOS los demás sitios del mismo servidor
  3. Si alguno falla, alertar inmediatamente y ofrecer rollback

### F8 — No hay backup automático pre-deploy
- **Severidad:** CRÍTICA
- **Qué pasó:** Guillermo se eliminó y no había backup reciente accesible automáticamente
- **Causa:** deploy no hace backup antes de modificar nada
- **Fix necesario:** Antes de cualquier deploy/update, hacer backup de DB + wp-content del sitio afectado. Mantener al menos 1 backup pre-deploy

### F9 — Build React falla exit 127 sin mensaje claro
- **Severidad:** MEDIA
- **Qué pasó:** `Error compilando React:` con exit 127 pero sin decir qué comando falló
- **Causa:** El manager ejecuta el build pero `vite` no estaba en PATH (node_modules no existía por F3)
- **Fix necesario:** Capturar stderr/stdout del build y mostrar el error real. Verificar que el build command existe antes de ejecutar

### F10 — Rollback destructivo en deploy fallido
- **Severidad:** ALTA
- **Qué pasó:** Cuando el build React falló, el manager hizo rollback del tema y Glory, dejando el sitio peor que antes
- **Causa:** El rollback revierte git pero no reinstala dependencias ni rebuild
- **Fix necesario:** El rollback debe ser completo: revertir git + reinstalar deps + rebuild. O mejor: no hacer rollback automático si el sitio ya estaba funcionando antes del deploy

### F11 — No detecta temas desaparecidos en `health`
- **Severidad:** ALTA
- **Qué pasó:** health solo verifica HTTP status, no contenido. Un sitio puede devolver 200 con tema incorrecto
- **Fix necesario:** health debe verificar:
  1. HTTP 200
  2. Que el HTML contiene indicadores del tema correcto (title, CSS class, etc.)
  3. Que no hay errores PHP en los logs recientes

### F12 — Contenedor WordPress pierde todo en recreate
- **Severidad:** CRÍTICA (root cause de F1)
- **Qué pasó:** Los temas, vendor/, node_modules/, dist/ se pierden al recrear contenedor
- **Causa:** Estos archivos se instalan en runtime pero no están en volúmenes persistentes
- **Fix necesario:** La plantilla docker-compose de WordPress debe incluir:
  - Volumen persistente para `wp-content/themes/{tema}` O
  - Script de entrypoint que clone/instale tema + deps al arrancar
  - El tema debe tratarse como parte del "estado" del sitio, no como runtime efímero

## Prioridad de Implementación

1. **F7 + F8** — ✅ Health check + backup pre-deploy (deploy_theme + deploy_service)
2. **F12 + F1** — ✅ Persistencia: dockerfile_inline con deps + entrypoint glory-init self-healing
3. **F3 + F4** — ✅ Verificar existencia de vendor/ y node_modules/ antes de saltar install
4. **F5** — ✅ Labels Traefik explícitos en wordpress-stack.yaml y kamples-stack.yaml
5. **F2** — ✅ Pre-deploy safety check (verifica sitios en Coolify API antes de deploy)
6. **F10** — ✅ Rollback completo: git reset + composer + npm install + build
7. **F6** — ✅ Ya estaba corregido (usa docker cp + archivo, no args)
8. **F9** — ✅ Mejor diagnóstico: stdout+stderr + hint para exit 127
9. **F11** — ✅ Health check valida contenido HTML (detecta tema incorrecto/twentytwenty)
8. Lee las notas

## Notas — ✅ Todas las notas atendidas (commit 8d06d07)

- coolify-manager-rs binario está en: `.agent/coolify-manager-rs/`
- Settings.json con todos los sitios: `.agent/coolify-manager-rs/config/settings.json`
- ✅ F1-F12 implementadas, bloqueantes eliminados

- ✅ **DB backup naming**: El sistema de backup ahora usa SSH directo con verificación de integridad post-upload (comparación de tamaño). Las estructura de carpetas es explícita: `{baseDir}/{sitio}/{tier}/{backup_id}.tar.gz`.

- ✅ **Google Drive reemplazado**: Nuevo backend `sshremote` para backups en VPS2 (173.249.50.44). Google Drive se mantiene como opción legacy pero ya no es el default.

- ✅ **Robustez mejorada**: Pre-deploy safety check, backup automático pre-deploy, health check de todos los sitios post-deploy, rollback completo, persistencia de tema con entrypoint self-healing, alertas por email cuando un sitio cae.

- Si un sitio de cae tiene que avisar a mi correo andoryyu@gmail.com 

- tenemos un segundo vps para respaldos pero no se porque no se esta usando, necesito que en verdad todos los sitios tengan un respaldo en el segundo vps, dejemos de usar google drive, esto complicada demasiado las cosas usar google drive, no nos funciono, asi que usemos otro vps y ya
se puede acceder con
VPS2_IP=(env)
VPS2_SSH_PASSWORD=(env)

las copias de seguridad tienen que ser diaria, (3 maximas), y 1 semana (2 maxima)
- ✅ **Alertas email**: `health --all --alert` verifica todos los sitios y envía email a andoryyu@gmail.com via Brevo SMTP si alguno está caído. Resumen consolidado si múltiples caídos.

- ✅ **VPS2 para backups**: settings.json actualizado con `type: "sshremote"` apuntando a VPS2 (173.249.50.44). Política: daily (max 3) + weekly (max 2) per site. Upload/download streamed sin limite de ARG_MAX.

- ✅ **Glory sync integrado**: `glory_sync.php` se ejecuta 3 veces automáticamente al instalar tema (paso 8 de install_glory_theme). Busca en `scripts/` y `Glory/scripts/`.

- ✅ **README actualizado**: Documentados todos los cambios en README.md del coolify-manager-rs.

