/* [195A-1] Órdenes de dominio self-service.
 * El usuario paga la reserva/registro; el alta final queda pendiente para completar handles WHOIS correctamente. */
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct DomainOrder {
    pub id: Uuid,
    pub user_id: Uuid,
    pub domain: String,
    pub tld: String,
    pub status: String,
    pub base_cost_cents: i32,
    pub price_cents: i32,
    pub stripe_session_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateDomainCheckoutRequest {
    #[validate(length(min = 4, max = 253))]
    pub domain: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DomainPriceQuote {
    pub domain: String,
    pub available: bool,
    pub tld: String,
    pub base_cost_cents: i32,
    pub price_cents: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DomainCheckoutResponse {
    pub order: DomainOrder,
    pub checkout_url: String,
}
