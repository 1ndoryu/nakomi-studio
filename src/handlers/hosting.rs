/* [054A-2] Handlers de hosting: CRUD suscripciones + eventos.
 * Solo admin puede gestionar hosting. Los clientes ven sus propias suscripciones
 * pero no pueden modificarlas directamente. */

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;
use validator::Validate;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{
    CreateHostingRequest, HostingEvent, HostingSubscriptionResponse,
    UpdateHostingRequest, UpdateHostingStatusRequest, UserRole,
};
use crate::repositories::{CreateHostingParams, HostingRepository};
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
    let _ = HostingRepository::add_event(
        &state.pool,
        sub.id,
        "created",
        Some(serde_json::json!({"plan": req.plan, "by": auth.user_id.to_string()})),
    )
    .await;

    Ok((StatusCode::CREATED, Json(sub.into())))
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

    /* Registrar evento */
    let _ = HostingRepository::add_event(
        &state.pool,
        id,
        "status_change",
        Some(serde_json::json!({
            "new_status": req.status,
            "reason": req.reason,
            "by": auth.user_id.to_string()
        })),
    )
    .await;

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

/* [074A-65] Actualizar suscripción de hosting (solo admin) */
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
    auth.require_role(&[UserRole::Admin])?;
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let price = plan_price_cents(&req.plan).ok_or_else(|| {
        AppError::Validation(format!("Plan inválido: {}. Opciones: basico, pro, ecommerce, custom", req.plan))
    })?;
    let storage = plan_storage_mb(&req.plan);

    HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    let updated = HostingRepository::update(
        &state.pool,
        id,
        &req.plan,
        req.domain.as_deref(),
        price,
        storage,
    )
    .await?;

    let _ = HostingRepository::add_event(
        &state.pool,
        id,
        "updated",
        Some(serde_json::json!({
            "plan": req.plan,
            "domain": req.domain,
            "by": auth.user_id.to_string()
        })),
    )
    .await;

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
}
