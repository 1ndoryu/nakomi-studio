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
