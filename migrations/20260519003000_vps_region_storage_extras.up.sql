-- [205A-1] Añade costos variables de región y storage al catálogo VPS.
-- Ahora el precio total en Stripe incluye automáticamente el extra de región y storage elegidos.
-- Precios de región VPS10/20: Contabo oficial. Resto: estimación proporcional revisable.
-- Precios de storage extra VPS30+: estimación proporcional; actualizar cuando Contabo los confirme.
ALTER TABLE vps_plan_configs
    ADD COLUMN IF NOT EXISTS region_extra_cents JSONB NOT NULL DEFAULT '{}',
    ADD COLUMN IF NOT EXISTS storage_extra_cents JSONB NOT NULL DEFAULT '{}';

-- Actualizar storage_options para incluir todas las variantes (base gratis + doblada de pago)
-- y poblar los mapas de costos extra.
UPDATE vps_plan_configs SET
    storage_options      = ARRAY['75 GB NVMe', '150 GB NVMe', '150 GB SSD', '300 GB SSD'],
    storage_extra_cents  = '{"75 GB NVMe": 0, "150 GB NVMe": 185, "150 GB SSD": 0, "300 GB SSD": 155}'::jsonb,
    region_extra_cents   = '{"EU": 0, "UK": 95, "US-EAST": 200, "US-WEST": 200, "SIN": 300, "AUS": 300}'::jsonb
WHERE tier_name = 'vps1';

UPDATE vps_plan_configs SET
    storage_options      = ARRAY['100 GB NVMe', '200 GB NVMe', '200 GB SSD', '400 GB SSD'],
    storage_extra_cents  = '{"100 GB NVMe": 0, "200 GB NVMe": 255, "200 GB SSD": 0, "400 GB SSD": 195}'::jsonb,
    region_extra_cents   = '{"EU": 0, "UK": 145, "US-EAST": 300, "US-WEST": 300, "SIN": 450, "AUS": 450}'::jsonb
WHERE tier_name = 'vps2';

UPDATE vps_plan_configs SET
    storage_options      = ARRAY['200 GB NVMe', '400 GB NVMe', '400 GB SSD', '800 GB SSD'],
    storage_extra_cents  = '{"200 GB NVMe": 0, "400 GB NVMe": 350, "400 GB SSD": 0, "800 GB SSD": 250}'::jsonb,
    region_extra_cents   = '{"EU": 0, "UK": 195, "US-EAST": 400, "US-WEST": 400, "SIN": 600, "AUS": 600}'::jsonb
WHERE tier_name = 'vps3';

UPDATE vps_plan_configs SET
    storage_options      = ARRAY['250 GB NVMe', '500 GB NVMe', '500 GB SSD', '1 TB SSD'],
    storage_extra_cents  = '{"250 GB NVMe": 0, "500 GB NVMe": 450, "500 GB SSD": 0, "1 TB SSD": 300}'::jsonb,
    region_extra_cents   = '{"EU": 0, "UK": 295, "US-EAST": 600, "US-WEST": 600, "SIN": 900, "AUS": 900}'::jsonb
WHERE tier_name = 'vps4';

UPDATE vps_plan_configs SET
    storage_options      = ARRAY['300 GB NVMe', '600 GB NVMe', '600 GB SSD', '1.2 TB SSD'],
    storage_extra_cents  = '{"300 GB NVMe": 0, "600 GB NVMe": 550, "600 GB SSD": 0, "1.2 TB SSD": 350}'::jsonb,
    region_extra_cents   = '{"EU": 0, "UK": 395, "US-EAST": 800, "US-WEST": 800, "SIN": 1200, "AUS": 1200}'::jsonb
WHERE tier_name = 'vps5';

UPDATE vps_plan_configs SET
    storage_options      = ARRAY['350 GB NVMe', '700 GB NVMe', '700 GB SSD', '1.4 TB SSD'],
    storage_extra_cents  = '{"350 GB NVMe": 0, "700 GB NVMe": 650, "700 GB SSD": 0, "1.4 TB SSD": 400}'::jsonb,
    region_extra_cents   = '{"EU": 0, "UK": 495, "US-EAST": 1000, "US-WEST": 1000, "SIN": 1500, "AUS": 1500}'::jsonb
WHERE tier_name = 'vps6';
