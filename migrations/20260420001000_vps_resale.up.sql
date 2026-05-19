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
    setup_fee_cents INT NOT NULL DEFAULT 0,
    cpu_cores INT NOT NULL,
    ram_mb INT NOT NULL,
    disk_mb INT NOT NULL,
    storage_type VARCHAR(20) NOT NULL DEFAULT 'NVMe',
    storage_options TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    port_speed_mbps INT NOT NULL DEFAULT 200,
    bandwidth_label VARCHAR(120) NOT NULL DEFAULT 'Tráfico ilimitado con uso justo',
    snapshot_count INT NOT NULL DEFAULT 1,
    region VARCHAR(20) NOT NULL DEFAULT 'EU',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    approval_required BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO vps_plan_configs
    (tier_name, display_name, description, contabo_product_id, base_cost_cents, monthly_price_cents, setup_fee_cents, cpu_cores, ram_mb, disk_mb, storage_type, storage_options, port_speed_mbps, bandwidth_label, snapshot_count, region)
VALUES
    ('vps1', 'Cloud VPS 10', 'Entrada dedicada para automatizaciones, paneles internos y servicios pequeños con acceso root.', 'V91', 450, 473, 450, 4, 8192, 76800, 'NVMe', ARRAY['75 GB NVMe', '150 GB SSD'], 200, 'Tráfico ilimitado con uso justo', 1, 'EU'),
    ('vps2', 'Cloud VPS 20', 'Servidor balanceado para APIs, SaaS liviano y cargas sostenidas con más memoria.', 'V92', 840, 882, 0, 6, 12288, 102400, 'NVMe', ARRAY['100 GB NVMe', '200 GB SSD'], 1000, 'Tráfico ilimitado con uso justo', 2, 'EU'),
    ('vps3', 'VPS 30', 'Nodo dedicado para workloads medianos, workers concurrentes y aplicaciones con tráfico constante.', 'V93', 1680, 1764, 0, 8, 24576, 204800, 'NVMe', ARRAY['200 GB NVMe', '400 GB SSD'], 1000, 'Tráfico ilimitado con uso justo', 3, 'EU'),
    ('vps4', 'VPS 40', 'Capacidad dedicada para pipelines pesados, bases de datos exigentes y servicios con mucha memoria.', 'V94', 3000, 3150, 0, 12, 49152, 256000, 'NVMe', ARRAY['250 GB NVMe', '500 GB SSD'], 1000, 'Tráfico ilimitado con uso justo', 3, 'EU'),
    ('vps5', 'VPS 50', 'Servidor de alta memoria para cargas intensivas, colas y stacks con varios servicios.', 'V95', 4450, 4673, 0, 16, 65536, 307200, 'NVMe', ARRAY['300 GB NVMe', '600 GB SSD'], 1000, 'Tráfico ilimitado con uso justo', 3, 'EU'),
    ('vps6', 'VPS 60', 'Capacidad máxima del catálogo VPS para workloads pesados y crecimiento sostenido.', 'V96', 5880, 6174, 0, 18, 98304, 358400, 'NVMe', ARRAY['350 GB NVMe', '700 GB SSD'], 1000, 'Tráfico ilimitado con uso justo', 3, 'EU');

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