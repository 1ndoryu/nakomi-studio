use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    /* [096A-7] Métricas internas para diagnóstico de starvation/crash */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_timing_loops: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registered_sessions: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_permits_available: Option<usize>,
}

/// Endpoint de health check — siempre público
#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "Servicio funcionando", body = HealthResponse)
    )
)]
pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let (loops, sessions, permits) = state.chat_timing.metrics();
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        active_timing_loops: Some(loops),
        registered_sessions: Some(sessions),
        ai_permits_available: Some(permits),
    })
}

/* [135A-1] /healthz queda fuera del namespace /api para que Docker/Coolify
 * no dependan del rate limit ni de contratos publicos de API al decidir liveness. */
pub fn root_routes() -> Router<AppState> {
    Router::new().route("/healthz", get(health_check))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}
