use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{AuthResponse, LoginRequest, QuickRegisterRequest, RegisterRequest, UserRole};
use crate::repositories::UserRepository;

/* [044A-38] Claims extendidos con role y effective_role.
 * El effective_role es el rol con el que el usuario opera: para admins
 * puede ser diferente de role si tienen active_role configurado.
 * [084A-1] impersonator: UUID del admin que inició impersonación.
 * Si Some, sub es el usuario impersonado y role/effective_role son los de ese usuario. */

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub role: UserRole,
    pub effective_role: UserRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impersonator: Option<Uuid>,
    pub exp: usize,
}

pub struct AuthService;

/* [104A-3] Verifica si el email está en GLORY_ADMIN_EMAILS y promueve a admin.
 * Env var es comma-separated, case-insensitive. Si no existe, no promueve a nadie. */
fn is_admin_email(email: &str) -> bool {
    std::env::var("GLORY_ADMIN_EMAILS")
        .unwrap_or_default()
        .split(',')
        .any(|e| e.trim().eq_ignore_ascii_case(email))
}

impl AuthService {
    /// Registra un nuevo usuario: valida unicidad, hashea contraseña, genera JWT
    pub async fn register(
        pool: &PgPool,
        req: RegisterRequest,
        jwt_secret: &str,
    ) -> Result<AuthResponse, AppError> {
        if UserRepository::find_by_email(pool, &req.email)
            .await?
            .is_some()
        {
            return Err(AppError::Conflict("Email ya registrado".into()));
        }

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(req.password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(format!("Error al hashear contraseña: {e}")))?
            .to_string();

        let user = UserRepository::create(pool, &req.email, &password_hash).await?;

        /* [104A-3] Auto-promote admin emails on registration */
        let user = if is_admin_email(&req.email) {
            UserRepository::update_role(pool, user.id, UserRole::Admin).await?
        } else {
            user
        };

        let effective = user.effective_role();
        let token = Self::generate_token(user.id, user.role, effective, None, jwt_secret)?;

        Ok(AuthResponse {
            token,
            user_id: user.id,
            role: user.role,
            effective_role: effective,
            impersonating: false,
        })
    }

    /* [064A-3] Registro rapido solo con email (flujo de compra).
     * Genera password aleatorio; el usuario puede cambiarlo desde el panel.
     * Si el email ya existe retorna Conflict(409) para que el frontend pida password. */
    pub async fn quick_register(
        pool: &PgPool,
        req: QuickRegisterRequest,
        jwt_secret: &str,
    ) -> Result<AuthResponse, AppError> {
        if UserRepository::find_by_email(pool, &req.email)
            .await?
            .is_some()
        {
            return Err(AppError::Conflict("Email ya registrado".into()));
        }

        let random_password: String = {
            use argon2::password_hash::rand_core::RngCore;
            let mut buf = [0u8; 32];
            OsRng.fill_bytes(&mut buf);
            hex::encode(buf)
        };

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(random_password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(format!("Error al hashear contraseña: {e}")))?
            .to_string();

        let user = UserRepository::create(pool, &req.email, &password_hash).await?;

        /* [104A-3] Auto-promote admin emails on quick registration */
        let user = if is_admin_email(&req.email) {
            UserRepository::update_role(pool, user.id, UserRole::Admin).await?
        } else {
            user
        };

        let effective = user.effective_role();
        let token = Self::generate_token(user.id, user.role, effective, None, jwt_secret)?;

        Ok(AuthResponse {
            token,
            user_id: user.id,
            role: user.role,
            effective_role: effective,
            impersonating: false,
        })
    }

    /// Inicia sesión: verifica credenciales y genera JWT
    pub async fn login(
        pool: &PgPool,
        req: LoginRequest,
        jwt_secret: &str,
    ) -> Result<AuthResponse, AppError> {
        let user = UserRepository::find_by_email(pool, &req.email)
            .await?
            .ok_or(AppError::Unauthorized)?;

        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| AppError::Internal(format!("Hash almacenado inválido: {e}")))?;

        Argon2::default()
            .verify_password(req.password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized)?;

        let effective = user.effective_role();
        let token = Self::generate_token(user.id, user.role, effective, None, jwt_secret)?;

        Ok(AuthResponse {
            token,
            user_id: user.id,
            role: user.role,
            effective_role: effective,
            impersonating: false,
        })
    }

    /// Genera un JWT con expiración de 24 horas, incluye role y `effective_role`.
    /// Si `impersonator` es Some, el token representa una sesión impersonada por un admin.
    pub fn generate_token(
        user_id: Uuid,
        role: UserRole,
        effective_role: UserRole,
        impersonator: Option<Uuid>,
        secret: &str,
    ) -> Result<String, AppError> {
        let timestamp = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .ok_or_else(|| AppError::Internal("Error calculando expiración del token".into()))?
            .timestamp();
        let exp = usize::try_from(timestamp)
            .map_err(|_| AppError::Internal("Timestamp fuera de rango".into()))?;

        let claims = Claims {
            sub: user_id,
            role,
            effective_role,
            impersonator,
            exp,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("Error generando token: {e}")))
    }

    /// Verifica un JWT y retorna los claims
    pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| AppError::Unauthorized)
    }
}
