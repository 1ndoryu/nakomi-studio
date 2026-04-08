/* [074A-8] Handler CRUD de servicios para panel admin.
 * Endpoints: listar todos (incluyendo inactivos), crear, actualizar, archivar.
 * Solo accesible por admins. Los planes se incluyen en la respuesta de listado. */

use axum::{
    extract::{Path, State},
    routing::{get, put},
    Json, Router,
};
use uuid::Uuid;
use validator::Validate;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{
    AdminServiceResponse, CreateServiceRequest, SaveServicePlansRequest,
    ServicePlanPhaseResponse, ServicePlanResponse, UpdateServiceRequest, UserRole,
};
use crate::repositories::OrderRepository;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/admin/services", get(list_all).post(create))
        .route("/admin/services/:id", put(update).delete(archive))
        .route("/admin/services/:id/destroy", axum::routing::post(destroy))
        .route("/admin/services/:id/plans", put(save_plans))
}

/// Lista todos los servicios (incluyendo inactivos/draft) con sus planes
#[utoipa::path(
    get,
    path = "/api/admin/services",
    responses(
        (status = 200, description = "Lista completa de servicios", body = Vec<AdminServiceResponse>)
    ),
    security(("bearer_auth" = []))
)]
async fn list_all(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<AdminServiceResponse>>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let services = OrderRepository::list_all_services(&state.pool).await?;
    let mut result = Vec::with_capacity(services.len());

    for svc in services {
        let plans = OrderRepository::list_plans_for_service(&state.pool, svc.id).await?;
        let mut plan_responses = Vec::with_capacity(plans.len());

        for plan in plans {
            let phases = OrderRepository::list_plan_phases(&state.pool, plan.id).await?;
            plan_responses.push(ServicePlanResponse {
                id: plan.id,
                slug: plan.slug,
                name: plan.name,
                price_cents: plan.price_cents,
                description: plan.description,
                features: plan.features,
                is_highlighted: plan.is_highlighted,
                is_custom: plan.is_custom,
                phases: phases.into_iter().map(ServicePlanPhaseResponse::from).collect(),
            });
        }

        result.push(AdminServiceResponse {
            id: svc.id,
            slug: svc.slug,
            title: svc.title,
            description: svc.description,
            base_price_cents: svc.base_price_cents,
            currency: svc.currency,
            is_active: svc.is_active,
            sort_order: svc.sort_order,
            image_url: svc.image_url,
            gallery: svc.gallery,
            skills: svc.skills,
            content: svc.content,
            meta_title: svc.meta_title,
            meta_description: svc.meta_description,
            status: svc.status,
            created_at: svc.created_at,
            updated_at: svc.updated_at,
            plans: plan_responses,
        });
    }

    Ok(Json(result))
}

/// Crea un servicio nuevo
#[utoipa::path(
    post,
    path = "/api/admin/services",
    request_body = CreateServiceRequest,
    responses(
        (status = 201, description = "Servicio creado", body = AdminServiceResponse)
    ),
    security(("bearer_auth" = []))
)]
async fn create(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateServiceRequest>,
) -> Result<(axum::http::StatusCode, Json<AdminServiceResponse>), AppError> {
    auth.require_role(&[UserRole::Admin])?;
    body.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let svc = OrderRepository::create_service(&state.pool, &body).await?;

    let response = AdminServiceResponse {
        id: svc.id,
        slug: svc.slug,
        title: svc.title,
        description: svc.description,
        base_price_cents: svc.base_price_cents,
        currency: svc.currency,
        is_active: svc.is_active,
        sort_order: svc.sort_order,
        image_url: svc.image_url,
        gallery: svc.gallery,
        skills: svc.skills,
        content: svc.content,
        meta_title: svc.meta_title,
        meta_description: svc.meta_description,
        status: svc.status,
        created_at: svc.created_at,
        updated_at: svc.updated_at,
        plans: vec![],
    };

    Ok((axum::http::StatusCode::CREATED, Json(response)))
}

/// Actualiza un servicio existente (solo campos proporcionados)
#[utoipa::path(
    put,
    path = "/api/admin/services/{id}",
    params(("id" = Uuid, Path, description = "ID del servicio")),
    request_body = UpdateServiceRequest,
    responses(
        (status = 200, description = "Servicio actualizado", body = AdminServiceResponse)
    ),
    security(("bearer_auth" = []))
)]
async fn update(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateServiceRequest>,
) -> Result<Json<AdminServiceResponse>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    /* Verificar que el servicio existe */
    OrderRepository::find_service_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Servicio no encontrado".into()))?;

    let svc = OrderRepository::update_service(&state.pool, id, &body).await?;

    /* Incluir planes actualizados en la respuesta */
    let plans = OrderRepository::list_plans_for_service(&state.pool, svc.id).await?;
    let mut plan_responses = Vec::with_capacity(plans.len());

    for plan in plans {
        let phases = OrderRepository::list_plan_phases(&state.pool, plan.id).await?;
        plan_responses.push(ServicePlanResponse {
            id: plan.id,
            slug: plan.slug,
            name: plan.name,
            price_cents: plan.price_cents,
            description: plan.description,
            features: plan.features,
            is_highlighted: plan.is_highlighted,
            is_custom: plan.is_custom,
            phases: phases.into_iter().map(ServicePlanPhaseResponse::from).collect(),
        });
    }

    Ok(Json(AdminServiceResponse {
        id: svc.id,
        slug: svc.slug,
        title: svc.title,
        description: svc.description,
        base_price_cents: svc.base_price_cents,
        currency: svc.currency,
        is_active: svc.is_active,
        sort_order: svc.sort_order,
        image_url: svc.image_url,
        gallery: svc.gallery,
        skills: svc.skills,
        content: svc.content,
        meta_title: svc.meta_title,
        meta_description: svc.meta_description,
        status: svc.status,
        created_at: svc.created_at,
        updated_at: svc.updated_at,
        plans: plan_responses,
    }))
}

/// Archiva un servicio (soft delete)
#[utoipa::path(
    delete,
    path = "/api/admin/services/{id}",
    params(("id" = Uuid, Path, description = "ID del servicio")),
    responses(
        (status = 204, description = "Servicio archivado")
    ),
    security(("bearer_auth" = []))
)]
async fn archive(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    OrderRepository::find_service_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Servicio no encontrado".into()))?;

    OrderRepository::archive_service(&state.pool, id).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

/* [074A-66] Guarda (reemplaza) todos los planes de un servicio.
 * Estrategia: DELETE CASCADE + INSERT en transacción — simple y consistente. */
#[utoipa::path(
    put,
    path = "/api/admin/services/{id}/plans",
    params(("id" = Uuid, Path, description = "ID del servicio")),
    request_body = SaveServicePlansRequest,
    responses(
        (status = 200, description = "Planes guardados", body = Vec<ServicePlanResponse>)
    ),
    security(("bearer_auth" = []))
)]
async fn save_plans(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<SaveServicePlansRequest>,
) -> Result<Json<Vec<ServicePlanResponse>>, AppError> {
    auth.require_role(&[UserRole::Admin])?;
    body.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    OrderRepository::find_service_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Servicio no encontrado".into()))?;

    OrderRepository::save_plans_for_service(&state.pool, id, &body.plans).await?;

    let plans = OrderRepository::list_plans_for_service(&state.pool, id).await?;
    let mut responses = Vec::with_capacity(plans.len());
    for plan in plans {
        let phases = OrderRepository::list_plan_phases(&state.pool, plan.id).await?;
        responses.push(ServicePlanResponse {
            id: plan.id,
            slug: plan.slug,
            name: plan.name,
            price_cents: plan.price_cents,
            description: plan.description,
            features: plan.features,
            is_highlighted: plan.is_highlighted,
            is_custom: plan.is_custom,
            phases: phases.into_iter().map(ServicePlanPhaseResponse::from).collect(),
        });
    }

    Ok(Json(responses))
}

/* [084A-10] Eliminación permanente de un servicio.
 * Rechaza con 409 Conflict si existen órdenes que lo referencian. */
#[utoipa::path(
    post,
    path = "/api/admin/services/{id}/destroy",
    params(("id" = Uuid, Path, description = "ID del servicio")),
    responses(
        (status = 204, description = "Servicio eliminado permanentemente"),
        (status = 404, description = "Servicio no encontrado"),
        (status = 409, description = "No se puede eliminar: existen órdenes vinculadas")
    ),
    security(("bearer_auth" = []))
)]
async fn destroy(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    OrderRepository::find_service_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Servicio no encontrado".into()))?;

    if OrderRepository::service_has_orders(&state.pool, id).await? {
        return Err(AppError::Conflict(
            "No se puede eliminar: existen órdenes vinculadas a este servicio".into(),
        ));
    }

    OrderRepository::hard_delete_service(&state.pool, id).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
