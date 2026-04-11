# Plan: SSH/SFTP Seguro por Despliegue de Hosting

> Creado: 2026-04-16
> Estado: **En progreso** — Fase 2 completada (openssh-server + resource limits + wp-cli + hardening + plan configs admin). Pendiente: Fase 1 (verificación VFS), Fase 3 (panel recursos), Fase 4 (migración existentes).
> Contexto: Cada hosting usa `linuxserver/openssh-server` con SSH+SFTP. Recursos configurables por plan desde BD.

---

## Objetivo

Dar a cada cliente de hosting acceso SSH shell + SFTP enjaulado en su volumen WordPress, con límites de CPU/RAM/disco por contenedor, sin que pueda ver ni afectar otros hostings.

---

## Estado actual (post 114A-3)

- **SSH + SFTP funciona**: `linuxserver/openssh-server` con `dockerfile_inline` (PHP + wp-cli + sshd hardening)
- **wp-cli disponible**: PHP 8.3 + wp-cli instalados en el contenedor SSH vía dockerfile_inline
- **SSH hardening**: AllowTcpForwarding=no, X11Forwarding=no, PermitTunnel=no, GatewayPorts=no (custom-cont-init.d)
- **Límites de recursos**: Dinámicos desde `hosting_plan_configs` (admin-configurable vía API)
- **backend_net**: SSH container en backend_net para wp-cli → MariaDB
- **Panel recursos**: Pendiente (Fase 3)
- **Disk quota**: Pendiente (requiere verificación VFS en VPS2)

---

## Arquitectura propuesta

### 1. Reemplazar `atmoz/sftp` por `linuxserver/openssh-server`

**Por qué:** openssh-server soporta SSH shell + SFTP sobre el mismo puerto/conexión. El cliente puede hacer `ssh` (shell completo) y `sftp`/`scp` (transferencia de archivos) con las mismas credenciales.

**Compose por hosting:**
```yaml
ssh:
  image: linuxserver/openssh-server:latest
  environment:
    - PUID=33           # www-data UID (mismo que WordPress)
    - PGID=33
    - TZ=Europe/Madrid
    - USER_NAME={ssh_user}
    - USER_PASSWORD={ssh_password}
    - PASSWORD_ACCESS=true
    - SUDO_ACCESS=false  # NUNCA dar sudo
    - LOG_STDOUT=true
  volumes:
    - wordpress-data:/home/{ssh_user}/html  # Enjaulado en su WP
  ports:
    - '{ssh_port}:2222'
  restart: unless-stopped
  deploy:
    resources:
      limits:
        cpus: '0.50'
        memory: 512M
      reservations:
        cpus: '0.10'
        memory: 64M
```

### 2. ChrootDirectory — Enjaulamiento SSH

**Configuración sshd_config custom (montar como volumen):**
```
# /config/sshd_config (override del default de linuxserver)
Port 2222
PermitRootLogin no
PasswordAuthentication yes
AllowTcpForwarding no
X11Forwarding no
PermitTunnel no
GatewayPorts no

# Enjaular al usuario en su home
Match User *
    ChrootDirectory /home/%u
    ForceCommand internal-sftp   # OPCIÓN A: solo SFTP (más seguro)
    # ForceCommand /bin/bash     # OPCIÓN B: shell completo enjaulado
    AllowTcpForwarding no
    X11Forwarding no
```

**Decisión pendiente:** ¿Shell completo o solo SFTP?
- **Solo SFTP** (`ForceCommand internal-sftp`): más seguro, el cliente solo sube/baja archivos. Para wp-cli necesitaría un panel web.
- **Shell enjaulado** (sin ForceCommand + ChrootDirectory): el cliente puede usar wp-cli, nano, etc. pero solo dentro de su directorio. Requiere instalar bash, coreutils, wp-cli dentro del chroot.
- **Recomendación**: Empezar con SFTP solamente. Agregar shell como upgrade premium si hay demanda.

### 3. Límites de recursos por contenedor

| Recurso | Contenedor WP | Contenedor SSH | Contenedor DB |
|---------|---------------|----------------|---------------|
| CPU | 1.0 core | 0.5 core | 0.5 core |
| RAM | 512M | 256M | 512M |
| Disco | 5GB quota* | compartido con WP | 2GB quota* |

*Nota: Docker storage quotas (`storage_opt.size`) solo funcionan con `overlay2` + XFS con `pquota`. Verificar en VPS2 antes de implementar.*

**Verificación necesaria (pre-implementación):**
```bash
# En VPS2, verificar filesystem y quotas
ssh root@173.249.50.44 "docker info | grep -i storage"
ssh root@173.249.50.44 "mount | grep ' / '"
ssh root@173.249.50.44 "cat /etc/docker/daemon.json"
```

Si NO hay XFS con pquota, las alternativas son:
- `tmpfs` con size limit (pierde datos en restart — NO viable)
- Volumen con `driver_opts.o=size=5G` (solo algunos drivers)
- Monitoreo + alerta cuando excede umbral (approach recomendado como fallback)

### 4. Monitoreo de recursos → Panel

**Backend:**
- Nuevo endpoint `GET /api/admin/hosting/{id}/resources` que hace SSH al VPS y ejecuta:
  ```bash
  docker stats --no-stream --format '{{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}\t{{.BlockIO}}' {container_prefix}*
  ```
- Parsear y devolver JSON con usage por contenedor
- Cache 30s (no hacer docker stats en cada request)

**Frontend:**
- Tab "Recursos" en HostingDetalleTabs (admin + cliente)
- Barras de progreso: CPU %, RAM usada/límite, Disco usado/quota
- Badge de alerta si algún recurso supera 80%

### 5. Migración de `atmoz/sftp` → `openssh-server`

**Pasos para hostings existentes:**
1. Actualizar compose template en `src/services/coolify.rs`
2. Reutilizar mismas credenciales (sftp_user, sftp_password, sftp_port)
3. El puerto se mantiene igual (el cliente no nota el cambio)
4. Redeploy del servicio Coolify → el nuevo container arranca con openssh-server
5. Renombrar columnas BD si se decide (sftp_* → ssh_*, o agregar ssh_* y deprecar sftp_*)

**Riesgos:**
- linuxserver/openssh-server usa puerto 2222 internamente (vs 22 en atmoz/sftp) — ajustar mapping
- El volumen se monta en `/home/{user}/html` (misma ruta que atmoz/sftp) — compatible
- PUID/PGID 33 (www-data) para que archivos subidos sean legibles por WordPress

### 6. Campos en BD

Decisión: ¿renombrar `sftp_*` a `ssh_*`?
- **Opción A** (recomendada): Mantener `sftp_user`, `sftp_password`, `sftp_port` — son las mismas credenciales, SSH y SFTP comparten el mismo servidor OpenSSH.
- **Opción B**: Agregar `ssh_enabled BOOLEAN DEFAULT false` para toggle SSH vs SFTP-only por hosting.

---

## Fases de implementación

### Fase 1 — Verificar infraestructura (pre-requisitos)
- [ ] Verificar overlay2/XFS/pquota en VPS2
- [ ] Test manual: crear container linuxserver/openssh-server con ChrootDirectory en VPS2
- [ ] Confirmar que SFTP y SSH funcionan con el setup propuesto
- [ ] Medir overhead de RAM del container SSH idle (~15-30MB esperado)

### Fase 2 — Actualizar compose template
- [ ] Reemplazar `atmoz/sftp` por `linuxserver/openssh-server` en `coolify.rs`
- [ ] Agregar sshd_config custom como volumen o inline en compose
- [ ] Agregar `deploy.resources.limits` a TODOS los contenedores del compose (wp, ssh, db)
- [ ] Test: provisionar nuevo hosting y verificar SSH + SFTP + limits

### Fase 3 — Panel de recursos
- [ ] Backend: endpoint Docker stats via SSH
- [ ] Frontend: Tab Recursos con barras de progreso
- [ ] Alertas: badge si recurso >80%

### Fase 4 — Migración existentes
- [ ] Script para redeploy hostings existentes con nuevo compose
- [ ] Verificar que credenciales existentes funcionan con openssh-server
- [ ] Rollback plan si algo falla

---

## Decisiones pendientes (requieren input del usuario)
1. ¿Shell completo o solo SFTP? (recomendación: solo SFTP inicialmente). Ambas cosas, pero enfocarse en la seguridad desde el inicio.
2. ¿Quota de disco por hosting? (depende de infraestructura VPS) Si.
3. ¿Límites de recursos diferenciados por plan de hosting? (ej: básico=0.5 CPU, pro=1 CPU) Si, y tiene que ser modificable por el admin y adaptarse automaticamente si se cambian los planes.
4. ¿wp-cli accesible desde SSH? (requiere shell + instalación dentro del chroot) Si. 
