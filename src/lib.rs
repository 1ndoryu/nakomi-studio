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

use crate::services::{AiChatConfig, ChatHub, ChatTimingService, ContaboService, HostingStripeConfig, NotificationHub};

/// Estado compartido de la aplicación — accesible desde handlers y middleware
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub http_client: reqwest::Client,
    pub stripe_secret_key: Option<String>,
    pub stripe_webhook_secret: Option<String>,
    pub chat_hub: ChatHub,
    pub ai_config: AiChatConfig,
    pub notification_hub: NotificationHub,
    pub chat_timing: ChatTimingService,
    pub contabo_service: Option<ContaboService>,
    pub hosting_stripe_config: Option<HostingStripeConfig>,
}
