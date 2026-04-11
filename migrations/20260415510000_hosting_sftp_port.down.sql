-- Revertir columna sftp_port
ALTER TABLE hosting_subscriptions DROP COLUMN IF EXISTS sftp_port;
