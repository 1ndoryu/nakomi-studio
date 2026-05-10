use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;

#[derive(Serialize, ToSchema)]
pub struct PublicConfigResponse {
    pub stripe_publishable_key: Option<String>,
}

/// Configuracion publica que el frontend puede cargar en runtime.
#[utoipa::path(
    get,
    path = "/api/public-config",
    responses(
        (status = 200, description = "Configuracion publica", body = PublicConfigResponse)
    )
)]
pub async fn get_public_config(State(state): State<AppState>) -> Json<PublicConfigResponse> {
    Json(PublicConfigResponse {
        stripe_publishable_key: state.stripe_publishable_key,
    })
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/public-config", get(get_public_config))
}
