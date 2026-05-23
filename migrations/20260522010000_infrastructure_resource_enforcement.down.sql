DROP INDEX IF EXISTS idx_hosting_subscriptions_server_status;
DROP INDEX IF EXISTS idx_hosting_subscriptions_user_status;

ALTER TABLE user_profiles DROP COLUMN IF EXISTS max_active_subscriptions;
ALTER TABLE hosting_plan_configs DROP COLUMN IF EXISTS usage_alert_threshold_pct;
ALTER TABLE hosting_subscriptions DROP COLUMN IF EXISTS bandwidth_limit_gb;

DROP TABLE IF EXISTS server_capacity;
DROP TABLE IF EXISTS vps_monitor_state;
DROP TABLE IF EXISTS bandwidth_usage;
DROP TABLE IF EXISTS bandwidth_snapshots;
DROP TABLE IF EXISTS infrastructure_resource_samples;
DROP TABLE IF EXISTS infrastructure_servers;