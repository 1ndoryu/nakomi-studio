-- [104A-18] Credenciales SFTP generadas al provisionar hosting WordPress.
-- sftp_user: nombre de usuario del contenedor atmoz/sftp.
-- sftp_password: contraseña generada aleatoriamente (almacenada en texto, no es auth de sistema).
ALTER TABLE hosting_subscriptions
    ADD COLUMN IF NOT EXISTS sftp_user TEXT,
    ADD COLUMN IF NOT EXISTS sftp_password TEXT;
