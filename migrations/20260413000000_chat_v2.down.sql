/* [P-2 Chatbot v2] Rollback */

ALTER TABLE orders DROP COLUMN IF EXISTS ai_summary;
ALTER TABLE orders DROP COLUMN IF EXISTS ai_intermediary_enabled;

ALTER TABLE chat_messages DROP COLUMN IF EXISTS metadata;
ALTER TABLE chat_messages DROP COLUMN IF EXISTS message_type;

DROP TABLE IF EXISTS chat_attachments;
DROP TABLE IF EXISTS visitor_profiles;
