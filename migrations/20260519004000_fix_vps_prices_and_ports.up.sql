-- [205B-1] Correcciones de datos VPS con precios oficiales Contabo.
-- Velocidades de puerto reales: VPS20=300Mbit/s, VPS30=600Mbit/s, VPS40=800Mbit/s.
-- Precios de storage/región corregidos con cifras oficiales de contabo.com.

-- ─── Port speeds ───
UPDATE vps_plan_configs SET port_speed_mbps = 300 WHERE tier_name = 'vps2';
UPDATE vps_plan_configs SET port_speed_mbps = 600 WHERE tier_name = 'vps3';
UPDATE vps_plan_configs SET port_speed_mbps = 800 WHERE tier_name = 'vps4';

-- ─── Storage options labels (usar notación TB igual que Contabo) ───
UPDATE vps_plan_configs SET
    storage_options = ARRAY['250 GB NVMe', '500 GB NVMe', '500 GB SSD', '1 TB SSD']
WHERE tier_name = 'vps4';

UPDATE vps_plan_configs SET
    storage_options = ARRAY['300 GB NVMe', '600 GB NVMe', '600 GB SSD', '1.2 TB SSD']
WHERE tier_name = 'vps5';

UPDATE vps_plan_configs SET
    storage_options = ARRAY['350 GB NVMe', '700 GB NVMe', '700 GB SSD', '1.4 TB SSD']
WHERE tier_name = 'vps6';

-- ─── Storage extra cents (precios oficiales Contabo) ───
UPDATE vps_plan_configs SET
    storage_extra_cents = '{"200 GB NVMe": 0, "400 GB NVMe": 325, "400 GB SSD": 0, "800 GB SSD": 385}'::jsonb
WHERE tier_name = 'vps3';

UPDATE vps_plan_configs SET
    storage_extra_cents = '{"250 GB NVMe": 0, "500 GB NVMe": 385, "500 GB SSD": 0, "1 TB SSD": 495}'::jsonb
WHERE tier_name = 'vps4';

UPDATE vps_plan_configs SET
    storage_extra_cents = '{"300 GB NVMe": 0, "600 GB NVMe": 475, "600 GB SSD": 0, "1.2 TB SSD": 615}'::jsonb
WHERE tier_name = 'vps5';

UPDATE vps_plan_configs SET
    storage_extra_cents = '{"350 GB NVMe": 0, "700 GB NVMe": 575, "700 GB SSD": 0, "1.4 TB SSD": 695}'::jsonb
WHERE tier_name = 'vps6';

-- ─── Region extra cents (UK oficial Contabo; US/Asia/AUS proporcional) ───
UPDATE vps_plan_configs SET
    region_extra_cents = '{"EU": 0, "UK": 295, "US-EAST": 600, "US-WEST": 600, "SIN": 900, "AUS": 900}'::jsonb
WHERE tier_name = 'vps3';

UPDATE vps_plan_configs SET
    region_extra_cents = '{"EU": 0, "UK": 525, "US-EAST": 1100, "US-WEST": 1100, "SIN": 1600, "AUS": 1600}'::jsonb
WHERE tier_name = 'vps4';

UPDATE vps_plan_configs SET
    region_extra_cents = '{"EU": 0, "UK": 780, "US-EAST": 1600, "US-WEST": 1600, "SIN": 2400, "AUS": 2400}'::jsonb
WHERE tier_name = 'vps5';

UPDATE vps_plan_configs SET
    region_extra_cents = '{"EU": 0, "UK": 1030, "US-EAST": 2100, "US-WEST": 2100, "SIN": 3200, "AUS": 3200}'::jsonb
WHERE tier_name = 'vps6';
