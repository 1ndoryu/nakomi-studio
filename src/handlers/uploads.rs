/* [074A-6] Handler genérico de upload de imágenes para CMS.
 * Guarda archivos en uploads/content/ con nombre hasheado.
 * Solo admin. Whitelist MIME para imágenes. Max 5 MB. */

use axum::extract::{DefaultBodyLimit, Multipart, State};
use axum::routing::post;
use axum::{Json, Router};
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::UserRole;
use crate::AppState;

const UPLOAD_DIR: &str = "uploads/content";
const MAX_IMAGE_SIZE: u64 = 5 * 1024 * 1024;

const ALLOWED_IMAGE_MIMES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/gif",
    "image/svg+xml",
];

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct UploadResponse {
    pub url: String,
    pub file_name: String,
}

/// Upload de imagen para CMS (admin-only)
#[utoipa::path(
    post,
    path = "/api/admin/uploads",
    responses(
        (status = 200, description = "Imagen subida", body = UploadResponse),
        (status = 400, description = "Archivo inválido"),
        (status = 401, description = "No autorizado"),
        (status = 403, description = "Solo admin"),
    ),
    security(("bearer_auth" = [])),
    tag = "admin"
)]
pub async fn upload_image(
    State(_state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Error leyendo multipart: {e}")))?
        .ok_or_else(|| AppError::BadRequest("No se recibió ningún archivo".into()))?;

    let original_name = field.file_name().unwrap_or("imagen").to_string();
    let content_type = field
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_string();

    if !ALLOWED_IMAGE_MIMES.contains(&content_type.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Tipo de archivo no permitido: {content_type}. Solo imágenes."
        )));
    }

    let data = field
        .bytes()
        .await
        .map_err(|e| AppError::BadRequest(format!("Error leyendo archivo: {e}")))?;

    #[allow(clippy::cast_possible_truncation)]
    let file_size = data.len() as u64;
    if file_size > MAX_IMAGE_SIZE {
        return Err(AppError::BadRequest(format!(
            "Imagen demasiado grande: {file_size} bytes (máx 5 MB)"
        )));
    }

    /* Generar nombre único con UUID para evitar colisiones */
    let extension = original_name
        .rsplit('.')
        .next()
        .unwrap_or("bin");
    let unique_name = format!("{}.{}", Uuid::new_v4(), extension);

    let upload_path = PathBuf::from(UPLOAD_DIR);
    fs::create_dir_all(&upload_path)
        .await
        .map_err(|e| AppError::Internal(format!("Error creando directorio: {e}")))?;

    let file_path = upload_path.join(&unique_name);
    fs::write(&file_path, &data)
        .await
        .map_err(|e| AppError::Internal(format!("Error guardando archivo: {e}")))?;

    tracing::info!(
        user_id = %auth.user_id,
        file = %original_name,
        mime = %content_type,
        size = file_size,
        "Imagen CMS subida"
    );

    let url = format!("/{UPLOAD_DIR}/{unique_name}");

    Ok(Json(UploadResponse {
        url,
        file_name: original_name,
    }))
}

/* [204A-15] DefaultBodyLimit de 5 MB para que axum no rechace antes del handler.
 * Sin esto, axum limita a 2 MB por defecto y devuelve 400 silencioso. */
#[allow(clippy::cast_possible_truncation)] /* 5 MB: safe on all platforms */
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/admin/uploads", post(upload_image))
        .layer(DefaultBodyLimit::max(MAX_IMAGE_SIZE as usize))
}
