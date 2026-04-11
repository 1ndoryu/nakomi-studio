ALTER TABLE hosting_subscriptions
    DROP COLUMN IF EXISTS sftp_user,
    DROP COLUMN IF EXISTS sftp_password;
