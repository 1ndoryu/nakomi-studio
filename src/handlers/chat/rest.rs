/* sentinel-disable-file sqlx-query-as-sin-macro: chat REST usa runtime query_as
 * para query ad-hoc de sesiones con tipos FromRow genéricos. */
/* [P-1 Chatbot v2] REST endpoints para chat.
 * CRUD sesiones, mensajes, notas, renombrar visitante.
 * [T-5] Upload de archivos en chat (multipart + procesamiento IA).
 * Todos bajo /api/chat/ y protegidos con JWT. */

use axum::extract::{Multipart, Path, Query, State};
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
    /* [104A-36] Rate limiting para REST — reutiliza ChatTimingService con user_id como key.
     * Solo aplica a clientes; staff/admin no tienen límite. */
    if auth.effective_role == crate::models::UserRole::Client {
        let (result, _msg) = state.chat_timing.check_rate(&auth.user_id.to_string());
        if matches!(result, crate::services::RateCheckResult::Muted) {
            return Err(AppError::TooManyRequests(
                "Has enviado demasiados mensajes. Espera un momento.".to_string(),
            ));
        }
    }

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
            /* [T-10] IA intermediaria: si sesión de orden con intermediary habilitado */
            if let Some(order_id) = s.order_id {
                spawn_intermediary_if_enabled(
                    &state, session_id, order_id, auth.user_id, &req.content,
                );
            } else if s.ai_enabled && s.assigned_staff_id.is_none() {
                let ai_resp = AiChatService::generate_response(
                    &state.pool,
                    &state.ai_config,
                    &state.http_client,
                    state.stripe_secret_key.as_deref(),
                    crate::services::AiSessionContext {
                        session_id,
                        visitor_id: s.visitor_id.as_deref(),
                        user_id: None,
                        context: None,
                    },
                    &req.content,
                )
                .await
                .unwrap_or_else(|e| crate::services::AiResponse {
                    text: format!("Error IA: {e}"),
                    needs_escalation: true,
                    rich_messages: Vec::new(),
                });

                /* [T-2] Enviar rich messages antes del texto */
                for rm in &ai_resp.rich_messages {
                    let _ = state.chat_hub
                        .send_rich_message(
                            session_id, "ai", Some("ai"),
                            &rm.content, &rm.message_type, &rm.metadata,
                        )
                        .await;
                }

                let _ = state
                    .chat_hub
                    .send_message(session_id, "ai", Some("ai"), &ai_resp.text)
                    .await;

                /* [T-6] Escalación: notificar admins si la IA lo señala */
                if ai_resp.needs_escalation {
                    if let Ok(admin_ids) =
                        crate::repositories::UserRepository::admin_ids(&state.pool).await
                    {
                        if !admin_ids.is_empty() {
                            let visitor = s.visitor_name.as_deref().unwrap_or("Visitante");
                            let base = crate::models::CreateNotification {
                                user_id: uuid::Uuid::nil(),
                                notification_type: crate::models::NOTIF_ESCALATION_NEEDED
                                    .to_string(),
                                title: format!("Escalación: {visitor} necesita ayuda"),
                                body: Some(
                                    "La IA detectó que se requiere intervención humana en la sesión de chat."
                                        .to_string()
                                ),
                                link: Some(format!("/admin/chat?session={session_id}")),
                                reference_type: Some("chat_session".to_string()),
                                reference_id: Some(session_id),
                            };
                            let _ = state.notification_hub.notify_many(&admin_ids, &base).await;
                        }
                    }
                }
            }
        }
    }

    Ok((StatusCode::CREATED, Json(msg)))
}

/* [T-10] Lanza respuesta IA intermediaria en background si la orden lo tiene habilitado */
fn spawn_intermediary_if_enabled(
    state: &AppState,
    session_id: Uuid,
    order_id: Uuid,
    user_id: Uuid,
    content: &str,
) {
    let pool = state.pool.clone();
    let ai_config = state.ai_config.clone();
    let http_client = state.http_client.clone();
    let hub = state.chat_hub.clone();
    let content = content.to_string();
    tokio::spawn(async move {
        /* Verificar si la orden tiene intermediary habilitado */
        let order = match crate::repositories::OrderRepository::find_order_by_id(&pool, order_id)
            .await
        {
            Ok(Some(o)) if o.ai_intermediary_enabled.unwrap_or(false) => o,
            _ => return,
        };

        /* Generar respuesta con prompt de intermediario */
        let ai_resp = match AiChatService::generate_intermediary_response(
            &pool, &ai_config, &http_client, session_id, &order, user_id, &content,
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Error IA intermediaria orden #{}: {e}", order.order_number);
                return;
            }
        };

        /* Enviar respuesta como ai_intermediary */
        let _ = hub
            .send_message(session_id, "ai_intermediary", Some("ai"), &ai_resp.text)
            .await;

        /* [T-10c] Auto-resumen si >5 mensajes en la sesión */
        if let Ok(msgs) =
            crate::repositories::ChatRepository::list_messages(&pool, session_id, 100, 0).await
        {
            if msgs.len() > 5 {
                AiChatService::maybe_update_order_summary(
                    &pool, &ai_config, &http_client, order_id, &msgs,
                )
                .await;
            }
        }
    });
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
   [T-5] UPLOAD DE ARCHIVOS EN CHAT
   ============================================================ */

const CHAT_UPLOAD_DIR: &str = "uploads/chat";
const MAX_CHAT_FILE_SIZE: u64 = 10 * 1024 * 1024; /* 10 MB */

/* MIME types permitidos para chat: imágenes, audio, PDF, documentos */
const ALLOWED_CHAT_MIMES: &[&str] = &[
    "image/jpeg", "image/png", "image/webp", "image/gif",
    "audio/mpeg", "audio/ogg", "audio/wav", "audio/webm", "audio/mp4", "audio/flac",
    "application/pdf",
];

/* [T-5] Respuesta del endpoint de upload de archivos en chat */
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ChatUploadResponse {
    pub message_id: Uuid,
    pub attachment_id: Uuid,
    pub file_name: String,
    pub mime_type: String,
    pub ai_description: Option<String>,
}

/* [084A-30] Mapea MIME type a extensión de archivo segura.
 * Nunca confiar en la extensión del nombre original del archivo — un atacante
 * puede subir "malware.php" con content-type "image/jpeg". Derivamos la extensión
 * exclusivamente del MIME ya validado contra ALLOWED_CHAT_MIMES. */
fn mime_to_safe_extension(mime: &str) -> &'static str {
    match mime {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "audio/mpeg" => "mp3",
        "audio/ogg" => "ogg",
        "audio/wav" => "wav",
        "audio/webm" => "webm",
        "audio/mp4" => "m4a",
        "audio/flac" => "flac",
        "application/pdf" => "pdf",
        _ => "bin",
    }
}

// [T-5] Determina el message_type según MIME.
// image -> image, audio -> audio, application/pdf -> file
fn mime_to_message_type(mime: &str) -> &'static str {
    if mime.starts_with("image/") {
        "image"
    } else if mime.starts_with("audio/") {
        "audio"
    } else {
        "file"
    }
}

/// Upload de archivo en sesión de chat (imágenes, audio, PDF)
#[utoipa::path(
    post,
    path = "/api/chat/sessions/{session_id}/upload",
    params(("session_id" = Uuid, Path, description = "ID de la sesión")),
    responses(
        (status = 201, description = "Archivo subido", body = ChatUploadResponse),
        (status = 400, description = "Archivo inválido"),
        (status = 413, description = "Archivo demasiado grande"),
    ),
    tag = "chat"
)]
pub async fn upload_chat_file(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<ChatUploadResponse>), AppError> {
    /* Verificar que la sesión existe y está activa */
    let session = crate::repositories::ChatRepository::find_session_by_id(&state.pool, session_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Sesión no encontrada".into()))?;
    if session.status == "closed" {
        return Err(AppError::BadRequest("La sesión está cerrada".into()));
    }

    /* Leer archivo del multipart */
    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Error multipart: {e}")))?
        .ok_or_else(|| AppError::BadRequest("No se recibió archivo".into()))?;

    let original_name = field.file_name().unwrap_or("archivo").to_string();
    let content_type = field
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_string();

    if !ALLOWED_CHAT_MIMES.contains(&content_type.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Tipo no permitido: {content_type}. Permitidos: imágenes, audio, PDF."
        )));
    }

    let data = field
        .bytes()
        .await
        .map_err(|e| AppError::BadRequest(format!("Error leyendo archivo: {e}")))?;

    #[allow(clippy::cast_possible_truncation)]
    let file_size = data.len() as u64;
    if file_size > MAX_CHAT_FILE_SIZE {
        return Err(AppError::BadRequest(format!(
            "Archivo demasiado grande: {file_size} bytes (máx 10 MB)"
        )));
    }

    /* [084A-30] Seguridad: mapear MIME type a extensión segura.
     * Nunca confiar en la extensión del nombre original (puede ser .php, .exe, etc.) */
    let ext = mime_to_safe_extension(&content_type);
    let unique_name = format!("{}.{ext}", Uuid::new_v4());
    let upload_dir = std::path::PathBuf::from(CHAT_UPLOAD_DIR).join(session_id.to_string());
    tokio::fs::create_dir_all(&upload_dir)
        .await
        .map_err(|e| AppError::Internal(format!("Error creando directorio: {e}")))?;
    let file_path = upload_dir.join(&unique_name);
    tokio::fs::write(&file_path, &data)
        .await
        .map_err(|e| AppError::Internal(format!("Error guardando archivo: {e}")))?;

    let relative_path = format!("{CHAT_UPLOAD_DIR}/{session_id}/{unique_name}");
    let msg_type = mime_to_message_type(&content_type);

    /* Crear mensaje rico con metadata del archivo */
    let metadata = serde_json::json!({
        "file_name": original_name,
        "mime_type": content_type,
        "file_size_bytes": file_size,
        "file_url": format!("/{relative_path}"),
    });

    let display_content = match msg_type {
        "image" => format!("📷 {original_name}"),
        "audio" => format!("🎵 {original_name}"),
        _ => format!("📄 {original_name}"),
    };

    let msg = state.chat_hub
        .send_rich_message(session_id, "client", None, &display_content, msg_type, &metadata)
        .await?;

    /* Guardar attachment en BD */
    #[allow(clippy::cast_possible_wrap)]
    let attachment = crate::repositories::ChatRepository::save_attachment(
        &state.pool,
        msg.id,
        &original_name,
        &relative_path,
        &content_type,
        file_size as i64,
    )
    .await?;

    tracing::info!(
        session_id = %session_id,
        file = %original_name,
        mime = %content_type,
        size = file_size,
        "Chat file uploaded"
    );

    /* [T-5] Procesamiento IA en background */
    spawn_file_ai_processing(
        state.pool.clone(), state.ai_config.clone(), state.http_client.clone(),
        state.chat_hub.clone(), session_id, attachment.id,
        content_type.clone(), data.to_vec(), relative_path,
    );

    Ok((
        StatusCode::CREATED,
        Json(ChatUploadResponse {
            message_id: msg.id,
            attachment_id: attachment.id,
            file_name: original_name,
            mime_type: content_type,
            ai_description: None, /* se procesa en background */
        }),
    ))
}

/* [T-5] Lanza procesamiento IA del archivo en background (Vision, Whisper, PDF extract).
 * Si la sesión tiene IA habilitada, genera respuesta automática con el resultado. */
#[allow(clippy::too_many_arguments)]
fn spawn_file_ai_processing(
    pool: sqlx::PgPool,
    ai_config: crate::services::AiChatConfig,
    http_client: reqwest::Client,
    chat_hub: crate::services::ChatHub,
    session_id: Uuid,
    attachment_id: Uuid,
    mime_type: String,
    data: Vec<u8>,
    file_path: String,
) {
    tokio::spawn(async move {
        let description = process_file_with_ai(
            &pool, &ai_config, &http_client, &mime_type, &data, &file_path,
        ).await;

        if let Some(desc) = &description {
            let _ = crate::repositories::ChatRepository::update_attachment_description(
                &pool, attachment_id, desc,
            ).await;

            if let Ok(Some(s)) = crate::repositories::ChatRepository::find_session_by_id(&pool, session_id).await {
                if s.ai_enabled {
                    let user_msg = match mime_type.as_str() {
                        m if m.starts_with("image/") => format!("[El cliente envió una imagen. Descripción: {desc}]"),
                        m if m.starts_with("audio/") => format!("[El cliente envió un audio. Transcripción: {desc}]"),
                        _ => format!("[El cliente envió un archivo PDF. Contenido: {desc}]"),
                    };

                    if let Ok(ai_resp) = AiChatService::generate_response(
                        &pool, &ai_config, &http_client, None,
                        crate::services::AiSessionContext {
                            session_id,
                            visitor_id: s.visitor_id.as_deref(),
                            user_id: None,
                            context: None,
                        },
                        &user_msg,
                    ).await {
                        for rm in &ai_resp.rich_messages {
                            let _ = chat_hub.send_rich_message(
                                session_id, "ai", Some("ai"),
                                &rm.content, &rm.message_type, &rm.metadata,
                            ).await;
                        }
                        let _ = chat_hub.send_message(session_id, "ai", Some("ai"), &ai_resp.text).await;
                    }
                }
            }
        }
    });
}

/* [T-5] Procesar archivo con IA según tipo MIME.
 * Imágenes → Groq Vision (Llama 4 Scout multimodal)
 * Audio → Groq Whisper STT
 * PDF → extracción de texto con pdf-extract */
async fn process_file_with_ai(
    _pool: &sqlx::PgPool,
    config: &crate::services::AiChatConfig,
    http_client: &reqwest::Client,
    mime_type: &str,
    data: &[u8],
    _file_path: &str,
) -> Option<String> {
    if mime_type.starts_with("image/") {
        process_image_vision(config, http_client, data, mime_type).await
    } else if mime_type.starts_with("audio/") {
        process_audio_whisper(config, http_client, data, mime_type).await
    } else if mime_type == "application/pdf" {
        process_pdf_extract(data)
    } else {
        None
    }
}

/* [T-5] Groq Vision: envía imagen como base64 al modelo multimodal.
 * Usa llama-3.2-90b-vision-preview que soporta imágenes inline. */
async fn process_image_vision(
    config: &crate::services::AiChatConfig,
    _http_client: &reqwest::Client,
    data: &[u8],
    mime_type: &str,
) -> Option<String> {
    use base64::Engine;
    let api_key = config.next_key()?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(data);

    let body = serde_json::json!({
        "model": "llama-3.2-90b-vision-preview",
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:{mime_type};base64,{b64}")
                        }
                    },
                    {
                        "type": "text",
                        "text": "Describe brevemente esta imagen en español. Si contiene texto, transcríbelo. Si es un mockup o diseño, describe los elementos."
                    }
                ]
            }
        ],
        "max_tokens": 300
    });

    let resp = reqwest::Client::new()
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let json: serde_json::Value = r.json().await.ok()?;
            let text = json["choices"][0]["message"]["content"].as_str()?;
            tracing::info!("Vision: imagen descrita ({} chars)", text.len());
            Some(text.to_string())
        }
        Ok(r) => {
            let status = r.status();
            let body_text = r.text().await.unwrap_or_default();
            tracing::error!("Vision API error HTTP {status}: {body_text}");
            None
        }
        Err(e) => {
            tracing::error!("Vision API error: {e}");
            None
        }
    }
}

/* [T-5] Groq Whisper STT: transcribe audio a texto.
 * Usa whisper-large-v3-turbo. Max 25MB. */
async fn process_audio_whisper(
    config: &crate::services::AiChatConfig,
    _http_client: &reqwest::Client,
    data: &[u8],
    mime_type: &str,
) -> Option<String> {
    let api_key = config.next_key()?;

    /* Determinar extensión para el campo filename del multipart */
    let ext = match mime_type {
        "audio/ogg" => "ogg",
        "audio/wav" => "wav",
        "audio/webm" => "webm",
        "audio/mp4" => "m4a",
        "audio/flac" => "flac",
        _ => "mp3",
    };

    let file_part = reqwest::multipart::Part::bytes(data.to_vec())
        .file_name(format!("audio.{ext}"))
        .mime_str(mime_type)
        .ok()?;

    let form = reqwest::multipart::Form::new()
        .text("model", "whisper-large-v3-turbo")
        .text("language", "es")
        .part("file", file_part);

    let resp = reqwest::Client::new()
        .post("https://api.groq.com/openai/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {api_key}"))
        .multipart(form)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let json: serde_json::Value = r.json().await.ok()?;
            let text = json["text"].as_str()?;
            tracing::info!("Whisper: audio transcrito ({} chars)", text.len());
            Some(text.to_string())
        }
        Ok(r) => {
            let status = r.status();
            let body_text = r.text().await.unwrap_or_default();
            tracing::error!("Whisper API error HTTP {status}: {body_text}");
            None
        }
        Err(e) => {
            tracing::error!("Whisper API error: {e}");
            None
        }
    }
}

/* [T-5] Extraer texto de PDF con pdf-extract.
 * Limitado a las primeras 3000 palabras para no saturar el context window de la IA. */
fn process_pdf_extract(data: &[u8]) -> Option<String> {
    match pdf_extract::extract_text_from_mem(data) {
        Ok(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                tracing::info!("PDF: sin texto extraíble (probablemente escaneado)");
                return Some("(PDF sin texto extraíble — puede ser un documento escaneado)".to_string());
            }
            /* Limitar a 3000 palabras */
            let words: Vec<&str> = trimmed.split_whitespace().collect();
            let limited = if words.len() > 3000 {
                let truncated: String = words[..3000].join(" ");
                format!("{truncated}\n[... documento truncado a 3000 palabras]")
            } else {
                trimmed.to_string()
            };
            tracing::info!("PDF: {} palabras extraídas", words.len().min(3000));
            Some(limited)
        }
        Err(e) => {
            tracing::error!("PDF extract error: {e}");
            None
        }
    }
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
        /* [T-5] Upload de archivos en chat (sin JWT — visitantes también suben archivos) */
        .route(
            "/chat/sessions/:session_id/upload",
            axum::routing::post(upload_chat_file),
        )
}
