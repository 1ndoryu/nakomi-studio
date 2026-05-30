DROP TABLE IF EXISTS hosting_email_aliases;
DROP TABLE IF EXISTS hosting_email_mailboxes;
ALTER TABLE hosting_plan_configs DROP COLUMN IF EXISTS included_aliases;
ALTER TABLE hosting_plan_configs DROP COLUMN IF EXISTS included_mailboxes;
