/* [225A-4] Recursos hosting: inventario persistente, muestras promediadas,
 * bandwidth mensual, capacidad de servidor y límites por usuario. */

CREATE TABLE infrastructure_servers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    label VARCHAR(120) NOT NULL,
    provider VARCHAR(40) NOT NULL DEFAULT 'coolify',
    provider_instance_id VARCHAR(80),
    server_ip VARCHAR(64) NOT NULL,
    coolify_base_url TEXT,
    coolify_server_uuid VARCHAR(120),
    coolify_project_uuid VARCHAR(120),
    secret_ref VARCHAR(120),
    ssh_secret_ref VARCHAR(120),
    status VARCHAR(40) NOT NULL DEFAULT 'configured',
    status_checked_at TIMESTAMPTZ,
    status_failures INT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(server_ip, coolify_base_url)
);

CREATE UNIQUE INDEX idx_infrastructure_servers_coolify_uuid
    ON infrastructure_servers(coolify_server_uuid)
    WHERE coolify_server_uuid IS NOT NULL;
CREATE INDEX idx_infrastructure_servers_active ON infrastructure_servers(is_active);

CREATE TABLE infrastructure_resource_samples (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_kind TEXT NOT NULL CHECK (entity_kind IN ('server', 'deployment')),
    server_id UUID NOT NULL REFERENCES infrastructure_servers(id) ON DELETE CASCADE,
    deployment_uuid TEXT,
    sampled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    cpu_percent DOUBLE PRECISION,
    ram_used_mb DOUBLE PRECISION,
    ram_limit_mb DOUBLE PRECISION,
    disk_used_mb DOUBLE PRECISION,
    disk_limit_mb DOUBLE PRECISION,
    source TEXT NOT NULL DEFAULT 'ssh_sampler'
);

CREATE INDEX idx_infra_samples_entity_server_time
    ON infrastructure_resource_samples(entity_kind, server_id, sampled_at DESC);
CREATE INDEX idx_infra_samples_deployment_time
    ON infrastructure_resource_samples(deployment_uuid, sampled_at DESC)
    WHERE deployment_uuid IS NOT NULL;

CREATE TABLE bandwidth_snapshots (
    subscription_id UUID NOT NULL REFERENCES hosting_subscriptions(id) ON DELETE CASCADE,
    deployment_uuid TEXT NOT NULL,
    server_id UUID NOT NULL REFERENCES infrastructure_servers(id) ON DELETE CASCADE,
    net_input_mb DOUBLE PRECISION NOT NULL,
    net_output_mb DOUBLE PRECISION NOT NULL,
    sampled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY(subscription_id, deployment_uuid)
);

CREATE TABLE bandwidth_usage (
    subscription_id UUID NOT NULL REFERENCES hosting_subscriptions(id) ON DELETE CASCADE,
    month_start DATE NOT NULL,
    bytes_rx BIGINT NOT NULL DEFAULT 0,
    bytes_tx BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY(subscription_id, month_start)
);

CREATE TABLE server_capacity (
    server_uuid VARCHAR(120) PRIMARY KEY,
    server_id UUID REFERENCES infrastructure_servers(id) ON DELETE SET NULL,
    cpu_cores NUMERIC NOT NULL DEFAULT 0,
    ram_mb INT NOT NULL DEFAULT 0,
    disk_mb INT NOT NULL DEFAULT 0,
    cpu_allocated NUMERIC NOT NULL DEFAULT 0,
    ram_allocated INT NOT NULL DEFAULT 0,
    disk_allocated INT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (cpu_cores >= 0 AND ram_mb >= 0 AND disk_mb >= 0),
    CHECK (cpu_allocated >= 0 AND ram_allocated >= 0 AND disk_allocated >= 0)
);

CREATE TABLE vps_monitor_state (
    subscription_id UUID PRIMARY KEY REFERENCES vps_subscriptions(id) ON DELETE CASCADE,
    provider_status VARCHAR(40),
    failure_count INT NOT NULL DEFAULT 0,
    checked_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE hosting_subscriptions
    ADD COLUMN IF NOT EXISTS bandwidth_limit_gb INT NOT NULL DEFAULT 50;

UPDATE hosting_subscriptions hs
SET bandwidth_limit_gb = hpc.bandwidth_limit_gb
FROM hosting_plan_configs hpc
WHERE hs.plan = hpc.plan_name;

ALTER TABLE hosting_plan_configs
    ADD COLUMN IF NOT EXISTS usage_alert_threshold_pct INT NOT NULL DEFAULT 80;

ALTER TABLE user_profiles
    ADD COLUMN IF NOT EXISTS max_active_subscriptions INT NOT NULL DEFAULT 20;

UPDATE user_profiles
SET max_active_subscriptions = 100
WHERE user_id IN (SELECT id FROM users WHERE role = 'admin');

ALTER TABLE user_profiles
    ALTER COLUMN max_active_subscriptions SET DEFAULT 5;

CREATE INDEX IF NOT EXISTS idx_hosting_subscriptions_user_status
    ON hosting_subscriptions(user_id, status)
    WHERE user_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_hosting_subscriptions_server_status
    ON hosting_subscriptions(server_uuid, status)
    WHERE server_uuid IS NOT NULL;