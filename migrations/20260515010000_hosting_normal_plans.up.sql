INSERT INTO hosting_plan_configs (
    plan_name,
    monthly_price_cents,
    wp_cpu_millicores,
    wp_memory_mb,
    db_cpu_millicores,
    db_memory_mb,
    ssh_cpu_millicores,
    ssh_memory_mb,
    storage_limit_mb,
    bandwidth_limit_gb
)
SELECT
    'normal-' || plan_name,
    CEIL(monthly_price_cents::numeric * 1.30)::integer,
    wp_cpu_millicores,
    wp_memory_mb,
    db_cpu_millicores,
    db_memory_mb,
    ssh_cpu_millicores,
    ssh_memory_mb,
    storage_limit_mb,
    bandwidth_limit_gb
FROM hosting_plan_configs
WHERE plan_name IN ('basico', 'pro', 'ecommerce')
ON CONFLICT (plan_name) DO UPDATE SET
    monthly_price_cents = EXCLUDED.monthly_price_cents,
    wp_cpu_millicores = EXCLUDED.wp_cpu_millicores,
    wp_memory_mb = EXCLUDED.wp_memory_mb,
    db_cpu_millicores = EXCLUDED.db_cpu_millicores,
    db_memory_mb = EXCLUDED.db_memory_mb,
    ssh_cpu_millicores = EXCLUDED.ssh_cpu_millicores,
    ssh_memory_mb = EXCLUDED.ssh_memory_mb,
    storage_limit_mb = EXCLUDED.storage_limit_mb,
    bandwidth_limit_gb = EXCLUDED.bandwidth_limit_gb,
    updated_at = NOW();