/* [P-1 Chatbot v2] Endpoints REST para notas de sesión y renombrar visitante.
 * Separado de rest.rs para mantener el límite de líneas por archivo. */

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{ChatSessionNote, CreateSessionNoteRequest, UpdateVisitorNameRequest};
use crate::AppState;

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
