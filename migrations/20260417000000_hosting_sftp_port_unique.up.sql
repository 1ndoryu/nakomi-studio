-- [164A-16] UNIQUE constraint en sftp_port para prevenir colisiones.
-- Solo aplica a puertos no nulos (suscripciones ya provisionadas).
-- Rango restringido en código: 10000-49151 (evita puertos efímeros 49152-65535).
ALTER TABLE hosting_subscriptions
    ADD CONSTRAINT hosting_subscriptions_sftp_port_key UNIQUE (sftp_port);
