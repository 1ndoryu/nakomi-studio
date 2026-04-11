-- [104A-18] Puerto SFTP asignado dinámicamente al provisionar hosting WordPress.
-- Cada hosting recibe un puerto único (10000-65000) para el contenedor atmoz/sftp.
ALTER TABLE hosting_subscriptions
    ADD COLUMN IF NOT EXISTS sftp_port INT;
