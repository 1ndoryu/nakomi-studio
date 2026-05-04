/* [064A-62] Handler admin para gestión de datos de prueba (seed).
 * Solo admin. Permite recrear o borrar datos de prueba desde el panel.
 * [025B-1] recreate_seed ahora corre fixture_manager.sync_all() antes del seed
 * para garantizar que los usuarios de prueba existen incluso si FIXTURES_SYNC=false. */

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde::Serialize;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::UserRole;
use crate::services::SeedService;
use crate::AppState;

#[derive(Serialize)]
pub struct SeedResponse {
    pub message: String,
}

/// Recrear datos de prueba (borrar + crear nuevos)
pub async fn recreate_seed(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<SeedResponse>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    /* [025B-1] Sincroniza fixtures antes de crear datos suplementarios.
     * Sin esto, los usuarios de prueba (cliente@test.com / empleado@test.com)
     * no existen en BD cuando FIXTURES_SYNC=false (producción o arrancado sin sync). */
    if let Some(fm) = &state.fixture_manager {
        match fm.sync_all().await {
            Ok(report) => tracing::info!("[seed] Fixtures sincronizados: {}", report.summary()),
            Err(e) => tracing::warn!("[seed] No se pudieron sincronizar fixtures: {e}"),
        }
    }

    let message = SeedService::recreate_test_data(&state.pool, auth.user_id)
        .await
        .map_err(|e| AppError::Internal(format!("Error en seed: {e}")))?;

    Ok(Json(SeedResponse { message }))
}

/// Borrar datos de prueba
pub async fn delete_seed(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<(StatusCode, Json<SeedResponse>), AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let deleted = SeedService::delete_test_data(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error al borrar seed: {e}")))?;

    Ok((
        StatusCode::OK,
        Json(SeedResponse {
            message: format!("Datos de prueba eliminados ({deleted} usuarios borrados)."),
        }),
    ))
}

pub fn seed_routes() -> Router<AppState> {
    Router::new().route("/admin/seed", post(recreate_seed).delete(delete_seed))
}
