# Gestión de Secrets — Procedimiento de Rotación

> **Fecha:** 2026-04-11
> **Contexto:** Documentación de todos los secrets del sistema y proceso para rotarlos.

---

## Inventario de secrets

| Secret | Ubicación | Rotación | Impacto de rotación |
|--------|-----------|----------|---------------------|
| DATABASE_URL | .env / Coolify env | Raro (cambio de host/password) | Downtime — requiere restart |
| JWT_SECRET | .env / Coolify env | Semestral mínimo | Invalida TODOS los tokens activos — todos los usuarios deben re-login |
| GROQ_API_1/2/3 | .env / Coolify env | Cuando se sospeche compromiso | Chatbot offline temporalmente mientras se genera nueva key |
| GOOGLE_GEMINI_API | .env / Coolify env | Cuando se sospeche compromiso | Solo afecta fallback del chatbot |
| COOLIFY_API_TOKEN | .env / Coolify env | Trimestral | Provisioning/management de hosting offline hasta actualizar |
| STRIPE_SECRET_KEY | .env / Coolify env | Anual o cuando se sospeche compromiso | Pagos offline — actualizar ASAP |
| STRIPE_WEBHOOK_SECRET | .env / Coolify env | Al rotar Stripe key o recrear webhook | Webhooks rechazados hasta actualizar |
| CONTABO_* | .env / Coolify env | Cuando se sospeche compromiso | Solo afecta panel admin VPS |
| SMTP_PASSWORD | .env / Coolify env | Cuando se sospeche compromiso | Notificaciones email offline |
| Hosting SFTP passwords | BD (hosting_subscriptions) | Vía endpoint /rotate-credentials | Acceso SFTP del cliente se resetea |

---

## Procedimiento por secret

### JWT_SECRET
1. Generar nuevo secret: `openssl rand -base64 64`
2. Actualizar en Coolify env vars del servicio nakomi
3. Hacer restart/redeploy del servicio
4. **Advertencia:** TODOS los tokens JWT activos se invalidan. Los usuarios deberán hacer login de nuevo.

### COOLIFY_API_TOKEN
1. Ir a Coolify panel → Settings → API Tokens
2. Revocar token anterior, crear uno nuevo
3. Usar permisos mínimos: `services:read`, `services:create`, `services:update`, `services:delete`
4. Actualizar en .env local y en Coolify env vars del servicio principal

### STRIPE_SECRET_KEY + WEBHOOK_SECRET
1. En Stripe Dashboard → Developers → API Keys → Roll key
2. Stripe mantiene la key anterior activa 72h (grace period)
3. Actualizar GLORY_STRIPE_SECRET_KEY en prod
4. Si se recrea webhook endpoint: actualizar GLORY_STRIPE_WEBHOOK_SECRET

### SFTP credentials (por hosting)
1. Desde panel admin: POST /api/hosting/subscriptions/{id}/rotate-credentials
2. El endpoint genera nueva contraseña, actualiza BD y Coolify compose, reinicia SSH
3. Comunicar nueva contraseña al cliente
4. La contraseña anterior deja de funcionar inmediatamente tras restart

---

## Reglas de seguridad

- **Nunca** commitear secrets en código fuente
- `.env` debe estar en `.gitignore` (verificado ✅)
- JWT_SECRET en producción debe ser ≥64 caracteres (`openssl rand -base64 64`)
- En producción: inyectar secrets vía Coolify env vars, no archivos en disco
- Rotar inmediatamente si se sospecha compromiso de cualquier secret
- Coolify API token: usar permisos mínimos necesarios
