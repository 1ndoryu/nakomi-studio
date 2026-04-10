-- [104A-40] Tiempo real staff: último tiempo de conexión del visitante.
-- Permite mostrar en el panel "En línea" o "Última conexión: hace Xm".
ALTER TABLE chat_sessions ADD COLUMN IF NOT EXISTS visitor_last_connected_at TIMESTAMPTZ;
