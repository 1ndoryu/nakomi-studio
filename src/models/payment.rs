/* [044A-38 Fase 3] Modelos de pagos: mapean a order_payments + tipos de request/response.
 * PaymentStatus mapea al enum PostgreSQL payment_status. */

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use super::PaymentMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "payment_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Held,
    Released,
    Refunded,
    Failed,
}

#[derive(Debug, Clone, FromRow)]
pub struct OrderPayment {
    pub id: Uuid,
    pub order_id: Uuid,
    pub phase_id: Option<Uuid>,
    pub amount_cents: i32,
    pub currency: String,
    pub status: PaymentStatus,
    pub payment_mode: PaymentMode,
    pub stripe_payment_intent_id: Option<String>,
    pub stripe_charge_id: Option<String>,
    pub held_at: Option<DateTime<Utc>>,
    pub released_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/* Request para iniciar un pago */
#[derive(Debug, Deserialize, ToSchema)]
pub struct InitiatePaymentRequest {
    /// Número de fase (solo requerido en modo phased)
    pub phase_number: Option<i32>,
}

/* Response con el client_secret de Stripe para confirmar en el frontend */
#[derive(Debug, Serialize, ToSchema)]
pub struct PaymentIntentResponse {
    pub payment_id: Uuid,
    pub client_secret: String,
    pub amount_cents: i32,
    pub currency: String,
    pub bypassed: bool,
}

/* Response para historial de pagos */
#[derive(Debug, Serialize, ToSchema)]
pub struct PaymentResponse {
    pub id: Uuid,
    pub order_id: Uuid,
    pub phase_number: Option<i32>,
    pub amount_cents: i32,
    pub currency: String,
    pub status: PaymentStatus,
    pub payment_mode: PaymentMode,
    pub description: Option<String>,
    pub bypassed: bool,
    pub created_at: DateTime<Utc>,
}
