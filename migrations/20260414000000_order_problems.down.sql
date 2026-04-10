ALTER TABLE orders DROP COLUMN IF EXISTS cancel_reason;
DROP TABLE IF EXISTS order_problems;
DROP TYPE IF EXISTS problem_status;
