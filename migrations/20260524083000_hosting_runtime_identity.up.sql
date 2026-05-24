-- [245A-6] Persistir la identidad de runtime para desacoplar control y borrado
-- de los campos legacy específicos de Coolify.
ALTER TABLE hosting_subscriptions
    ADD COLUMN IF NOT EXISTS runtime_kind TEXT NOT NULL DEFAULT 'coolify',
    ADD COLUMN IF NOT EXISTS deployment_id TEXT;

UPDATE hosting_subscriptions
SET runtime_kind = 'coolify'
WHERE runtime_kind IS DISTINCT FROM 'coolify';

UPDATE hosting_subscriptions
SET deployment_id = server_uuid
WHERE deployment_id IS NULL
  AND server_uuid IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_hosting_subscriptions_runtime_deployment_unique
    ON hosting_subscriptions(runtime_kind, deployment_id)
    WHERE deployment_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_hosting_subscriptions_runtime_status
    ON hosting_subscriptions(runtime_kind, status);