/* [265A-5] Politica CPU por plan.
 * `contention_throttle` pasa a ser el default comercial: los hostings quedan
 * sin cap fuera de contencion y solo reciben un limite temporal cuando la VPS
 * entra en presion alta sostenida. `baseline_burst` conserva el modelo previo
 * para planes o clientes que requieran baseline fijo. */
ALTER TABLE hosting_plan_configs
    ADD COLUMN cpu_scaling_policy VARCHAR(32) NOT NULL DEFAULT 'contention_throttle';

ALTER TABLE hosting_plan_configs
    ADD CONSTRAINT hosting_plan_configs_cpu_scaling_policy_check
    CHECK (cpu_scaling_policy IN ('baseline_burst', 'contention_throttle'));