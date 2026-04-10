-- [104A-42] Añadir campos de servidor Coolify a hosting_subscriptions.
-- server_uuid: UUID del servicio en Coolify (para gestión: stop, delete, redeploy)
-- server_ip: IP pública del VPS donde está desplegado el hosting

ALTER TABLE hosting_subscriptions ADD COLUMN IF NOT EXISTS server_uuid TEXT;
ALTER TABLE hosting_subscriptions ADD COLUMN IF NOT EXISTS server_ip TEXT;
