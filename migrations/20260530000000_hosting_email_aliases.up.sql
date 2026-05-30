/* [265A-11] Producto de correo: aliases (Opcion A activa) y preparacion de buzones (Opcion B).
 * Cloudflare Email Routing: forwarding gratis sin costo operativo.
 * Migadu/MXroute: tablas preparadas para Fase 2, NO activa aun. */

CREATE TABLE hosting_email_aliases (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    subscription_id UUID NOT NULL REFERENCES hosting_subscriptions(id) ON DELETE CASCADE,
    alias VARCHAR(100) NOT NULL, -- ej: info, ventas, soporte
    domain VARCHAR(253) NOT NULL,
    destination VARCHAR(254) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(subscription_id, alias, domain)
);

CREATE INDEX idx_hosting_email_aliases_subscription ON hosting_email_aliases(subscription_id);
CREATE INDEX idx_hosting_email_aliases_status ON hosting_email_aliases(status);

/* [265A-12] Preparacion para Fase 2 (buzones IMAP). Tabla inactiva hasta activacion manual. */
CREATE TABLE hosting_email_mailboxes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    subscription_id UUID NOT NULL REFERENCES hosting_subscriptions(id) ON DELETE CASCADE,
    email VARCHAR(354) NOT NULL,
    password_hash VARCHAR(255),
    provider VARCHAR(20) NOT NULL DEFAULT 'migadu',
    provider_mailbox_id VARCHAR(100),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    storage_used_mb INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(subscription_id, email)
);

CREATE INDEX idx_hosting_email_mailboxes_subscription ON hosting_email_mailboxes(subscription_id);
CREATE INDEX idx_hosting_email_mailboxes_status ON hosting_email_mailboxes(status);

/* [265A-11] Limite de aliases incluidos por plan */
ALTER TABLE hosting_plan_configs
    ADD COLUMN included_aliases INT NOT NULL DEFAULT 0;

/* [265A-12] Limite de buzones incluidos por plan (preparado, sin efecto aun) */
ALTER TABLE hosting_plan_configs
    ADD COLUMN included_mailboxes INT NOT NULL DEFAULT 0;

/* [265A-11] Defaults por plan: Pro=3 aliases, Avanzado=5 aliases + 1 mailbox */
UPDATE hosting_plan_configs SET included_aliases = 0 WHERE plan_name = 'basico';
UPDATE hosting_plan_configs SET included_aliases = 3 WHERE plan_name = 'pro';
UPDATE hosting_plan_configs SET included_aliases = 5 WHERE plan_name = 'ecommerce';
UPDATE hosting_plan_configs SET included_aliases = 0 WHERE plan_name LIKE 'normal-%';

UPDATE hosting_plan_configs SET included_mailboxes = 0;
UPDATE hosting_plan_configs SET included_mailboxes = 1 WHERE plan_name = 'ecommerce';
