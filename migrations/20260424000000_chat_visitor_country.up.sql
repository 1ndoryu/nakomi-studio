-- [124A-PAIS] Agrega campo visitor_country a chat_sessions para mostrar país en el panel de info.
ALTER TABLE chat_sessions ADD COLUMN IF NOT EXISTS visitor_country VARCHAR(100);
