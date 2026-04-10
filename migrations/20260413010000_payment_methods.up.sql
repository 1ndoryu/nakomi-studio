ALTER TABLE users
ADD COLUMN IF NOT EXISTS stripe_customer_id VARCHAR(100);

CREATE TABLE user_payment_methods (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    stripe_payment_method_id VARCHAR(100) NOT NULL UNIQUE,
    card_fingerprint VARCHAR(255) NOT NULL,
    brand VARCHAR(50) NOT NULL,
    last_four VARCHAR(4) NOT NULL,
    exp_month INT NOT NULL CHECK (exp_month BETWEEN 1 AND 12),
    exp_year INT NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_user_payment_methods_fingerprint
ON user_payment_methods(user_id, card_fingerprint);

CREATE UNIQUE INDEX idx_user_payment_methods_default
ON user_payment_methods(user_id)
WHERE is_default = TRUE;

CREATE INDEX idx_user_payment_methods_user
ON user_payment_methods(user_id, created_at DESC);