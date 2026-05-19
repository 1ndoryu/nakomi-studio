-- [195A-1] Actualiza catálogo VPS a precios Contabo visibles + margen operativo 5%.
-- Se añade metadata pública para que el configurador muestre setup fee, storage, velocidad y tráfico sin texto hardcodeado.
ALTER TABLE vps_plan_configs
    ADD COLUMN IF NOT EXISTS setup_fee_cents INT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS storage_type VARCHAR(20) NOT NULL DEFAULT 'NVMe',
    ADD COLUMN IF NOT EXISTS storage_options TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    ADD COLUMN IF NOT EXISTS port_speed_mbps INT NOT NULL DEFAULT 200,
    ADD COLUMN IF NOT EXISTS bandwidth_label VARCHAR(120) NOT NULL DEFAULT 'Tráfico ilimitado con uso justo',
    ADD COLUMN IF NOT EXISTS snapshot_count INT NOT NULL DEFAULT 1;

INSERT INTO vps_plan_configs
    (tier_name, display_name, description, contabo_product_id, base_cost_cents, monthly_price_cents, setup_fee_cents, cpu_cores, ram_mb, disk_mb, storage_type, storage_options, port_speed_mbps, bandwidth_label, snapshot_count, region, is_active, approval_required)
VALUES
    ('vps1', 'Cloud VPS 10', 'Entrada dedicada para automatizaciones, paneles internos y servicios pequeños con acceso root.', 'V91', 450, 473, 450, 4, 8192, 76800, 'NVMe', ARRAY['75 GB NVMe', '150 GB SSD'], 200, 'Tráfico ilimitado con uso justo', 1, 'EU', TRUE, TRUE),
    ('vps2', 'Cloud VPS 20', 'Servidor balanceado para APIs, SaaS liviano y cargas sostenidas con más memoria.', 'V92', 840, 882, 0, 6, 12288, 102400, 'NVMe', ARRAY['100 GB NVMe', '200 GB SSD'], 1000, 'Tráfico ilimitado con uso justo', 2, 'EU', TRUE, TRUE),
    ('vps3', 'VPS 30', 'Nodo dedicado para workloads medianos, workers concurrentes y aplicaciones con tráfico constante.', 'V93', 1680, 1764, 0, 8, 24576, 204800, 'NVMe', ARRAY['200 GB NVMe', '400 GB SSD'], 1000, 'Tráfico ilimitado con uso justo', 3, 'EU', TRUE, TRUE),
    ('vps4', 'VPS 40', 'Capacidad dedicada para pipelines pesados, bases de datos exigentes y servicios con mucha memoria.', 'V94', 3000, 3150, 0, 12, 49152, 256000, 'NVMe', ARRAY['250 GB NVMe', '500 GB SSD'], 1000, 'Tráfico ilimitado con uso justo', 3, 'EU', TRUE, TRUE),
    ('vps5', 'VPS 50', 'Servidor de alta memoria para cargas intensivas, colas y stacks con varios servicios.', 'V95', 4450, 4673, 0, 16, 65536, 307200, 'NVMe', ARRAY['300 GB NVMe', '600 GB SSD'], 1000, 'Tráfico ilimitado con uso justo', 3, 'EU', TRUE, TRUE),
    ('vps6', 'VPS 60', 'Capacidad máxima del catálogo VPS para workloads pesados y crecimiento sostenido.', 'V96', 5880, 6174, 0, 18, 98304, 358400, 'NVMe', ARRAY['350 GB NVMe', '700 GB SSD'], 1000, 'Tráfico ilimitado con uso justo', 3, 'EU', TRUE, TRUE)
ON CONFLICT (tier_name) DO UPDATE SET
    display_name = EXCLUDED.display_name,
    description = EXCLUDED.description,
    contabo_product_id = EXCLUDED.contabo_product_id,
    base_cost_cents = EXCLUDED.base_cost_cents,
    monthly_price_cents = EXCLUDED.monthly_price_cents,
    setup_fee_cents = EXCLUDED.setup_fee_cents,
    cpu_cores = EXCLUDED.cpu_cores,
    ram_mb = EXCLUDED.ram_mb,
    disk_mb = EXCLUDED.disk_mb,
    storage_type = EXCLUDED.storage_type,
    storage_options = EXCLUDED.storage_options,
    port_speed_mbps = EXCLUDED.port_speed_mbps,
    bandwidth_label = EXCLUDED.bandwidth_label,
    snapshot_count = EXCLUDED.snapshot_count,
    region = EXCLUDED.region,
    is_active = EXCLUDED.is_active,
    approval_required = EXCLUDED.approval_required,
    updated_at = NOW();