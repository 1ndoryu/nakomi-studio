/* [164A-17] Dominio VPS revendido: catálogo, suscripciones dedicadas y flujo de aprobación.
 * Se separa de hosting compartido para no mezclar inventario Contabo con ventas reales.
 * Gotcha: no persistimos la contraseña inicial en BD; se entrega por email al aprobar. */

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct VpsPlanConfig {
    pub id: Uuid,
    pub tier_name: String,
    pub display_name: String,
    pub description: String,
    pub contabo_product_id: String,
    pub base_cost_cents: i32,
    pub monthly_price_cents: i32,
    pub cpu_cores: i32,
    pub ram_mb: i32,
    pub disk_mb: i32,
    pub region: String,
    pub is_active: bool,
    pub approval_required: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PublicVpsPlan {
    pub tier_name: String,
    pub display_name: String,
    pub description: String,
    pub monthly_price_cents: i32,
    pub cpu_cores: i32,
    pub ram_mb: i32,
    pub disk_mb: i32,
    pub region: String,
    pub features: Vec<String>,
    pub approval_required: bool,
    pub recommended: bool,
}

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct VpsSubscription {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub client_name: String,
    pub client_email: String,
    pub tier_name: String,
    pub requested_hostname: Option<String>,
    pub status: String,
    pub stripe_subscription_id: Option<String>,
    pub monthly_price_cents: i32,
    pub contabo_instance_id: Option<i64>,
    pub provisioning_ip: Option<String>,
    pub access_username: Option<String>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub provisioned_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    pub client_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct VpsEvent {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub event_type: String,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SelfSubscribeVpsRequest {
    #[validate(length(min = 1, max = 20))]
    pub tier: String,
    #[validate(length(max = 253))]
    pub hostname: Option<String>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RejectVpsRequest {
    #[validate(length(min = 3, max = 1000))]
    pub reason: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VpsSubscriptionResponse {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub client_name: String,
    pub client_email: String,
    pub tier_name: String,
    pub requested_hostname: Option<String>,
    pub status: String,
    pub stripe_subscription_id: Option<String>,
    pub monthly_price_cents: i32,
    pub contabo_instance_id: Option<i64>,
    pub provisioning_ip: Option<String>,
    pub access_username: Option<String>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub provisioned_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    pub client_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<VpsSubscription> for VpsSubscriptionResponse {
    fn from(subscription: VpsSubscription) -> Self {
        Self {
            id: subscription.id,
            user_id: subscription.user_id,
            client_name: subscription.client_name,
            client_email: subscription.client_email,
            tier_name: subscription.tier_name,
            requested_hostname: subscription.requested_hostname,
            status: subscription.status,
            stripe_subscription_id: subscription.stripe_subscription_id,
            monthly_price_cents: subscription.monthly_price_cents,
            contabo_instance_id: subscription.contabo_instance_id,
            provisioning_ip: subscription.provisioning_ip,
            access_username: subscription.access_username,
            approved_by: subscription.approved_by,
            approved_at: subscription.approved_at,
            provisioned_at: subscription.provisioned_at,
            rejected_reason: subscription.rejected_reason,
            client_notes: subscription.client_notes,
            created_at: subscription.created_at,
            updated_at: subscription.updated_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SelfSubscribeVpsResponse {
    pub subscription: VpsSubscriptionResponse,
    pub checkout_url: String,
}