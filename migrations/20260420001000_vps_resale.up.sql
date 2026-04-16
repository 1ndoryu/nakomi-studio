-- [164A-17] Reventa VPS: catálogo configurable + suscripciones dedicadas + auditoría de eventos.
-- product_id queda editable desde BD porque Contabo puede variar el SKU exacto por región/catálogo.
CREATE TABLE vps_plan_configs (
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

INSERT INTO vps_plan_configs
    (tier_name, display_name, description, contabo_product_id, base_cost_cents, monthly_price_cents, cpu_cores, ram_mb, disk_mb, region)
VALUES
    ('vps1', 'Cloud VPS 1', 'Instancia dedicada para proyectos pequeños, automatizaciones y entornos privados con acceso root.', 'V91', 550, 688, 4, 8192, 204800, 'EU'),
    ('vps2', 'Cloud VPS 2', 'Servidor balanceado para SaaS liviano, APIs, staging persistente y sitios con más margen operativo.', 'V92', 990, 1238, 6, 16384, 409600, 'EU'),
    ('vps3', 'Cloud VPS 3', 'Nodo dedicado para cargas medianas, workers concurrentes y tiendas o apps con tráfico sostenido.', 'V93', 1650, 2063, 8, 30720, 819200, 'EU'),
    ('vps4', 'Cloud VPS 4', 'Capacidad dedicada para cargas intensivas, pipelines pesados y aplicaciones con mucha memoria.', 'V94', 2970, 3713, 12, 49152, 1638400, 'EU');

CREATE TABLE vps_subscriptions (
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

CREATE INDEX idx_vps_subscriptions_user_id ON vps_subscriptions(user_id);
CREATE INDEX idx_vps_subscriptions_status ON vps_subscriptions(status);
CREATE INDEX idx_vps_subscriptions_tier_status ON vps_subscriptions(tier_name, status);

CREATE TABLE vps_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES vps_subscriptions(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vps_events_subscription_id ON vps_events(subscription_id);
CREATE INDEX idx_vps_events_created_at ON vps_events(created_at DESC);