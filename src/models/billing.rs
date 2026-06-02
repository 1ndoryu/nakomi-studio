/* [205A-1] Facturacion pendiente desacoplada de hosting_subscriptions.
 * Un hosting puede seguir activo mientras la plataforma muestra el cobro que falta activar. */
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct BillingItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub amount_cents: i32,
    pub currency: String,
    pub billing_period: String,
    pub status: String,
    pub due_at: DateTime<Utc>,
    pub grace_period_ends_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub stripe_session_id: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BillingCheckoutMode {
    Subscription,
    PrepayYear,
}

impl std::fmt::Display for BillingCheckoutMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Subscription => write!(f, "subscription"),
            Self::PrepayYear => write!(f, "prepay_year"),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateBillingCheckoutRequest {
    pub item_ids: Option<Vec<Uuid>>,
    pub mode: BillingCheckoutMode,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BillingCheckoutResponse {
    pub checkout_url: String,
}

/* [026B-1] Respuesta admin de billing_items con email del usuario. */
#[derive(Debug, Serialize, ToSchema)]
pub struct AdminBillingItemResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub amount_cents: i32,
    pub currency: String,
    pub billing_period: String,
    pub status: String,
    pub due_at: DateTime<Utc>,
    pub grace_period_ends_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/* [026B-1] Request para cambiar status de un billing_item desde admin. */
#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminUpdateBillingStatusRequest {
    pub status: String,
}
