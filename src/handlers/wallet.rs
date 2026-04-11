/* [154A-15a] Handlers de wallet: saldo virtual de usuarios.
 * GET /api/wallet — obtener saldo actual
 * GET /api/wallet/transactions — historial paginado
 * Incluye también endpoints de cancellation requests. */

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{
    CancellationRequestResponse, CreateCancellationRequest, CreateNotification, Order,
    RespondCancellationRequest, NOTIF_ORDER_CANCELLED,
};
use crate::repositories::{CancellationRequestRepository, OrderRepository, WalletRepository};
use crate::services::WalletService;
use crate::AppState;

/* ============================================================
   GET /api/wallet — Saldo actual del usuario
   ============================================================ */

#[utoipa::path(
    get,
    path = "/api/wallet",
    responses(
        (status = 200, description = "Saldo actual", body = WalletResponse),
    ),
    security(("bearer" = []))
)]
pub async fn get_balance(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let wallet = WalletService::get_balance(&state.pool, auth.user_id).await?;
    Ok(Json(wallet))
}

/* ============================================================
   GET /api/wallet/transactions — Historial paginado
   ============================================================ */

#[derive(Debug, Deserialize)]
pub struct TransactionQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/api/wallet/transactions",
    params(
        ("page" = Option<i64>, Query, description = "Página (1-indexed)"),
        ("per_page" = Option<i64>, Query, description = "Registros por página (max 100)"),
    ),
    responses(
        (status = 200, description = "Historial de transacciones", body = WalletTransactionsPage),
    ),
    security(("bearer" = []))
)]
pub async fn list_transactions(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<TransactionQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = WalletService::list_transactions(
        &state.pool,
        auth.user_id,
        q.page.unwrap_or(1),
        q.per_page.unwrap_or(20),
    )
    .await?;
    Ok(Json(page))
}

/* ============================================================
   POST /api/orders/:order_id/cancel-request — Empleado solicita cancelación
   ============================================================ */

#[utoipa::path(
    post,
    path = "/api/orders/{order_id}/cancel-request",
    request_body = CreateCancellationRequest,
    responses(
        (status = 201, description = "Solicitud de cancelación creada", body = CancellationRequestResponse),
        (status = 400, description = "Ya existe solicitud pendiente"),
        (status = 403, description = "Solo el responsable puede solicitar cancelación"),
        (status = 404, description = "Orden no encontrada"),
    ),
    params(("order_id" = Uuid, Path, description = "ID de la orden")),
    security(("bearer" = []))
)]
pub async fn create_cancellation_request(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(order_id): Path<Uuid>,
    Json(body): Json<CreateCancellationRequest>,
) -> Result<impl IntoResponse, AppError> {
    let order = OrderRepository::find_order_by_id(&state.pool, order_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Orden no encontrada".into()))?;

    /* Solo el empleado asignado puede solicitar cancelación (admin usa cancel directo) */
    if order.assigned_employee_id != Some(auth.user_id) {
        return Err(AppError::Forbidden(
            "Solo el responsable puede solicitar cancelación".into(),
        ));
    }

    /* Verificar que no haya solicitud pendiente */
    if let Some(_pending) =
        CancellationRequestRepository::find_pending_by_order(&state.pool, order_id).await?
    {
        return Err(AppError::BadRequest(
            "Ya existe una solicitud de cancelación pendiente".into(),
        ));
    }

    /* Crear solicitud */
    let req =
        CancellationRequestRepository::create(&state.pool, order_id, auth.user_id, &body.reason)
            .await?;

    /* Notificar al cliente */
    let notif = CreateNotification {
        user_id: order.client_id,
        notification_type: "cancellation_requested".to_string(),
        title: format!("Solicitud de cancelación — Orden #{}", order.order_number),
        body: Some(format!(
            "El responsable solicita cancelar tu orden. Motivo: {}",
            truncate_str(&body.reason, 120)
        )),
        link: Some(format!("/panel?seccion=proyectos&orden={order_id}")),
        reference_type: Some("order".to_string()),
        reference_id: Some(order_id),
    };
    let _ = state.notification_hub.notify(notif).await;

    /* Registrar en activity_log */
    let _ = sqlx::query!(
        r#"INSERT INTO activity_log (user_id, entity_type, entity_id, action, details)
           VALUES ($1, 'order', $2, 'cancellation_requested', $3)"#,
        auth.user_id,
        order_id,
        serde_json::json!({ "reason": body.reason })
    )
    .execute(&state.pool)
    .await;

    let resp = CancellationRequestResponse {
        id: req.id,
        order_id: req.order_id,
        order_number: Some(order.order_number),
        requested_by: req.requested_by,
        requester_name: None,
        reason: req.reason,
        status: req.status,
        resolved_by: req.resolved_by,
        resolved_at: req.resolved_at.map(|t| t.to_rfc3339()),
        created_at: req.created_at.to_rfc3339(),
    };
    Ok((StatusCode::CREATED, Json(resp)))
}

/* ============================================================
   POST /api/orders/:order_id/cancel-request/:request_id/respond
   — Cliente acepta o rechaza cancelación
   ============================================================ */

#[utoipa::path(
    post,
    path = "/api/orders/{order_id}/cancel-request/{request_id}/respond",
    request_body = RespondCancellationRequest,
    responses(
        (status = 200, description = "Respuesta procesada", body = CancellationRequestResponse),
        (status = 403, description = "Solo el cliente puede responder"),
        (status = 404, description = "Solicitud no encontrada"),
    ),
    params(
        ("order_id" = Uuid, Path, description = "ID de la orden"),
        ("request_id" = Uuid, Path, description = "ID de la solicitud"),
    ),
    security(("bearer" = []))
)]
pub async fn respond_cancellation_request(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((order_id, request_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<RespondCancellationRequest>,
) -> Result<impl IntoResponse, AppError> {
    let order = OrderRepository::find_order_by_id(&state.pool, order_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Orden no encontrada".into()))?;

    /* Solo el cliente dueño de la orden puede responder */
    if order.client_id != auth.user_id {
        return Err(AppError::Forbidden(
            "Solo el cliente puede responder a esta solicitud".into(),
        ));
    }

    let req = CancellationRequestRepository::find_by_id(&state.pool, request_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Solicitud no encontrada".into()))?;

    if req.status != "pending" {
        return Err(AppError::BadRequest(
            "Esta solicitud ya fue resuelta".into(),
        ));
    }

    if req.order_id != order_id {
        return Err(AppError::NotFound("Solicitud no pertenece a esta orden".into()));
    }

    /* Resolver la solicitud */
    let resolved =
        CancellationRequestRepository::resolve(&state.pool, request_id, auth.user_id, body.accept)
            .await?;

    if body.accept {
        handle_cancellation_accepted(&state, &order, order_id, auth.user_id).await;
    } else {
        handle_cancellation_rejected(&state, &order, order_id, auth.user_id).await;
    }

    let resp = CancellationRequestResponse {
        id: resolved.id,
        order_id: resolved.order_id,
        order_number: Some(order.order_number),
        requested_by: resolved.requested_by,
        requester_name: None,
        reason: resolved.reason,
        status: resolved.status,
        resolved_by: resolved.resolved_by,
        resolved_at: resolved.resolved_at.map(|t| t.to_rfc3339()),
        created_at: resolved.created_at.to_rfc3339(),
    };
    Ok(Json(resp))
}

/* ============================================================
   HELPERS - Lógica de aceptación/rechazo de cancelación
   ============================================================ */

async fn handle_cancellation_accepted(
    state: &AppState,
    order: &Order,
    order_id: Uuid,
    resolved_by: Uuid,
) {
    /* Cancelar orden */
    let _ = sqlx::query!(
        "UPDATE orders SET status = 'cancelled', cancelled_at = NOW(), updated_at = NOW() WHERE id = $1",
        order_id
    )
    .execute(&state.pool)
    .await;

    /* Acreditar wallet del cliente */
    let _ = WalletRepository::credit(
        &state.pool,
        order.client_id,
        order.final_price_cents,
        "refund_credit",
        Some("order"),
        Some(order_id),
        Some(&format!("Reembolso por cancelación de orden #{}", order.order_number)),
    )
    .await;

    /* Notificar al empleado */
    if let Some(emp_id) = order.assigned_employee_id {
        let notif = CreateNotification {
            user_id: emp_id,
            notification_type: NOTIF_ORDER_CANCELLED.to_string(),
            title: format!("Orden #{} cancelada", order.order_number),
            body: Some("El cliente aceptó tu solicitud de cancelación.".into()),
            link: None,
            reference_type: Some("order".into()),
            reference_id: Some(order_id),
        };
        let _ = state.notification_hub.notify(notif).await;
    }

    /* Activity log */
    let _ = sqlx::query!(
        r#"INSERT INTO activity_log (user_id, entity_type, entity_id, action, details)
           VALUES ($1, 'order', $2, 'cancellation_accepted', $3)"#,
        resolved_by,
        order_id,
        serde_json::json!({ "refund_cents": order.final_price_cents })
    )
    .execute(&state.pool)
    .await;
}

async fn handle_cancellation_rejected(
    state: &AppState,
    order: &Order,
    order_id: Uuid,
    resolved_by: Uuid,
) {
    /* Orden vuelve a awaiting_assignment sin empleado */
    let _ = sqlx::query!(
        r#"UPDATE orders
           SET status = 'awaiting_assignment',
               assigned_employee_id = NULL,
               assigned_at = NULL,
               open_to_employees = true,
               updated_at = NOW()
           WHERE id = $1"#,
        order_id
    )
    .execute(&state.pool)
    .await;

    /* Notificar al empleado */
    if let Some(emp_id) = order.assigned_employee_id {
        let notif = CreateNotification {
            user_id: emp_id,
            notification_type: "cancellation_rejected".into(),
            title: format!("Cancelación rechazada — Orden #{}", order.order_number),
            body: Some("El cliente rechazó tu solicitud. La orden volverá a estar disponible.".into()),
            link: None,
            reference_type: Some("order".into()),
            reference_id: Some(order_id),
        };
        let _ = state.notification_hub.notify(notif).await;
    }

    /* Notificar a empleados disponibles */
    let employees = sqlx::query_scalar!(
        "SELECT user_id FROM employee_profiles WHERE availability = 'available'"
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    for emp_id in employees {
        let notif = CreateNotification {
            user_id: emp_id,
            notification_type: "order_available".into(),
            title: format!("Orden #{} disponible", order.order_number),
            body: Some("Una orden ha quedado disponible para ser tomada.".into()),
            link: Some("/panel?seccion=disponibles".into()),
            reference_type: Some("order".into()),
            reference_id: Some(order_id),
        };
        let _ = state.notification_hub.notify(notif).await;
    }

    /* Activity log */
    let _ = sqlx::query!(
        r#"INSERT INTO activity_log (user_id, entity_type, entity_id, action, details)
           VALUES ($1, 'order', $2, 'cancellation_rejected', NULL)"#,
        resolved_by,
        order_id,
    )
    .execute(&state.pool)
    .await;
}

/* ============================================================
   GET /api/orders/:order_id/cancel-request — Lista solicitudes de cancelación
   ============================================================ */

#[utoipa::path(
    get,
    path = "/api/orders/{order_id}/cancel-request",
    responses(
        (status = 200, description = "Solicitudes de cancelación", body = Vec<CancellationRequestResponse>),
        (status = 403, description = "Sin permiso"),
        (status = 404, description = "Orden no encontrada"),
    ),
    params(("order_id" = Uuid, Path, description = "ID de la orden")),
    security(("bearer" = []))
)]
pub async fn list_cancellation_requests(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(order_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let order = OrderRepository::find_order_by_id(&state.pool, order_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Orden no encontrada".into()))?;

    /* Solo cliente, empleado asignado o admin pueden ver solicitudes */
    let is_involved = order.client_id == auth.user_id
        || order.assigned_employee_id == Some(auth.user_id)
        || auth.effective_role == crate::models::UserRole::Admin;
    if !is_involved {
        return Err(AppError::Forbidden("Sin permiso para ver solicitudes".into()));
    }

    let requests = CancellationRequestRepository::list_by_order(&state.pool, order_id).await?;
    let resp: Vec<CancellationRequestResponse> = requests
        .into_iter()
        .map(|r| CancellationRequestResponse {
            id: r.id,
            order_id: r.order_id,
            order_number: Some(order.order_number),
            requested_by: r.requested_by,
            requester_name: None,
            reason: r.reason,
            status: r.status,
            resolved_by: r.resolved_by,
            resolved_at: r.resolved_at.map(|t| t.to_rfc3339()),
            created_at: r.created_at.to_rfc3339(),
        })
        .collect();
    Ok(Json(resp))
}

/* ============================================================
   ROUTES
   ============================================================ */

pub fn wallet_routes() -> Router<AppState> {
    Router::new()
        .route("/wallet", get(get_balance))
        .route("/wallet/transactions", get(list_transactions))
}

pub fn cancellation_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/orders/{order_id}/cancel-request",
            get(list_cancellation_requests).post(create_cancellation_request),
        )
        .route(
            "/orders/{order_id}/cancel-request/{request_id}/respond",
            post(respond_cancellation_request),
        )
}

/* Helper para truncar strings en notificaciones */
fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}
