/* [T-5] Upload de archivos en chat + procesamiento IA (Vision, Whisper, PDF).
 * Separado de rest.rs para mantener el límite de líneas por archivo.
 * Soporta imágenes, audio y PDF con límite de 10 MB.
 * Extensiones derivadas del MIME validado — nunca del nombre original. */

use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::errors::AppError;
use crate::services::AiChatService;
use crate::AppState;

use super::file_ai::process_file_with_ai;

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

/* [T-5] Determina el message_type según MIME.
 * image -> image, audio -> audio, application/pdf -> file */
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
    spawn_file_ai_processing(FileAiContext {
        pool: state.pool.clone(),
        ai_config: state.ai_config.clone(),
        http_client: state.http_client.clone(),
        chat_hub: state.chat_hub.clone(),
        session_id,
        attachment_id: attachment.id,
        mime_type: content_type.clone(),
        data: data.to_vec(),
        file_path: relative_path,
    });

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

/* [174A-2] Contexto para procesamiento IA de archivos en background */
struct FileAiContext {
    pool: sqlx::PgPool,
    ai_config: crate::services::AiChatConfig,
    http_client: reqwest::Client,
    chat_hub: crate::services::ChatHub,
    session_id: Uuid,
    attachment_id: Uuid,
    mime_type: String,
    data: Vec<u8>,
    file_path: String,
}

/* [T-5] Lanza procesamiento IA del archivo en background (Vision, Whisper, PDF extract).
 * Si la sesión tiene IA habilitada, genera respuesta automática con el resultado. */
fn spawn_file_ai_processing(ctx: FileAiContext) {
    tokio::spawn(async move {
        let description = process_file_with_ai(
            &ctx.pool, &ctx.ai_config, &ctx.http_client, &ctx.mime_type, &ctx.data, &ctx.file_path,
        ).await;

        if let Some(desc) = &description {
            let _ = crate::repositories::ChatRepository::update_attachment_description(
                &ctx.pool, ctx.attachment_id, desc,
            ).await;

            if let Ok(Some(s)) = crate::repositories::ChatRepository::find_session_by_id(&ctx.pool, ctx.session_id).await {
                if s.ai_enabled {
                    let user_msg = match ctx.mime_type.as_str() {
                        m if m.starts_with("image/") => format!("[El cliente envió una imagen. Descripción: {desc}]"),
                        m if m.starts_with("audio/") => format!("[El cliente envió un audio. Transcripción: {desc}]"),
                        _ => format!("[El cliente envió un archivo PDF. Contenido: {desc}]"),
                    };

                    if let Ok(ai_resp) = AiChatService::generate_response(
                        &ctx.pool, &ctx.ai_config, &ctx.http_client, None,
                        crate::services::AiSessionContext {
                            session_id: ctx.session_id,
                            visitor_id: s.visitor_id.as_deref(),
                            user_id: None,
                            context: None,
                        },
                        &user_msg,
                    ).await {
                        for rm in &ai_resp.rich_messages {
                            let _ = ctx.chat_hub.send_rich_message(
                                ctx.session_id, "ai", Some("ai"),
                                &rm.content, &rm.message_type, &rm.metadata,
                            ).await;
                        }
                        let _ = ctx.chat_hub.send_message(ctx.session_id, "ai", Some("ai"), &ai_resp.text).await;
                    }
                }
            }
        }
    });
}

