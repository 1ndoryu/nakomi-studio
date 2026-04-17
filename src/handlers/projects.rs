/* [074A-12] Handler CRUD de proyectos/portfolio.
 * Endpoints públicos: listar publicados, detalle por slug.
 * Endpoints admin: listar todos, crear, actualizar, archivar. */

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use uuid::Uuid;
use validator::Validate;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{
    CreateProjectRequest, Project, ProjectResponse, ReorderProjectsRequest,
    UpdateProjectRequest, UserRole,
};
use crate::repositories::{CreateProjectParams, ProjectRepository, UpdateProjectParams};
use crate::AppState;

pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/projects", get(list_published))
        .route("/projects/:slug", get(get_by_slug))
}

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/admin/projects", get(list_all).post(create))
        .route(
            "/admin/projects/:id",
            axum::routing::put(update).delete(archive),
        )
        .route("/admin/projects/:id/destroy", axum::routing::post(destroy))
        .route("/admin/projects/reorder", axum::routing::put(reorder))
}

/// Lista proyectos publicados (público)
#[utoipa::path(
    get,
    path = "/api/projects",
    responses(
        (status = 200, description = "Proyectos publicados", body = Vec<ProjectResponse>)
    )
)]
pub async fn list_published(
    State(state): State<AppState>,
) -> Result<Json<Vec<ProjectResponse>>, AppError> {
    let projects = ProjectRepository::list_published(&state.pool).await?;
    let responses: Vec<ProjectResponse> = projects.into_iter().map(Project::into_response).collect();
    Ok(Json(responses))
}

/// Detalle de proyecto publicado por slug (público)
#[utoipa::path(
    get,
    path = "/api/projects/{slug}",
    params(("slug" = String, Path, description = "Slug del proyecto")),
    responses(
        (status = 200, description = "Proyecto encontrado", body = ProjectResponse),
        (status = 404, description = "Proyecto no encontrado")
    )
)]
pub async fn get_by_slug(
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ProjectResponse>, AppError> {
    let project = ProjectRepository::find_published_by_slug(&state.pool, &slug)
        .await?
        .ok_or(AppError::NotFound("Project not found".into()))?;

    Ok(Json(project.into_response()))
}

/// Lista todos los proyectos (admin, incluyendo borradores/archivados)
#[utoipa::path(
    get,
    path = "/api/admin/projects",
    responses(
        (status = 200, description = "Todos los proyectos", body = Vec<ProjectResponse>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<ProjectResponse>>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let projects = ProjectRepository::list_all(&state.pool).await?;
    let responses: Vec<ProjectResponse> = projects.into_iter().map(Project::into_response).collect();
    Ok(Json(responses))
}

/// Crear nuevo proyecto (admin)
#[utoipa::path(
    post,
    path = "/api/admin/projects",
    request_body = CreateProjectRequest,
    responses(
        (status = 201, description = "Proyecto creado", body = ProjectResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateProjectRequest>,
) -> Result<(axum::http::StatusCode, Json<ProjectResponse>), AppError> {
    auth.require_role(&[UserRole::Admin])?;
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let description = body.description.as_deref().unwrap_or("");
    let status = body.status.as_deref().unwrap_or("published");
    let sort_order = body.sort_order.unwrap_or(0);
    let is_featured = body.is_featured.unwrap_or(true);
    let in_carousel = body.in_carousel.unwrap_or(true);

    let gallery =
        serde_json::to_value(body.gallery.unwrap_or_default())
            .map_err(|e| AppError::Internal(e.to_string()))?;
    let categories =
        serde_json::to_value(body.categories.unwrap_or_default())
            .map_err(|e| AppError::Internal(e.to_string()))?;
    let technologies =
        serde_json::to_value(body.technologies.unwrap_or_default())
            .map_err(|e| AppError::Internal(e.to_string()))?;
    let links =
        serde_json::to_value(body.links.unwrap_or_default())
            .map_err(|e| AppError::Internal(e.to_string()))?;
    let skills =
        serde_json::to_value(body.skills.unwrap_or_default())
            .map_err(|e| AppError::Internal(e.to_string()))?;

    let project = ProjectRepository::create(
        &state.pool,
        &CreateProjectParams {
            title: &body.title,
            slug: &body.slug,
            client: body.client.as_deref(),
            description,
            featured_image: body.featured_image.as_deref(),
            gallery_image: body.gallery_image.as_deref(),
            gallery: &gallery,
            categories: &categories,
            technologies: &technologies,
            links: &links,
            skills: &skills,
            status,
            sort_order,
            is_featured,
            in_carousel,
            showcase_category: body.showcase_category.as_deref(),
            detail_title: body.detail_title.as_deref(),
            use_first_gallery_image: body.use_first_gallery_image.unwrap_or(false),
            meta_title: body.meta_title.as_deref(),
            meta_description: body.meta_description.as_deref(),
        },
    )
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint() == Some("projects_slug_key") {
                return AppError::Conflict(format!("El slug '{}' ya existe", body.slug));
            }
        }
        AppError::from(e)
    })?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(project.into_response()),
    ))
}

/// Actualizar proyecto (admin, parcial)
#[utoipa::path(
    put,
    path = "/api/admin/projects/{id}",
    params(("id" = Uuid, Path, description = "ID del proyecto")),
    request_body = UpdateProjectRequest,
    responses(
        (status = 200, description = "Proyecto actualizado", body = ProjectResponse),
        (status = 404, description = "Proyecto no encontrado")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update(
    auth: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(body): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectResponse>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    /* Verificar que existe */
    let _existing = ProjectRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Project not found".into()))?;

    let gallery = body
        .gallery
        .as_ref()
        .map(|v| serde_json::to_value(v).unwrap_or_default());
    let categories = body
        .categories
        .as_ref()
        .map(|v| serde_json::to_value(v).unwrap_or_default());
    let technologies = body
        .technologies
        .as_ref()
        .map(|v| serde_json::to_value(v).unwrap_or_default());
    let links = body
        .links
        .as_ref()
        .map(|v| serde_json::to_value(v).unwrap_or_default());
    let skills = body
        .skills
        .as_ref()
        .map(|v| serde_json::to_value(v).unwrap_or_default());

    let project = ProjectRepository::update(
        &state.pool,
        id,
        &UpdateProjectParams {
            title: body.title.as_deref(),
            slug: body.slug.as_deref(),
            client: body.client.as_deref(),
            description: body.description.as_deref(),
            featured_image: body.featured_image.as_deref(),
            gallery_image: body.gallery_image.as_deref(),
            gallery: gallery.as_ref(),
            categories: categories.as_ref(),
            technologies: technologies.as_ref(),
            links: links.as_ref(),
            skills: skills.as_ref(),
            status: body.status.as_deref(),
            sort_order: body.sort_order,
            is_featured: body.is_featured,
            in_carousel: body.in_carousel,
            showcase_category: body.showcase_category.as_deref(),
            detail_title: body.detail_title.as_deref(),
            use_first_gallery_image: body.use_first_gallery_image,
            meta_title: body.meta_title.as_deref(),
            meta_description: body.meta_description.as_deref(),
        },
    )
    .await?;

    Ok(Json(project.into_response()))
}

/// Archivar proyecto (admin, soft delete)
#[utoipa::path(
    delete,
    path = "/api/admin/projects/{id}",
    params(("id" = Uuid, Path, description = "ID del proyecto")),
    responses(
        (status = 204, description = "Proyecto archivado"),
        (status = 404, description = "Proyecto no encontrado")
    ),
    security(("bearer_auth" = []))
)]
pub async fn archive(
    auth: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<axum::http::StatusCode, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let _existing = ProjectRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Project not found".into()))?;

    ProjectRepository::archive(&state.pool, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

/* [084A-10] Eliminación permanente de un proyecto */
#[utoipa::path(
    post,
    path = "/api/admin/projects/{id}/destroy",
    params(("id" = Uuid, Path, description = "ID del proyecto")),
    responses(
        (status = 204, description = "Proyecto eliminado permanentemente"),
        (status = 404, description = "Proyecto no encontrado")
    ),
    security(("bearer_auth" = []))
)]
pub async fn destroy(
    auth: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<axum::http::StatusCode, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let deleted = ProjectRepository::hard_delete(&state.pool, id).await?;
    if !deleted {
        return Err(AppError::NotFound("Project not found".into()));
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}

/* [124A-CMS3] Reordenar proyectos en batch (admin).
 * Recibe array de {id, sort_order} y actualiza todos en una sola query. */
#[utoipa::path(
    put,
    path = "/api/admin/projects/reorder",
    request_body = ReorderProjectsRequest,
    responses(
        (status = 204, description = "Proyectos reordenados")
    ),
    security(("bearer_auth" = []))
)]
pub async fn reorder(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<ReorderProjectsRequest>,
) -> Result<axum::http::StatusCode, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let items: Vec<(Uuid, i32)> = body.items.iter().map(|i| (i.id, i.sort_order)).collect();
    ProjectRepository::reorder(&state.pool, &items).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
