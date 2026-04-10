/* [044A-38 Fase 8] Handlers de reviews.
 * POST crear review, POST responder, GET por orden, GET listar por empleado.
 * Al crear review, actualiza average_rating del empleado. */

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;
use validator::Validate;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{
    CreateNotification, CreateReviewBody, OrderStatus, RespondReviewBody, ReviewResponse, UserRole,
    NOTIF_NEW_REVIEW, NOTIF_REVIEW_RESPONSE,
};
use crate::repositories::{OrderRepository, ReviewRepository};
use crate::AppState;

/* ============================================================
   POST /api/orders/:order_id/review — Cliente deja review
   ============================================================ */

#[utoipa::path(
    post,
    path = "/api/orders/{order_id}/review",
    request_body = CreateReviewBody,
    responses(
        (status = 201, description = "Review creada", body = ReviewResponse),
        (status = 400, description = "Orden no completada o ya tiene review"),
        (status = 403, description = "No es el dueño de la orden"),
    ),
    params(("order_id" = Uuid, Path, description = "ID de la orden")),
    security(("bearer" = []))
)]
pub async fn create_review(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(order_id): Path<Uuid>,
    Json(body): Json<CreateReviewBody>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::BadRequest(format!("Validación fallida: {e}")))?;

    let order = OrderRepository::find_order_by_id(&state.pool, order_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Orden no encontrada".into()))?;

    /* Solo el cliente dueño puede dejar review */
    if order.client_id != auth.user_id {
        return Err(AppError::Forbidden("No tienes acceso a esta orden".into()));
    }

    /* Solo órdenes completadas pueden recibir review */
    if order.status != OrderStatus::Completed {
        return Err(AppError::BadRequest(
            "Solo puedes dejar review en órdenes completadas".into(),
        ));
    }

    let employee_id = order.assigned_employee_id.ok_or_else(|| {
        AppError::BadRequest("La orden no tiene empleado asignado".into())
    })?;

    let review = ReviewRepository::create(
        &state.pool,
        order_id,
        auth.user_id,
        employee_id,
        body.rating,
        body.comment.as_deref(),
    )
    .await?;

    /* Actualizar average_rating del empleado */
    ReviewRepository::update_employee_average(&state.pool, employee_id).await?;

    /* [104A-38] Notificar al empleado que recibió una review */
    let _ = state.notification_hub.notify(CreateNotification {
        user_id: employee_id,
        notification_type: NOTIF_NEW_REVIEW.to_string(),
        title: format!("Nueva review en orden #{}", order.order_number),
        body: Some(format!("Rating: {}/5", body.rating)),
        link: Some(format!("/panel/orders/{order_id}")),
        reference_type: Some("order".to_string()),
        reference_id: Some(order_id),
    }).await;

    Ok((StatusCode::CREATED, Json(ReviewResponse::from(review))))
}

/* ============================================================
   POST /api/reviews/:review_id/respond — Empleado responde
   ============================================================ */

#[utoipa::path(
    post,
    path = "/api/reviews/{review_id}/respond",
    request_body = RespondReviewBody,
    responses(
        (status = 200, description = "Respuesta guardada", body = ReviewResponse),
        (status = 403, description = "No es el empleado de esta review"),
        (status = 404, description = "Review no encontrada"),
    ),
    params(("review_id" = Uuid, Path, description = "ID de la review")),
    security(("bearer" = []))
)]
pub async fn respond_review(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(review_id): Path<Uuid>,
    Json(body): Json<RespondReviewBody>,
) -> Result<Json<ReviewResponse>, AppError> {
    body.validate()
        .map_err(|e| AppError::BadRequest(format!("Validación fallida: {e}")))?;

    /* Buscar la review por ID — necesitamos iterar o añadir find_by_id.
     * Como no hay find_by_id directo, consultamos por order y filtramos.
     * Alternativa: query directa aquí. */
    let review = sqlx::query_as!(
        crate::models::OrderReview,
        r#"SELECT id, order_id, client_id, employee_id, rating, comment,
                  employee_response, employee_responded_at, created_at
           FROM order_reviews WHERE id = $1"#,
        review_id,
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Error buscando review: {e}")))?
    .ok_or_else(|| AppError::NotFound("Review no encontrada".into()))?;

    /* Solo el empleado de la review puede responder */
    if review.employee_id != auth.user_id {
        return Err(AppError::Forbidden(
            "Solo el empleado asignado puede responder".into(),
        ));
    }

    if review.employee_response.is_some() {
        return Err(AppError::BadRequest(
            "Ya respondiste a esta review".into(),
        ));
    }

    let updated = ReviewRepository::respond(&state.pool, review_id, &body.response).await?;

    /* [104A-38] Notificar al cliente que el empleado respondió su review */
    let _ = state.notification_hub.notify(CreateNotification {
        user_id: review.client_id,
        notification_type: NOTIF_REVIEW_RESPONSE.to_string(),
        title: "Respuesta a tu review".to_string(),
        body: Some(body.response.chars().take(100).collect::<String>()),
        link: Some(format!("/panel/orders/{}", review.order_id)),
        reference_type: Some("order".to_string()),
        reference_id: Some(review.order_id),
    }).await;

    Ok(Json(ReviewResponse::from(updated)))
}

/* ============================================================
   GET /api/orders/:order_id/review — Ver review de una orden
   ============================================================ */

#[utoipa::path(
    get,
    path = "/api/orders/{order_id}/review",
    responses(
        (status = 200, description = "Review de la orden", body = ReviewResponse),
        (status = 404, description = "No hay review para esta orden"),
    ),
    params(("order_id" = Uuid, Path, description = "ID de la orden")),
    security(("bearer" = []))
)]
pub async fn get_order_review(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(order_id): Path<Uuid>,
) -> Result<Json<ReviewResponse>, AppError> {
    let order = OrderRepository::find_order_by_id(&state.pool, order_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Orden no encontrada".into()))?;

    /* Dueño, empleado asignado o admin */
    let is_owner = order.client_id == auth.user_id;
    let is_assigned = order.assigned_employee_id == Some(auth.user_id);
    let is_admin = auth.role == UserRole::Admin;
    if !is_owner && !is_assigned && !is_admin {
        return Err(AppError::Forbidden("No tienes acceso a esta orden".into()));
    }

    let review = ReviewRepository::find_by_order(&state.pool, order_id)
        .await?
        .ok_or_else(|| AppError::NotFound("No hay review para esta orden".into()))?;

    Ok(Json(ReviewResponse::from(review)))
}

/* ============================================================
   GET /api/reviews — Admin: todas, Employee: propias
   ============================================================ */

#[utoipa::path(
    get,
    path = "/api/reviews",
    responses(
        (status = 200, description = "Lista de reviews", body = Vec<ReviewResponse>),
    ),
    security(("bearer" = []))
)]
pub async fn list_reviews(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<ReviewResponse>>, AppError> {
    let reviews = if auth.role == UserRole::Admin {
        ReviewRepository::list_all(&state.pool).await?
    } else {
        ReviewRepository::list_by_employee(&state.pool, auth.user_id).await?
    };

    let response: Vec<ReviewResponse> = reviews.into_iter().map(ReviewResponse::from).collect();
    Ok(Json(response))
}

pub fn routes() -> Router<AppState> {
    /* [074A-49] Sin /api/ — ya se nestan bajo .nest("/api", api_routes()) */
    Router::new()
        .route("/orders/:order_id/review", post(create_review).get(get_order_review))
        .route("/reviews/:review_id/respond", post(respond_review))
        .route("/reviews", get(list_reviews))
}
