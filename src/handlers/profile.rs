/* [044A-43] Endpoints de perfil de usuario: obtener perfil y subir avatar.
Upload multipart con validación MIME (image), max 2MB, guardado en uploads/avatars/.
[074A-23] PATCH /api/profile para actualizar display_name y campos extendidos. */
use axum::extract::{Multipart, State};
use axum::Json;
use axum::Router;
use chrono::Utc;
use validator::Validate;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{UpdateProfileRequest, UserResponse};
use crate::repositories::UserRepository;
use crate::AppState;

const MAX_AVATAR_SIZE: usize = 2 * 1024 * 1024;
const ALLOWED_MIME_PREFIXES: &[&str] = &["image/jpeg", "image/png", "image/webp", "image/gif"];

/* GET /api/profile — obtiene el perfil del usuario autenticado */
#[utoipa::path(
    get,
    path = "/api/profile",
    responses(
        (status = 200, description = "Perfil del usuario", body = UserResponse),
        (status = 401, description = "No autenticado"),
    ),
    security(("bearer_auth" = [])),
    tag = "profile"
)]
pub async fn get_profile(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<UserResponse>, AppError> {
    let user = UserRepository::find_by_id(&state.pool, auth.user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Usuario no encontrado".into()))?;
    Ok(Json(user.into()))
}

/* POST /api/profile/avatar — sube imagen de avatar (multipart) */
#[utoipa::path(
    post,
    path = "/api/profile/avatar",
    responses(
        (status = 200, description = "Avatar actualizado", body = AvatarResponse),
        (status = 400, description = "Archivo inválido"),
        (status = 401, description = "No autenticado"),
    ),
    security(("bearer_auth" = [])),
    tag = "profile"
)]
pub async fn upload_avatar(
    auth: AuthUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<AvatarResponse>, AppError> {
    let mut file_data: Option<(Vec<u8>, String)> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Error leyendo multipart: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name != "avatar" {
            continue;
        }

        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        if !ALLOWED_MIME_PREFIXES
            .iter()
            .any(|m| content_type.starts_with(m))
        {
            return Err(AppError::BadRequest(
                "Tipo de archivo no permitido. Usa JPG, PNG, WebP o GIF.".into(),
            ));
        }

        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(format!("Error leyendo archivo: {e}")))?;

        if data.len() > MAX_AVATAR_SIZE {
            return Err(AppError::BadRequest(
                "El archivo excede el límite de 2MB.".into(),
            ));
        }

        let ext = match content_type.as_str() {
            "image/png" => "png",
            "image/webp" => "webp",
            "image/gif" => "gif",
            _ => "jpg",
        };

        file_data = Some((data.to_vec(), ext.to_string()));
    }

    let (data, ext) = file_data
        .ok_or_else(|| AppError::BadRequest("No se encontró el campo 'avatar'.".into()))?;

    /* [054A-3] Guardar con nombre versionado para invalidar cache del navegador.
     * Antes de escribir, eliminamos avatares anteriores del mismo usuario para no acumular basura. */
    let dir = std::path::Path::new("uploads/avatars");
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(|e| AppError::Internal(format!("Error creando directorio: {e}")))?;

    let avatar_prefix = auth.user_id.to_string();
    let mut entries = tokio::fs::read_dir(dir)
        .await
        .map_err(|e| AppError::Internal(format!("Error listando avatares previos: {e}")))?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| AppError::Internal(format!("Error leyendo avatar previo: {e}")))?
    {
        let file_name = entry.file_name();
        if file_name.to_string_lossy().starts_with(&avatar_prefix) {
            if let Err(error) = tokio::fs::remove_file(entry.path()).await {
                tracing::warn!(
                    user_id = %auth.user_id,
                    %error,
                    "No se pudo eliminar una version anterior del avatar"
                );
            }
        }
    }

    let filename = format!("{}-{}.{ext}", auth.user_id, Utc::now().timestamp_millis());
    let filepath = dir.join(&filename);
    tokio::fs::write(&filepath, &data)
        .await
        .map_err(|e| AppError::Internal(format!("Error guardando archivo: {e}")))?;

    let avatar_url = format!("/uploads/avatars/{filename}");
    UserRepository::update_avatar(&state.pool, auth.user_id, &avatar_url).await?;

    Ok(Json(AvatarResponse { avatar_url }))
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct AvatarResponse {
    pub avatar_url: String,
}

/* Rutas de perfil — montadas bajo /api en mod.rs */
pub fn routes() -> Router<AppState> {
    use axum::routing::{get, post};
    Router::new()
        .route("/profile", get(get_profile).patch(update_profile))
        .route("/profile/avatar", post(upload_avatar))
}

/* [074A-23] PATCH /api/profile — actualiza display_name y campos extendidos */
#[utoipa::path(
    patch,
    path = "/api/profile",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Perfil actualizado", body = UserResponse),
        (status = 400, description = "Datos inválidos"),
        (status = 401, description = "No autenticado"),
    ),
    security(("bearer_auth" = [])),
    tag = "profile"
)]
pub async fn update_profile(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserResponse>, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    UserRepository::update_profile(
        &state.pool,
        auth.user_id,
        req.display_name.as_deref(),
        req.bio.as_deref(),
        req.linkedin.as_deref(),
        req.twitter.as_deref(),
        req.website.as_deref(),
    )
    .await
    .map_err(|e| AppError::Internal(format!("Error actualizando perfil: {e}")))?;

    let user = UserRepository::find_by_id(&state.pool, auth.user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Usuario no encontrado".into()))?;

    Ok(Json(user.into()))
}
