-- [124A-ESC] Persistir estado de escalación en chat_sessions.
-- is_escalated = true cuando la IA detectó que se necesita intervención humana.
-- DEFAULT false: sesiones existentes no tienen escalación activa.
ALTER TABLE chat_sessions
    ADD COLUMN IF NOT EXISTS is_escalated BOOLEAN NOT NULL DEFAULT false;
