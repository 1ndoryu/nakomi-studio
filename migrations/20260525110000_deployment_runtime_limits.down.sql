ALTER TABLE infrastructure_resource_samples
    DROP COLUMN IF EXISTS ssh_ram_limit_mb,
    DROP COLUMN IF EXISTS ssh_cpu_limit_cores,
    DROP COLUMN IF EXISTS db_ram_limit_mb,
    DROP COLUMN IF EXISTS db_cpu_limit_cores,
    DROP COLUMN IF EXISTS site_ram_limit_mb,
    DROP COLUMN IF EXISTS site_cpu_limit_cores;