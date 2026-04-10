/* Revertir: restaurar pending_payment como estado inicial de órdenes */
UPDATE orders SET status = 'pending_payment' WHERE status = 'payment_held'
    AND started_at IS NULL AND assigned_employee_id IS NULL;
ALTER TABLE orders ALTER COLUMN status SET DEFAULT 'pending_payment';
