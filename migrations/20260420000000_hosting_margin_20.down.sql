UPDATE hosting_plan_configs
SET monthly_price_cents = CASE plan_name
    WHEN 'basico' THEN 500
    WHEN 'pro' THEN 1000
    WHEN 'ecommerce' THEN 1500
    ELSE monthly_price_cents
END,
updated_at = NOW()
WHERE plan_name IN ('basico', 'pro', 'ecommerce');