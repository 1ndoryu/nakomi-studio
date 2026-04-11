/* [114A-3] Tabla de configuración de recursos por plan de hosting.
 * Centraliza precios, límites de CPU/RAM/almacenamiento/bandwidth por plan.
 * Millicores: 1000 = 1.0 CPU (ej: 500 = 0.50 cores).
 * Admin puede modificar desde el panel; los hostings usan estos valores al provisionar. */
CREATE TABLE hosting_plan_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_name VARCHAR(20) UNIQUE NOT NULL,
    monthly_price_cents INT NOT NULL,
    wp_cpu_millicores INT NOT NULL DEFAULT 1000,
    wp_memory_mb INT NOT NULL DEFAULT 512,
    db_cpu_millicores INT NOT NULL DEFAULT 500,
    db_memory_mb INT NOT NULL DEFAULT 512,
    ssh_cpu_millicores INT NOT NULL DEFAULT 500,
    ssh_memory_mb INT NOT NULL DEFAULT 256,
    storage_limit_mb INT NOT NULL DEFAULT 5120,
    bandwidth_limit_gb INT NOT NULL DEFAULT 50,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

/* Seed: valores por defecto coherentes con los límites actuales hardcodeados.
 * basico: ligero (0.50/0.25/0.25 CPU). pro: medio (1.0/0.50/0.50). ecommerce: alto (1.5/0.75/0.50). */
INSERT INTO hosting_plan_configs
    (plan_name, monthly_price_cents, wp_cpu_millicores, wp_memory_mb, db_cpu_millicores, db_memory_mb, ssh_cpu_millicores, ssh_memory_mb, storage_limit_mb, bandwidth_limit_gb)
VALUES
    ('basico',    500,  500, 256, 250, 256, 250, 128,  5120,  50),
    ('pro',      1000, 1000, 512, 500, 512, 500, 256, 20480, 200),
    ('ecommerce',1500, 1500,1024, 750, 512, 500, 256, 51200, 500);
