/* [154A-15a] Wallet system: tablas para saldo virtual de usuarios.
 * user_wallets: un wallet por usuario con balance en cents.
 * wallet_transactions: auditoría de cada movimiento de saldo.
 * cancellation_requests: solicitudes de cancelación con flujo de aprobación. */

CREATE TABLE IF NOT EXISTS user_wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    balance_cents INT NOT NULL DEFAULT 0,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS wallet_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_id UUID NOT NULL REFERENCES user_wallets(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    amount_cents INT NOT NULL,
    transaction_type VARCHAR(30) NOT NULL,
    reference_type VARCHAR(30),
    reference_id UUID,
    description TEXT,
    balance_after_cents INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS cancellation_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    requested_by UUID NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

/* Orden: campo para ventana exclusiva admin 48h */
ALTER TABLE orders ADD COLUMN IF NOT EXISTS open_to_employees BOOLEAN NOT NULL DEFAULT false;

/* IA intermediaria desactivada por defecto en ordenes */
ALTER TABLE orders ALTER COLUMN ai_intermediary_enabled SET DEFAULT false;

CREATE INDEX IF NOT EXISTS idx_wallet_user ON user_wallets(user_id);
CREATE INDEX IF NOT EXISTS idx_wallet_tx_user ON wallet_transactions(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_wallet_tx_ref ON wallet_transactions(reference_type, reference_id);
CREATE INDEX IF NOT EXISTS idx_cancel_req_order ON cancellation_requests(order_id, status);
CREATE INDEX IF NOT EXISTS idx_orders_open_employees ON orders(open_to_employees) WHERE status = 'awaiting_assignment';
