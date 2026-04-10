DROP INDEX IF EXISTS idx_orders_open_employees;
DROP INDEX IF EXISTS idx_cancel_req_order;
DROP INDEX IF EXISTS idx_wallet_tx_ref;
DROP INDEX IF EXISTS idx_wallet_tx_user;
DROP INDEX IF EXISTS idx_wallet_user;

ALTER TABLE orders ALTER COLUMN ai_intermediary_enabled SET DEFAULT NULL;
ALTER TABLE orders DROP COLUMN IF EXISTS open_to_employees;

DROP TABLE IF EXISTS cancellation_requests;
DROP TABLE IF EXISTS wallet_transactions;
DROP TABLE IF EXISTS user_wallets;
