-- [195A-1] Órdenes de dominio self-service con checkout Stripe.
CREATE TABLE domain_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    domain VARCHAR(253) NOT NULL,
    tld VARCHAR(32) NOT NULL,
    status VARCHAR(40) NOT NULL DEFAULT 'pending_payment',
    base_cost_cents INT NOT NULL,
    price_cents INT NOT NULL,
    stripe_session_id VARCHAR(255) UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_domain_orders_user_id ON domain_orders(user_id);
CREATE INDEX idx_domain_orders_status ON domain_orders(status);
CREATE INDEX idx_domain_orders_domain ON domain_orders(domain);
