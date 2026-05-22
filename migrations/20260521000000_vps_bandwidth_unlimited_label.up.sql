-- [215A-4] El catálogo VPS visible debe decir solo "Tráfico ilimitado".
UPDATE vps_plan_configs
SET bandwidth_label = 'Tráfico ilimitado',
    updated_at = NOW()
WHERE bandwidth_label = 'Tráfico ilimitado con uso justo';