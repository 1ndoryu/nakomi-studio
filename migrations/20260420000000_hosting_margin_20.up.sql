-- [164A-17] Ajuste operativo: hosting compartido ahora apunta a un margen bruto del 20%.
-- El precio final se calcula sobre la densidad conservadora del VPS2 actual para evitar
-- vender por debajo del costo cuando todavía no existe autoscaling por nodo.
UPDATE hosting_plan_configs
SET monthly_price_cents = CASE plan_name
    WHEN 'basico' THEN 248
    WHEN 'pro' THEN 413
    WHEN 'ecommerce' THEN 619
    ELSE monthly_price_cents
END,
updated_at = NOW()
WHERE plan_name IN ('basico', 'pro', 'ecommerce');