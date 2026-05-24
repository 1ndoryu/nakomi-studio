use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use std::collections::HashMap;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{CoolifyDeploymentResponse, UserRole};
use crate::repositories::{HostingRepository, InfrastructureRepository};
use crate::services::infrastructure::coolify_server_targets;
use crate::services::{
    CoolifyConfig, HostingRuntimeDeploymentSummary, HostingRuntimeKind, HostingRuntimeService,
};
use crate::AppState;

fn map_runtime_deployments(
    services: Vec<HostingRuntimeDeploymentSummary>,
    label: &str,
    subscriptions_by_uuid: &HashMap<&str, &crate::models::HostingSubscription>,
    subscriptions_by_name: &HashMap<&str, &crate::models::HostingSubscription>,
) -> Vec<CoolifyDeploymentResponse> {
    services
        .into_iter()
        .map(|service| {
            let linked_subscription = subscriptions_by_uuid
                .get(service.deployment_id.as_str())
                .copied()
                .or_else(|| subscriptions_by_name.get(service.name.as_str()).copied());

            CoolifyDeploymentResponse {
                uuid: service.deployment_id.clone(),
                runtime_kind: service.runtime_kind.as_str().to_string(),
                deployment_id: service.deployment_id,
                name: service.name,
                status: service.status,
                fqdn: service.fqdn,
                server_uuid: service.target_id,
                server_name: service.target_name,
                project_uuid: service.project_id,
                environment_name: service.environment_name,
                linked_subscription_id: linked_subscription.map(|subscription| subscription.id),
                linked_subscription_domain: linked_subscription
                    .and_then(|subscription| subscription.domain.clone()),
                linked_subscription_status: linked_subscription
                    .map(|subscription| subscription.status.clone()),
                linked_subscription_plan: linked_subscription
                    .map(|subscription| subscription.plan.clone()),
                server_label: label.to_string(),
                linked_subscription_client: linked_subscription
                    .map(|subscription| subscription.client_name.clone()),
                storage_limit_mb: linked_subscription
                    .map(|subscription| subscription.storage_limit_mb),
                cpu_percent: None,
                ram_used_mb: None,
                ram_limit_mb: None,
                storage_used_mb: None,
            }
        })
        .collect()
}

fn f64_to_i64_rounded(value: f64) -> Option<i64> {
    if !value.is_finite() {
        return None;
    }
    format!("{value:.0}").parse::<i64>().ok()
}

/* [225A-4] Enriquece despliegues desde snapshots del sampler, no desde SSH en render.
 * Si aún no hay muestras, el panel muestra guiones hasta que el loop background
 * capture el primer promedio. */
async fn enrich_deployment_resources(
    state: &AppState,
    deployments: &mut [CoolifyDeploymentResponse],
) {
    for deployment in deployments {
        match InfrastructureRepository::latest_deployment_sample(&state.pool, &deployment.uuid)
            .await
        {
            Ok(Some(sample)) => {
                deployment.cpu_percent = sample.cpu_percent;
                deployment.ram_used_mb = sample.ram_used_mb;
                deployment.ram_limit_mb = sample.ram_limit_mb;
                deployment.storage_used_mb = sample.disk_used_mb.and_then(f64_to_i64_rounded);
            }
            Ok(None) => {}
            Err(error) => tracing::warn!(
                "[deployments] No se pudo leer snapshot para {}: {error}",
                deployment.uuid
            ),
        }
    }
}

struct DeploymentsCacheEntry {
    fetched_at: std::time::Instant,
    deployments: Vec<CoolifyDeploymentResponse>,
}

static DEPLOYMENTS_CACHE: std::sync::OnceLock<tokio::sync::RwLock<DeploymentsCacheEntry>> =
    std::sync::OnceLock::new();

/// Listar despliegues reales de todas las VPS configuradas en Coolify (admin only)
#[utoipa::path(
    get,
    path = "/api/hosting/deployments",
    responses(
        (status = 200, description = "Lista de despliegues reales en Coolify (todas las VPS)", body = Vec<CoolifyDeploymentResponse>),
        (status = 403, description = "Sin permisos"),
        (status = 503, description = "Coolify no configurado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
#[allow(clippy::too_many_lines)]
pub(super) async fn list_deployments(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<CoolifyDeploymentResponse>>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    if coolify_server_targets(
        state.coolify_config_vps1.as_ref(),
        state.coolify_config.as_ref(),
    )
    .is_empty()
    {
        return Err(AppError::ServiceUnavailable(
            "Coolify no está configurado para listar despliegues".into(),
        ));
    }

    let cache = DEPLOYMENTS_CACHE.get_or_init(|| {
        tokio::sync::RwLock::new(DeploymentsCacheEntry {
            // Empezar con una fecha antigua para forzar el primer refresh
            fetched_at: std::time::Instant::now()
                .checked_sub(std::time::Duration::from_secs(9999))
                .unwrap_or_else(std::time::Instant::now),
            deployments: Vec::new(),
        })
    });

    let needs_refresh;
    let mut current_deployments = Vec::new();
    {
        let cache_guard = cache.read().await;
        if cache_guard.deployments.is_empty() {
            needs_refresh = true;
        } else {
            current_deployments = cache_guard.deployments.clone();
            // Stale-while-revalidate: cache dura 30 segundos, pero devolvemos viejo mientras carga el nuevo
            needs_refresh = cache_guard.fetched_at.elapsed() > std::time::Duration::from_secs(30);
        }
    }

    if needs_refresh {
        if current_deployments.is_empty() {
            tracing::info!("[deployments] Primer carga, esperando datos...");
            current_deployments = build_deployments(state.clone()).await?;
            let mut cache_guard = cache.write().await;
            cache_guard.deployments.clone_from(&current_deployments);
            cache_guard.fetched_at = std::time::Instant::now();
        } else {
            tracing::info!(
                "[deployments] Devolviendo de caché (stale), refrescando en background..."
            );
            let state_clone = state.clone();
            tokio::spawn(async move {
                if let Ok(new_deployments) = build_deployments(state_clone).await {
                    if let Some(cache) = DEPLOYMENTS_CACHE.get() {
                        let mut cache_guard = cache.write().await;
                        cache_guard.deployments = new_deployments;
                        cache_guard.fetched_at = std::time::Instant::now();
                        tracing::info!("[deployments] Caché refrescado en background");
                    } else {
                        tracing::warn!(
                            "[deployments] Cache no inicializado al refrescar background"
                        );
                    }
                } else {
                    tracing::warn!("[deployments] Falló el refresco en background");
                }
            });
        }
    } else {
        tracing::info!("[deployments] Devolviendo de caché (fresco)");
    }

    Ok(Json(current_deployments))
}

async fn build_deployments(state: AppState) -> Result<Vec<CoolifyDeploymentResponse>, AppError> {
    tracing::info!("[deployments] -> Inicio build_deployments");

    tracing::info!("[deployments] Consultando repositorios...");
    let subscriptions = HostingRepository::list_all(&state.pool).await?;
    let subscriptions_by_uuid: HashMap<&str, _> = subscriptions
        .iter()
        .filter(|subscription| subscription.is_coolify_runtime())
        .filter_map(|subscription| {
            subscription
                .deployment_id_or_legacy()
                .map(|deployment_id| (deployment_id, subscription))
        })
        .collect();
    let subscriptions_by_name: HashMap<&str, _> = subscriptions
        .iter()
        .filter(|subscription| subscription.is_coolify_runtime())
        .filter_map(|subscription| {
            subscription
                .coolify_site_name
                .as_deref()
                .map(|site_name| (site_name, subscription))
        })
        .collect();

    let mut deployments: Vec<CoolifyDeploymentResponse> = Vec::new();

    let targets = coolify_server_targets(
        state.coolify_config_vps1.as_ref(),
        state.coolify_config.as_ref(),
    );

    for target in &targets {
        tracing::info!("[deployments] Consultando {} en Coolify...", target.label);
        match HostingRuntimeService::list_deployments(
            &state.http_client,
            Some(target.config),
            Some(HostingRuntimeKind::Coolify),
        )
        .await
        {
            Ok(services) => {
                tracing::info!(
                    "[deployments] {} devolvió {} servicios",
                    target.label,
                    services.len()
                );
                deployments.extend(map_runtime_deployments(
                    services,
                    &target.label,
                    &subscriptions_by_uuid,
                    &subscriptions_by_name,
                ));
            }
            Err(error) => tracing::warn!("[deployments] Error listando {}: {error}", target.label),
        }
    }

    tracing::info!("[deployments] Iniciando enrich_deployment_resources...");
    enrich_deployment_resources(&state, &mut deployments).await;
    tracing::info!("[deployments] Finalizó enrich_deployment_resources");

    deployments.sort_by(|left, right| {
        right
            .linked_subscription_id
            .is_some()
            .cmp(&left.linked_subscription_id.is_some())
            .then(left.name.cmp(&right.name))
            .then(left.uuid.cmp(&right.uuid))
    });

    tracing::info!("[deployments] -> Fin build_deployments");
    Ok(deployments)
}

/* [165A-4] Permite limpiar despliegues huérfanos desde el panel admin.
 * Solo borra stacks sin vínculo en BD para evitar desalinear suscripciones reales. */
#[utoipa::path(
    delete,
    path = "/api/hosting/deployments/{deployment_uuid}",
    params(("deployment_uuid" = String, Path, description = "UUID del despliegue en Coolify")),
    responses(
        (status = 204, description = "Despliegue eliminado"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "Despliegue no encontrado"),
        (status = 409, description = "El despliegue ya está vinculado a una suscripción"),
        (status = 503, description = "Coolify no configurado o no disponible"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub(super) async fn delete_deployment(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(deployment_uuid): Path<String>,
) -> Result<StatusCode, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let targets = coolify_server_targets(
        state.coolify_config_vps1.as_ref(),
        state.coolify_config.as_ref(),
    );

    if targets.is_empty() {
        return Err(AppError::ServiceUnavailable(
            "Coolify no está configurado para eliminar despliegues".into(),
        ));
    }

    let subscriptions = HostingRepository::list_all(&state.pool).await?;
    let mut lookup_failed = false;
    let mut target_config: Option<&CoolifyConfig> = None;
    let mut target_name: Option<String> = None;

    for target in &targets {
        match HostingRuntimeService::list_deployments(
            &state.http_client,
            Some(target.config),
            Some(HostingRuntimeKind::Coolify),
        )
        .await
        {
            Ok(services) => {
                if let Some(service) = services
                    .into_iter()
                    .find(|service| service.deployment_id == deployment_uuid)
                {
                    target_name = Some(service.name);
                    target_config = Some(target.config);
                    break;
                }
            }
            Err(error) => {
                lookup_failed = true;
                tracing::warn!(
                    "[deployments] Error buscando despliegue {} en {}: {}",
                    deployment_uuid,
                    target.label,
                    error
                );
            }
        }
    }

    let target_config = match target_config {
        Some(config) => config,
        None if lookup_failed => {
            return Err(AppError::ServiceUnavailable(
                "No se pudo consultar Coolify para ubicar el despliegue".into(),
            ));
        }
        None => {
            return Err(AppError::NotFound(
                "Despliegue no encontrado en Coolify".into(),
            ));
        }
    };

    let target_name = target_name.expect("deployment name must exist when config is found");
    let linked_subscription = subscriptions.iter().find(|subscription| {
        subscription.deployment_id_or_legacy() == Some(deployment_uuid.as_str())
            || subscription.coolify_site_name.as_deref() == Some(target_name.as_str())
    });

    if let Some(subscription) = linked_subscription {
        return Err(AppError::Conflict(format!(
            "El despliegue {} ya está vinculado a la suscripción {}. Elimínalo desde la suscripción para no dejar datos huérfanos.",
            target_name,
            subscription.id
        )));
    }

    HostingRuntimeService::delete_deployment(
        &state.http_client,
        Some(target_config),
        Some(HostingRuntimeKind::Coolify),
        &deployment_uuid,
        true,
    )
    .await?;
    tracing::info!(
        "[deployments] Despliegue huérfano {} ({}) eliminado desde el panel admin.",
        target_name,
        deployment_uuid
    );

    Ok(StatusCode::NO_CONTENT)
}
