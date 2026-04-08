/* [054A-2] Handlers de hosting: CRUD suscripciones + eventos.
 * Admin puede gestionar todo. Clientes pueden actualizar dominio/plan de sus propias
 * suscripciones y solicitar cancelación.
 * [084A-4] Cliente ahora puede editar su suscripción y solicitar cancelación.
 * [084A-24] Endpoints Contabo VPS + Stripe hosting checkout. */

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;
use validator::Validate;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{
    CreateHostingRequest, HostingEvent, HostingStatsResponse, HostingSubscriptionResponse,
    SelfSubscribeRequest, SelfSubscribeResponse,
    UpdateHostingRequest, UpdateHostingStatusRequest, UserRole,
};
use crate::repositories::{CreateHostingParams, HostingRepository, UserRepository};
use crate::AppState;

/* [084A-10] Precios por plan en centavos: Básico $5, Pro $10, E-commerce $15 */
fn plan_price_cents(plan: &str) -> Option<i32> {
    match plan {
        "basico" => Some(500),
        "pro" => Some(1000),
        "ecommerce" => Some(1500),
        _ => None,
    }
}

/* Límite de almacenamiento por plan en MB */
fn plan_storage_mb(plan: &str) -> i32 {
    match plan {
        "pro" => 20_480,
        "ecommerce" => 51_200,
        "custom" => 102_400,
        /* basico y cualquier plan desconocido: 5120 MB */
        _ => 5120,
    }
}

/* ============================================================
   HANDLERS
   ============================================================ */

/// Listar suscripciones de hosting (admin: todas, cliente: las suyas)
#[utoipa::path(
    get,
    path = "/api/hosting/subscriptions",
    responses(
        (status = 200, description = "Lista de suscripciones", body = Vec<HostingSubscriptionResponse>),
        (status = 401, description = "No autorizado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn list_subscriptions(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<HostingSubscriptionResponse>>, AppError> {
    let subs = HostingRepository::list_all(&state.pool).await?;
    let filtered: Vec<HostingSubscriptionResponse> = if auth.effective_role == UserRole::Admin
        || auth.effective_role == UserRole::Employee
    {
        subs.into_iter().map(Into::into).collect()
    } else {
        subs.into_iter()
            .filter(|s| s.user_id == Some(auth.user_id))
            .map(Into::into)
            .collect()
    };
    Ok(Json(filtered))
}

/// Crear suscripción de hosting (semi-automático: crea registro, admin provisiona después)
#[utoipa::path(
    post,
    path = "/api/hosting/subscriptions",
    request_body = CreateHostingRequest,
    responses(
        (status = 201, description = "Suscripción creada", body = HostingSubscriptionResponse),
        (status = 400, description = "Datos inválidos"),
        (status = 401, description = "No autorizado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn create_subscription(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateHostingRequest>,
) -> Result<(StatusCode, Json<HostingSubscriptionResponse>), AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let price = plan_price_cents(&req.plan).ok_or_else(|| {
        AppError::Validation(format!("Plan inválido: {}. Opciones: basico, pro, ecommerce, custom", req.plan))
    })?;
    let storage = plan_storage_mb(&req.plan);

    let sub = HostingRepository::create(
        &state.pool,
        CreateHostingParams {
            user_id: Some(auth.user_id),
            client_name: &req.client_name,
            client_email: &req.client_email,
            plan: &req.plan,
            domain: req.domain.as_deref(),
            monthly_price_cents: price,
            storage_limit_mb: storage,
        },
    )
    .await?;

    /* Registrar evento de creación */
    /* [094A-9] Log de errores en evento, no silenciar */
    if let Err(e) = HostingRepository::add_event(
        &state.pool,
        sub.id,
        "created",
        Some(serde_json::json!({"plan": req.plan, "by": auth.user_id.to_string()})),
    )
    .await {
        tracing::warn!("Error registrando evento created para {}: {e}", sub.id);
    }

    Ok((StatusCode::CREATED, Json(sub.into())))
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
pub async fn subscribe_self(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<SelfSubscribeRequest>,
) -> Result<(StatusCode, Json<SelfSubscribeResponse>), AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let price = plan_price_cents(&req.plan).ok_or_else(|| {
        AppError::Validation(format!(
            "Plan inválido: {}. Opciones: basico, pro, ecommerce",
            req.plan
        ))
    })?;
    let storage = plan_storage_mb(&req.plan);

    /* Obtener nombre y email del perfil del usuario autenticado */
    let user = UserRepository::find_by_id(&state.pool, auth.user_id)
        .await?
        .ok_or(AppError::NotFound("Usuario no encontrado".into()))?;

    let client_name = user.display_name.unwrap_or_else(|| user.email.clone());
    let client_email = user.email;

    /* Crear la suscripción en estado pending */
    let sub = HostingRepository::create(
        &state.pool,
        CreateHostingParams {
            user_id: Some(auth.user_id),
            client_name: &client_name,
            client_email: &client_email,
            plan: &req.plan,
            domain: req.domain.as_deref(),
            monthly_price_cents: price,
            storage_limit_mb: storage,
        },
    )
    .await?;

    /* [094A-9] Log de errores en evento, no silenciar */
    if let Err(e) = HostingRepository::add_event(
        &state.pool,
        sub.id,
        "created",
        Some(serde_json::json!({
            "plan": req.plan,
            "by": auth.user_id.to_string(),
            "source": "self-service"
        })),
    )
    .await {
        tracing::warn!("Error registrando evento created (self-service) para {}: {e}", sub.id);
    }

    /* Crear Stripe Checkout Session inmediatamente */
    let stripe_key = state
        .stripe_secret_key
        .as_deref()
        .ok_or_else(|| AppError::ServiceUnavailable("Stripe no configurado".into()))?;

    let config = state
        .hosting_stripe_config
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Precios de hosting no configurados".into()))?;

    let base_url = std::env::var("GLORY_PUBLIC_URL")
        .unwrap_or_else(|_| "http://localhost:5173".to_string());
    let success_url = format!(
        "{base_url}/panel?hosting=success&session_id={{CHECKOUT_SESSION_ID}}"
    );
    let cancel_url = format!("{base_url}/panel?hosting=cancelled");

    let checkout_url = crate::services::HostingStripeService::create_checkout_session(
        &crate::services::CheckoutParams {
            http_client: &state.http_client,
            stripe_key,
            config,
            subscription_id: sub.id,
            plan: &sub.plan,
            customer_email: &client_email,
            success_url: &success_url,
            cancel_url: &cancel_url,
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

/// Obtener suscripción por ID
#[utoipa::path(
    get,
    path = "/api/hosting/subscriptions/{id}",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    responses(
        (status = 200, description = "Suscripción", body = HostingSubscriptionResponse),
        (status = 404, description = "No encontrada"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn get_subscription(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<HostingSubscriptionResponse>, AppError> {
    let sub = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    /* Clientes solo ven sus propias suscripciones */
    if auth.effective_role == UserRole::Client && sub.user_id != Some(auth.user_id) {
        return Err(AppError::Forbidden("Sin permisos".into()));
    }

    Ok(Json(sub.into()))
}

/// Actualizar status de suscripción (solo admin)
#[utoipa::path(
    patch,
    path = "/api/hosting/subscriptions/{id}/status",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    request_body = UpdateHostingStatusRequest,
    responses(
        (status = 204, description = "Status actualizado"),
        (status = 400, description = "Status inválido"),
        (status = 403, description = "Sin permisos"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn update_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateHostingStatusRequest>,
) -> Result<StatusCode, AppError> {
    auth.require_role(&[UserRole::Admin])?;
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let valid_statuses = ["pending", "provisioning", "active", "suspended", "cancelled"];
    if !valid_statuses.contains(&req.status.as_str()) {
        return Err(AppError::Validation(format!(
            "Status inválido: {}. Opciones: {}",
            req.status,
            valid_statuses.join(", ")
        )));
    }

    /* Verificar que existe */
    HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    HostingRepository::update_status(&state.pool, id, &req.status).await?;

    /* [094A-9] Log de errores en evento, no silenciar */
    if let Err(e) = HostingRepository::add_event(
        &state.pool,
        id,
        "status_change",
        Some(serde_json::json!({
            "new_status": req.status,
            "reason": req.reason,
            "by": auth.user_id.to_string()
        })),
    )
    .await {
        tracing::warn!("Error registrando evento status_change para {id}: {e}");
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Listar eventos de una suscripción
#[utoipa::path(
    get,
    path = "/api/hosting/subscriptions/{id}/events",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    responses(
        (status = 200, description = "Eventos", body = Vec<HostingEvent>),
        (status = 404, description = "Suscripción no encontrada"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn list_events(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<HostingEvent>>, AppError> {
    /* Verificar acceso */
    let sub = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    if auth.effective_role == UserRole::Client && sub.user_id != Some(auth.user_id) {
        return Err(AppError::Forbidden("Sin permisos".into()));
    }

    let events = HostingRepository::list_events(&state.pool, id, 100).await?;
    Ok(Json(events))
}

/* ============================================================
   ROUTES
   ============================================================ */

/* [074A-65] Actualizar suscripción de hosting (admin: cualquiera, cliente: solo las suyas)
 * [084A-4] Relajado de admin-only a role-aware: cliente valida ownership. */
#[utoipa::path(
    put,
    path = "/api/hosting/subscriptions/{id}",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    request_body = UpdateHostingRequest,
    responses(
        (status = 200, description = "Suscripción actualizada", body = HostingSubscriptionResponse),
        (status = 400, description = "Datos inválidos"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "No encontrada"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn update_subscription(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateHostingRequest>,
) -> Result<Json<HostingSubscriptionResponse>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let sub = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    /* [084A-4] Clientes solo pueden editar sus propias suscripciones */
    if auth.effective_role == UserRole::Client && sub.user_id != Some(auth.user_id) {
        return Err(AppError::Forbidden("Sin permisos para editar esta suscripción".into()));
    }

    let price = plan_price_cents(&req.plan).ok_or_else(|| {
        AppError::Validation(format!("Plan inválido: {}. Opciones: basico, pro, ecommerce, custom", req.plan))
    })?;
    let storage = plan_storage_mb(&req.plan);

    let updated = HostingRepository::update(
        &state.pool,
        id,
        &req.plan,
        req.domain.as_deref(),
        price,
        storage,
    )
    .await?;

    /* [094A-9] Log de errores en evento, no silenciar */
    if let Err(e) = HostingRepository::add_event(
        &state.pool,
        id,
        "updated",
        Some(serde_json::json!({
            "plan": req.plan,
            "domain": req.domain,
            "by": auth.user_id.to_string()
        })),
    )
    .await {
        tracing::warn!("Error registrando evento updated para {id}: {e}");
    }

    Ok(Json(updated.into()))
}

/* [074A-65] Eliminar suscripción de hosting (solo admin) */
#[utoipa::path(
    delete,
    path = "/api/hosting/subscriptions/{id}",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    responses(
        (status = 204, description = "Eliminada"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "No encontrada"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn delete_subscription(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    /* CASCADE borra eventos asociados automáticamente */
    HostingRepository::delete(&state.pool, id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/* [084A-4] Solicitar cancelación de hosting (cliente o admin).
 * El cliente puede cancelar sus propias suscripciones activas.
 * No se usa DELETE directamente para clientes — esto cambia el status a cancelled
 * y registra un evento, preservando el registro para auditoría. */
#[utoipa::path(
    post,
    path = "/api/hosting/subscriptions/{id}/cancel",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    responses(
        (status = 204, description = "Cancelación procesada"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "No encontrada"),
        (status = 409, description = "Ya cancelada"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn request_cancel(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let sub = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    /* Clientes solo cancelan las suyas */
    if auth.effective_role == UserRole::Client && sub.user_id != Some(auth.user_id) {
        return Err(AppError::Forbidden("Sin permisos para cancelar esta suscripción".into()));
    }

    if sub.status == "cancelled" {
        return Err(AppError::Conflict("La suscripción ya está cancelada".into()));
    }

    HostingRepository::update_status(&state.pool, id, "cancelled").await?;

    /* [094A-9] Log de errores en evento, no silenciar */
    if let Err(e) = HostingRepository::add_event(
        &state.pool,
        id,
        "status_change",
        Some(serde_json::json!({
            "new_status": "cancelled",
            "reason": "Cancelación solicitada por el usuario",
            "by": auth.user_id.to_string()
        })),
    )
    .await {
        tracing::warn!("Error registrando evento cancellation para {id}: {e}");
    }

    Ok(StatusCode::NO_CONTENT)
}

/* ============================================================
   STRIPE CHECKOUT — Suscripción de hosting
   [084A-24] Crea Stripe Checkout Session para pagar hosting mensual
   ============================================================ */

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
pub async fn create_checkout(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let sub = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    /* Solo el dueño o admin puede generar checkout */
    if auth.effective_role == UserRole::Client && sub.user_id != Some(auth.user_id) {
        return Err(AppError::Forbidden("Sin permisos".into()));
    }

    /* Ya tiene suscripción Stripe activa */
    if sub.stripe_subscription_id.is_some() && sub.status == "active" {
        return Err(AppError::Conflict(
            "La suscripción ya tiene un pago activo en Stripe".into(),
        ));
    }

    let stripe_key = state
        .stripe_secret_key
        .as_deref()
        .ok_or_else(|| AppError::ServiceUnavailable("Stripe no configurado".into()))?;

    let config = state
        .hosting_stripe_config
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Precios de hosting no configurados".into()))?;

    /* URLs de retorno (frontend SPA) */
    let base_url = std::env::var("GLORY_PUBLIC_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    let success_url = format!("{base_url}/panel?hosting=success&session_id={{CHECKOUT_SESSION_ID}}");
    let cancel_url = format!("{base_url}/panel?hosting=cancelled");

    let url = crate::services::HostingStripeService::create_checkout_session(
        &crate::services::CheckoutParams {
            http_client: &state.http_client,
            stripe_key,
            config,
            subscription_id: id,
            plan: &sub.plan,
            customer_email: &sub.client_email,
            success_url: &success_url,
            cancel_url: &cancel_url,
        },
    )
    .await?;

    Ok(Json(serde_json::json!({ "checkout_url": url })))
}

/* ============================================================
   HOSTING STATS — Estadísticas reales de una suscripción
   [094A-8] Calcula uptime desde eventos, muestra límites reales del plan.
   ============================================================ */

/* Límite de ancho de banda por plan en GB */
fn plan_bandwidth_gb(plan: &str) -> i32 {
    match plan {
        "pro" => 200,
        "ecommerce" => 500,
        _ => 50,
    }
}

/* [094A-8] Calcula el porcentaje de uptime analizando transiciones de status en eventos.
 * Recorre los eventos cronológicamente, contando el tiempo total en estado "active".
 * Si la suscripción actualmente está activa, el período abierto se extiende hasta ahora. */
#[allow(clippy::cast_precision_loss)] /* Duración en segundos cabe en f64 sin pérdida práctica */
fn calculate_uptime(
    created_at: chrono::DateTime<chrono::Utc>,
    current_status: &str,
    events: &[crate::models::HostingEvent],
) -> (f64, Option<chrono::DateTime<chrono::Utc>>) {
    let now = chrono::Utc::now();
    let total_duration = (now - created_at).num_seconds().max(1) as f64;

    /* Recorrer eventos cronológicamente para rastrear períodos activos */
    let mut active_seconds: f64 = 0.0;
    let mut last_active_start: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut first_active: Option<chrono::DateTime<chrono::Utc>> = None;

    /* Eventos vienen DESC del repo, revertir para orden cronológico */
    let mut sorted_events: Vec<&crate::models::HostingEvent> = events.iter().collect();
    sorted_events.sort_by_key(|e| e.created_at);

    for event in &sorted_events {
        if event.event_type != "status_change" {
            continue;
        }
        let new_status = event
            .details
            .as_ref()
            .and_then(|d| d.get("new_status"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match new_status {
            "active" => {
                last_active_start = Some(event.created_at);
                if first_active.is_none() {
                    first_active = Some(event.created_at);
                }
            }
            _ => {
                /* Transición fuera de active: cerrar el período activo abierto */
                if let Some(start) = last_active_start.take() {
                    active_seconds += (event.created_at - start).num_seconds().max(0) as f64;
                }
            }
        }
    }

    /* Si actualmente está activo, cerrar el período abierto hasta ahora */
    if current_status == "active" {
        if let Some(start) = last_active_start {
            active_seconds += (now - start).num_seconds().max(0) as f64;
        } else if first_active.is_none() {
            /* Nunca hubo un evento status_change a "active" explícito pero el status es active
             * (puede pasar con suscripciones migradas). Asumir active desde creación. */
            first_active = Some(created_at);
            active_seconds = total_duration;
        }
    }

    let uptime = if total_duration > 0.0 {
        (active_seconds / total_duration * 100.0).min(100.0)
    } else {
        0.0
    };

    (uptime, first_active)
}

/// Estadísticas de uso de una suscripción de hosting
#[utoipa::path(
    get,
    path = "/api/hosting/subscriptions/{id}/stats",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    responses(
        (status = 200, description = "Estadísticas de la suscripción", body = HostingStatsResponse),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "Suscripción no encontrada"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn get_hosting_stats(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<HostingStatsResponse>, AppError> {
    let sub = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    /* Clientes solo ven stats de sus propias suscripciones */
    if auth.effective_role == UserRole::Client && sub.user_id != Some(auth.user_id) {
        return Err(AppError::Forbidden("Sin permisos".into()));
    }

    let events = HostingRepository::list_events(&state.pool, id, 1000).await?;
    let (uptime_percent, active_since) = calculate_uptime(sub.created_at, &sub.status, &events);

    let total_events = i64::try_from(events.len()).unwrap_or(i64::MAX);
    let last_event_at = events.first().map(|e| e.created_at);
    let monitoring_available = sub.coolify_site_name.is_some();

    Ok(Json(HostingStatsResponse {
        storage_limit_mb: sub.storage_limit_mb,
        storage_used_mb: None,
        bandwidth_limit_gb: plan_bandwidth_gb(&sub.plan),
        bandwidth_used_gb: None,
        uptime_percent,
        active_since,
        total_events,
        last_event_at,
        monitoring_available,
    }))
}

/* ============================================================
   VPS STATS — Proxy a Contabo API (solo admin)
   [084A-24] Permite al panel admin ver estado real de las VPS
   ============================================================ */

/// Listar instancias VPS (admin only — proxy Contabo API)
#[utoipa::path(
    get,
    path = "/api/hosting/vps",
    responses(
        (status = 200, description = "Lista de VPS"),
        (status = 403, description = "Sin permisos"),
        (status = 503, description = "Contabo no configurado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn list_vps(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let service = state
        .contabo_service
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Contabo API no configurada".into()))?;

    let instances = service
        .list_instances()
        .await
        .map_err(|e| AppError::Internal(e.clone()))?;

    Ok(Json(serde_json::json!({ "data": instances })))
}

/// Obtener instancia VPS por ID (admin only)
#[utoipa::path(
    get,
    path = "/api/hosting/vps/{instance_id}",
    params(("instance_id" = i64, Path, description = "ID de instancia Contabo")),
    responses(
        (status = 200, description = "Detalles de la VPS"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "Instancia no encontrada"),
        (status = 503, description = "Contabo no configurado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn get_vps(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(instance_id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let service = state
        .contabo_service
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Contabo API no configurada".into()))?;

    let instance = service
        .get_instance(instance_id)
        .await
        .map_err(|e| {
            if e.contains("not found") {
                AppError::NotFound(format!("VPS {instance_id} no encontrada"))
            } else {
                AppError::Internal(e)
            }
        })?;

    Ok(Json(serde_json::json!({ "data": instance })))
}

pub fn hosting_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/hosting/subscriptions",
            get(list_subscriptions).post(create_subscription),
        )
        .route(
            "/hosting/subscriptions/:id",
            get(get_subscription)
                .put(update_subscription)
                .delete(delete_subscription),
        )
        .route(
            "/hosting/subscriptions/:id/status",
            axum::routing::patch(update_status),
        )
        .route(
            "/hosting/subscriptions/:id/events",
            get(list_events),
        )
        /* [094A-8] Stats reales de una suscripción */
        .route(
            "/hosting/subscriptions/:id/stats",
            get(get_hosting_stats),
        )
        .route(
            "/hosting/subscriptions/:id/cancel",
            axum::routing::post(request_cancel),
        )
        /* [084A-24] Stripe checkout para hosting */
        .route(
            "/hosting/subscriptions/:id/checkout",
            axum::routing::post(create_checkout),
        )
        /* [094A-3] Self-service: cliente contrata + paga en un solo paso */
        .route(
            "/hosting/subscribe",
            axum::routing::post(subscribe_self),
        )
        /* [084A-24] VPS stats: proxy a Contabo API */
        .route("/hosting/vps", get(list_vps))
        .route("/hosting/vps/:instance_id", get(get_vps))
}
