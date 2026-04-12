-- [124A-PAIS] Revertir campo visitor_country
ALTER TABLE chat_sessions DROP COLUMN IF EXISTS visitor_country;
