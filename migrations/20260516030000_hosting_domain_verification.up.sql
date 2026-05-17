ALTER TABLE hosting_subscriptions
ADD COLUMN domain_verification_status VARCHAR(32) NOT NULL DEFAULT 'none',
ADD COLUMN domain_verification_token VARCHAR(120),
ADD COLUMN domain_verified_at TIMESTAMPTZ;

UPDATE hosting_subscriptions
SET domain_verification_status = CASE
        WHEN domain IS NOT NULL AND btrim(domain) <> '' THEN 'active'
        ELSE 'none'
    END,
    domain_verified_at = CASE
        WHEN domain IS NOT NULL AND btrim(domain) <> '' THEN COALESCE(updated_at, NOW())
        ELSE NULL
    END
WHERE domain_verification_status = 'none';