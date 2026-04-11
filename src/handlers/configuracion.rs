/* [114A-12] Handler admin para configuración del sistema.
 * Endpoint de rotación de API keys: GET status + PATCH toggle.
 * Solo admin. Permite activar/desactivar rotación desde el panel. */

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::UserRole;
use crate::services::AiChatConfig;
use crate::AppState;

#[derive(Serialize, ToSchema)]
pub struct RotacionStatusResponse {
    pub enabled: bool,
    pub total_keys: usize,
    pub current_index: usize,
    pub model: String,
    pub has_fallback: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct ToggleRotacionRequest {
    pub enabled: bool,
}

/// Obtener estado actual de la rotación de API keys
#[utoipa::path(
    get,
    path = "/api/admin/configuracion/rotacion",
    responses(
        (status = 200, description = "Estado de rotación", body = RotacionStatusResponse),
        (status = 401, description = "No autenticado"),
        (status = 403, description = "No autorizado"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_rotation_status(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<RotacionStatusResponse>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    Ok(Json(RotacionStatusResponse {
        enabled: AiChatConfig::is_rotation_enabled(),
        total_keys: state.ai_config.total_keys(),
        current_index: state.ai_config.current_key_index(),
        model: state.ai_config.model.clone(),
        has_fallback: state.ai_config.gemini_key.is_some(),
    }))
}

/// Activar o desactivar rotación de API keys
#[utoipa::path(
    patch,
    path = "/api/admin/configuracion/rotacion",
    request_body = ToggleRotacionRequest,
    responses(
        (status = 200, description = "Rotación actualizada", body = RotacionStatusResponse),
        (status = 401, description = "No autenticado"),
        (status = 403, description = "No autorizado"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn toggle_rotation(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<ToggleRotacionRequest>,
) -> Result<Json<RotacionStatusResponse>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    AiChatConfig::set_rotation_enabled(body.enabled);

    Ok(Json(RotacionStatusResponse {
        enabled: AiChatConfig::is_rotation_enabled(),
        total_keys: state.ai_config.total_keys(),
        current_index: state.ai_config.current_key_index(),
        model: state.ai_config.model.clone(),
        has_fallback: state.ai_config.gemini_key.is_some(),
    }))
}

pub fn configuracion_routes() -> Router<AppState> {
    Router::new()
        .route("/admin/configuracion/rotacion", get(get_rotation_status).patch(toggle_rotation))
}
