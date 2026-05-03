use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{post, put};
use axum::{Json, Router};
use validator::Validate;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{
    AuthResponse, LoginRequest, QuickRegisterRequest, RegisterRequest, SetPasswordRequest,
};
use crate::services::{AuditService, AuthService};
use crate::AppState;

/// Registrar nuevo usuario
#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Usuario registrado", body = AuthResponse),
        (status = 409, description = "Email ya registrado", body = crate::errors::ErrorResponse),
        (status = 422, description = "Error de validación", body = crate::errors::ErrorResponse)
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let response = AuthService::register(&state.pool, req, &state.jwt_secret).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// Iniciar sesión
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login exitoso", body = AuthResponse),
        (status = 401, description = "Credenciales inválidas", body = crate::errors::ErrorResponse)
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let email = req.email.clone();
    match AuthService::login(&state.pool, req, &state.jwt_secret).await {
        Ok(response) => {
            /* [064A-73] Audit: login exitoso */
            AuditService::log(
                &state.pool,
                "login_success",
                Some(response.user_id),
                None,
                serde_json::json!({"email": email}),
            )
            .await;
            Ok(Json(response))
        }
        Err(e) => {
            /* [064A-73] Audit: login fallido */
            AuditService::log(
                &state.pool,
                "login_failed",
                None,
                None,
                serde_json::json!({"email": email}),
            )
            .await;
            Err(e)
        }
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/quick-register", post(quick_register))
        .route("/auth/login", post(login))
        .route("/auth/set-password", put(set_password))
}

/* [064A-3] Registro rapido: solo email, sin password.
 * Pensado para el flujo de compra — crea cuenta con password aleatorio.
 * Si el email ya existe retorna 409 para que el frontend pida password. */
#[utoipa::path(
    post,
    path = "/api/auth/quick-register",
    request_body = QuickRegisterRequest,
    responses(
        (status = 201, description = "Usuario registrado (sin password)", body = AuthResponse),
        (status = 409, description = "Email ya registrado", body = crate::errors::ErrorResponse),
        (status = 422, description = "Error de validación", body = crate::errors::ErrorResponse)
    )
)]
pub async fn quick_register(
    State(state): State<AppState>,
    Json(req): Json<QuickRegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let response = AuthService::quick_register(&state.pool, req, &state.jwt_secret).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/* [154A-5] Establece contraseña para usuarios de quick_register (password_set = false).
 * Requiere autenticación. Si el usuario ya tiene contraseña, retorna 400. */
#[utoipa::path(
    put,
    path = "/api/auth/set-password",
    request_body = SetPasswordRequest,
    responses(
        (status = 200, description = "Contraseña establecida"),
        (status = 400, description = "Ya tiene contraseña", body = crate::errors::ErrorResponse),
        (status = 401, description = "No autenticado", body = crate::errors::ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn set_password(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<SetPasswordRequest>,
) -> Result<StatusCode, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    AuthService::set_password(&state.pool, auth.user_id, req).await?;
    Ok(StatusCode::OK)
}
