/* [104A-29] Eliminar pending_payment como estado inicial de órdenes.
 * Las órdenes ahora inician en payment_held (el pago se inicia al crear la orden).
 * pending_payment solo permanece en phase_status para pagos inter-fase. */

/* Migrar órdenes existentes con pending_payment a payment_held */
UPDATE orders SET status = 'payment_held' WHERE status = 'pending_payment';

/* Cambiar DEFAULT de la columna status */
ALTER TABLE orders ALTER COLUMN status SET DEFAULT 'payment_held';
