-- Revert fix: restore previous port speeds and pricing estimates
UPDATE vps_plan_configs SET port_speed_mbps = 1000 WHERE tier_name IN ('vps2', 'vps3', 'vps4');
