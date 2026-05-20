-- Reversa de 20260420001001_vps_tables_repair.
-- Solo aplica en fresh DB (IF EXISTS = no-op en producción donde las tablas vienen de 20260420001000).
DROP TABLE IF EXISTS vps_events;
DROP TABLE IF EXISTS vps_subscriptions;
DROP TABLE IF EXISTS vps_plan_configs;
