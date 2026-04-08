/* [054A-2] Modelos de hosting: suscripciones y eventos.
 * hosting_subscriptions: registra planes de hosting contratados por clientes.
 * hosting_events: log de eventos del ciclo de vida (provisioned, backup, health_fail, etc). */

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/* ============================================================
   MODELOS DE BD
   ============================================================ */

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct HostingSubscription {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub client_name: String,
    pub client_email: String,
    pub plan: String,
    pub domain: Option<String>,
    pub coolify_site_name: Option<String>,
    pub status: String,
    pub stripe_subscription_id: Option<String>,
    pub monthly_price_cents: i32,
    pub storage_limit_mb: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct HostingEvent {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub event_type: String,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/* ============================================================
   REQUESTS
   ============================================================ */

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateHostingRequest {
    #[validate(length(min = 1, max = 200))]
    pub client_name: String,
    #[validate(email)]
    pub client_email: String,
    #[validate(length(min = 1, max = 20))]
    pub plan: String,
    #[validate(length(max = 253))]
    pub domain: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateHostingStatusRequest {
    #[validate(length(min = 1, max = 20))]
    pub status: String,
    pub reason: Option<String>,
}

/* [094A-3] Self-service: cliente contrata hosting sin form admin.
 * Solo necesita plan y dominio opcional — nombre/email se toman del perfil del usuario. */
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SelfSubscribeRequest {
    #[validate(length(min = 1, max = 20))]
    pub plan: String,
    #[validate(length(max = 253))]
    pub domain: Option<String>,
}

/* [074A-65] Request para editar suscripción (plan, dominio) */
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateHostingRequest {
    #[validate(length(min = 1, max = 20))]
    pub plan: String,
    #[validate(length(max = 253))]
    pub domain: Option<String>,
}

/* ============================================================
   RESPONSES
   ============================================================ */

#[derive(Debug, Serialize, ToSchema)]
pub struct HostingSubscriptionResponse {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub client_name: String,
    pub client_email: String,
    pub plan: String,
    pub domain: Option<String>,
    pub coolify_site_name: Option<String>,
    pub status: String,
    pub monthly_price_cents: i32,
    pub storage_limit_mb: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<HostingSubscription> for HostingSubscriptionResponse {
    fn from(s: HostingSubscription) -> Self {
        Self {
            id: s.id,
            user_id: s.user_id,
            client_name: s.client_name,
            client_email: s.client_email,
            plan: s.plan,
            domain: s.domain,
            coolify_site_name: s.coolify_site_name,
            status: s.status,
            monthly_price_cents: s.monthly_price_cents,
            storage_limit_mb: s.storage_limit_mb,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

/* [094A-3] Respuesta self-service: suscripción creada + URL de Stripe Checkout */
#[derive(Debug, Serialize, ToSchema)]
pub struct SelfSubscribeResponse {
    pub subscription: HostingSubscriptionResponse,
    pub checkout_url: String,
}

/* [094A-8] Estadísticas reales de una suscripción de hosting.
 * Uptime se calcula desde el historial de eventos (status_change).
 * Storage/bandwidth son límites del plan; uso real requiere monitoreo futuro (Coolify/cAdvisor). */
#[derive(Debug, Serialize, ToSchema)]
pub struct HostingStatsResponse {
    pub storage_limit_mb: i32,
    /// null = monitoring no disponible aún
    pub storage_used_mb: Option<i64>,
    pub bandwidth_limit_gb: i32,
    /// null = monitoring no disponible aún
    pub bandwidth_used_gb: Option<i64>,
    /// Calculado desde historial de eventos (tiempo en status "active")
    pub uptime_percent: f64,
    /// Timestamp desde que la suscripción está activa (null si nunca se activó)
    pub active_since: Option<DateTime<Utc>>,
    pub total_events: i64,
    pub last_event_at: Option<DateTime<Utc>>,
    /// true si hay agente de monitoreo configurado (`coolify_site_name` != null)
    pub monitoring_available: bool,
}
