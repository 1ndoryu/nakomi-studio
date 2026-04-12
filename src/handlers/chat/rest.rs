/* sentinel-disable-file sqlx-query-as-sin-macro: chat REST usa runtime query_as
 * para query ad-hoc de sesiones con tipos FromRow genéricos. */
/* [P-1 Chatbot v2] REST endpoints para sesiones de chat.
 * CRUD sesiones (listar, crear, cerrar, marcar vista).
 * Mensajes, notas y upload separados en rest_messages, rest_notes, rest_upload. */

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{ChatSessionResponse, CreateChatSessionRequest};
use crate::repositories::OrderRepository;
use crate::AppState;

pub use super::rest_messages::{get_messages, send_message};
pub use super::rest_notes::{create_session_note, list_session_notes, update_visitor_name};
pub use super::rest_upload::upload_chat_file;

/* Re-exportar structs __path_* generados por utoipa para OpenAPI */
pub use super::rest_messages::*;

/* ============================================================
   REST API ENDPOINTS
   ============================================================ */

/// Listar sesiones de chat del usuario
#[utoipa::path(
    get,
    path = "/api/chat/sessions",
    responses(
        (status = 200, description = "Sesiones activas", body = Vec<ChatSessionResponse>),
        (status = 401, description = "No autorizado"),
    ),
    security(("bearer_auth" = [])),
    tag = "chat"
)]
pub async fn list_sessions(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<ChatSessionResponse>>, AppError> {
    let sessions = if auth.effective_role == crate::models::UserRole::Admin
        || auth.effective_role == crate::models::UserRole::Employee
    {
        state.chat_hub.list_all_active_sessions().await?
    } else {
        state.chat_hub.list_sessions_for_user(auth.user_id).await?
    };
    Ok(Json(sessions))
}

/// Crear sesión de chat (para órdenes desde frontend)
#[utoipa::path(
    post,
    path = "/api/chat/sessions",
    request_body = CreateChatSessionRequest,
    responses(
        (status = 201, description = "Sesión creada", body = ChatSessionResponse),
        (status = 401, description = "No autorizado"),
    ),
    security(("bearer_auth" = [])),
    tag = "chat"
)]
pub async fn create_session(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateChatSessionRequest>,
) -> Result<(StatusCode, Json<ChatSessionResponse>), AppError> {
    let session = if let Some(order_id) = req.order_id {
        state
            .chat_hub
            .get_or_create_order_session(order_id, auth.user_id)
            .await?
    } else {
        let vid = req
            .visitor_id
            .unwrap_or_else(|| auth.user_id.to_string());
        state
            .chat_hub
            .get_or_create_visitor_session(&vid, req.visitor_name.as_deref(), None, None, None)
            .await?
    };

    /* [064A-31] Obtener order_number si la sesión está vinculada a una orden */
    let order_number: Option<i32> = if let Some(oid) = session.order_id {
        OrderRepository::order_number_by_id(&state.pool, oid).await.unwrap_or(None)
    } else {
        None
    };

    let response = ChatSessionResponse {
        id: session.id,
        order_id: session.order_id,
        order_number,
        status: session.status,
        ai_enabled: session.ai_enabled,
        assigned_staff_id: session.assigned_staff_id,
        last_message: None,
        last_message_at: None,
        created_at: session.created_at,
        visitor_name: session.visitor_name,
        visitor_ip: session.visitor_ip,
        visitor_user_agent: session.visitor_user_agent,
        visitor_country: session.visitor_country,
        last_viewed_at: session.last_viewed_at,
        visitor_last_connected_at: session.visitor_last_connected_at,
        is_escalated: session.is_escalated,
        client_name: None,
        client_avatar_url: None,
        employee_name: None,
        employee_avatar_url: None,
    };
    Ok((StatusCode::CREATED, Json(response)))
}

/* [054A-9] Cerrar sesión de chat via REST (staff/admin) */
#[utoipa::path(
    post,
    path = "/api/chat/sessions/{session_id}/close",
    params(("session_id" = Uuid, Path, description = "ID de la sesión")),
    responses(
        (status = 204, description = "Sesión cerrada"),
        (status = 401, description = "No autorizado"),
        (status = 403, description = "Sin permisos"),
    ),
    security(("bearer_auth" = [])),
    tag = "chat"
)]
pub async fn close_session(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(session_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    auth.require_role(&[crate::models::UserRole::Admin, crate::models::UserRole::Employee])?;
    state.chat_hub.close_session(session_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/* [104A-39] Marcar sesión como vista — actualiza last_viewed_at para el cálculo del badge.
 * Llamado por el frontend al abrir/seleccionar una sesión en el panel.
 * Solo staff puede marcar como visto (clientes no tienen panel). */
pub async fn mark_session_viewed(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(session_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    auth.require_role(&[crate::models::UserRole::Admin, crate::models::UserRole::Employee])?;
    crate::repositories::ChatRepository::mark_session_viewed(&state.pool, session_id).await
        .map_err(AppError::Database)?;
    Ok(StatusCode::NO_CONTENT)
}

/* ============================================================
   ROUTES (REST — montadas bajo /api)
   ============================================================ */

pub fn rest_routes() -> Router<AppState> {
    Router::new()
        .route("/chat/sessions", get(list_sessions).post(create_session))
        .route(
            "/chat/sessions/:session_id/messages",
            get(get_messages).post(send_message),
        )
        .route(
            "/chat/sessions/:session_id/close",
            axum::routing::post(close_session),
        )
        .route(
            "/chat/sessions/:session_id/mark-viewed",
            axum::routing::patch(mark_session_viewed),
        )
        .route(
            "/chat/sessions/:session_id/notes",
            get(list_session_notes).post(create_session_note),
        )
        .route(
            "/chat/sessions/:session_id/visitor-name",
            axum::routing::patch(update_visitor_name),
        )
        /* [T-5] Upload de archivos en chat (sin JWT — visitantes también suben archivos) */
        .route(
            "/chat/sessions/:session_id/upload",
            axum::routing::post(upload_chat_file),
        )
}
