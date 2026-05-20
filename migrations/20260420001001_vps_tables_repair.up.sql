-- [bug-fix] La migración 20260420001000_vps_resale.up.sql fue guardada sin saltos de línea;
-- el archivo entero quedó como un comentario SQL de una sola línea y no creó nada.
-- Esta migración usa IF NOT EXISTS para ser idempotente:
--   · Fresh DB (local/CI): crea las tres tablas y sus índices.
--   · Producción (tablas ya existen): no-op.

CREATE TABLE IF NOT EXISTS vps_plan_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tier_name VARCHAR(20) UNIQUE NOT NULL,
    display_name VARCHAR(80) NOT NULL,
    description TEXT NOT NULL,
    contabo_product_id VARCHAR(20) NOT NULL,
    base_cost_cents INT NOT NULL,
    monthly_price_cents INT NOT NULL,
    cpu_cores INT NOT NULL,
    ram_mb INT NOT NULL,
    disk_mb INT NOT NULL,
    region VARCHAR(20) NOT NULL DEFAULT 'EU',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    approval_required BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS vps_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    client_name VARCHAR(200) NOT NULL,
    client_email VARCHAR(255) NOT NULL,
    tier_name VARCHAR(20) NOT NULL REFERENCES vps_plan_configs(tier_name),
    requested_hostname VARCHAR(253),
    status VARCHAR(32) NOT NULL DEFAULT 'pending_payment',
    stripe_subscription_id VARCHAR(255) UNIQUE,
    monthly_price_cents INT NOT NULL,
    contabo_instance_id BIGINT UNIQUE,
    provisioning_ip VARCHAR(64),
    access_username VARCHAR(64),
    approved_by UUID REFERENCES users(id) ON DELETE SET NULL,
    approved_at TIMESTAMPTZ,
    provisioned_at TIMESTAMPTZ,
    rejected_reason TEXT,
    client_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_vps_subscriptions_user_id ON vps_subscriptions(user_id);
CREATE INDEX IF NOT EXISTS idx_vps_subscriptions_status ON vps_subscriptions(status);
CREATE INDEX IF NOT EXISTS idx_vps_subscriptions_tier_status ON vps_subscriptions(tier_name, status);

CREATE TABLE IF NOT EXISTS vps_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES vps_subscriptions(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_vps_events_subscription_id ON vps_events(subscription_id);
CREATE INDEX IF NOT EXISTS idx_vps_events_created_at ON vps_events(created_at DESC);
