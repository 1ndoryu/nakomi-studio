/* [074A-13] Handler CRUD de miembros del equipo.
 * Endpoints públicos: listar publicados.
 * Endpoints admin: listar todos, crear, actualizar, archivar. */

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use axum::http::StatusCode;
use uuid::Uuid;
use validator::Validate;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{
    CreateTeamMemberRequest, TeamMember, TeamMemberResponse, UpdateTeamMemberRequest, UserRole,
};
use crate::repositories::{CreateTeamMemberParams, TeamMemberRepository, UpdateTeamMemberParams};
use crate::AppState;

pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/team", get(list_published))
}

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/admin/team", get(list_all).post(create))
        .route(
            "/admin/team/:id",
            axum::routing::put(update).delete(archive),
        )
        .route("/admin/team/:id/destroy", axum::routing::post(destroy))
}

/// Lista miembros del equipo publicados (público)
#[utoipa::path(
    get,
    path = "/api/team",
    responses(
        (status = 200, description = "Miembros publicados", body = Vec<TeamMemberResponse>)
    )
)]
pub async fn list_published(
    State(state): State<AppState>,
) -> Result<Json<Vec<TeamMemberResponse>>, AppError> {
    let members = TeamMemberRepository::list_published(&state.pool).await?;
    let responses: Vec<TeamMemberResponse> = members.into_iter().map(TeamMember::into_response).collect();
    Ok(Json(responses))
}

/// Lista todos los miembros (admin)
#[utoipa::path(
    get,
    path = "/api/admin/team",
    responses(
        (status = 200, description = "Todos los miembros", body = Vec<TeamMemberResponse>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    _user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<TeamMemberResponse>>, AppError> {
    let members = TeamMemberRepository::list_all(&state.pool).await?;
    let responses: Vec<TeamMemberResponse> = members.into_iter().map(TeamMember::into_response).collect();
    Ok(Json(responses))
}

/// Crear miembro del equipo (admin)
#[utoipa::path(
    post,
    path = "/api/admin/team",
    request_body = CreateTeamMemberRequest,
    responses(
        (status = 201, description = "Miembro creado", body = TeamMemberResponse),
        (status = 400, description = "Datos inválidos"),
        (status = 403, description = "Sin permisos")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateTeamMemberRequest>,
) -> Result<(StatusCode, Json<TeamMemberResponse>), AppError> {
    if user.role != UserRole::Admin {
        return Err(AppError::Forbidden("Solo admin puede crear miembros".into()));
    }
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let params = CreateTeamMemberParams {
        name: &body.name,
        slug: &body.slug,
        role: body.role.as_deref().unwrap_or(""),
        bio: body.bio.as_deref().unwrap_or(""),
        avatar: body.avatar.as_deref(),
        linkedin: body.linkedin.as_deref(),
        twitter: body.twitter.as_deref(),
        github: body.github.as_deref(),
        status: body.status.as_deref().unwrap_or("published"),
        sort_order: body.sort_order.unwrap_or(0),
    };

    let member = TeamMemberRepository::create(&state.pool, params).await?;
    Ok((StatusCode::CREATED, Json(member.into_response())))
}

/// Actualizar miembro (admin, parcial via COALESCE)
#[utoipa::path(
    put,
    path = "/api/admin/team/{id}",
    params(("id" = Uuid, Path, description = "ID del miembro")),
    request_body = UpdateTeamMemberRequest,
    responses(
        (status = 200, description = "Miembro actualizado", body = TeamMemberResponse),
        (status = 404, description = "Miembro no encontrado"),
        (status = 403, description = "Sin permisos")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update(
    user: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(body): Json<UpdateTeamMemberRequest>,
) -> Result<Json<TeamMemberResponse>, AppError> {
    if user.role != UserRole::Admin {
        return Err(AppError::Forbidden("Solo admin puede editar miembros".into()));
    }

    let params = UpdateTeamMemberParams {
        name: body.name.as_deref(),
        slug: body.slug.as_deref(),
        role: body.role.as_deref(),
        bio: body.bio.as_deref(),
        avatar: body.avatar.as_deref(),
        linkedin: body.linkedin.as_deref(),
        twitter: body.twitter.as_deref(),
        github: body.github.as_deref(),
        status: body.status.as_deref(),
        sort_order: body.sort_order,
    };

    let member = TeamMemberRepository::update(&state.pool, id, params)
        .await?
        .ok_or(AppError::NotFound("Team member not found".into()))?;

    Ok(Json(member.into_response()))
}

/// Archivar miembro (soft delete, admin)
#[utoipa::path(
    delete,
    path = "/api/admin/team/{id}",
    params(("id" = Uuid, Path, description = "ID del miembro")),
    responses(
        (status = 204, description = "Miembro archivado"),
        (status = 404, description = "Miembro no encontrado"),
        (status = 403, description = "Sin permisos")
    ),
    security(("bearer_auth" = []))
)]
pub async fn archive(
    user: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    if user.role != UserRole::Admin {
        return Err(AppError::Forbidden("Solo admin puede archivar miembros".into()));
    }

    let archived = TeamMemberRepository::archive(&state.pool, id).await?;
    if archived {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Team member not found or already archived".into()))
    }
}

/* [084A-10] Eliminación permanente de un miembro del equipo */
#[utoipa::path(
    post,
    path = "/api/admin/team/{id}/destroy",
    params(("id" = Uuid, Path, description = "ID del miembro")),
    responses(
        (status = 204, description = "Miembro eliminado permanentemente"),
        (status = 404, description = "Miembro no encontrado")
    ),
    security(("bearer_auth" = []))
)]
pub async fn destroy(
    user: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    if user.role != UserRole::Admin {
        return Err(AppError::Forbidden("Solo admin puede eliminar miembros".into()));
    }

    let deleted = TeamMemberRepository::hard_delete(&state.pool, id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Team member not found".into()))
    }
}
