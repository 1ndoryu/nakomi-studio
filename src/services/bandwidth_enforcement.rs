/* [225A-4] Enforcement mensual de bandwidth.
 * El sampler acumula deltas de Docker stats en `bandwidth_usage`; este loop usa
 * Coolify para suspender/restaurar servicios cuando cruzan su límite mensual. */

use std::time::Duration;

use sqlx::PgPool;

use crate::errors::AppError;
use crate::repositories::{
    BandwidthEnforcementCandidate, HostingRepository, InfrastructureRepository,
};
use crate::services::coolify::CoolifyConfig;
use crate::services::infrastructure::coolify_server_targets;
use crate::services::CoolifyService;

const BANDWIDTH_ENFORCEMENT_INTERVAL: Duration = Duration::from_hours(1);

fn find_config_for<'a>(
    candidate: &BandwidthEnforcementCandidate,
    vps1_config: Option<&'a CoolifyConfig>,
    default_config: Option<&'a CoolifyConfig>,
) -> Option<&'a CoolifyConfig> {
    coolify_server_targets(vps1_config, default_config)
        .into_iter()
        .find(|target| {
            candidate
                .server_ip
                .as_deref()
                .is_some_and(|server_ip| server_ip == target.config.server_ip)
        })
        .map(|target| target.config)
}

async fn suspend_over_limit(
    pool: &PgPool,
    http_client: &reqwest::Client,
    candidate: &BandwidthEnforcementCandidate,
    config: &CoolifyConfig,
) -> Result<(), AppError> {
    let Some(deployment_uuid) = candidate.deployment_uuid.as_deref() else {
        return Ok(());
    };
    CoolifyService::stop_service(http_client, config, deployment_uuid).await?;
    HostingRepository::update_status(pool, candidate.subscription_id, "suspended_bandwidth")
        .await?;
    HostingRepository::add_event(
        pool,
        candidate.subscription_id,
        "bandwidth_exceeded",
        Some(serde_json::json!({
            "used_gb": candidate.bandwidth_used_gb,
            "limit_gb": candidate.bandwidth_limit_gb,
            "action": "coolify_stop_service"
        })),
    )
    .await?;
    Ok(())
}

async fn restore_under_limit(
    pool: &PgPool,
    http_client: &reqwest::Client,
    candidate: &BandwidthEnforcementCandidate,
    config: &CoolifyConfig,
) -> Result<(), AppError> {
    let Some(deployment_uuid) = candidate.deployment_uuid.as_deref() else {
        return Ok(());
    };
    CoolifyService::start_service(http_client, config, deployment_uuid).await?;
    HostingRepository::update_status(pool, candidate.subscription_id, "active").await?;
    HostingRepository::add_event(
        pool,
        candidate.subscription_id,
        "bandwidth_restored",
        Some(serde_json::json!({
            "used_gb": candidate.bandwidth_used_gb,
            "limit_gb": candidate.bandwidth_limit_gb,
            "action": "coolify_start_service"
        })),
    )
    .await?;
    Ok(())
}

pub async fn enforce_bandwidth_once(
    pool: &PgPool,
    http_client: &reqwest::Client,
    vps1_config: Option<&CoolifyConfig>,
    default_config: Option<&CoolifyConfig>,
) -> Result<(), AppError> {
    let candidates = InfrastructureRepository::bandwidth_enforcement_candidates(pool).await?;
    for candidate in candidates {
        let over_limit = candidate.bandwidth_limit_gb >= 0
            && candidate.bandwidth_used_gb > f64::from(candidate.bandwidth_limit_gb);
        let should_restore = candidate.status == "suspended_bandwidth" && !over_limit;

        if !over_limit && !should_restore {
            continue;
        }

        let Some(config) = find_config_for(&candidate, vps1_config, default_config) else {
            tracing::warn!(
                "[bandwidth-enforcement] Sin Coolify config para {:?}",
                candidate.server_ip
            );
            continue;
        };

        let result = if over_limit && candidate.status != "suspended_bandwidth" {
            suspend_over_limit(pool, http_client, &candidate, config).await
        } else if should_restore {
            restore_under_limit(pool, http_client, &candidate, config).await
        } else {
            Ok(())
        };

        if let Err(error) = result {
            tracing::warn!(
                "[bandwidth-enforcement] Acción fallida para {}: {error}",
                candidate.subscription_id
            );
        }
    }
    Ok(())
}

pub async fn bandwidth_enforcement_loop(
    pool: PgPool,
    http_client: reqwest::Client,
    vps1_config: Option<CoolifyConfig>,
    default_config: Option<CoolifyConfig>,
) {
    loop {
        if let Err(error) = enforce_bandwidth_once(
            &pool,
            &http_client,
            vps1_config.as_ref(),
            default_config.as_ref(),
        )
        .await
        {
            tracing::warn!("[bandwidth-enforcement] ciclo incompleto: {error}");
        }
        tokio::time::sleep(BANDWIDTH_ENFORCEMENT_INTERVAL).await;
    }
}
