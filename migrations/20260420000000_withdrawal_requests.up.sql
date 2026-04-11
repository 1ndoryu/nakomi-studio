/* [184A-1] Withdrawal requests: tabla para solicitudes de retiro de saldo.
 * Los usuarios pueden solicitar retirar fondos de su wallet.
 * El admin aprueba o rechaza; al aprobar se debita del wallet. */

CREATE TABLE IF NOT EXISTS withdrawal_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    amount_cents INT NOT NULL CHECK (amount_cents > 0),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    payment_method VARCHAR(100),
    payment_details TEXT,
    admin_notes TEXT,
    resolved_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_withdrawal_user ON withdrawal_requests(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_withdrawal_status ON withdrawal_requests(status) WHERE status = 'pending';
