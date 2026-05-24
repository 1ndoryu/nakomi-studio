DROP INDEX IF EXISTS idx_hosting_subscriptions_runtime_status;
DROP INDEX IF EXISTS idx_hosting_subscriptions_runtime_deployment_unique;

ALTER TABLE hosting_subscriptions
    DROP COLUMN IF EXISTS deployment_id,
    DROP COLUMN IF EXISTS runtime_kind;