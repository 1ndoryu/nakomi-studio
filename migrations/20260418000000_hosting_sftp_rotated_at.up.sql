-- [114A-1] Track credential rotation timestamps for security auditing
ALTER TABLE hosting_subscriptions
ADD COLUMN sftp_credentials_rotated_at TIMESTAMPTZ;
