use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use uuid::Uuid;
use validator::Validate;

use super::domain::{build_domain_verification_state, normalize_domain};
use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{
    HostingSubscriptionResponse, SelfSubscribeRequest, SelfSubscribeResponse, UserRole,
};
use crate::repositories::{CreateHostingParams, HostingRepository, UserRepository};
use crate::services::{is_checkout_bypass_email, CoolifyConfig, HostingStripeService};
use crate::AppState;

/// Deriva la URL base pública desde Origin header, env var, o fallback dev.
fn resolve_public_base_url(headers: &HeaderMap) -> String {
    if let Some(origin) = headers.get("origin").and_then(|v| v.to_str().ok()) {
        let trimmed = origin.trim_end_matches('/');
        if !trimmed.is_empty() && !trimmed.contains("localhost") {
            return trimmed.to_string();
        }
    }

    if let Some(referer) = headers.get("referer").and_then(|v| v.to_str().ok()) {
        if let Some(idx) = referer.find("://") {
            let after_scheme = &referer[idx + 3..];
            let host_end = after_scheme.find('/').unwrap_or(after_scheme.len());
            let base = &referer[..idx + 3 + host_end];
            if !base.contains("localhost") {
                return base.to_string();
            }
        }
    }

    if let Ok(env_url) = std::env::var("GLORY_PUBLIC_URL") {
        return env_url.trim_end_matches('/').to_string();
    }

    "http://localhost:5173".to_string()
}

fn normalize_hosting_billing_cycle(value: Option<i32>) -> Result<i32, AppError> {
    let months = value.unwrap_or(1);
    match months {
        1 | 6 | 12 => Ok(months),
        _ => Err(AppError::Validation(
            "El periodo de pago debe ser 1, 6 o 12 meses".into(),
        )),
    }
}

fn hosting_billing_discount_cents(months: i32) -> i32 {
    match months {
        12 => 2000,
        6 => 1000,
        _ => 0,
    }
}

fn hosting_period_amount_cents(monthly_price_cents: i32, months: i32) -> i32 {
    (monthly_price_cents * months - hosting_billing_discount_cents(months)).max(0)
}

fn ensure_single_line_secret(value: Option<&str>, label: &str) -> Result<(), AppError> {
    if value.is_some_and(|candidate| candidate.contains(['\n', '\r'])) {
        return Err(AppError::Validation(format!(
            "{label} no puede contener saltos de línea"
        )));
    }
    Ok(())
}

fn validate_hosting_checkout_request(req: &SelfSubscribeRequest) -> Result<i32, AppError> {
    let billing_cycle_months = normalize_hosting_billing_cycle(req.billing_cycle_months)?;
    ensure_single_line_secret(req.wp_admin_password.as_deref(), "La contraseña wp-admin")?;
    ensure_single_line_secret(req.sftp_password.as_deref(), "La contraseña SFTP")?;
    if let Some(language) = req.wp_language.as_deref() {
        if !matches!(language, "es_ES" | "en_US" | "ja") {
            return Err(AppError::Validation("Idioma WordPress no soportado".into()));
        }
    }
    Ok(billing_cycle_months)
}

fn hosting_checkout_config_details(
    req: &SelfSubscribeRequest,
    billing_cycle_months: i32,
    period_amount_cents: i32,
) -> serde_json::Value {
    serde_json::json!({
        "billing_cycle_months": billing_cycle_months,
        "period_amount_cents": period_amount_cents,
        "discount_cents": hosting_billing_discount_cents(billing_cycle_months),
        "wp_admin_username": req.wp_admin_username.as_deref(),
        "wp_admin_password": req.wp_admin_password.as_deref(),
        "wp_language": req.wp_language.as_deref(),
        "sftp_user": req.sftp_user.as_deref(),
        "sftp_password": req.sftp_password.as_deref(),
    })
}

/* [094A-3] Self-service: cliente contrata hosting + paga en un solo paso.
 * Toma plan y dominio opcional, obtiene nombre/email del perfil del usuario autenticado,
 * crea la suscripción y la Stripe Checkout Session, retorna URL de pago. */
#[utoipa::path(
    post,
    path = "/api/hosting/subscribe",
    request_body = SelfSubscribeRequest,
    responses(
        (status = 201, description = "Suscripción creada + URL de checkout", body = SelfSubscribeResponse),
        (status = 400, description = "Plan inválido"),
        (status = 401, description = "No autorizado"),
        (status = 503, description = "Stripe no configurado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
#[allow(clippy::too_many_lines)]
pub(super) async fn subscribe_self(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(req): Json<SelfSubscribeRequest>,
) -> Result<(StatusCode, Json<SelfSubscribeResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let billing_cycle_months = validate_hosting_checkout_request(&req)?;

    let plan_config = HostingRepository::get_plan_config(&state.pool, &req.plan)
        .await?
        .ok_or_else(|| {
            AppError::Validation(format!(
                "Plan inválido: {}. Opciones disponibles en /hosting/plan-configs",
                req.plan
            ))
        })?;
    let price = plan_config.monthly_price_cents;
    let storage = plan_config.storage_limit_mb;
    let period_amount_cents = hosting_period_amount_cents(price, billing_cycle_months);

    let user = UserRepository::find_by_id(&state.pool, auth.user_id)
        .await?
        .ok_or(AppError::NotFound("Usuario no encontrado".into()))?;

    let client_name = user.display_name.unwrap_or_else(|| user.email.clone());
    let client_email = user.email;
    let requested_domain = normalize_domain(req.domain.as_deref());
    let (domain_verification_status, domain_verification_token, domain_verified_at) =
        build_domain_verification_state(requested_domain.as_deref());

    let sub = HostingRepository::create(
        &state.pool,
        CreateHostingParams {
            user_id: Some(auth.user_id),
            client_name: &client_name,
            client_email: &client_email,
            plan: &req.plan,
            domain: requested_domain.as_deref(),
            domain_verification_status: &domain_verification_status,
            domain_verification_token: domain_verification_token.as_deref(),
            domain_verified_at,
            coolify_site_name: None,
            monthly_price_cents: price,
            storage_limit_mb: storage,
        },
    )
    .await?;

    if let Err(e) = HostingRepository::add_event(
        &state.pool,
        sub.id,
        "created",
        Some(serde_json::json!({
            "plan": req.plan,
            "by": auth.user_id.to_string(),
            "source": "self-service",
            "checkout_config": hosting_checkout_config_details(&req, billing_cycle_months, period_amount_cents)
        })),
    )
    .await
    {
        tracing::warn!(
            "Error registrando evento created (self-service) para {}: {e}",
            sub.id
        );
    }

    let base_url = resolve_public_base_url(&headers);
    if let Some(response) = complete_test_hosting_checkout(
        &state.pool,
        &state.http_client,
        state.coolify_config.as_ref(),
        &base_url,
        sub.clone(),
        &client_email,
        "self-service",
    )
    .await?
    {
        return Ok((StatusCode::CREATED, Json(response)));
    }

    let stripe_key = state
        .stripe_secret_key
        .as_deref()
        .ok_or_else(|| AppError::ServiceUnavailable("Stripe no configurado".into()))?;

    let success_url =
        format!("{base_url}/panel?hosting=success&session_id={{CHECKOUT_SESSION_ID}}");
    let cancel_url = format!("{base_url}/panel?hosting=cancelled");

    let checkout_url = crate::services::HostingStripeService::create_checkout_session(
        &crate::services::CheckoutParams {
            http_client: &state.http_client,
            stripe_key,
            subscription_id: sub.id,
            plan: &sub.plan,
            amount_cents: period_amount_cents,
            customer_email: &client_email,
            success_url: &success_url,
            cancel_url: &cancel_url,
            billing_cycle_months,
        },
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(SelfSubscribeResponse {
            subscription: sub.into(),
            checkout_url,
        }),
    ))
}

async fn complete_test_hosting_checkout(
    pool: &sqlx::PgPool,
    http_client: &reqwest::Client,
    coolify_config: Option<&CoolifyConfig>,
    base_url: &str,
    sub: crate::models::HostingSubscription,
    client_email: &str,
    source: &str,
) -> Result<Option<SelfSubscribeResponse>, AppError> {
    if !is_checkout_bypass_email(client_email) {
        return Ok(None);
    }

    HostingRepository::update_status(pool, sub.id, "active").await?;
    if let Err(e) = HostingRepository::add_event(
        pool,
        sub.id,
        "test_checkout_bypassed",
        Some(serde_json::json!({"email": client_email, "source": source})),
    )
    .await
    {
        tracing::warn!("Error registrando bypass test hosting para {}: {e}", sub.id);
    }

    HostingStripeService::try_auto_provision_subscription(
        pool,
        http_client,
        coolify_config,
        &sub,
        "test_checkout_bypassed",
    )
    .await;

    let subscription = HostingRepository::find_by_id(pool, sub.id)
        .await?
        .unwrap_or(sub);
    let checkout_url = format!(
        "{base_url}/panel?hosting=test-bypass&subscription_id={}",
        subscription.id
    );

    Ok(Some(SelfSubscribeResponse {
        subscription: subscription.into(),
        checkout_url,
    }))
}

/* [165A-1] Los hostings de prueba creados antes del fix quedaron `active` pero sin
 * `server_uuid` ni credenciales SFTP. Cuando el cliente test vuelve al panel,
 * intentamos provisionarlos una sola vez usando el mismo flujo automático. */
pub(super) async fn maybe_backfill_test_hosting_access(
    state: &AppState,
    sub: crate::models::HostingSubscription,
    source: &str,
) -> Result<crate::models::HostingSubscription, AppError> {
    if sub.status != "active"
        || sub.server_uuid.is_some()
        || !is_checkout_bypass_email(&sub.client_email)
    {
        return Ok(sub);
    }

    HostingStripeService::try_auto_provision_subscription(
        &state.pool,
        &state.http_client,
        state.coolify_config.as_ref(),
        &sub,
        source,
    )
    .await;

    Ok(HostingRepository::find_by_id(&state.pool, sub.id)
        .await?
        .unwrap_or(sub))
}

/// Crear Checkout Session de Stripe para suscripción de hosting
#[utoipa::path(
    post,
    path = "/api/hosting/subscriptions/{id}/checkout",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    responses(
        (status = 200, description = "URL de checkout"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "No encontrada"),
        (status = 503, description = "Stripe no configurado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub(super) async fn create_checkout(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let sub = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    if auth.effective_role == UserRole::Client && sub.user_id != Some(auth.user_id) {
        return Err(AppError::Forbidden("Sin permisos".into()));
    }

    if sub.stripe_subscription_id.is_some() && sub.status == "active" {
        return Err(AppError::Conflict(
            "La suscripción ya tiene un pago activo en Stripe".into(),
        ));
    }

    let base_url = resolve_public_base_url(&headers);
    if is_checkout_bypass_email(&sub.client_email) {
        HostingRepository::update_status(&state.pool, sub.id, "active").await?;
        if let Err(e) = HostingRepository::add_event(
            &state.pool,
            sub.id,
            "test_checkout_bypassed",
            Some(serde_json::json!({"email": sub.client_email, "source": "existing-subscription-checkout"})),
        )
        .await
        {
            tracing::warn!(
                "Error registrando bypass test hosting existente para {}: {e}",
                sub.id
            );
        }
        return Ok(Json(serde_json::json!({
            "checkout_url": format!("{base_url}/panel?hosting=test-bypass&subscription_id={}", sub.id)
        })));
    }

    let stripe_key = state
        .stripe_secret_key
        .as_deref()
        .ok_or_else(|| AppError::ServiceUnavailable("Stripe no configurado".into()))?;

    let success_url =
        format!("{base_url}/panel?hosting=success&session_id={{CHECKOUT_SESSION_ID}}");
    let cancel_url = format!("{base_url}/panel?hosting=cancelled");

    let url = crate::services::HostingStripeService::create_checkout_session(
        &crate::services::CheckoutParams {
            http_client: &state.http_client,
            stripe_key,
            subscription_id: id,
            plan: &sub.plan,
            amount_cents: sub.monthly_price_cents,
            customer_email: &sub.client_email,
            success_url: &success_url,
            cancel_url: &cancel_url,
            billing_cycle_months: 1,
        },
    )
    .await?;

    Ok(Json(serde_json::json!({ "checkout_url": url })))
}

/// Admin test: crear hosting sin Stripe (para testing)
#[utoipa::path(
    post,
    path = "/api/hosting/admin-test-subscribe",
    request_body = SelfSubscribeRequest,
    responses(
        (status = 201, description = "Suscripción creada y activada (test)"),
        (status = 403, description = "Solo admin"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub(super) async fn admin_test_subscribe(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<SelfSubscribeRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    auth.require_role(&[UserRole::Admin])?;
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let billing_cycle_months = validate_hosting_checkout_request(&req)?;

    let plan_config = HostingRepository::get_plan_config(&state.pool, &req.plan)
        .await?
        .ok_or_else(|| AppError::Validation(format!("Plan inválido: {}", req.plan)))?;

    let user = UserRepository::find_by_id(&state.pool, auth.user_id)
        .await?
        .ok_or(AppError::NotFound("Usuario no encontrado".into()))?;
    let requested_domain = normalize_domain(req.domain.as_deref());
    let (domain_verification_status, domain_verification_token, domain_verified_at) =
        build_domain_verification_state(requested_domain.as_deref());

    let sub = HostingRepository::create(
        &state.pool,
        CreateHostingParams {
            user_id: Some(auth.user_id),
            client_name: &user.display_name.unwrap_or_else(|| user.email.clone()),
            client_email: &user.email,
            plan: &req.plan,
            domain: requested_domain.as_deref(),
            domain_verification_status: &domain_verification_status,
            domain_verification_token: domain_verification_token.as_deref(),
            domain_verified_at,
            coolify_site_name: None,
            monthly_price_cents: plan_config.monthly_price_cents,
            storage_limit_mb: plan_config.storage_limit_mb,
        },
    )
    .await?;

    HostingRepository::update_status(&state.pool, sub.id, "active").await?;

    if let Err(e) = HostingRepository::add_event(
        &state.pool,
        sub.id,
        "created",
        Some(serde_json::json!({
            "plan": req.plan,
            "by": auth.user_id.to_string(),
            "source": "admin-test",
            "note": "Suscripción de prueba sin Stripe",
            "checkout_config": hosting_checkout_config_details(
                &req,
                billing_cycle_months,
                hosting_period_amount_cents(plan_config.monthly_price_cents, billing_cycle_months)
            )
        })),
    )
    .await
    {
        tracing::warn!(
            "Error registrando evento admin-test-subscribe para {}: {e}",
            sub.id
        );
    }

    let updated = HostingRepository::find_by_id(&state.pool, sub.id)
        .await?
        .ok_or_else(|| AppError::Internal("Suscripción perdida post-create".into()))?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "subscription": HostingSubscriptionResponse::from(updated),
            "message": "Suscripción de prueba creada. Usa /provision para provisionarla.",
        })),
    ))
}
