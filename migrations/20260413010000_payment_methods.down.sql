DROP INDEX IF EXISTS idx_user_payment_methods_user;
DROP INDEX IF EXISTS idx_user_payment_methods_default;
DROP INDEX IF EXISTS idx_user_payment_methods_fingerprint;

DROP TABLE IF EXISTS user_payment_methods;

ALTER TABLE users
DROP COLUMN IF EXISTS stripe_customer_id;