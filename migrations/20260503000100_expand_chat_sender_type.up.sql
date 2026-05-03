-- [035A-12] IA intermediaria de órdenes usa sender_type = 'ai_intermediary'.
-- La columna original VARCHAR(10) truncaba/fallaba al persistir ese valor.

ALTER TABLE chat_messages
ALTER COLUMN sender_type TYPE VARCHAR(32);