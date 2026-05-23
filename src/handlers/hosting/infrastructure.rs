use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{DeploymentMetricsResponse, ResourceUsageReportItem, UserRole};
use crate::repositories::InfrastructureRepository;
use crate::services::infrastructure::coolify_server_targets;
use crate::services::infrastructure_metrics::sample_infrastructure_once;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct MetricsQuery {
    range: Option<String>,
}

fn range_to_hours(range: Option<&str>) -> i64 {
    match range.unwrap_or("24h") {
        "1h" => 1,
        "6h" => 6,
        "7d" => 168,
        _ => 24,
    }
}

pub(super) async fn list_infrastructure_servers(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    for target in coolify_server_targets(
        state.coolify_config_vps1.as_ref(),
        state.coolify_config.as_ref(),
    ) {
        InfrastructureRepository::upsert_configured_server(
            &state.pool,
            crate::repositories::ConfiguredServerInput {
                label: &target.label,
                config: target.config,
                secret_ref: if target.label.contains("VPS1") || target.label.contains("Principal") {
                    "COOLIFY_VPS1_API_TOKEN"
                } else {
                    "COOLIFY_API_TOKEN"
                },
                ssh_secret_ref: target.config.ssh_key_path.as_ref().map(|_| {
                    if target.label.contains("VPS1") || target.label.contains("Principal") {
                        "COOLIFY_VPS1_SSH_KEY_PATH"
                    } else {
                        "COOLIFY_SSH_KEY_PATH"
                    }
                }),
            },
        )
        .await?;
    }

    let servers = InfrastructureRepository::list_servers_with_metrics(&state.pool).await?;
    Ok(Json(serde_json::json!({ "data": servers })))
}

pub(super) async fn refresh_infrastructure_metrics(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_role(&[UserRole::Admin])?;
    sample_infrastructure_once(
        state.pool.clone(),
        state.http_client.clone(),
        state.coolify_config_vps1.clone(),
        state.coolify_config.clone(),
    )
    .await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub(super) async fn deployment_metrics(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(deployment_uuid): Path<String>,
    Query(query): Query<MetricsQuery>,
) -> Result<Json<DeploymentMetricsResponse>, AppError> {
    auth.require_role(&[UserRole::Admin])?;
    let range = query.range.unwrap_or_else(|| "24h".to_string());
    let points = InfrastructureRepository::deployment_metric_points(
        &state.pool,
        &deployment_uuid,
        range_to_hours(Some(&range)),
    )
    .await?;

    Ok(Json(DeploymentMetricsResponse {
        deployment_uuid,
        range,
        points,
    }))
}

pub(super) async fn resource_usage_report(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<ResourceUsageReportItem>>, AppError> {
    auth.require_role(&[UserRole::Admin])?;
    let report = InfrastructureRepository::resource_usage_report(&state.pool).await?;
    Ok(Json(report))
}
