/* [205A-1] Endpoints de cobros pendientes del panel.
 * Mantiene facturacion legacy separada de provisioning para evitar caidas por cobro. */
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{BillingCheckoutResponse, BillingItem, CreateBillingCheckoutRequest, UserRole};
use crate::repositories::{BillingRepository, UserRepository};
use crate::services::{is_checkout_bypass_email, BillingCheckoutParams, BillingStripeService};
use crate::AppState;

fn resolve_public_base_url(headers: &HeaderMap) -> String {
    if let Some(origin) = headers.get("origin").and_then(|value| value.to_str().ok()) {
        let trimmed = origin.trim_end_matches('/');
        if !trimmed.is_empty() && !trimmed.contains("localhost") {
            return trimmed.to_string();
        }
    }

    if let Ok(env_url) = std::env::var("GLORY_PUBLIC_URL") {
        return env_url.trim_end_matches('/').to_string();
    }

    "http://localhost:5173".to_string()
}

pub async fn list_billing_items(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<BillingItem>>, AppError> {
    auth.require_role(&[UserRole::Client, UserRole::Admin])?;
    let items = BillingRepository::list_for_user(&state.pool, auth.user_id).await?;
    Ok(Json(items))
}

pub async fn create_billing_checkout(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(req): Json<CreateBillingCheckoutRequest>,
) -> Result<(StatusCode, Json<BillingCheckoutResponse>), AppError> {
    auth.require_role(&[UserRole::Client, UserRole::Admin])?;

    let requested_ids = req.item_ids.as_deref();
    if requested_ids.is_some_and(<[uuid::Uuid]>::is_empty) {
        return Err(AppError::Validation("Selecciona al menos un cobro".into()));
    }

    let items =
        BillingRepository::pending_for_checkout(&state.pool, auth.user_id, requested_ids).await?;
    if items.is_empty() {
        return Err(AppError::Validation(
            "No hay cobros pendientes para pagar".into(),
        ));
    }

    let user = UserRepository::find_by_id(&state.pool, auth.user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Usuario no encontrado".into()))?;

    /* [195A-1] Test bypass: evita requerir Stripe en entorno de desarrollo. */
    if is_checkout_bypass_email(&user.email) {
        let base_url = resolve_public_base_url(&headers);
        let item_ids = items.iter().map(|item| item.id).collect::<Vec<_>>();
        let fake_session = format!("test-bypass-{}", user.id);
        BillingRepository::set_checkout_session(&state.pool, &item_ids, &fake_session).await?;
        BillingRepository::mark_paid_by_session(&state.pool, &fake_session).await?;
        return Ok((
            StatusCode::CREATED,
            Json(BillingCheckoutResponse {
                checkout_url: format!("{base_url}/panel?seccion=hosting&billing=success"),
            }),
        ));
    }

    let stripe_key = state
        .stripe_secret_key
        .as_deref()
        .ok_or_else(|| AppError::ServiceUnavailable("Stripe no configurado".into()))?;

    let base_url = resolve_public_base_url(&headers);
    let success_url = format!("{base_url}/panel?seccion=hosting&billing=success");
    let cancel_url = format!("{base_url}/panel?seccion=hosting&billing=cancelled");

    let (session_id, checkout_url) =
        BillingStripeService::create_checkout_session(&BillingCheckoutParams {
            http_client: &state.http_client,
            stripe_key,
            items: &items,
            mode: req.mode,
            customer_email: &user.email,
            success_url: &success_url,
            cancel_url: &cancel_url,
        })
        .await?;

    let item_ids = items.iter().map(|item| item.id).collect::<Vec<_>>();
    BillingRepository::set_checkout_session(&state.pool, &item_ids, &session_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(BillingCheckoutResponse { checkout_url }),
    ))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/billing/items", get(list_billing_items))
        .route("/billing/checkout", post(create_billing_checkout))
}
