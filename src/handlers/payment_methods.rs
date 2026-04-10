use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{PaymentMethodResponse, SavePaymentMethodRequest, SetupIntentResponse, UserRole};
use crate::services::PaymentMethodService;
use crate::AppState;

#[utoipa::path(
    post,
    path = "/api/payment-methods/setup-intent",
    responses(
        (status = 200, description = "SetupIntent listo para guardar tarjeta", body = SetupIntentResponse),
        (status = 401, description = "No autorizado", body = crate::errors::ErrorResponse),
        (status = 400, description = "Stripe no pudo preparar la tarjeta", body = crate::errors::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "payment_methods"
)]
pub async fn create_setup_intent(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<SetupIntentResponse>, AppError> {
    auth.require_role(&[UserRole::Client, UserRole::Admin])?;

    let stripe_key = state
        .stripe_secret_key
        .as_ref()
        .ok_or_else(|| AppError::Internal("Stripe no esta configurado".into()))?;

    let response = PaymentMethodService::create_setup_intent(
        &state.pool,
        &state.http_client,
        stripe_key,
        auth.user_id,
    )
    .await?;

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/payment-methods",
    responses(
        (status = 200, description = "Tarjetas guardadas del usuario", body = Vec<PaymentMethodResponse>),
        (status = 401, description = "No autorizado", body = crate::errors::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "payment_methods"
)]
pub async fn list_payment_methods(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<PaymentMethodResponse>>, AppError> {
    auth.require_role(&[UserRole::Client, UserRole::Admin])?;

    let methods = PaymentMethodService::list_payment_methods(&state.pool, auth.user_id).await?;
    Ok(Json(methods))
}

#[utoipa::path(
    post,
    path = "/api/payment-methods",
    request_body = SavePaymentMethodRequest,
    responses(
        (status = 200, description = "Tarjeta guardada", body = PaymentMethodResponse),
        (status = 401, description = "No autorizado", body = crate::errors::ErrorResponse),
        (status = 400, description = "SetupIntent inválido o no confirmado", body = crate::errors::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "payment_methods"
)]
pub async fn save_payment_method(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<SavePaymentMethodRequest>,
) -> Result<Json<PaymentMethodResponse>, AppError> {
    auth.require_role(&[UserRole::Client, UserRole::Admin])?;

    let stripe_key = state
        .stripe_secret_key
        .as_ref()
        .ok_or_else(|| AppError::Internal("Stripe no esta configurado".into()))?;

    let method = PaymentMethodService::save_payment_method(
        &state.pool,
        &state.http_client,
        stripe_key,
        auth.user_id,
        &req.setup_intent_id,
    )
    .await?;

    Ok(Json(method))
}

#[utoipa::path(
    delete,
    path = "/api/payment-methods/{payment_method_id}",
    params(("payment_method_id" = Uuid, Path, description = "ID local del metodo de pago")),
    responses(
        (status = 204, description = "Tarjeta eliminada"),
        (status = 401, description = "No autorizado", body = crate::errors::ErrorResponse),
        (status = 404, description = "Tarjeta no encontrada", body = crate::errors::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "payment_methods"
)]
pub async fn delete_payment_method(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(payment_method_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    auth.require_role(&[UserRole::Client, UserRole::Admin])?;

    let stripe_key = state
        .stripe_secret_key
        .as_ref()
        .ok_or_else(|| AppError::Internal("Stripe no esta configurado".into()))?;

    PaymentMethodService::delete_payment_method(
        &state.pool,
        &state.http_client,
        stripe_key,
        auth.user_id,
        payment_method_id,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/payment-methods", get(list_payment_methods).post(save_payment_method))
        .route("/payment-methods/setup-intent", post(create_setup_intent))
        .route("/payment-methods/:payment_method_id", delete(delete_payment_method))
}