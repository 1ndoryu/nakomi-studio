use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::UserRole;
use crate::repositories::HostingRepository;
use crate::services::{CoolifyConfig, CoolifyService};
use crate::AppState;

async fn resolve_provisioned_sub(
    state: &AppState,
    auth: &AuthUser,
    id: Uuid,
) -> Result<(crate::models::HostingSubscription, String, CoolifyConfig), AppError> {
    let sub = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;
    if auth.effective_role != UserRole::Admin && sub.user_id != Some(auth.user_id) {
        return Err(AppError::Forbidden("Sin permisos".into()));
    }
    let server_uuid = sub
        .server_uuid
        .clone()
        .ok_or(AppError::Validation("Hosting no provisionado".into()))?;
    let config = state
        .coolify_config
        .clone()
        .ok_or(AppError::ServiceUnavailable(
            "Coolify no configurado".into(),
        ))?;
    Ok((sub, server_uuid, config))
}

async fn record_control_event(state: &AppState, id: Uuid, event: &str, actor_id: Uuid) {
    if let Err(error) = HostingRepository::add_event(
        &state.pool,
        id,
        event,
        Some(serde_json::json!({"by": actor_id.to_string()})),
    )
    .await
    {
        tracing::warn!("Error registrando evento {event} para {id}: {error}");
    }
}

/// Reiniciar el servicio `WordPress`
#[utoipa::path(
    post,
    path = "/api/hosting/subscriptions/{id}/restart",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    responses(
        (status = 200, description = "Servicio reiniciado"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "No encontrada"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub(super) async fn restart_hosting(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (_sub, server_uuid, config) = resolve_provisioned_sub(&state, &auth, id).await?;
    CoolifyService::restart_service(&state.http_client, &config, &server_uuid).await?;
    record_control_event(&state, id, "restarted", auth.user_id).await;
    Ok(Json(serde_json::json!({"message": "WordPress reiniciado"})))
}

/// Detener el servicio `WordPress`
#[utoipa::path(
    post,
    path = "/api/hosting/subscriptions/{id}/stop",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    responses(
        (status = 200, description = "Servicio detenido"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "No encontrada"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub(super) async fn stop_hosting(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (_sub, server_uuid, config) = resolve_provisioned_sub(&state, &auth, id).await?;
    CoolifyService::stop_service(&state.http_client, &config, &server_uuid).await?;
    record_control_event(&state, id, "stopped", auth.user_id).await;
    Ok(Json(serde_json::json!({"message": "WordPress detenido"})))
}

/// Arrancar el servicio `WordPress`
#[utoipa::path(
    post,
    path = "/api/hosting/subscriptions/{id}/start",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    responses(
        (status = 200, description = "Servicio iniciado"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "No encontrada"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub(super) async fn start_hosting(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (_sub, server_uuid, config) = resolve_provisioned_sub(&state, &auth, id).await?;
    CoolifyService::start_service(&state.http_client, &config, &server_uuid).await?;
    record_control_event(&state, id, "started", auth.user_id).await;
    Ok(Json(serde_json::json!({"message": "WordPress iniciado"})))
}
