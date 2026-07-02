# Estrategia de Backups — 1 julio 2026

> **Autor:** Agente / MiMo-v2.5-pro
> **Contexto:** Post-incidente nakomi-dataloss. El sistema anterior nunca ejecutó un backup real.
> **Principio:** Los backups corren **en el servidor** (VPS1), no desde Windows.

---

## 1. Situación actual

### Qué había
- `backupPolicy.enabled: true` en todos los sitios.
- `schedule-backup` crea tareas en **Windows Task Scheduler** (requiere PC encendido).
- `backup` conecta por SSH desde Windows → VPS → docker exec → pg_dump → descarga a Windows → sube a Google Drive o VPS2.
- **Nunca se ejecutó.** El PC no estaba encendido a las 3am, o las tareas no existían.

### Qué falló
- nakomi.studio perdió todos los datos el 1 julio 2026 al 02:22 UTC.
- No había ningún backup de PostgreSQL. Cero. Datos irrecuperables.

### Qué se necesita
- Backups que corran **en el servidor** automáticamente.
- Sin dependencia del PC Windows.
- Retención simple: **máximo 4 backups por sitio** (2 diarios + 2 semanales).
- Sitios con dumps >500MB: **limitar a 1 semanal** (ahorrar espacio en VPS).
- Organización clara para `coolify-manager backup --list`.

---

## 2. Arquitectura

```
VPS1 (66.94.100.241)
└── /data/backups/
    ├── studio/
    │   ├── daily/
    │   │   ├── 2026-07-01_0300.sql.gz      (52MB)
    │   │   └── 2026-07-02_0300.sql.gz      (53MB)
    │   └── weekly/
    │       ├── 2026-06-29_0400.sql.gz      (51MB)
    │       └── 2026-07-06_0400.sql.gz      (54MB)
    ├── nakomi/
    │   ├── daily/
    │   │   ├── 2026-07-01_0315.sql.gz      (2MB)
    │   │   └── 2026-07-02_0315.sql.gz      (2MB)
    │   └── weekly/
    │       └── 2026-07-06_0415.sql.gz      (2MB)
    ├── kamples/
    │   └── weekly/                          ← >500MB, solo semanal
    │       └── 2026-07-06_0430.sql.gz      (820MB)
    └── ...
```

### Componentes
| Componente | Ubicación | Responsabilidad |
|---|---|---|
| `backup-server.sh` | `/usr/local/bin/backup-server.sh` en VPS1 | Script principal: dump, comprimir, rotar |
| crontab root | `/var/spool/cron/crontabs/root` en VPS1 | Ejecución automática diaria (3:00 UTC) |
| `coolify-manager backup --list` | CLI local (Windows) | Descubre backups leyendo el servidor por SSH |

---

## 3. Política de retención

### Regla general
| Tier | Frecuencia | Retención | Ejemplo |
|---|---|---|---|
| Daily | Cada día | **2 últimos** | Solo lun-mar si hoy es miércoles |
| Weekly | Cada domingo | **2 últimos** | Solo los 2 domingos más recientes |
| **Total máximo** | — | **4 por sitio** | — |

### Excepción: dumps >500MB
| Tier | Frecuencia | Retención | Razón |
|---|---|---|---|
| Weekly | Cada domingo | **1 último** | VPS tiene ~20GB libres; 800MB × 4 = 3.2GB es demasiado |
| Daily | — | **0** | Deshabilitado para estos sitios |

### Clasificación de sitios por tamaño estimado

| Sitio | DB Engine | Tamaño estimado | Política |
|---|---|---|---|
| studio (nakomi.rust) | PostgreSQL | ~50-80MB | 2 daily + 2 weekly |
| nakomi (WordPress) | MariaDB | ~2-5MB | 2 daily + 2 weekly |
| kamples | PostgreSQL | **>500MB** (scraper data) | **1 weekly** |
| glory-rest | PostgreSQL | ~10-20MB | 2 daily + 2 weekly |
| guillermo | MariaDB | ~5-15MB | 2 daily + 2 weekly |
| padel | MariaDB | ~5-10MB | 2 daily + 2 weekly |
| cap | MariaDB | ~5-10MB | 2 daily + 2 weekly |
| wandori | MariaDB | ~10-20MB | 2 daily + 2 weekly |

> **Nota:** El tamaño real se determina dinámicamente al ejecutar el dump. Si un dump supera 500MB, el script lo mueve a `weekly/` y borra cualquier `daily/` existente para ese sitio.

---

## 4. Flujo del script `backup-server.sh`

```
INICIO
  │
  ├─ Para cada sitio con backupPolicy.enabled:
  │   │
  │   ├─ 1. Detectar DB engine (postgres vs mariadb)
  │   │      postgres → container postgres-{uuid}
  │   │      mariadb  → container mariadb-{uuid}
  │   │
  │   ├─ 2. docker exec → pg_dump / mariadb-dump
  │   │      → pipe a gzip → archivo temporal
  │   │
  │   ├─ 3. Medir tamaño del dump
  │   │      >500MB → tier=weekly, skip_daily=true
  │   │      ≤500MB → tier según día (daily o weekly)
  │   │
  │   ├─ 4. Mover a /data/backups/{site}/{tier}/
  │   │
  │   ├─ 5. Rotar:
  │   │      daily/  → mantener 2 más recientes
  │   │      weekly/ → mantener 2 más recientes (1 si >500MB)
  │   │
  │   └─ 6. Log: timestamp, sitio, tier, tamaño, archivo
  │
  FIN
```

### Criterio de tier
- **Domingo** → `weekly/` (siempre)
- **Lunes-Sábado** → `daily/` (si el sitio no tiene la excepción de >500MB)
- **Sitio >500MB** → `weekly/` siempre, ignora el día

---

## 5. Integración con coolify-manager

### `backup --list` (lectura remota)
El comando `backup --list --name {site}` se modifica para:
1. Intentar listar backups en el servidor (SSH → `ls -la /data/backups/{site}/`)
2. Si existen, mostrar: archivo, tier, tamaño, fecha
3. Mantener compatibilidad con backups locales si existen

### `backup --name {site} --tier {tier}` (trigger manual)
El comando existente se modifica para:
1. Ejecutar `ssh root@VPS1 "bash /usr/local/bin/backup-server.sh --site {site} --tier {tier}"`
2. Retornar el resultado al usuario

### `backup --install` (nuevo)
Instala el script y el crontab en VPS1:
1. Sube `backup-server.sh` a `/usr/local/bin/`
2. Ejecuta `chmod +x`
3. Agrega entrada al crontab root

---

## 6. Formato de archivos

### Nombre de archivo
```
{YYYY-MM-DD}_{HHMM}.sql.gz
```
Ejemplo: `2026-07-01_0300.sql.gz`

### Log
```
/data/backups/backup.log
```
Formato: `{timestamp} | {site} | {tier} | {size_mb}MB | {filename} | {status}`

---

## 7. Monitoreo y alertas

### Checks automáticos (en el propio script)
- Si `docker exec` falla → log `FAILED`, no borrar backup anterior
- Si dump = 0 bytes → log `EMPTY`, no guardar
- Si disco < 1GB libre → log `DISK_LOW`, saltar sitio

### Descubrimiento desde coolify-manager
- `backup --list` lee el servidor directamente
- Muestra: sitio, tier, fecha, tamaño
- Permite al usuario verificar sin SSH manual

---

## 8. Cambios pendientes en settings.json

```json
{
  "backupPolicy": {
    "enabled": true,
    "dailyKeep": 2,
    "weeklyKeep": 2,
    "maxSizeMbDaily": 500
  }
}
```

> **Nota:** `weeklyKeep` cambia de 3 → 2. Se agrega `maxSizeMbDaily: 500`.

---

## 9. Timeline de implementación

1. ✅ Documentación (este archivo)
2. 🔄 Crear `backup-server.sh` (script bash para VPS1)
3. ⏳ Subir e instalar script + crontab en VPS1
4. ⏳ Modificar `coolify-manager backup --list` para descubrir backups remotos
5. ⏳ Ejecutar backup manual de studio para verificar
6. ⏳ Verificar que crontab ejecuta al día siguiente
