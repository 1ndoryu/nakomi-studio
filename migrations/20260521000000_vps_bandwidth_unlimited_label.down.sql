-- [215A-4] Reversión del copy anterior de VPS.
UPDATE vps_plan_configs
SET bandwidth_label = 'Tráfico ilimitado con uso justo',
    updated_at = NOW()
WHERE bandwidth_label = 'Tráfico ilimitado';