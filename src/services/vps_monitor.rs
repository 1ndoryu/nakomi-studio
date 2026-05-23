/* [225A-4] Monitor básico de VPS de proveedor.
 * Contabo se usa solo para estado/proveedor; CPU/RAM/disco usados salen del
 * sampler SSH de infraestructura para evitar depender de métricas no expuestas. */

use std::time::Duration;

use sqlx::PgPool;

use crate::errors::AppError;
use crate::repositories::{InfrastructureRepository, VpsRepository};
use crate::services::ContaboService;

const VPS_MONITOR_INTERVAL: Duration = Duration::from_hours(1);
const VPS_FAILURE_THRESHOLD: i32 = 3;

pub async fn monitor_vps_once(
    pool: &PgPool,
    contabo_service: &ContaboService,
) -> Result<(), AppError> {
    let subscriptions = VpsRepository::list_all(pool).await?;
    for subscription in subscriptions {
        let Some(instance_id) = subscription.contabo_instance_id else {
            continue;
        };
        if !matches!(
            subscription.status.as_str(),
            "active" | "payment_overdue" | "provider_attention"
        ) {
            continue;
        }

        match contabo_service.get_instance(instance_id).await {
            Ok(instance) => {
                InfrastructureRepository::upsert_vps_monitor_state(
                    pool,
                    subscription.id,
                    Some(&instance.status),
                    0,
                )
                .await?;
                if instance.status != "running" && subscription.status != "provider_attention" {
                    VpsRepository::update_status(pool, subscription.id, "provider_attention")
                        .await?;
                    VpsRepository::add_event(
                        pool,
                        subscription.id,
                        "provider_status_warning",
                        Some(serde_json::json!({
                            "instance_id": instance_id,
                            "provider_status": instance.status,
                        })),
                    )
                    .await?;
                } else if instance.status == "running"
                    && subscription.status == "provider_attention"
                {
                    VpsRepository::update_status(pool, subscription.id, "active").await?;
                    VpsRepository::add_event(
                        pool,
                        subscription.id,
                        "provider_status_restored",
                        Some(serde_json::json!({"instance_id": instance_id})),
                    )
                    .await?;
                }
            }
            Err(error) => {
                let failures =
                    InfrastructureRepository::vps_monitor_failure_count(pool, subscription.id)
                        .await?
                        .saturating_add(1);
                InfrastructureRepository::upsert_vps_monitor_state(
                    pool,
                    subscription.id,
                    None,
                    failures,
                )
                .await?;
                if failures >= VPS_FAILURE_THRESHOLD {
                    VpsRepository::add_event(
                        pool,
                        subscription.id,
                        "provider_monitor_unavailable",
                        Some(serde_json::json!({"instance_id": instance_id, "error": error})),
                    )
                    .await?;
                }
            }
        }
    }
    Ok(())
}

pub async fn vps_monitor_loop(pool: PgPool, contabo_service: ContaboService) {
    loop {
        if let Err(error) = monitor_vps_once(&pool, &contabo_service).await {
            tracing::warn!("[vps-monitor] ciclo incompleto: {error}");
        }
        tokio::time::sleep(VPS_MONITOR_INTERVAL).await;
    }
}
