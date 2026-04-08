/* sentinel-disable-file sqlx-query-as-sin-macro: chat REST usa runtime query_as
 * para query ad-hoc de sesiones con tipos FromRow genéricos. */
/* [P-1 Chatbot v2] REST endpoints para chat.
 * CRUD sesiones, mensajes, notas, renombrar visitante.
 * Todos bajo /api/chat/ y protegidos con JWT. */

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{
    ChatMessage, ChatMessageResponse, ChatSessionNote, ChatSessionResponse,
    CreateChatSessionRequest, CreateSessionNoteRequest, SendMessageRequest,
    UpdateVisitorNameRequest,
};
use crate::services::AiChatService;
use crate::AppState;

use super::{enrich_messages, MessagesQuery};

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

/// Obtener mensajes de una sesión
#[utoipa::path(
    get,
    path = "/api/chat/sessions/{session_id}/messages",
    params(
        ("session_id" = Uuid, Path, description = "ID de la sesión"),
        ("limit" = Option<i64>, Query, description = "Límite de mensajes"),
        ("offset" = Option<i64>, Query, description = "Offset para paginación"),
    ),
    responses(
        (status = 200, description = "Mensajes", body = Vec<ChatMessage>),
        (status = 401, description = "No autorizado"),
    ),
    security(("bearer_auth" = [])),
    tag = "chat"
)]
pub async fn get_messages(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(session_id): Path<Uuid>,
    Query(params): Query<MessagesQuery>,
) -> Result<Json<Vec<ChatMessageResponse>>, AppError> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);
    let messages =
        crate::repositories::ChatRepository::list_messages(&state.pool, session_id, limit, offset)
            .await?;

    /* [064A-70] Enriquecer mensajes con avatar + nombre del sender */
    let enriched = enrich_messages(&state.pool, messages).await;

    Ok(Json(enriched))
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
            .get_or_create_visitor_session(&vid, req.visitor_name.as_deref(), None, None)
            .await?
    };

    /* [064A-31] Obtener order_number si la sesión está vinculada a una orden */
    let order_number: Option<i32> = if let Some(oid) = session.order_id {
        sqlx::query_scalar("SELECT order_number FROM orders WHERE id = $1")
            .bind(oid)
            .fetch_optional(&state.pool)
            .await
            .unwrap_or(None)
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
    };
    Ok((StatusCode::CREATED, Json(response)))
}

/// Enviar mensaje REST (alternativa a WebSocket)
#[utoipa::path(
    post,
    path = "/api/chat/sessions/{session_id}/messages",
    params(("session_id" = Uuid, Path, description = "ID de la sesión")),
    request_body = SendMessageRequest,
    responses(
        (status = 201, description = "Mensaje enviado", body = ChatMessage),
        (status = 401, description = "No autorizado"),
    ),
    security(("bearer_auth" = [])),
    tag = "chat"
)]
pub async fn send_message(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(session_id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Result<(StatusCode, Json<ChatMessage>), AppError> {
    let sender_type = match auth.effective_role {
        crate::models::UserRole::Admin => "admin",
        crate::models::UserRole::Employee => "employee",
        crate::models::UserRole::Client => "client",
    };

    let msg = state
        .chat_hub
        .send_message(session_id, sender_type, Some(&auth.user_id.to_string()), &req.content)
        .await?;

    /* Si IA habilitada y sender es client, generar respuesta */
    if sender_type == "client" {
        if let Ok(Some(s)) =
            crate::repositories::ChatRepository::find_session_by_id(&state.pool, session_id).await
        {
            if s.ai_enabled && s.assigned_staff_id.is_none() {
                let ai_response = AiChatService::generate_response(
                    &state.pool,
                    &state.ai_config,
                    session_id,
                    &req.content,
                )
                .await
                .unwrap_or_else(|e| format!("Error IA: {e}"));

                let _ = state
                    .chat_hub
                    .send_message(session_id, "ai", Some("ai"), &ai_response)
                    .await;
            }
        }
    }

    Ok((StatusCode::CREATED, Json(msg)))
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

/* ============================================================
   [064A-72] NOTAS Y RENOMBRAR VISITANTE
   ============================================================ */

/// Listar notas de una sesión
pub async fn list_session_notes(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(session_id): Path<Uuid>,
) -> Result<Json<Vec<ChatSessionNote>>, AppError> {
    let notes =
        crate::repositories::ChatRepository::list_session_notes(&state.pool, session_id).await?;
    Ok(Json(notes))
}

/// Crear nota en una sesión (solo staff/admin)
pub async fn create_session_note(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(session_id): Path<Uuid>,
    Json(req): Json<CreateSessionNoteRequest>,
) -> Result<(StatusCode, Json<ChatSessionNote>), AppError> {
    auth.require_role(&[crate::models::UserRole::Admin, crate::models::UserRole::Employee])?;
    let note = crate::repositories::ChatRepository::create_session_note(
        &state.pool,
        session_id,
        auth.user_id,
        &req.content,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(note)))
}

/// Renombrar visitante de una sesión (solo staff/admin)
pub async fn update_visitor_name(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(session_id): Path<Uuid>,
    Json(req): Json<UpdateVisitorNameRequest>,
) -> Result<StatusCode, AppError> {
    auth.require_role(&[crate::models::UserRole::Admin, crate::models::UserRole::Employee])?;
    crate::repositories::ChatRepository::update_visitor_name(&state.pool, session_id, &req.name)
        .await?;
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
            "/chat/sessions/:session_id/notes",
            get(list_session_notes).post(create_session_note),
        )
        .route(
            "/chat/sessions/:session_id/visitor-name",
            axum::routing::patch(update_visitor_name),
        )
}
