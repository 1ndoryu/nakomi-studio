#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

pub mod config;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod services;

use sqlx::PgPool;

use crate::services::{AiChatConfig, ChatHub, ChatTimingService, ContaboService, CoolifyConfig, EmailConfig, NotificationHub};
use crate::services::docker_stats::DockerStatsCache;

/// Estado compartido de la aplicación — accesible desde handlers y middleware
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub static_dir: Option<String>,
    pub http_client: reqwest::Client,
    pub stripe_secret_key: Option<String>,
    pub stripe_webhook_secret: Option<String>,
    pub chat_hub: ChatHub,
    pub ai_config: AiChatConfig,
    pub notification_hub: NotificationHub,
    pub chat_timing: ChatTimingService,
    pub contabo_service: Option<ContaboService>,
    /// [104A-42] Config de Coolify para provisioning automático de hostings
    pub coolify_config: Option<CoolifyConfig>,
    /// [154A-15c] Config de SMTP para emails transaccionales
    pub email_config: Option<EmailConfig>,
    /// [114A-15+] Cache de stats de contenedores Docker (30s TTL)
    pub docker_stats_cache: DockerStatsCache,
    /* [154A-2] Fixture manager para sincronizar archivos TOML de content/ con la BD desde el panel admin.
     * Arc porque ContentManager no es Clone (contiene Box<dyn Fn>). None si content/ no existe. */
    pub fixture_manager: Option<std::sync::Arc<glory_rs::fixtures::ContentManager>>,
}
