# Sistema de Backups Automáticos — VPS1

> **Fecha:** 2026-07-01
> **Motivo:** Incidente nakomi-dataloss (PostgreSQL sin volumen persistente) reveló que no existían backups automáticos de bases de datos.

---

## 1. Resumen

Backups automáticos de **todas las bases de datos** (PostgreSQL + MariaDB) ejecutados directamente en el servidor VPS1 (66.94.100.241), con rotación organizada y límites de tamaño.

**Script:** `/usr/local/bin/backup-server.sh`
**Crontab root:** `0 3 * * * /usr/local/bin/backup-server.sh`
**Directorio:** `/data/backups/{sitio}/{daily|weekly}/`
**Log:** `/data/backups/backup.log`

---

## 2. Política de retención

| Condición | Daily | Weekly | Total máximo |
|-----------|-------|--------|--------------|
| Dump ≤ 500MB | 2 últimos | 2 últimos | 4 archivos/sitio |
| Dump > 500MB | 0 (saltado) | 1 último | 1 archivo/sitio |

- **Daily:** Se ejecuta de lunes a sábado (días 1-6).
- **Weekly:** Se ejecuta los domingos (día 7).
- **Heavy sites (>500MB):** Solo 1 backup semanal, sin daily. Esto evita llenar el disco con dumps grandes.

### Estructura de directorios

```
/data/backups/
├── backup.log                          ← log global
├── backup-sites.conf                   ← overrides por sitio (opcional)
├── {stack_uuid}/                       ← nombre = UUID del stack (auto-descubierto)
│   ├── daily/
│   │   ├── 2026-07-01_0300.sql.gz
│   │   └── 2026-06-30_0300.sql.gz
│   └── weekly/
│       ├── 2026-06-29_0300.sql.gz
│       └── 2026-06-22_0300.sql.gz
└── ...
```

---

## 3. Arquitectura auto-descubridora

**ZERO HARDCODING:** El script detecta automáticamente todos los containers PostgreSQL y MariaDB corriendo en el VPS. No importa si agregas, eliminas o renombras sitios.

### Descubrimiento
- **PostgreSQL:** `docker ps | grep '^postgres-'` → extrae UUID → `docker exec postgres-{uuid} printenv POSTGRES_USER/POSTGRES_DB`
- **MariaDB:** `docker ps | grep 'mariadb'` → `docker exec mariadb-{uuid} printenv MARIADB_USER/MARIADB_PASSWORD/MARIADB_DATABASE` (fallback `MYSQL_*`)

### Config overrides (opcional)
`/etc/backup-sites.conf` permite overrides por stack UUID:
```
# STACK_UUID|daily_keep|weekly_keep|max_daily_mb
mo4so4440c488g8woow4cow0|3|3|1000
```
Si no hay override, usa defaults: 2 daily, 2 weekly, 500MB threshold.

### Containers detectados (2026-07-01)
| Container | Tipo | UUID | Backup |
|-----------|------|------|--------|
| `postgres-b8s0cks444o0sogo8kg8wcgw` | PostgreSQL | glory-rest | ✅ |
| `postgres-do8k4w8swccwwogoc0os0ck0` | PostgreSQL | studio | ✅ |
| `postgres-mo4so4440c488g8woow4cow0` | PostgreSQL | kamples | ❌ (container caído) |
| `mariadb-owck8sww4ogk8gskgwcsk4w0` | MariaDB | guillermo | ✅ |
| `mariadb-qgskgw8wwc08o444o08wko8o` | MariaDB | cap | ✅ |
| `mariadb-zkcc040cc0scock4kcooowkc` | MariaDB | padel | ✅ |

---

## 4. Ejecución

### Automática (crontab)
```
0 3 * * * /usr/local/bin/backup-server.sh >> /data/backups/backup-cron.log 2>&1
```

### Manual (SSH directo)
```bash
ssh root@66.94.100.241

# Dry-run: ver qué containers detectaría
/usr/local/bin/backup-server.sh --dry-run

# Todos los containers encontrados
/usr/local/bin/backup-server.sh

# Match por nombre parcial
/usr/local/bin/backup-server.sh --site studio

# Forzar tier
/usr/local/bin/backup-server.sh --site studio --tier daily
/usr/local/bin/backup-server.sh --site studio --tier weekly
```

### Via coolify-manager-rs
```powershell
$cm = "C:\Users\Owner\OneDrive\Documentos\WP\app\public\wp-content\themes\glorytemplate\.agent\coolify-manager-rs\target\release\coolify-manager.exe"

# Instalar/actualizar el sistema de backups
& $cm install-backups

# Verificar containers detectados (dry-run)
# (se ejecuta automáticamente al instalar)
```

---

## 5. Restauración

### PostgreSQL (Rust)
```bash
# 1. Copiar el dump al container
docker cp /data/backups/{stack_uuid}/daily/2026-07-01_0300.sql.gz postgres-{stack_uuid}:/tmp/

# 2. Descomprimir y restaurar
docker exec postgres-{stack_uuid} bash -c \
  "gunzip -c /tmp/2026-07-01_0300.sql.gz | psql -U rust_app -d rust_db"

# 3. Verificar
docker exec postgres-{stack_uuid} psql -U rust_app -d rust_db -c "SELECT count(*) FROM _sqlx_migrations;"
```

### MariaDB (WordPress)
```bash
# 1. Copiar el dump al container
docker cp /data/backups/{stack_uuid}/daily/2026-07-01_0300.sql.gz mariadb-{stack_uuid}:/tmp/

# 2. Descomprimir y restaurar
docker exec mariadb-{stack_uuid} bash -c \
  "gunzip -c /tmp/2026-07-01_0300.sql.gz | mariadb -u wordpress wordpress"

# 3. Verificar
docker exec mariadb-{stack_uuid} mariadb -u wordpress wordpress -e "SELECT count(*) FROM wp_posts;"
```

---

## 6. Monitoreo

### Verificar que el crontab está instalado
```bash
crontab -l | grep backup-server
```

### Ver último log
```bash
tail -30 /data/backups/backup.log
```

### Verificar espacio en disco
```bash
du -sh /data/backups/
df -h /data/backups/
```

### Listar backups disponibles
```bash
find /data/backups -name '*.sql.gz' -type f -printf '%T@ %s %p\n' | sort -rn | head -20
```

---

## 7. Protecciones

| Protección | Descripción |
|-----------|-------------|
| **Verificación de contenedor** | Solo hace backup si el container está `running` |
| **Tamaño mínimo** | Dumps < 100 bytes se descartan como vacíos |
| **Espacio en disco** | Rechaza todo si hay < 1GB libre |
| **Límite 500MB** | Sitios pesados → solo weekly, sin daily |
| **Rotación automática** | Elimina los más antiguos al exceder el límite |
| **Logging** | Cada acción queda registrada con timestamp |

---

## 8. Incidente de origen

**nakomi-dataloss-2026-07-01:** Nakomi.studio perdió toda su base de datos PostgreSQL porque el `docker_compose_raw` no tenía el volumen `pg_data:/var/lib/postgresql/data`. Los datos eran efímeros. Se descubrió al revisar las imágenes rotas.

**Lección:** Los backups no son opcionales. Ni Docker volumes ni Coolify protegen contra errores de configuración del compose.

---

## 9. Estado

- [x] Script backup-server.sh reescrito como auto-descubridor (zero hardcoding)
- [x] Integración con coolify-manager-rs (`install-backups` lee settings.json, genera `/etc/backup-sites.conf`)
- [x] Instalado en VPS1 (`/usr/local/bin/backup-server.sh` + crontab `0 3 * * *`)
- [x] Primer backup ejecutado (2 PostgreSQL + 3 MariaDB detectados)
- [ ] kamples: container postgres caído (503) — no se pudo hacer backup
- [ ] wandori/nakomi: containers WordPress no encontrados — MariaDB sin backup
- [ ] Añadir comando `list-backups` a coolify-manager-rs para consultar backups disponibles
- [ ] Monitorear primeros días que la rotación funcione correctamente
