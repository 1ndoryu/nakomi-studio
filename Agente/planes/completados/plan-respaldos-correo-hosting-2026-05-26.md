# Plan: Pestañas de Respaldos y Correo en Detalle de Hosting

> **Fecha:** 2026-05-26
> **Estado:** Investigación completada, pendiente revisión del usuario
> **Contexto:** Al abrir un hosting en el panel (`/panel?seccion=hosting&hostingId=...`) faltan 2 pestañas prometidas en los planes: **Respaldos** y **Correo**.

---

## 1. INVESTIGACIÓN COMPLETA

### 1.1 Estado actual del frontend

**Archivo principal:** `frontend/src/components/panel/HostingDetalle.tsx`

Tabs existentes (6):
| Tab | Icon | Componente |
|-----|------|-----------|
| General | Server | `TabGeneral` (en `HostingDetalleTabs.tsx`) |
| Recursos | Zap | `TabRecursos` (en `HostingDetalleTabs.tsx`) |
| Dominio & SSL | Globe | `TabDominio` (en `HostingDetalleAccess.tsx`) |
| Acceso | Terminal | `TabAcceso` (en `HostingDetalleAccess.tsx`) |
| Facturación | CreditCard | `TabFacturacion` (en `TabFacturacion.tsx`) |
| Eventos | Clock | `TabEventos` (en `HostingDetalleTabs.tsx`) |

Definición de array `TABS` y `HostingDetalleTab` type en:
- `frontend/src/components/panel/HostingDetalle.tsx` (líneas ~30-40)
- `frontend/src/hooks/useHostingDetalle.ts` (línea ~10): `type HostingDetalleTab = 'general' | 'recursos' | 'dominio' | 'acceso' | 'facturacion' | 'eventos'`

**No existe hook de backups ni de correo** en frontend.

---

### 1.2 Estado del backend — Respaldos

**Ya existe API completa** en backend:

| Endpoint | Método | Handler | Estado |
|----------|--------|---------|--------|
| `/api/hosting/subscriptions/{id}/backups` | GET | `list_backups` (en `handlers/hosting/backups.rs`) | ✅ Implementado |
| `/api/hosting/subscriptions/{id}/backups` | POST | `create_backup` (en `handlers/hosting/backups.rs`) | ✅ Admin-only |
| `/api/hosting/subscriptions/{id}/restore` | POST | `restore_backup` (en `handlers/hosting/backups.rs`) | ✅ Admin-only |

**Pero hay un problema CRÍTICO:** El `HostingRuntimeService::list_backups()` devuelve error `unsupported` para `runtime_kind = Coolify`. Solo funciona para `Lightweight` (que llama a `coolify-manager-rs light-backup --list --json`).

Esto significa que **para hostings WordPress en Coolify** (la mayoría actual), los backups existen como sidecar dentro del compose (mariadb haciendo mysqldump + tar.gz a volumen `backup-data`), pero **no hay API que los liste desde el backend Rust**.

#### ¿Qué hay realmente en el sidecar de backup?

El compose de WordPress genera:
```yaml
services:
  backup:
    image: mariadb:11.4
    environment:
      - MYSQL_PWD=SERVICE_PASSWORD_DB
    command: |
      sleep 60; while true; do
        DT=$(date +%Y%m%d_%H%M%S)
        mysqldump -h mariadb -u wordpress wordpress > /backups/daily_$DT.sql
        tar czf /backups/daily_wp_$DT.tar.gz -C /wp-html .
        # + retención diaria 3 días / semanal 14 o 28 días
        sleep 86400
      done
    volumes:
      - 'wordpress-data:/wp-html:ro'
      - 'backup-data:/backups'
```

Los backups **se están generando** pero **no son visibles ni gestionables** por el usuario.

#### Opciones para hacer funcional los respaldos en Coolify:

**Opción A (recomendada):** Añadir al backend Rust un endpoint que ejecute `docker exec` via SSH en el contenedor backup para listar/crear/restaurar archivos en `backup-data`. Esto requiere:
1. Saber el UUID del servicio Coolify (ya se tiene en `hosting_subscriptions.server_uuid`)
2. SSH al VPS y ejecutar comandos Docker en el servicio
3. Parsear salida para el frontend

**Opción B:** Migrar hostings WordPress a runtime Lightweight (cambios mayores, no viable a corto plazo).

**Opción C:** Implementar un helper en `coolify-manager-rs` que liste archivos en el volumen `backup-data` del servicio Coolify vía SSH. Similar a cómo ya funciona `light-backup --list` pero para Coolify.

---

### 1.3 Estado del backend — Correo

**No existe nada.** Cero modelos, rutas o handlers para buzones de correo.

Solo existe `src/services/email.rs` que usa `lettre` con SMTP transaccional (Brevo) para enviar **emails de la plataforma** (confirmaciones de pedido, escalaciones de chat, etc.).

#### Documentación existente

Ya hay un documento completo: `Agente/documentacion/hosting/buzones-correo-clientes-2026-05-21.md`

**Conclusión clave:** Implementar buzones (IMAP/POP3/SMTP) self-hosted requiere:
- Postfix + Dovecot + Rspamd + Roundcube o similar
- Gestión de DNS (MX, SPF, DKIM, DMARC)
- Reputación IP (VPS2 es nuevo, caerá en spam)
- Monitoreo 24/7 de colas, bounce, blacklists
- Superficie de abuso masiva (spam, reenvío, cuentas robadas)

**Esto NO es recomendado para el modelo de negocio actual.** Alternativas:
1. **Alias/reenvío simple** (Cloudflare Email Routing): `info@dominiodelcliente → gmail del cliente`. Más simple, sin storage, sin IMAP.
2. **Proveedor externo** (Migadu, Zoho, MXroute): La plataforma gestiona DNS, el proveedor los buzones.
3. **Nada por ahora:** Ocultar la pestaña si no hay add-on contratado.

#### Lo que WordPress ya puede hacer para enviar correo

Un WordPress estándar usa `wp_mail()` que internamente usa `PHPMailer`. Sin configuración SMTP:
- En el contenedor Docker de Coolify, PHP usa `sendmail` del sistema → no hay MTA instalado
- Los formularios de contacto, recuperación de contraseña, notificaciones de WooCommerce **NO funcionarán**
- Solución estándar en hosting: plugin SMTP (WP Mail SMTP, Post SMTP) o configurar `wp-config.php` con las credenciales SMTP de la plataforma

**Estado actual:** los WordPress desplegados vía Coolify NO pueden enviar correo porque:
1. `DISALLOW_FILE_EDIT` está activo (impide instalar plugins vía admin) — pero se puede sobreescribir
2. No hay `SMTP_HOST` ni credenciales en el environment del compose
3. El contenedor no tiene MTA configurado

---

## 2. PLAN DE IMPLEMENTACIÓN

### Fase 1: Pestaña de Respaldos (Backups) — Frontend + Backend

#### 2.1 Backend: Soportar backups en Coolify runtime

**2.1.1** Crear helper en coolify-manager-rs: `coolify-backup --list --site {uuid|name}` que via SSH:
- Detecta el compose project del servicio
- Ejecuta `docker compose exec backup ls -la /backups/` o similar
- Retorna JSON con lista de archivos, fechas, tamaños

**2.1.2** En el backend Rust (`src/services/hosting_runtime.rs`), para `HostingRuntimeKind::Coolify`:
- `list_backups`: SSH al VPS, leer archivos del volumen `backup-data` en el proyecto del servicio. Opciones:
  - Usar `docker run --rm -v {project}_backup-data:/backups alpine:3.20 ls -la /backups/`
  - Parsear nombres de archivo para extraer fecha/tier
- `create_backup`: disparar `docker compose exec backup` para forzar backup inmediato
- `restore_backup`: detener servicio, montar backup, restaurar archivos + BD

**2.1.3** Alternativa más pragmática (recomendada para primera iteración):
- Ampliar `HostingRuntimeService` para Coolify usando SSH directo (ya existe infraestructura en `src/services/docker_stats.rs` que hace SSH)
- Implementar solo `list_backups` inicialmente (lectura), dejar `create`/`restore` para después

#### 2.2 Frontend: TabBackups

**2.2.1** Agregar tipo `HostingBackupEntry` en `frontend/src/api/hosting.ts`:
```typescript
export interface HostingBackupEntry {
    backup_id: string;
    tier: string;        // 'daily' | 'weekly' | 'manual'
    file_name: string;
    file_size_bytes: number | null;
    created_at: string;
    label?: string | null;
}
```

**2.2.2** Agregar endpoints API en `hosting.ts`:
```typescript
export async function apiListBackups(subscriptionId: string): Promise<HostingBackupEntry[]>
export async function apiCreateBackup(subscriptionId: string, label?: string): Promise<void>
export async function apiRestoreBackup(subscriptionId: string, backupId: string, password?: string): Promise<void>
export async function apiDeleteBackup(subscriptionId: string, backupId: string): Promise<void>
```

**2.2.3** Agregar `'backups'` a `HostingDetalleTab` en `useHostingDetalle.ts` y crear el hook `useHostingBackups.ts`:
- Fetch de lista al activar tab
- Mutaciones para crear/restaurar/eliminar

**2.2.4** Crear componente `TabBackups` (idealmente archivo nuevo `TabBackups.tsx`):
- Lista de respaldos con:
  - Nombre del archivo
  - Fecha del backup
  - Tipo (diario/semanal/manual)
  - Tamaño (bytes → human readable)
  - Botón "Restaurar" → modal de confirmación (con opción de contraseña SFTP)
  - Botón "Eliminar" → modal de confirmación
- Botón "Crear respaldo manual"
- Estados: vacío, cargando, error
- Mensaje informativo sobre periodicidad según el plan

**2.2.5** Conectar en `HostingDetalle.tsx`:
- Añadir tab "Respaldos" con icono `HardDrive` (o `Archive`)
- Renderizar `TabBackups`

### Fase 2: Pestaña de Correo — Investigación + MVP

#### 2.1 Lo que WordPress necesita para enviar correo

**Solución inmediata (configurable al provisionar):** Inyectar variables SMTP en el compose del WordPress:
```yaml
wordpress:
  environment:
    - WORDPRESS_CONFIG_EXTRA=define('DISALLOW_FILE_EDIT', true);\ndefine('SMTP_HOST', 'smtp.brevo.com');\ndefine('SMTP_PORT', 587);\ndefine('SMTP_USER', '...');\ndefine('SMTP_PASS', '...');\ndefine('SMTP_FROM', 'noreply@nakomi.studio');
```

Esto se hace agregando `WP_SMTP_*` envs al compose o parcheando `wp-config.php` vía wp-cli en el contenedor SSH.

**Alternativa:** Auto-instalar plugin WP Mail SMTP vía wp-cli durante el provisioning.

#### 2.2 Qué mostrar en la pestaña Correo (MVP)

**Opción 1 — Aliases simples (Cloudflare Email Routing):**
- Si el dominio está verificado y usa Cloudflare DNS:
  - Mostrar formulario para crear alias: `info@dominio → email del cliente`
  - Estado del alias
  - No requiere infraestructura de mailbox

**Opción 2 — Guía informativa:**
- Explicar que el correo del hosting está configurado
- Mostrar credenciales SMTP si se inyectaron
- Instrucciones para configurar en Outlook/Gmail/Thunderbird
- CTA para contratar buzones (próximamente)

**Opción 3 — Proveedor externo (post-MVP):**
- Tab que muestra estado del dominio de correo
- Botón "Contratar buzón" → Stripe add-on
- Provisión vía API del proveedor
- Gestión DNS (MX, SPF, DKIM)

#### 2.3 Frontend: TabCorreo (MVP informativo)

**2.3.1** Agregar endpoints si aplica, o solo UI informativa.

**2.3.2** Agregar `'correo'` a `HostingDetalleTab` en `useHostingDetalle.ts`.

**2.3.3** Crear componente `TabCorreo.tsx`:
- Estado actual del correo en el hosting
- Si es WordPress: mostrar que `wp_mail()` está configurado vía SMTP de Nakomi
- Botón "Configurar correo" → modal/acciones (según opción elegida)

**2.3.4** Conectar en `HostingDetalle.tsx`.

---

## 3. DEPENDENCIAS Y RIESGOS

| Ítem | Riesgo | Mitigación |
|------|--------|-----------|
| Backups Coolify: SSH al VPS para leer backups | Nuevo código de SSH en runtime | Reutilizar infraestructura `docker_stats.rs` |
| Backups Coolify: `docker run --rm -v` puede no encontrar el volumen | Nombre de volumen depende del proyecto UUID | Usar `docker volume ls --filter name={project}_backup` |
| Correo WordPress: `wp-config.php` vía wp-cli | Requiere contenedor SSH funcionando | Ya existe el contenedor SSH con wp-cli instalado |
| Correo WordPress: plugin SMTP vs wp-config | Plugin puede romperse con actualizaciones | Preferir wp-config injection + constantes |
| Backups: restore requiere parar servicio | Downtime | Modal de confirmación explicando el downtime |

---

## 4. ESTIMACIÓN DE ESFUERZO

| Componente | Archivos | Esfuerzo |
|-----------|----------|----------|
| Backend: list_backups para Coolify | `src/services/hosting_runtime.rs`, `src/services/coolify.rs` | 3-4h |
| Frontend: TabBackups | `TabBackups.tsx`, `hosting.ts`, `useHostingDetalle.ts`, `HostingDetalle.tsx` | 3-4h |
| Correo: inyectar SMTP en compose de WP | `src/services/coolify.rs`, `src/services/hosting_runtime.rs` | 2-3h |
| Frontend: TabCorreo (MVP informativo) | `TabCorreo.tsx`, `HostingDetalle.tsx` | 2-3h |
| **Total estimado** | | **10-14h** |

---

## 5. PREGUNTAS PENDIENTES PARA EL USUARIO

1. **Respaldos en Coolify:** ¿Aceptas la implementación vía SSH para listar archivos en el volumen `backup-data` directamente? O ¿prefieres que implemente un helper en `coolify-manager-rs` primero? R: lo que sea mejor, un helper suena mejor croe.

2. **Restaurar respaldos:** ¿Quieres que restore sea operable desde el panel (con modal de confirmación) o solo desde el backend/admin por ahora? R: desde el panel para los usuario, no tiene que fallar.

3. **Correo WordPress:** ¿Quieres que los WordPress nuevos ya puedan enviar correo (inyectando SMTP de Nakomi en el compose) o prefieres que el usuario configure su propio SMTP? R: no lo se, lo que sea normal en los hostings. 

4. **Pestaña Correo - MVP:** ¿Qué contenido inicial quieres?
   - a) Solo informativo: "tu WordPress puede enviar correo vía SMTP de Nakomi, aquí están las credenciales"
   - b) Alias/reenvío (info@dominio → email del cliente) vía Cloudflare Email Routing
   - c) Ocultar la pestaña hasta que haya un producto de correo real
   R: Hasta que ya un producto real.

5. **¿El hosting que abriste es el de prueba `0fa1d5da` que es WordPress en Coolify?** Esto confirma si los respaldos se generarían via sidecar Docker o si estamos ante un hosting Normal (Nginx+SFTP). R: Si es de prueba, es de wordpress.
