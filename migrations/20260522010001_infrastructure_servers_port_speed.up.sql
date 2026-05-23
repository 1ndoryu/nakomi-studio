ALTER TABLE infrastructure_servers
    ADD COLUMN IF NOT EXISTS port_speed_mbps INT NOT NULL DEFAULT 200;
