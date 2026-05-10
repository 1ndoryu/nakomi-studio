use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Variable de entorno requerida no encontrada: {0}")]
    MissingEnvVar(String),
    #[error("Puerto inválido: {0}")]
    InvalidPort(#[from] std::num::ParseIntError),
}

/// Configuración de la aplicación cargada desde variables de entorno
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub host: String,
    pub port: u16,
    pub static_dir: Option<String>,
    pub stripe_publishable_key: Option<String>,
    pub stripe_secret_key: Option<String>,
    pub stripe_webhook_secret: Option<String>,
    /* [064A-73] Orígenes permitidos para CORS (separados por coma en env).
     * Si vacío, permite todo (dev). En producción, definir GLORY_ALLOWED_ORIGINS. */
    pub allowed_origins: Vec<String>,
}

impl AppConfig {
    /// Carga la configuración desde variables de entorno.
    /// Requiere `JWT_SECRET`. `DATABASE_URL`, `HOST` y `PORT` son opcionales.
    /// Si `DATABASE_URL` no está definida, usa la DB local por defecto del proyecto.
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:root@localhost:5432/nakomi_db".to_string()
            }),
            jwt_secret: std::env::var("JWT_SECRET")
                .map_err(|_| ConfigError::MissingEnvVar("JWT_SECRET".into()))?,
            host: std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
            static_dir: std::env::var("STATIC_DIR").ok(),
            /* [095A-23] La publishable key es publica y se expone tambien en runtime.
             * Esto evita depender de que Vite reciba VITE_* durante builds remotos. */
            stripe_publishable_key: std::env::var("VITE_STRIPE_PUBLISHABLE_KEY")
                .or_else(|_| std::env::var("GLORY_STRIPE_PUBLISHABLE_KEY"))
                .or_else(|_| std::env::var("STRIPE_PUBLISHABLE_KEY"))
                .ok(),
            /* [064A-65] Admite ambas variantes: GLORY_STRIPE_* (convención del proyecto) y STRIPE_* */
            stripe_secret_key: std::env::var("GLORY_STRIPE_SECRET_KEY")
                .or_else(|_| std::env::var("STRIPE_SECRET_KEY"))
                .ok(),
            stripe_webhook_secret: std::env::var("GLORY_STRIPE_WEBHOOK_SECRET")
                .or_else(|_| std::env::var("STRIPE_WEBHOOK_SECRET"))
                .ok(),
            /* [064A-73] CORS: leer orígenes de GLORY_ALLOWED_ORIGINS (separados por coma) */
            allowed_origins: std::env::var("GLORY_ALLOWED_ORIGINS")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        })
    }
}
