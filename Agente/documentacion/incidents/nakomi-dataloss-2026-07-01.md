# Incidente: Pérdida total de datos PostgreSQL — nakomi.studio
**Fecha:** 2026-07-01  
**Severidad:** CRÍTICA  
**Servicio afectado:** nakomi.studio (Rust/PostgreSQL)  
**Stack UUID:** `do8k4w8swccwwogoc0os0ck0`  
**VPS:** 66.94.100.241

---

## Resumen ejecutivo

La base de datos PostgreSQL de nakomi.studio fue destruida y recreada desde cero durante un redeploy el 1 de julio de 2026 a las 02:22 UTC. **No existía ningún sistema de respaldo automático funcionando**, a pesar de que la configuración declaraba `backupPolicy.enabled: true`. Todos los datos dinámicos (usuarios, proyectos, chats, pedidos, facturación) se perdieron de forma irrecuperable.

---

## Causa raíz: Bug en Docker Compose template

El `docker_compose_raw` del servicio **no monta el volumen `pg_data` en el contenedor postgres**:

```yaml
# docker_compose_raw — SECCIÓN POSTGRES (ACTUAL, CON BUG):
postgres:
    image: 'postgres:16-alpine'
    container_name: postgres-do8k4w8swccwwogoc0os0ck0
    environment:
      POSTGRES_DB: rust_db
      POSTGRES_USER: rust_app
      POSTGRES_PASSWORD: MYPASS
    # ❌ NO tiene volumes: ['pg_data:/var/lib/postgresql/data']

volumes:
  app_data: null
  pg_data: null    # ← Declarado pero NUNCA montado en el contenedor
```

Coolify además **eliminó `pg_data` por completo** del compose procesado. El resultado: PostgreSQL almacenaba datos en un directorio efímero del contenedor, destruido en cada recreación.

### Fix correcto

```yaml
# docker_compose_raw — SECCIÓN POSTGRES (CORREGIDO):
postgres:
    image: 'postgres:16-alpine'
    container_name: postgres-do8k4w8swccwwogoc0os0ck0
    environment:
      POSTGRES_DB: rust_db
      POSTGRES_USER: rust_app
      POSTGRES_PASSWORD: MYPASS
    volumes:
      - 'pg_data:/var/lib/postgresql/data'    # ← FIX: montar volumen persistente

volumes:
  app_data: null
  pg_data: null
```

---

## Timeline

| Hora (UTC) | Evento |
|------------|--------|
| 2026-04-04 17:06 | Servicio creado en Coolify |
| 2026-04-04 → 2026-06-30 | **Período de operación normal** — datos acumulados (usuarios, proyectos, chats, pedidos). PostgreSQL funcionaba con volumen anónimo que sobrevivía a restarts simples. |
| 2026-06-09 | Incidente connection leak (`std::sync::Mutex` en async WebSocket). Docker restart restauró servicio. PostgreSQL NO fue afectado — datos intactos. |
| **2026-07-01 02:22:44** | **Redeploy destruye contenedor postgres.** Coolify recrea contenedores. PostgreSQL levanta con directorio efímero nuevo. |
| 2026-07-01 02:22:45 | SQLx ejecuta las 67 migraciones sobre base vacía (0→67 en 2 segundos) |
| 2026-07-01 02:22:47 | Seed de marketplace migration: 7 servicios básicos sin imágenes |
| 2026-07-01 03:49 | Primer redeploy de la sesión actual (ya con datos perdidos) |
| 2026-07-01 04:03 | Segundo redeploy completado |
| 2026-07-01 04:22 | Descubrimiento: users=0, projects=0, database_size=11MB |

---

## Datos perdidos

| Tabla | Registros esperados | Registros actuales |
|-------|--------------------|--------------------|
| `users` | Usuarios registrados | **0** |
| `projects` | 5+ proyectos con imágenes | **0** |
| `orders` | Pedidos/facturación | **0** |
| `chat_sessions` | Conversaciones | **0** |
| `chat_messages` | Mensajes | **0** |
| `blog_posts` | Artículos | **0** |
| `team_members` | Equipo | **0** |
| `services` | Servicios CMS | 7 (seed únicamente) |

---

## ¿Por qué no había backups?

### Lo que estaba PLANEADO (settings.json)

```json
"backupPolicy": {
  "enabled": true,
  "dailyKeep": 2,
  "weeklyKeep": 3,
  "manualKeep": 5,
  "sourcePaths": ["/app/uploads"]
}
```

### Lo que realmente EXISTÍA para PostgreSQL/Rust

| Componente | Estado | Detalle |
|-----------|--------|---------|
| **Sidecar backup en compose** | ❌ No existe | El compose Rust NO tiene contenedor `backup` — solo `app` y `postgres` |
| **Cron server-side** | ❌ No existe | No hay cron en VPS1, ni `/usr/local/bin/backup-studio.sh`, ni systemd timer |
| **Windows Task Scheduler** | ❌ No ejecutado | `schedule-backup` existe como comando pero crea tareas locales (solo corren con PC encendida) |
| **Coolify scheduled_tasks** | ❌ `null` | Nunca se configuró en la API de Coolify |
| **Coolify backup_configs** | ❌ `[]` | Vacío para la BD postgres |
| **Panel UI (TabBackups)** | ❌ `unsupported` | `HostingRuntimeService::list_backups()` devuelve error para runtime Coolify |
| **Backup manual CLI** | ✅ Funcional | `coolify-manager-rs backup --name studio` funciona pero nadie lo ejecutó |
| **Pre-deploy backup** | ⚠️ Parcial | Solo en `deploy-service`, no en `redeploy` ni `deploy --update` |

### La brecha

```
PLANEADO:   backupPolicy.enabled = true  ✅
REALIDAD:   Ningún mecanismo automático ejecutándose  ❌

El "plan" existía como configuración declarativa.
La implementación nunca se completó para el runtime Rust/PostgreSQL.
```

**La infraestructura de `coolify-manager-rs` tiene capacidad** de hacer backups PostgreSQL (`database_manager::export_postgres_database()` funciona), pero:
1. No se creó sidecar container en el compose
2. No se configuró cron server-side
3. No se ejecutó `schedule-backup`
4. El panel nunca se conectó para Rust/Coolify
5. `HostingRuntimeService::list_backups()` devuelve `unsupported` para Coolify

---

## Datos recuperables

| Fuente | ¿Recuperable? | Contenido |
|--------|--------------|-----------|
| `content/projects.toml` | ✅ Sí | 5 proyectos (KAMPLES, MABUHAY, Guillermo Chatbot, Task Manager, Material de Pádel) |
| `content/services.toml` | ✅ Sí | 4 servicios (diseño-web, desarrollo-apps, agentes-ia, branding) |
| Uploads bind mount (`/data/uploads/studio`) | ✅ Sí | ~70 archivos de imágenes CMS |
| Assets estáticos (build) | ✅ Sí | 5 imágenes de servicios (jpg+webp), covers de proyectos |
| Datos dinámicos (users, orders, chats) | ❌ No | Sin backup existente |

---

## Acciones correctivas inmediatas

### 1. Fix compose (previene recurrencia)
PATCH `docker_compose_raw` vía Coolify API para añadir `pg_data:/var/lib/postgresql/data` al servicio postgres.

### 2. Cargar fixtures (recupera contenido público)
`POST /api/admin/fixtures/sync` (admin auth) para poblar proyectos y servicios desde TOML.

### 3. Configurar backup automático (previene futura pérdida)
- Crear cron server-side: `pg_dump` diario con retención 7 daily + 4 weekly
- Script: `/usr/local/bin/backup-studio.sh`
- Almacenamiento: VPS2 o bind mount persistente

### 4. Implementar en coolify-manager-rs
- Sidecar backup container para compose Rust
- `HostingRuntimeService::list_backups()` para Coolify runtime
- Panel UI backup tab funcional para PostgreSQL

---

## Lecciones aprendidas

1. **`backupPolicy.enabled = true` en settings.json NO significa que haya backups corriendo.** La configuración declarativa sin implementación es peligrosa — da falsa sensación de seguridad.

2. **El template Docker Compose debe revisarse antes de producción.** El bug del volumen postgres existía desde la creación del servicio (abril 2026). Un review del compose on-disk vs raw habría detectado el problema.

3. **Los compose de Coolify necesitan validación post-deploy.** Verificar que los volúmenes declarados en `docker_compose_raw` aparecen correctamente en `docker_compose` procesado.

4. **Los backups deben verificarse con restore real.** Un backup que nunca se testea con restore es un backup que no existe.

5. **PostgreSQL sin volumen nombrado = datos efímeros.** Siempre verificar `docker inspect {container} --format '{{json .Mounts}}'` tras el primer deploy.

---

## Estado de acciones correctivas

- [ ] Fix compose: añadir `pg_data:/var/lib/postgresql/data`
- [ ] Cargar fixtures vía API
- [ ] Configurar cron backup server-side
- [ ] Implementar sidecar backup en coolify-manager-rs
- [ ] Implementar list_backups para Coolify runtime
- [ ] Documentar en lecciones-aprendidas.md
