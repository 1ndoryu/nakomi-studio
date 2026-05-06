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
    parse_service_categories, AdminServiceResponse, CreateServiceRequest, ReorderRequest,
    SaveServicePlansRequest, ServicePlanPhaseResponse, ServicePlanResponse, UpdateServiceRequest,
    UserRole,
};
use crate::repositories::{ServiceRepository, UpdateServiceParams};
use crate::AppState;

/* [045A-2] Los writes de services reportan conflictos/validaciones comunes como 4xx.
 * El CMS deja de ver un 500 genérico ante slugs duplicados o campos truncados. */
fn map_service_write_error(error: sqlx::Error, slug: Option<&str>) -> AppError {
    if let sqlx::Error::Database(ref db_err) = error {
        match db_err.code().as_deref() {
            Some("23505") if db_err.constraint() == Some("services_slug_key") => {
                let slug = slug.unwrap_or("slug");
                return AppError::Conflict(format!("El slug '{slug}' ya existe"));
            }
            Some("22001") => {
                return AppError::Validation(
                    "Uno de los campos del servicio excede la longitud permitida".into(),
                );
            }
            Some("23514") => {
                return AppError::Validation(
                    "Los datos del servicio no cumplen las restricciones esperadas".into(),
                );
            }
            _ => {}
        }
    }

    AppError::from(error)
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/admin/services", get(list_all).post(create))
        .route("/admin/services/reorder", axum::routing::put(reorder))
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

    let services = ServiceRepository::list_all_services(&state.pool).await?;
    let mut result = Vec::with_capacity(services.len());

    for svc in services {
        let plans = ServiceRepository::list_plans_for_service(&state.pool, svc.id).await?;
        let mut plan_responses = Vec::with_capacity(plans.len());

        for plan in plans {
            let phases = ServiceRepository::list_plan_phases(&state.pool, plan.id).await?;
            plan_responses.push(ServicePlanResponse {
                id: plan.id,
                slug: plan.slug,
                name: plan.name,
                price_cents: plan.price_cents,
                description: plan.description,
                features: plan.features,
                is_highlighted: plan.is_highlighted,
                is_custom: plan.is_custom,
                phases: phases
                    .into_iter()
                    .map(ServicePlanPhaseResponse::from)
                    .collect(),
            });
        }

        result.push(AdminServiceResponse {
            id: svc.id,
            slug: svc.slug,
            title: svc.title,
            description: svc.description,
            categories: parse_service_categories(&svc.categories),
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
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let svc = ServiceRepository::create_service(&state.pool, &body)
        .await
        .map_err(|error| map_service_write_error(error, Some(body.slug.as_str())))?;

    let response = AdminServiceResponse {
        id: svc.id,
        slug: svc.slug,
        title: svc.title,
        description: svc.description,
        categories: parse_service_categories(&svc.categories),
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
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    /* Verificar que el servicio existe */
    ServiceRepository::find_service_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Servicio no encontrado".into()))?;

    let categories_json = body
        .categories
        .as_ref()
        .map(|categories| serde_json::json!(categories));

    let params = UpdateServiceParams {
        title: body.title.as_deref(),
        slug: body.slug.as_deref(),
        description: body.description.as_deref(),
        categories: categories_json.as_ref(),
        base_price_cents: body.base_price_cents,
        currency: body.currency.as_deref(),
        is_active: body.is_active,
        image_url: body.image_url.as_deref(),
        gallery: body.gallery.as_ref(),
        skills: body.skills.as_ref(),
        content: body.content.as_deref(),
        meta_title: body.meta_title.as_deref(),
        meta_description: body.meta_description.as_deref(),
        status: body.status.as_deref(),
        sort_order: body.sort_order,
    };

    let svc = ServiceRepository::update_service(&state.pool, id, &params)
        .await
        .map_err(|error| map_service_write_error(error, body.slug.as_deref()))?;

    /* Incluir planes actualizados en la respuesta */
    let plans = ServiceRepository::list_plans_for_service(&state.pool, svc.id).await?;
    let mut plan_responses = Vec::with_capacity(plans.len());

    for plan in plans {
        let phases = ServiceRepository::list_plan_phases(&state.pool, plan.id).await?;
        plan_responses.push(ServicePlanResponse {
            id: plan.id,
            slug: plan.slug,
            name: plan.name,
            price_cents: plan.price_cents,
            description: plan.description,
            features: plan.features,
            is_highlighted: plan.is_highlighted,
            is_custom: plan.is_custom,
            phases: phases
                .into_iter()
                .map(ServicePlanPhaseResponse::from)
                .collect(),
        });
    }

    Ok(Json(AdminServiceResponse {
        id: svc.id,
        slug: svc.slug,
        title: svc.title,
        description: svc.description,
        categories: parse_service_categories(&svc.categories),
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

    ServiceRepository::find_service_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Servicio no encontrado".into()))?;

    ServiceRepository::archive_service(&state.pool, id).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

/* [074A-66] Guarda (reemplaza) todos los planes de un servicio.
 * Estrategia: DELETE CASCADE + INSERT en transacciÃ³n â€” simple y consistente. */
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
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    /* [035A-10] El checkout público siempre ofrece pago por fases para servicios.
     * Si el CMS guarda un plan sin fases, la orden queda sin estructura de trabajo.
     * Se corta en el boundary admin para que el catálogo siga siendo fuente de verdad. */
    if let Some(plan) = body.plans.iter().find(|plan| plan.phases.is_empty()) {
        return Err(AppError::Validation(format!(
            "El plan '{}' debe tener al menos una fase configurada",
            plan.name
        )));
    }

    ServiceRepository::find_service_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Servicio no encontrado".into()))?;

    ServiceRepository::save_plans_for_service(&state.pool, id, &body.plans).await?;

    let plans = ServiceRepository::list_plans_for_service(&state.pool, id).await?;
    let mut responses = Vec::with_capacity(plans.len());
    for plan in plans {
        let phases = ServiceRepository::list_plan_phases(&state.pool, plan.id).await?;
        responses.push(ServicePlanResponse {
            id: plan.id,
            slug: plan.slug,
            name: plan.name,
            price_cents: plan.price_cents,
            description: plan.description,
            features: plan.features,
            is_highlighted: plan.is_highlighted,
            is_custom: plan.is_custom,
            phases: phases
                .into_iter()
                .map(ServicePlanPhaseResponse::from)
                .collect(),
        });
    }

    Ok(Json(responses))
}

/* [124A-CMS10] Reordenar servicios en batch */
#[utoipa::path(
    put,
    path = "/api/admin/services/reorder",
    request_body = ReorderRequest,
    responses(
        (status = 204, description = "Servicios reordenados")
    ),
    security(("bearer_auth" = []))
)]
async fn reorder(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<ReorderRequest>,
) -> Result<axum::http::StatusCode, AppError> {
    auth.require_role(&[UserRole::Admin])?;
    let items: Vec<(uuid::Uuid, i32)> = body.items.iter().map(|i| (i.id, i.sort_order)).collect();
    ServiceRepository::reorder_services(&state.pool, &items).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

/* [084A-10] EliminaciÃ³n permanente de un servicio.
 * Rechaza con 409 Conflict si existen Ã³rdenes que lo referencian. */
#[utoipa::path(
    post,
    path = "/api/admin/services/{id}/destroy",
    params(("id" = Uuid, Path, description = "ID del servicio")),
    responses(
        (status = 204, description = "Servicio eliminado permanentemente"),
        (status = 404, description = "Servicio no encontrado"),
        (status = 409, description = "No se puede eliminar: existen Ã³rdenes vinculadas")
    ),
    security(("bearer_auth" = []))
)]
async fn destroy(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    ServiceRepository::find_service_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Servicio no encontrado".into()))?;

    if ServiceRepository::service_has_orders(&state.pool, id).await? {
        return Err(AppError::Conflict(
            "No se puede eliminar: existen Ã³rdenes vinculadas a este servicio".into(),
        ));
    }

    ServiceRepository::hard_delete_service(&state.pool, id).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
