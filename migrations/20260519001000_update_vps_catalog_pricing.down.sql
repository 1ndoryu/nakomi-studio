-- [195A-1] Reversión conservadora: deja columnas porque el código nuevo las consume.
UPDATE vps_plan_configs
SET is_active = FALSE,
    updated_at = NOW()
WHERE tier_name IN ('vps5', 'vps6');