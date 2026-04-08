/* [084A-7] Endpoints públicos de perfil de usuario estilo Fiverr.
 * Sin autenticación — accesibles por cualquier visitante.
 * Endpoints: perfil por username, reviews recibidas, reviews dadas, distribución de ratings. */

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::errors::AppError;
use crate::models::{
    PaginatedPublicReviews, PublicReviewItem, PublicUserProfile, RatingDistribution,
};
use crate::repositories::PublicProfileRepository;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ReviewPaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/users/:username", get(get_profile))
        .route("/users/:username/reviews/received", get(get_reviews_received))
        .route("/users/:username/reviews/given", get(get_reviews_given))
        .route("/users/:username/ratings", get(get_rating_distribution))
}

/// Perfil público de un usuario
#[utoipa::path(
    get,
    path = "/api/users/{username}",
    params(("username" = String, Path, description = "Username del usuario")),
    responses(
        (status = 200, description = "Perfil público", body = PublicUserProfile),
        (status = 404, description = "Usuario no encontrado")
    )
)]
async fn get_profile(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<PublicUserProfile>, AppError> {
    let row = PublicProfileRepository::find_by_username(&state.pool, &username)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Usuario '{username}' no encontrado")))?;

    Ok(Json(PublicUserProfile::from(row)))
}

/// Reviews recibidas como empleado (paginado)
#[utoipa::path(
    get,
    path = "/api/users/{username}/reviews/received",
    params(
        ("username" = String, Path, description = "Username del usuario"),
        ("page" = Option<i64>, Query, description = "Página (default 1)"),
        ("per_page" = Option<i64>, Query, description = "Reviews por página (default 10)")
    ),
    responses(
        (status = 200, description = "Reviews recibidas paginadas", body = PaginatedPublicReviews),
        (status = 404, description = "Usuario no encontrado")
    )
)]
async fn get_reviews_received(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(params): Query<ReviewPaginationParams>,
) -> Result<Json<PaginatedPublicReviews>, AppError> {
    let user_id = PublicProfileRepository::get_user_id_by_username(&state.pool, &username)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Usuario '{username}' no encontrado")))?;

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(10).clamp(1, 50);
    let offset = (page - 1) * per_page;

    let (rows, total) = PublicProfileRepository::list_reviews_received(
        &state.pool, user_id, per_page, offset,
    ).await?;

    let reviews: Vec<PublicReviewItem> = rows.into_iter().map(|r| PublicReviewItem {
        id: r.id.to_string(),
        rating: r.rating,
        comment: r.comment,
        employee_response: r.employee_response,
        author_name: r.client_name,
        author_avatar: r.client_avatar,
        author_username: r.client_username,
        service_title: r.service_title,
        created_at: r.created_at.to_rfc3339(),
    }).collect();

    Ok(Json(PaginatedPublicReviews { reviews, total, page, per_page }))
}

/// Reviews dadas como cliente (paginado)
#[utoipa::path(
    get,
    path = "/api/users/{username}/reviews/given",
    params(
        ("username" = String, Path, description = "Username del usuario"),
        ("page" = Option<i64>, Query, description = "Página (default 1)"),
        ("per_page" = Option<i64>, Query, description = "Reviews por página (default 10)")
    ),
    responses(
        (status = 200, description = "Reviews dadas paginadas", body = PaginatedPublicReviews),
        (status = 404, description = "Usuario no encontrado")
    )
)]
async fn get_reviews_given(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(params): Query<ReviewPaginationParams>,
) -> Result<Json<PaginatedPublicReviews>, AppError> {
    let user_id = PublicProfileRepository::get_user_id_by_username(&state.pool, &username)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Usuario '{username}' no encontrado")))?;

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(10).clamp(1, 50);
    let offset = (page - 1) * per_page;

    let (rows, total) = PublicProfileRepository::list_reviews_given(
        &state.pool, user_id, per_page, offset,
    ).await?;

    let reviews: Vec<PublicReviewItem> = rows.into_iter().map(|r| PublicReviewItem {
        id: r.id.to_string(),
        rating: r.rating,
        comment: r.comment,
        employee_response: r.employee_response,
        author_name: r.employee_name,
        author_avatar: r.employee_avatar,
        author_username: r.employee_username,
        service_title: r.service_title,
        created_at: r.created_at.to_rfc3339(),
    }).collect();

    Ok(Json(PaginatedPublicReviews { reviews, total, page, per_page }))
}

/// Distribución de ratings (estrellas 1-5) de un usuario como empleado
#[utoipa::path(
    get,
    path = "/api/users/{username}/ratings",
    params(("username" = String, Path, description = "Username del usuario")),
    responses(
        (status = 200, description = "Distribución de ratings", body = RatingDistribution),
        (status = 404, description = "Usuario no encontrado")
    )
)]
async fn get_rating_distribution(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<RatingDistribution>, AppError> {
    let user_id = PublicProfileRepository::get_user_id_by_username(&state.pool, &username)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Usuario '{username}' no encontrado")))?;

    let distribution = PublicProfileRepository::rating_distribution(&state.pool, user_id).await?;
    Ok(Json(distribution))
}
