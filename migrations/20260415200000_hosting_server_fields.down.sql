-- [104A-42] Revertir campos de servidor Coolify
ALTER TABLE hosting_subscriptions DROP COLUMN IF EXISTS server_uuid;
ALTER TABLE hosting_subscriptions DROP COLUMN IF EXISTS server_ip;
