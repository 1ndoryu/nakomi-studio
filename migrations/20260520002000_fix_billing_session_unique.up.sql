-- [205A-2] Un stripe_session_id puede cubrir varios billing_items (checkout multi-item).
-- El índice UNIQUE original impide asignar la misma sesión a más de un item → error 500.
DROP INDEX IF EXISTS idx_billing_items_stripe_session;
CREATE INDEX idx_billing_items_stripe_session
ON billing_items(stripe_session_id)
WHERE stripe_session_id IS NOT NULL;
