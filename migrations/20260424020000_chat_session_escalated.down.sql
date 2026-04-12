-- [124A-ESC] Revertir campo is_escalated.
ALTER TABLE chat_sessions
    DROP COLUMN IF EXISTS is_escalated;
