ALTER TABLE hosting_plan_configs
    DROP CONSTRAINT IF EXISTS hosting_plan_configs_cpu_scaling_policy_check;

ALTER TABLE hosting_plan_configs
    DROP COLUMN IF EXISTS cpu_scaling_policy;