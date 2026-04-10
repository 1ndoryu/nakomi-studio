use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/* [044A-38] Roles de usuario: admin puede cambiar genuinamente entre roles vía active_role.
 * El effective_role se calcula: si admin tiene active_role, usa ese; si no, usa role real. */

/// Roles del sistema — mapea al enum `user_role` de `PostgreSQL`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Employee,
    Client,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admin => write!(f, "admin"),
            Self::Employee => write!(f, "employee"),
            Self::Client => write!(f, "client"),
        }
    }
}

/// Modelo de usuario almacenado en base de datos
#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub active_role: Option<UserRole>,
    pub email_verified: bool,
    pub status: String,
    pub avatar_url: Option<String>,
    pub display_name: Option<String>,
    pub username: String,
    pub created_at: DateTime<Utc>,
    /* [154A-5] true si el usuario estableció su propia contraseña, false si fue quick_register */
    pub password_set: bool,
}

impl User {
    /// Retorna el rol efectivo: si admin tiene `active_role`, usa ese; si no, role real
    #[must_use]
    pub fn effective_role(&self) -> UserRole {
        if self.role == UserRole::Admin {
            self.active_role.unwrap_or(self.role)
        } else {
            self.role
        }
    }
}

/// Response público de usuario — sin datos sensibles
#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
    pub effective_role: UserRole,
    pub avatar_url: Option<String>,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        let effective_role = user.effective_role();
        Self {
            id: user.id,
            email: user.email,
            role: user.role,
            effective_role,
            avatar_url: user.avatar_url,
            display_name: user.display_name,
            created_at: user.created_at,
        }
    }
}

/// Request body para registrar un nuevo usuario
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(email(message = "Formato de email inválido"))]
    pub email: String,
    #[validate(length(min = 8, message = "La contraseña debe tener al menos 8 caracteres"))]
    pub password: String,
}

/* [064A-3] Request para registro rapido solo con email (flujo de compra).
 * No requiere password — se genera uno aleatorio en el backend. */
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct QuickRegisterRequest {
    #[validate(email(message = "Formato de email inválido"))]
    pub email: String,
}

/* [154A-5] Request para que un usuario sin contraseña establezca la suya */
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SetPasswordRequest {
    #[validate(length(min = 8, message = "La contraseña debe tener al menos 8 caracteres"))]
    pub password: String,
}

/// Request body para iniciar sesión
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

/// Response con token JWT después de autenticarse.
/// [084A-1] `impersonating` indica si la sesión es impersonada por un admin.
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: Uuid,
    pub role: UserRole,
    pub effective_role: UserRole,
    pub impersonating: bool,
    /* [154A-5] true si el usuario necesita crear su propia contraseña (quick_register sin password) */
    pub needs_password: bool,
}

/* [054A-1] Modelos para gestión de usuarios desde panel admin */

/// Elemento de la lista de usuarios para admin
#[derive(Debug, Serialize, ToSchema)]
pub struct AdminUserItem {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
    pub status: String,
    pub email_verified: bool,
    pub avatar_url: Option<String>,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Respuesta paginada de usuarios
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedUsers {
    pub users: Vec<AdminUserItem>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

/// Request para cambiar el rol de un usuario
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ChangeRoleRequest {
    pub role: UserRole,
}

/// Request para cambiar el status de un usuario (ban/unban)
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ChangeStatusRequest {
    #[validate(custom(function = "validate_user_status"))]
    pub status: String,
}

fn validate_user_status(status: &str) -> Result<(), validator::ValidationError> {
    match status {
        "active" | "banned" | "suspended" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_status");
            err.message = Some("Status debe ser: active, banned o suspended".into());
            Err(err)
        }
    }
}

/* [074A-23] Request para actualizar perfil (display_name + campos extendidos) */
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProfileRequest {
    #[validate(length(max = 100, message = "El nombre no puede exceder 100 caracteres"))]
    pub display_name: Option<String>,
    #[validate(length(max = 500, message = "La descripción no puede exceder 500 caracteres"))]
    pub bio: Option<String>,
    #[validate(length(max = 255, message = "URL de LinkedIn demasiado larga"))]
    pub linkedin: Option<String>,
    #[validate(length(max = 255, message = "URL de Twitter demasiado larga"))]
    pub twitter: Option<String>,
    #[validate(length(max = 500, message = "URL de website demasiado larga"))]
    pub website: Option<String>,
}
