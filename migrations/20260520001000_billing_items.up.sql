-- [205A-1] Cobros pendientes independientes del estado tecnico de hosting/dominios.
-- Permite mostrar y cobrar servicios legacy sin suspender ni reprovisionar hostings reales.
CREATE TABLE billing_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    resource_type VARCHAR(32) NOT NULL CHECK (resource_type IN ('hosting', 'domain', 'other')),
    resource_id UUID,
    title VARCHAR(160) NOT NULL,
    description TEXT,
    amount_cents INT NOT NULL CHECK (amount_cents > 0),
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    billing_period VARCHAR(20) NOT NULL CHECK (billing_period IN ('month', 'year', 'one_time')),
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'paid', 'cancelled')),
    due_at TIMESTAMPTZ NOT NULL,
    grace_period_ends_at TIMESTAMPTZ NOT NULL,
    paid_at TIMESTAMPTZ,
    stripe_session_id VARCHAR(255),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_billing_items_stripe_session
ON billing_items(stripe_session_id)
WHERE stripe_session_id IS NOT NULL;

CREATE INDEX idx_billing_items_user_status_due
ON billing_items(user_id, status, due_at);

CREATE INDEX idx_billing_items_resource
ON billing_items(resource_type, resource_id)
WHERE resource_id IS NOT NULL;