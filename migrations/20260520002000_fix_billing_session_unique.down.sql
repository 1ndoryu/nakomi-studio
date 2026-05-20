-- Revert: restaurar índice único en stripe_session_id
DROP INDEX IF EXISTS idx_billing_items_stripe_session;
CREATE UNIQUE INDEX idx_billing_items_stripe_session
ON billing_items(stripe_session_id)
WHERE stripe_session_id IS NOT NULL;
