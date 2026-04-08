/* [074A-10] Handler CRUD de blog posts.
 * Endpoints públicos: listar publicados (paginado), detalle por slug.
 * Endpoints admin: listar todos, crear, actualizar, archivar. */

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{
    BlogPostResponse, CreateBlogPostRequest, PaginatedBlogPosts, UpdateBlogPostRequest, UserRole,
};
use crate::repositories::{BlogRepository, CreateBlogPostParams, UpdateBlogPostParams};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct BlogPaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/blog", get(list_published))
        .route("/blog/:slug", get(get_by_slug))
}

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/admin/blog", get(list_all).post(create))
        .route("/admin/blog/:id", axum::routing::put(update).delete(archive))
        .route("/admin/blog/:id/destroy", axum::routing::post(destroy))
}

/// Lista posts publicados (público, paginado)
#[utoipa::path(
    get,
    path = "/api/blog",
    params(
        ("page" = Option<i64>, Query, description = "Página (default 1)"),
        ("per_page" = Option<i64>, Query, description = "Posts por página (default 10)")
    ),
    responses(
        (status = 200, description = "Posts publicados paginados", body = PaginatedBlogPosts)
    )
)]
pub async fn list_published(
    Query(params): Query<BlogPaginationParams>,
    State(state): State<AppState>,
) -> Result<Json<PaginatedBlogPosts>, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(10).clamp(1, 50);

    let (posts, total): (Vec<crate::models::BlogPost>, i64) =
        BlogRepository::list_published(&state.pool, page, per_page).await?;

    let mut responses = Vec::with_capacity(posts.len());
    for post in posts {
        let author_id = post.author_id;
        let name = BlogRepository::get_author_name(&state.pool, author_id).await?;
        responses.push(post.into_response(name));
    }

    Ok(Json(PaginatedBlogPosts {
        posts: responses,
        total,
        page,
        per_page,
    }))
}

/// Detalle de post publicado por slug (público)
#[utoipa::path(
    get,
    path = "/api/blog/{slug}",
    params(("slug" = String, Path, description = "Slug del post")),
    responses(
        (status = 200, description = "Post encontrado", body = BlogPostResponse),
        (status = 404, description = "Post no encontrado")
    )
)]
pub async fn get_by_slug(
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<BlogPostResponse>, AppError> {
    let post: crate::models::BlogPost = BlogRepository::find_published_by_slug(&state.pool, &slug)
        .await?
        .ok_or(AppError::NotFound("Blog post not found".into()))?;

    let name = BlogRepository::get_author_name(&state.pool, post.author_id).await?;
    Ok(Json(post.into_response(name)))
}

/// Lista todos los posts (admin, incluyendo borradores/archivados)
#[utoipa::path(
    get,
    path = "/api/admin/blog",
    responses(
        (status = 200, description = "Todos los posts", body = Vec<BlogPostResponse>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<BlogPostResponse>>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let posts: Vec<crate::models::BlogPost> = BlogRepository::list_all(&state.pool).await?;
    let mut responses = Vec::with_capacity(posts.len());
    for post in posts {
        let author_id = post.author_id;
        let name = BlogRepository::get_author_name(&state.pool, author_id).await?;
        responses.push(post.into_response(name));
    }

    Ok(Json(responses))
}

/// Crear nuevo blog post (admin)
#[utoipa::path(
    post,
    path = "/api/admin/blog",
    request_body = CreateBlogPostRequest,
    responses(
        (status = 201, description = "Post creado", body = BlogPostResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateBlogPostRequest>,
) -> Result<(axum::http::StatusCode, Json<BlogPostResponse>), AppError> {
    auth.require_role(&[UserRole::Admin])?;
    body.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let status = body.status.as_deref().unwrap_or("draft");
    let tags = serde_json::to_value(body.tags.unwrap_or_default())
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let post = BlogRepository::create(
        &state.pool,
        &CreateBlogPostParams {
            author_id: auth.user_id,
            title: &body.title,
            slug: &body.slug,
            excerpt: body.excerpt.as_deref(),
            content: &body.content,
            featured_image: body.featured_image.as_deref(),
            status,
            tags: &tags,
            meta_title: body.meta_title.as_deref(),
            meta_description: body.meta_description.as_deref(),
        },
    )
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint() == Some("blog_posts_slug_key") {
                return AppError::Conflict(format!("El slug '{}' ya existe", body.slug));
            }
        }
        AppError::from(e)
    })?;

    let name = BlogRepository::get_author_name(&state.pool, post.author_id).await?;
    Ok((axum::http::StatusCode::CREATED, Json(post.into_response(name))))
}

/// Actualizar blog post (admin, parcial)
#[utoipa::path(
    put,
    path = "/api/admin/blog/{id}",
    params(("id" = Uuid, Path, description = "ID del post")),
    request_body = UpdateBlogPostRequest,
    responses(
        (status = 200, description = "Post actualizado", body = BlogPostResponse),
        (status = 404, description = "Post no encontrado")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update(
    auth: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(body): Json<UpdateBlogPostRequest>,
) -> Result<Json<BlogPostResponse>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    /* Verificar que existe */
    let _existing: crate::models::BlogPost = BlogRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Blog post not found".into()))?;

    let tags = body.tags.as_ref().map(|t| {
        serde_json::to_value(t).unwrap_or_default()
    });

    let post = BlogRepository::update(
        &state.pool,
        id,
        &UpdateBlogPostParams {
            title: body.title.as_deref(),
            slug: body.slug.as_deref(),
            excerpt: body.excerpt.as_deref(),
            content: body.content.as_deref(),
            featured_image: body.featured_image.as_deref(),
            status: body.status.as_deref(),
            tags: tags.as_ref(),
            meta_title: body.meta_title.as_deref(),
            meta_description: body.meta_description.as_deref(),
        },
    )
    .await?;

    let name = BlogRepository::get_author_name(&state.pool, post.author_id).await?;
    Ok(Json(post.into_response(name)))
}

/// Archivar blog post (admin, soft delete)
#[utoipa::path(
    delete,
    path = "/api/admin/blog/{id}",
    params(("id" = Uuid, Path, description = "ID del post")),
    responses(
        (status = 204, description = "Post archivado"),
        (status = 404, description = "Post no encontrado")
    ),
    security(("bearer_auth" = []))
)]
pub async fn archive(
    auth: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<axum::http::StatusCode, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let _existing: crate::models::BlogPost = BlogRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Blog post not found".into()))?;

    BlogRepository::archive(&state.pool, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

/* [084A-10] Eliminación permanente de un blog post */
#[utoipa::path(
    post,
    path = "/api/admin/blog/{id}/destroy",
    params(("id" = Uuid, Path, description = "ID del post")),
    responses(
        (status = 204, description = "Post eliminado permanentemente"),
        (status = 404, description = "Post no encontrado")
    ),
    security(("bearer_auth" = []))
)]
pub async fn destroy(
    auth: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<axum::http::StatusCode, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let deleted = BlogRepository::hard_delete(&state.pool, id).await?;
    if !deleted {
        return Err(AppError::NotFound("Blog post not found".into()));
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}
