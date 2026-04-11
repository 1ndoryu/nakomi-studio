# Plan: Seguridad Integral del Servicio de Hosting

> **Fecha:** 2026-04-17 (actualizado)
> **Estado:** En progreso — Fases 2-3 completadas (164A-16)
> **Prioridad:** Crítica — varias vulnerabilidades de nivel alto detectadas
> **Contexto:** Auditoría de seguridad del servicio de hosting WordPress administrado (Coolify + Docker Compose)

---

## Resumen de hallazgos

| Área                | Riesgo     | Estado                                                           |
| ------------------- | ---------- | ---------------------------------------------------------------- |
| Secrets management  | 🔴 Crítico | .env en .gitignore ✅ — rotación pendiente                       |
| Backups             | 🔴 Crítico | Solo plan ecommerce (3 daily + 2 weekly) — pendiente implementar |
| SFTP credentials    | 🟡 Medio   | Generación fuerte, sin rotación — PENDIENTE                      |
| Container isolation | ✅ Resuelto | Network isolation + cap_drop ALL + no-new-privileges (164A-16)  |
| Resource limits     | ✅ Resuelto | CPU/mem/pids_limit/reservations (164A-16)                        |
| Port collisions     | ✅ Resuelto | UNIQUE constraint + retry loop 10000-49151 (164A-16)            |
| WordPress hardening | ✅ Resuelto | Imágenes pineadas, DISALLOW_FILE_EDIT (164A-16)                 |
| DB isolation        | ✅ Resuelto | MariaDB en backend_net internal only (164A-16)                   |
| DNS/Domain          | 🟡 Medio   | Sin validación de ownership de dominios custom — PENDIENTE       |
| API authorization   | 🟢 Bajo    | Role-based checks + Referrer-Policy/Permissions-Policy (164A-16)|
| Security headers    | ✅ Resuelto | HSTS + nosniff + DENY + referrer-policy + permissions (164A-16) |

---

## Fase 1 — Crítica: Secrets y Backups (prioridad máxima)

### 1.1 Secrets management

**Problema:** `.env` contiene COOLIFY*API_TOKEN, Stripe live keys (sk_live*\*), Contabo credentials, Google OAuth — todo plaintext.
**Solución:**

- [ ] Asegurar que `.env` está en `.gitignore` (verificar en producción)
- [ ] JWT_SECRET: generar secreto robusto de 64+ chars en producción
- [ ] Plan de rotación: documentar proceso para rotar cada token/key
- [ ] Coolify API token: verificar permisos mínimos necesarios (least privilege)
- [ ] En producción: inyectar secrets via Coolify env vars (no archivos en disco)

### 1.2 Backups automáticos

**Problema:** `delete_service()` elimina volúmenes sin backup. No hay snapshots diarios.
**Solución:**

- [ ] Antes de `delete_service()`: exportar dump SQL + tar de wp-content a storage externo
- [ ] Implementar backup cron: dump MariaDB + rsync wp-content → directorio de backups del VPS
- [ ] Retención: 7 días rotating
- [ ] Backup antes de cancelación obligatorio (grace period 48h antes de borrar)
- [ ] Endpoint admin: `GET /api/admin/hosting/{id}/backup` — trigger manual
- [ ] Documentar restore procedure

---

## Fase 2 — Alta: Aislamiento y Credenciales

### 2.1 Network isolation

**Problema:** Containers en bridge default — WordPress, MariaDB y SSH pueden hablar entre sí sin restricción.
**Solución:**

- [ ] Crear networks explícitas en compose: `frontend` (WP↔Traefik), `backend` (WP↔MariaDB), `ssh` (SSH↔wp-content volume)
- [ ] MariaDB: solo en network `backend` (no accesible desde SSH)
- [ ] SSH: solo acceso al volume de wp-content, no a la red de MariaDB
- [ ] WordPress: en `frontend` + `backend`

### 2.2 Container hardening

- [ ] Agregar `cap_drop: [ALL]` a todos los containers
- [ ] Agregar `cap_add` solo lo necesario: WP necesita `CHOWN`, `SETUID`, `SETGID`; SSH necesita `NET_BIND_SERVICE`
- [ ] `security_opt: [no-new-privileges:true]` en todos
- [ ] `read_only: true` donde sea viable (MariaDB: no, WP: parcialmente)
- [ ] `tmpfs: /tmp` para containers que necesiten escritura temporal

### 2.3 Credential rotation y SSH keys

**Problema:** Credenciales SFTP nunca rotan. Solo password auth.
**Solución:**

- [ ] Agregar campo `sftp_credentials_rotated_at` a `hosting_subscriptions`
- [ ] Endpoint: `POST /api/hosting/{id}/rotate-credentials` — genera nuevo password, actualiza env en Coolify
- [ ] Agregar soporte SSH key auth: campo `ssh_public_key` en subscription, montar en volume del container
- [ ] Recomendación al usuario: usar SSH key en vez de password
- [ ] Rate limit en intentos de login SSH (fail2ban o equiv dentro del container)

---

## Fase 3 — Media: Recursos, Puertos y WordPress

### 3.1 Port collision prevention

**Problema:** Puerto aleatorio 10000-65000 sin verificar unicidad.
**Solución:**

- [ ] Antes de asignar puerto: `SELECT sftp_port FROM hosting_subscriptions WHERE sftp_port = $port`
- [ ] Loop retry: si existe, generar otro (max 10 intentos)
- [ ] Agregar UNIQUE constraint a columna `sftp_port`
- [ ] Excluir rango ephemeral (49152-65535): usar solo 10000-49151

### 3.2 Resource limits mejorados

- [ ] Agregar `memswap_limit: 512M` (igual a mem limit para evitar swap infinito)
- [ ] Agregar `deploy.resources.reservations` para CPU/memory mínimos garantizados
- [ ] Considerar pids_limit (evitar fork bombs): `pids_limit: 200` para WP, `100` para SSH
- [ ] Disk quota: investigar `--storage-opt size=2G` (requiere overlay2 con xfs)

### 3.3 WordPress hardening

- [ ] Pinear versión de imagen: `wordpress:6.7-php8.3-apache` (no `latest`)
- [ ] Pinear MariaDB: `mariadb:11.4` (no `latest`)
- [ ] Agregar security headers via .htaccess o mod_headers en compose
- [ ] Deshabilitar XML-RPC: `WORDPRESS_XML_RPC_DISABLED=true` o bloquear en Apache
- [ ] Deshabilitar file editing en wp-admin: `WORDPRESS_CONFIG_EXTRA: define('DISALLOW_FILE_EDIT', true);`
- [ ] Limitar plugins instalables (evaluar feasibility)

### 3.4 DB encryption in transit

- [ ] Habilitar SSL entre WordPress y MariaDB (generar certs auto-signed)
- [ ] `MYSQL_REQUIRE_SECURE_TRANSPORT=ON` en MariaDB vars

---

## Fase 4 — Media: DNS, Dominios y Rate Limiting

### 4.1 Domain ownership validation

**Problema:** Clientes pueden asignar dominios custom sin verificar ownership.
**Solución:**

- [ ] Al asignar dominio custom: generar token TXT único (`_nakomi-verify.domain.com`)
- [ ] Endpoint: `POST /api/hosting/{id}/verify-domain` — chequear DNS TXT
- [ ] Status de dominio: `pending_verification` → `verified` → `active`
- [ ] No activar Traefik routing hasta verificación completa
- [ ] Auto-SSL solo para dominios verificados

### 4.2 Rate limiting en API

- [ ] Rate limit en `POST /api/hosting/subscribe`: max 3 por usuario por hora
- [ ] Rate limit en `POST /api/hosting/checkout`: max 5 por usuario por hora
- [ ] Rate limit global por IP: tower-governor o middleware custom
- [ ] SFTP login: fail2ban dentro del container SSH (o `MaxAuthTries 3` en sshd_config)

### 4.3 HTTPS enforcement

- [ ] Verificar que Let's Encrypt se provisiona automáticamente para dominios custom
- [ ] HSTS header en Traefik: `traefik.http.middlewares.hsts.headers.stsSeconds=31536000`
- [ ] Redirect HTTP→HTTPS obligatorio

---

## Fase 5 — Monitoreo y Auditoría

### 5.1 Logging de seguridad

- [ ] Log de todos los accesos SSH/SFTP (IP, usuario, hora, acción)
- [ ] Log de provisioning/deletion de hosting (quién, cuándo, qué)
- [ ] Alert on suspicious activity: múltiples intentos fallidos de login, port scan patterns
- [ ] Integrar con el activity_log existente del sistema

### 5.2 Health monitoring por hosting

- [ ] Cron healthcheck: verificar que cada hosting responde HTTP 200
- [ ] Si falla 3 checks consecutivos: notificar admin
- [ ] Dashboard admin: estado de todos los hostings activos
- [ ] Disk usage monitoring: alertar cuando volume > 80% capacity

---

## Priorización de implementación

1. **Immediato** (próxima sesión): 1.1 (secrets audit) + 3.1 (port collision UNIQUE constraint)
2. **Corto plazo**: 1.2 (backups) + 2.1 (network isolation) + 3.3 (WP hardening)
3. **Medio plazo**: 2.2 (container hardening) + 2.3 (credential rotation) + 4.1 (domain verification)
4. **Largo plazo**: 3.2 (resource refinement) + 4.2 (rate limiting) + 5.x (monitoring)

---

## Notas

- El análisis se basa en auditoría de `coolify.rs`, `hosting.rs` (handlers/repo), `.env`, compose template
- Las correcciones no deben romper hostings existentes — requieren migration strategy
- Network isolation requiere redeploy de hostings existentes
- Pinear versiones de imágenes Docker es la victoria más rápida en hardening
