/* [044A-38] Handlers de órdenes — CRUD con autenticación.
 * Cliente crea órdenes, admin/employee ven según rol, admin asigna. */

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, patch, post, put};
use axum::{Json, Router};
use sqlx;
use uuid::Uuid;
use validator::Validate;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{
    CreateNotification, CreateOrderRequest, OrderResponse, OrderStatus, SwitchRoleRequest,
    ToggleAiIntermediaryRequest, UpdateOrderPhaseDefinitionRequest,
    UpdateOrderProjectDescriptionRequest, UserRole,
    NOTIF_NEW_ORDER, NOTIF_ORDER_ASSIGNED, NOTIF_ORDER_CANCELLED,
    NOTIF_ORDER_COMPLETED, NOTIF_REVISION_REQUESTED,
};
use crate::repositories::{OrderRepository, UserRepository};
use crate::services::{AuthService, OrderService, PaymentService};
use crate::AppState;

/* ============================================================
   ÓRDENES
   ============================================================ */

/// Crear una nueva orden (solo clientes o admin operando como cliente)
#[utoipa::path(
    post,
    path = "/api/orders",
    request_body = CreateOrderRequest,
    responses(
        (status = 201, description = "Orden creada", body = OrderResponse),
        (status = 400, description = "Datos inválidos", body = crate::errors::ErrorResponse),
        (status = 401, description = "No autorizado", body = crate::errors::ErrorResponse),
        (status = 403, description = "Sin permisos", body = crate::errors::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "orders"
)]
pub async fn create_order(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateOrderRequest>,
) -> Result<(StatusCode, Json<OrderResponse>), AppError> {
    auth.require_role(&[UserRole::Client, UserRole::Admin])?;
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let order = OrderService::create_order(&state.pool, auth.user_id, req).await?;

    /* [104A-38] Notificar a admins sobre nueva orden */
    let admins = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM users WHERE role = 'admin' AND is_active = true",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    let base = CreateNotification {
        user_id: Uuid::nil(),
        notification_type: NOTIF_NEW_ORDER.to_string(),
        title: format!("Nueva orden #{}", order.order_number),
        body: Some(format!("{} — {}", order.service_title, order.plan_name)),
        link: Some(format!("/panel?seccion=ordenes&id={}", order.id)),
        reference_type: Some("order".to_string()),
        reference_id: Some(order.id),
    };
    let _ = state.notification_hub.notify_many(&admins, &base).await;

    Ok((StatusCode::CREATED, Json(order)))
}

/// Listar órdenes del usuario autenticado (filtrado por rol)
#[utoipa::path(
    get,
    path = "/api/orders",
    responses(
        (status = 200, description = "Lista de órdenes", body = Vec<OrderResponse>),
        (status = 401, description = "No autorizado", body = crate::errors::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "orders"
)]
pub async fn list_orders(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<OrderResponse>>, AppError> {
    /* [084A-1] Con impersonación, effective_role refleja el rol real del usuario
     * impersonado. Admin sin impersonar tiene effective_role=admin → ve todo.
     * Impersonando como client → filtra por client_id. Como employee → por assigned. */
    let orders =
        OrderService::list_orders_for_user(&state.pool, auth.user_id, auth.effective_role).await?;
    Ok(Json(orders))
}

/// Detalle de una orden con sus fases
#[utoipa::path(
    get,
    path = "/api/orders/{order_id}",
    params(("order_id" = Uuid, Path, description = "ID de la orden")),
    responses(
        (status = 200, description = "Detalle de la orden"),
        (status = 401, description = "No autorizado", body = crate::errors::ErrorResponse),
        (status = 404, description = "No encontrada", body = crate::errors::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "orders"
)]
pub async fn get_order(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(order_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (client_id, order, phases) = OrderService::get_order(&state.pool, order_id).await?;

    /* [074A-50] Admin real siempre puede ver cualquier orden, incluso con effective_role
     * switcheado a client/employee. El effective_role solo afecta la UI, no el acceso. */
    if auth.role != UserRole::Admin {
        match auth.effective_role {
            UserRole::Admin => { /* imposible si role != admin, pero por completitud */ }
            UserRole::Client => {
                if client_id != auth.user_id {
                    return Err(AppError::Forbidden("No tienes acceso a esta orden".into()));
                }
            }
            UserRole::Employee => {
                if order.assigned_employee_id != Some(auth.user_id) {
                    return Err(AppError::Forbidden("No tienes acceso a esta orden".into()));
                }
            }
        }
    }

    Ok(Json(serde_json::json!({
        "order": order,
        "phases": phases,
    })))
}

#[utoipa::path(
    patch,
    path = "/api/orders/{order_id}/project-description",
    params(("order_id" = Uuid, Path, description = "ID de la orden")),
    request_body = UpdateOrderProjectDescriptionRequest,
    responses(
        (status = 200, description = "Descripción del proyecto actualizada", body = OrderResponse),
        (status = 400, description = "Datos inválidos", body = crate::errors::ErrorResponse),
        (status = 401, description = "No autorizado", body = crate::errors::ErrorResponse),
        (status = 403, description = "Sin permisos", body = crate::errors::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "orders"
)]
pub async fn update_order_project_description_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(order_id): Path<Uuid>,
    Json(req): Json<UpdateOrderProjectDescriptionRequest>,
) -> Result<Json<OrderResponse>, AppError> {
    auth.require_role(&[UserRole::Client, UserRole::Admin])?;
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let order = OrderService::update_project_description(
        &state.pool,
        order_id,
        auth.user_id,
        auth.effective_role,
        req.project_description,
    )
    .await?;

    Ok(Json(order))
}

#[utoipa::path(
    patch,
    path = "/api/orders/{order_id}/phases/{phase_number}",
    params(
        ("order_id" = Uuid, Path, description = "ID de la orden"),
        ("phase_number" = i32, Path, description = "Número de fase"),
    ),
    request_body = UpdateOrderPhaseDefinitionRequest,
    responses(
        (status = 200, description = "Fase actualizada", body = crate::models::OrderPhaseResponse),
        (status = 400, description = "Datos inválidos", body = crate::errors::ErrorResponse),
        (status = 401, description = "No autorizado", body = crate::errors::ErrorResponse),
        (status = 403, description = "Sin permisos", body = crate::errors::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "orders"
)]
pub async fn update_order_phase_definition_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((order_id, phase_number)): Path<(Uuid, i32)>,
    Json(req): Json<UpdateOrderPhaseDefinitionRequest>,
) -> Result<Json<crate::models::OrderPhaseResponse>, AppError> {
    auth.require_role(&[UserRole::Employee, UserRole::Admin])?;

    let phase = OrderService::update_phase_definition(
        &state.pool,
        order_id,
        phase_number,
        auth.user_id,
        auth.effective_role,
        req,
    )
    .await?;

    Ok(Json(phase))
}

/// Asignar empleado a una orden (solo admin)
#[utoipa::path(
    put,
    path = "/api/orders/{order_id}/assign/{employee_id}",
    params(
        ("order_id" = Uuid, Path, description = "ID de la orden"),
        ("employee_id" = Uuid, Path, description = "ID del empleado"),
    ),
    responses(
        (status = 200, description = "Orden asignada"),
        (status = 401, description = "No autorizado", body = crate::errors::ErrorResponse),
        (status = 403, description = "Sin permisos", body = crate::errors::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "orders"
)]
pub async fn assign_order(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((order_id, employee_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let order = OrderService::assign_order(&state.pool, order_id, employee_id).await?;

    /* [104A-38] Notificar al empleado asignado */
    let _ = state.notification_hub.notify(CreateNotification {
        user_id: employee_id,
        notification_type: NOTIF_ORDER_ASSIGNED.to_string(),
        title: format!("Te asignaron la orden #{}", order.order_number),
        body: None,
        link: Some(format!("/panel?seccion=ordenes&id={}", order.id)),
        reference_type: Some("order".to_string()),
        reference_id: Some(order.id),
    }).await;

    Ok(Json(serde_json::json!({ "status": order.status, "assigned_employee_id": order.assigned_employee_id })))
}

/* ============================================================
   CAMBIO DE ROL (admin)
   ============================================================ */

/// Cambiar de vista impersonando un usuario real con el rol objetivo.
/// [084A-1] Si target es client/employee, busca un usuario real con ese rol y genera
/// token impersonado. Si target es admin, restaura el token del admin original.
#[utoipa::path(
    post,
    path = "/api/auth/switch-role",
    request_body = SwitchRoleRequest,
    responses(
        (status = 200, description = "Rol cambiado, nuevo token"),
        (status = 401, description = "No autorizado", body = crate::errors::ErrorResponse),
        (status = 403, description = "Solo admins pueden cambiar rol", body = crate::errors::ErrorResponse),
        (status = 404, description = "No hay usuario con ese rol", body = crate::errors::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "auth"
)]
pub async fn switch_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<SwitchRoleRequest>,
) -> Result<Json<crate::models::AuthResponse>, AppError> {
    /* Determinar el admin original: puede ser el caller directo o el impersonator */
    let admin_id = if auth.role == UserRole::Admin && auth.impersonator.is_none() {
        auth.user_id
    } else if let Some(imp) = auth.impersonator {
        imp
    } else {
        return Err(AppError::Forbidden(
            "Solo administradores pueden cambiar de rol".into(),
        ));
    };

    match req.role {
        UserRole::Admin => {
            /* Restaurar sesión del admin original */
            let admin = UserRepository::find_by_id(&state.pool, admin_id)
                .await?
                .ok_or_else(|| AppError::Internal("Admin original no encontrado".into()))?;
            let effective = admin.effective_role();
            let token = AuthService::generate_token(admin.id, admin.role, effective, None, &state.jwt_secret)?;
            Ok(Json(crate::models::AuthResponse {
                token,
                user_id: admin.id,
                role: admin.role,
                effective_role: effective,
                impersonating: false,
                needs_password: false,
            }))
        }
        target_role => {
            /* Impersonar un usuario real con el rol objetivo */
            let target = UserRepository::find_first_by_role(&state.pool, target_role)
                .await?
                .ok_or_else(|| AppError::NotFound(
                    format!("No hay usuario activo con rol {target_role}")
                ))?;
            let token = AuthService::generate_token(
                target.id, target.role, target.role, Some(admin_id), &state.jwt_secret,
            )?;
            Ok(Json(crate::models::AuthResponse {
                token,
                user_id: target.id,
                role: target.role,
                effective_role: target.role,
                impersonating: true,
                needs_password: false,
            }))
        }
    }
}

/* ============================================================
   [044A-38 Fase 2] CANCELAR, ENTREGAR, APROBAR, REVISIÓN
   ============================================================ */

/// Cancelar una orden (cliente dueño, empleado asignado con razón, o admin)
#[utoipa::path(
    post,
    path = "/api/orders/{order_id}/cancel",
    params(("order_id" = Uuid, Path, description = "ID de la orden")),
    request_body(content = Option<crate::models::CancelOrderRequest>, description = "Razón opcional (obligatoria para empleados)"),
    responses(
        (status = 200, description = "Orden cancelada"),
        (status = 400, description = "Estado no permite cancelación", body = crate::errors::ErrorResponse),
        (status = 401, description = "No autorizado", body = crate::errors::ErrorResponse),
        (status = 403, description = "Sin permisos", body = crate::errors::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "orders"
)]
pub async fn cancel_order_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(order_id): Path<Uuid>,
    body: Option<Json<crate::models::CancelOrderRequest>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let reason = body.and_then(|b| b.0.reason);
    let order = OrderService::cancel_order(
        &state.pool, order_id, auth.user_id, auth.effective_role, reason.as_deref(),
    ).await?;

    /* [104A-38] Notificar a las partes afectadas por la cancelación */
    let mut recipients = Vec::new();
    if auth.user_id != order.client_id {
        recipients.push(order.client_id);
    }
    if let Some(emp) = order.assigned_employee_id {
        if emp != auth.user_id {
            recipients.push(emp);
        }
    }
    let base = CreateNotification {
        user_id: Uuid::nil(),
        notification_type: NOTIF_ORDER_CANCELLED.to_string(),
        title: format!("Orden #{} cancelada", order.order_number),
        body: reason.as_deref().map(|r| r.chars().take(100).collect()),
        link: Some(format!("/panel?seccion=ordenes&id={}", order.id)),
        reference_type: Some("order".to_string()),
        reference_id: Some(order.id),
    };
    let _ = state.notification_hub.notify_many(&recipients, &base).await;

    Ok(Json(serde_json::json!({ "status": order.status })))
}

/* [044A-38 Fase 6] deliver_phase movido a handlers/deliverables.rs con soporte multipart.
 * La ruta POST /orders/:id/phases/:n/deliver ahora acepta archivos + notas. */

/// Cliente aprueba una fase entregada
#[utoipa::path(
    put,
    path = "/api/orders/{order_id}/phases/{phase_number}/approve",
    params(
        ("order_id" = Uuid, Path, description = "ID de la orden"),
        ("phase_number" = i32, Path, description = "Número de fase"),
    ),
    responses(
        (status = 200, description = "Fase aprobada", body = crate::models::OrderPhaseResponse),
        (status = 400, description = "Estado no permite aprobación", body = crate::errors::ErrorResponse),
        (status = 401, description = "No autorizado", body = crate::errors::ErrorResponse),
        (status = 403, description = "Solo el cliente dueño", body = crate::errors::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "orders"
)]
pub async fn approve_phase(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((order_id, phase_number)): Path<(Uuid, i32)>,
) -> Result<Json<crate::models::OrderPhaseResponse>, AppError> {
    auth.require_role(&[UserRole::Client, UserRole::Admin])?;
    let phase = OrderService::approve_phase(&state.pool, order_id, phase_number, auth.user_id).await?;

    /* [044A-38 Fase 3] Si la orden se completó, capturar los pagos retenidos en Stripe */
    if let Some(order) = OrderRepository::find_order_by_id(&state.pool, order_id).await? {
        if order.status == OrderStatus::Completed {
            /* [104A-38] Notificar cliente y empleado sobre orden completada */
            let mut recipients = vec![order.client_id];
            if let Some(emp) = order.assigned_employee_id {
                recipients.push(emp);
            }
            let base = CreateNotification {
                user_id: Uuid::nil(),
                notification_type: NOTIF_ORDER_COMPLETED.to_string(),
                title: format!("Orden #{} completada", order.order_number),
                body: Some("Todas las fases han sido aprobadas".to_string()),
                link: Some(format!("/panel?seccion=ordenes&id={}", order.id)),
                reference_type: Some("order".to_string()),
                reference_id: Some(order.id),
            };
            let _ = state.notification_hub.notify_many(&recipients, &base).await;

            if let Some(ref stripe_key) = state.stripe_secret_key {
                if let Err(e) = PaymentService::capture_held_payments(
                    &state.pool,
                    &state.http_client,
                    stripe_key,
                    order_id,
                )
                .await
                {
                    tracing::error!("Error capturando pagos de orden {order_id}: {e}");
                }
            }
        }
    }

    Ok(Json(crate::models::OrderPhaseResponse::from(phase)))
}

/// Cliente solicita revisión de una fase entregada
#[utoipa::path(
    put,
    path = "/api/orders/{order_id}/phases/{phase_number}/revision",
    params(
        ("order_id" = Uuid, Path, description = "ID de la orden"),
        ("phase_number" = i32, Path, description = "Número de fase"),
    ),
    responses(
        (status = 200, description = "Revisión solicitada", body = crate::models::OrderPhaseResponse),
        (status = 400, description = "No se puede pedir revisión", body = crate::errors::ErrorResponse),
        (status = 401, description = "No autorizado", body = crate::errors::ErrorResponse),
        (status = 403, description = "Solo el cliente dueño", body = crate::errors::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "orders"
)]
pub async fn request_revision(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((order_id, phase_number)): Path<(Uuid, i32)>,
) -> Result<Json<crate::models::OrderPhaseResponse>, AppError> {
    auth.require_role(&[UserRole::Client, UserRole::Admin])?;
    let phase = OrderService::request_revision(&state.pool, order_id, phase_number, auth.user_id).await?;

    /* [104A-38] Notificar al empleado asignado sobre la revisión solicitada */
    if let Some(order) = OrderRepository::find_order_by_id(&state.pool, order_id).await? {
        if let Some(emp) = order.assigned_employee_id {
            let _ = state.notification_hub.notify(CreateNotification {
                user_id: emp,
                notification_type: NOTIF_REVISION_REQUESTED.to_string(),
                title: format!("Revisión solicitada — Orden #{}, Fase {}", order.order_number, phase_number),
                body: Some("El cliente solicitó cambios en la entrega".to_string()),
                link: Some(format!("/panel?seccion=ordenes&id={}", order.id)),
                reference_type: Some("order".to_string()),
                reference_id: Some(order.id),
            }).await;
        }
    }

    Ok(Json(crate::models::OrderPhaseResponse::from(phase)))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/orders", post(create_order).get(list_orders))
        .route("/orders/:order_id", get(get_order))
        .route(
            "/orders/:order_id/project-description",
            patch(update_order_project_description_handler),
        )
        .route("/orders/:order_id/cancel", post(cancel_order_handler))
        .route(
            "/orders/:order_id/assign/:employee_id",
            put(assign_order),
        )
        .route(
            "/orders/:order_id/phases/:phase_number",
            patch(update_order_phase_definition_handler),
        )
        .route("/orders/:order_id/phases/:phase_number/approve", put(approve_phase))
        .route("/orders/:order_id/phases/:phase_number/revision", put(request_revision))
        .route("/orders/:order_id/ai-intermediary", put(toggle_ai_intermediary))
        .route("/auth/switch-role", post(switch_role))
}

/* [T-10] Toggle IA intermediaria por orden (solo admin/employee asignado) */
#[utoipa::path(
    put,
    path = "/api/orders/{order_id}/ai-intermediary",
    params(("order_id" = Uuid, Path, description = "ID de la orden")),
    request_body = ToggleAiIntermediaryRequest,
    responses(
        (status = 200, description = "Toggle actualizado", body = OrderResponse),
        (status = 401, description = "No autorizado"),
        (status = 403, description = "Sin permisos"),
    ),
    security(("bearer_auth" = [])),
    tag = "orders"
)]
pub async fn toggle_ai_intermediary(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(order_id): Path<Uuid>,
    Json(req): Json<ToggleAiIntermediaryRequest>,
) -> Result<Json<OrderResponse>, AppError> {
    auth.require_role(&[UserRole::Admin, UserRole::Employee])?;
    let order = OrderRepository::toggle_ai_intermediary(&state.pool, order_id, req.enabled).await?;

    let (svc_title, svc_slug, plan_name) =
        OrderRepository::get_order_display_info(&state.pool, order.service_id, order.plan_id)
            .await?;
    let phases = OrderRepository::list_order_phases(&state.pool, order.id).await?;
    let employee_name =
        OrderRepository::get_employee_display_name(&state.pool, order.assigned_employee_id)
            .await?;

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let total_phases = phases.len() as i32;

    Ok(Json(OrderResponse {
        id: order.id,
        order_number: order.order_number,
        client_id: order.client_id,
        client_name: None,
        service_title: svc_title,
        service_slug: svc_slug,
        plan_name,
        payment_mode: order.payment_mode,
        base_price_cents: order.base_price_cents,
        discount_percent: order.discount_percent,
        final_price_cents: order.final_price_cents,
        currency: order.currency,
        status: order.status,
        assigned_employee_id: order.assigned_employee_id,
        assigned_employee_name: employee_name,
        current_phase: order.current_phase,
        total_phases,
        project_description: order.project_description,
        client_notes: order.client_notes,
        started_at: order.started_at,
        created_at: order.created_at,
        ai_intermediary_enabled: order.ai_intermediary_enabled.unwrap_or(false),
        ai_summary: order.ai_summary,
    }))
}
