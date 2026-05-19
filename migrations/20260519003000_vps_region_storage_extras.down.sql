-- [205A-1] Revert: eliminar columnas de costos extra y restaurar storage_options originales.
ALTER TABLE vps_plan_configs
    DROP COLUMN IF EXISTS region_extra_cents,
    DROP COLUMN IF EXISTS storage_extra_cents;

UPDATE vps_plan_configs SET storage_options = ARRAY['75 GB NVMe', '150 GB SSD']    WHERE tier_name = 'vps1';
UPDATE vps_plan_configs SET storage_options = ARRAY['100 GB NVMe', '200 GB SSD']   WHERE tier_name = 'vps2';
UPDATE vps_plan_configs SET storage_options = ARRAY['200 GB NVMe', '400 GB SSD']   WHERE tier_name = 'vps3';
UPDATE vps_plan_configs SET storage_options = ARRAY['250 GB NVMe', '500 GB SSD']   WHERE tier_name = 'vps4';
UPDATE vps_plan_configs SET storage_options = ARRAY['300 GB NVMe', '600 GB SSD']   WHERE tier_name = 'vps5';
UPDATE vps_plan_configs SET storage_options = ARRAY['350 GB NVMe', '700 GB SSD']   WHERE tier_name = 'vps6';
