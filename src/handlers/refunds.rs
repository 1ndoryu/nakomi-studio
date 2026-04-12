/* [044A-38 Fase 7] Handlers de reembolsos.
 * POST solicitar, PATCH aprobar/rechazar, GET listar pendientes + por orden.
 * Stripe refund automático cuando admin aprueba. */

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{
    CreateNotification, OrderStatus, PaymentStatus, RefundResponse, RefundStatus,
    RequestRefundBody, ReviewAction, ReviewRefundBody, UserRole,
    NOTIF_REFUND_REQUESTED, NOTIF_REFUND_RESOLVED,
};
use crate::repositories::{OrderRepository, PaymentRepository, RefundRepository, UserRepository};
use crate::services::PaymentService;
use crate::AppState;

/* ============================================================
   POST /api/orders/:order_id/refund — Cliente solicita reembolso
   ============================================================ */

#[utoipa::path(
    post,
    path = "/api/orders/{order_id}/refund",
    request_body = RequestRefundBody,
    responses(
        (status = 201, description = "Reembolso solicitado", body = RefundResponse),
        (status = 400, description = "Ya existe solicitud activa o no reembolsable"),
        (status = 404, description = "Orden no encontrada"),
    ),
    params(("order_id" = Uuid, Path, description = "ID de la orden")),
    security(("bearer" = []))
)]
pub async fn request_refund(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(order_id): Path<Uuid>,
    Json(body): Json<RequestRefundBody>,
) -> Result<impl IntoResponse, AppError> {
    let order = OrderRepository::find_order_by_id(&state.pool, order_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Orden no encontrada".into()))?;

    /* Solo el cliente dueño de la orden puede solicitar reembolso */
    if order.client_id != auth.user_id {
        return Err(AppError::Forbidden(
            "No tienes acceso a esta orden".into(),
        ));
    }

    /* Validar que no exista ya una solicitud activa */
    if let Some(existing) =
        RefundRepository::find_active_for_order(&state.pool, order_id).await?
    {
        return Err(AppError::BadRequest(format!(
            "Ya existe una solicitud de reembolso activa (status: {:?})",
            existing.status
        )));
    }

    /* Buscar un pago reembolsable (held o released) */
    let payments = PaymentRepository::list_for_order(&state.pool, order_id).await?;
    let refundable_payment = payments
        .iter()
        .find(|p| p.status == PaymentStatus::Held || p.status == PaymentStatus::Released)
        .ok_or_else(|| {
            AppError::BadRequest("No hay pagos reembolsables para esta orden".into())
        })?;

    let refund = RefundRepository::create(
        &state.pool,
        order_id,
        refundable_payment.id,
        auth.user_id,
        refundable_payment.amount_cents,
        &body.reason,
    )
    .await?;

    /* [104A-38] Notificar a admins sobre solicitud de reembolso */
    let admins = UserRepository::admin_ids(&state.pool).await.unwrap_or_default();
    let base = CreateNotification {
        user_id: Uuid::nil(),
        notification_type: NOTIF_REFUND_REQUESTED.to_string(),
        title: format!("Reembolso solicitado — Orden #{}", order.order_number),
        body: Some(body.reason.chars().take(100).collect()),
        link: Some(format!("/panel?seccion=reembolsos&id={}", refund.id)),
        reference_type: Some("refund".to_string()),
        reference_id: Some(refund.id),
    };
    let _ = state.notification_hub.notify_many(&admins, &base).await;

    Ok((StatusCode::CREATED, Json(RefundResponse::from(refund))))
}

/* ============================================================
   PATCH /api/refunds/:refund_id — Admin aprueba o rechaza
   ============================================================ */

#[utoipa::path(
    patch,
    path = "/api/refunds/{refund_id}",
    request_body = ReviewRefundBody,
    responses(
        (status = 200, description = "Reembolso actualizado", body = RefundResponse),
        (status = 400, description = "Acción no válida para el estado actual"),
        (status = 404, description = "Reembolso no encontrado"),
    ),
    params(("refund_id" = Uuid, Path, description = "ID del reembolso")),
    security(("bearer" = []))
)]
pub async fn review_refund(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(refund_id): Path<Uuid>,
    Json(body): Json<ReviewRefundBody>,
) -> Result<Json<RefundResponse>, AppError> {
    /* Solo admin puede revisar reembolsos */
    auth.require_role(&[UserRole::Admin])?;

    let refund = RefundRepository::find_by_id(&state.pool, refund_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Reembolso no encontrado".into()))?;

    if refund.status != RefundStatus::Requested && refund.status != RefundStatus::UnderReview {
        return Err(AppError::BadRequest(
            "El reembolso no está en un estado revisable".into(),
        ));
    }

    match body.action {
        ReviewAction::Approve => {
            /* Aprobar → ejecutar refund en Stripe → marcar completado */
            let approved = RefundRepository::approve(
                &state.pool,
                refund_id,
                auth.user_id,
                body.admin_response.as_deref(),
            )
            .await?;

            /* Buscar el pago asociado para ejecutar el refund en Stripe */
            let payments =
                PaymentRepository::list_for_order(&state.pool, refund.order_id).await?;
            let payment = payments
                .iter()
                .find(|p| p.id == refund.payment_id)
                .ok_or_else(|| {
                    AppError::Internal("Pago asociado al reembolso no encontrado".into())
                })?;

            /* Intentar refund en Stripe (si hay key configurada) */
            let stripe_refund_id = match state.stripe_secret_key.as_ref() {
                None => {
                    /* Sin Stripe configurado, simular refund */
                    tracing::warn!("Stripe no configurado, simulando refund");
                    format!("simulated_refund_{refund_id}")
                }
                Some(key) => {
                    PaymentService::refund_payment(
                        &state.http_client,
                        key,
                        payment,
                    )
                    .await?
                }
            };

            /* Marcar pago como refunded */
            PaymentRepository::update_status(
                &state.pool,
                payment.id,
                PaymentStatus::Refunded,
            )
            .await?;

            /* Marcar reembolso como completado */
            let completed = RefundRepository::mark_completed(
                &state.pool,
                approved.id,
                &stripe_refund_id,
            )
            .await?;

            /* Cancelar la orden */
            OrderRepository::update_order_status(
                &state.pool,
                refund.order_id,
                OrderStatus::Cancelled,
            )
            .await?;

            /* [104A-38] Notificar al cliente que su reembolso fue aprobado */
            let _ = state.notification_hub.notify(CreateNotification {
                user_id: refund.requested_by,
                notification_type: NOTIF_REFUND_RESOLVED.to_string(),
                title: "Reembolso aprobado".to_string(),
                body: Some("Tu solicitud de reembolso ha sido procesada".to_string()),
                link: Some(format!("/panel?seccion=ordenes&id={}", refund.order_id)),
                reference_type: Some("refund".to_string()),
                reference_id: Some(refund.id),
            }).await;

            Ok(Json(RefundResponse::from(completed)))
        }
        ReviewAction::Reject => {
            let rejected = RefundRepository::reject(
                &state.pool,
                refund_id,
                auth.user_id,
                body.admin_response.as_deref(),
            )
            .await?;

            /* [104A-38] Notificar al cliente que su reembolso fue rechazado */
            let _ = state.notification_hub.notify(CreateNotification {
                user_id: refund.requested_by,
                notification_type: NOTIF_REFUND_RESOLVED.to_string(),
                title: "Reembolso rechazado".to_string(),
                body: body.admin_response.as_deref().map(|r| r.chars().take(100).collect()),
                link: Some(format!("/panel?seccion=ordenes&id={}", refund.order_id)),
                reference_type: Some("refund".to_string()),
                reference_id: Some(refund.id),
            }).await;

            Ok(Json(RefundResponse::from(rejected)))
        }
    }
}

/* ============================================================
   GET /api/refunds — Admin: listar pendientes
   ============================================================ */

#[utoipa::path(
    get,
    path = "/api/refunds",
    responses(
        (status = 200, description = "Lista de reembolsos", body = Vec<RefundResponse>),
    ),
    security(("bearer" = []))
)]
pub async fn list_refunds(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<RefundResponse>>, AppError> {
    let refunds = if auth.role == UserRole::Admin {
        RefundRepository::list_pending(&state.pool).await?
    } else {
        RefundRepository::list_for_user(&state.pool, auth.user_id).await?
    };

    let response: Vec<RefundResponse> = refunds.into_iter().map(RefundResponse::from).collect();
    Ok(Json(response))
}

/* ============================================================
   GET /api/orders/:order_id/refund — Ver reembolso de una orden
   ============================================================ */

#[utoipa::path(
    get,
    path = "/api/orders/{order_id}/refund",
    responses(
        (status = 200, description = "Reembolso de la orden", body = RefundResponse),
        (status = 404, description = "No hay reembolso para esta orden"),
    ),
    params(("order_id" = Uuid, Path, description = "ID de la orden")),
    security(("bearer" = []))
)]
pub async fn get_order_refund(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(order_id): Path<Uuid>,
) -> Result<Json<RefundResponse>, AppError> {
    let order = OrderRepository::find_order_by_id(&state.pool, order_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Orden no encontrada".into()))?;

    /* Solo el dueño o admin puede ver el reembolso */
    if order.client_id != auth.user_id && auth.role != UserRole::Admin {
        return Err(AppError::Forbidden(
            "No tienes acceso a esta orden".into(),
        ));
    }

    let refund = RefundRepository::find_for_order(&state.pool, order_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound("No hay reembolso para esta orden".into())
        })?;

    Ok(Json(RefundResponse::from(refund)))
}

/* ============================================================
   RUTAS
   ============================================================ */

pub fn routes() -> Router<AppState> {
    /* [074A-49] Rutas sin /api/ porque ya se nestan bajo .nest("/api", api_routes()) */
    Router::new()
        .route("/orders/:order_id/refund", post(request_refund).get(get_order_refund))
        .route("/refunds", get(list_refunds))
        .route("/refunds/:refund_id", patch(review_refund))
}
