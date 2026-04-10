use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct UserPaymentMethod {
    pub id: Uuid,
    pub user_id: Uuid,
    pub stripe_payment_method_id: String,
    pub card_fingerprint: String,
    pub brand: String,
    pub last_four: String,
    pub exp_month: i32,
    pub exp_year: i32,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaymentMethodResponse {
    pub id: Uuid,
    pub brand: String,
    pub last_four: String,
    pub exp_month: i32,
    pub exp_year: i32,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

impl From<UserPaymentMethod> for PaymentMethodResponse {
    fn from(value: UserPaymentMethod) -> Self {
        Self {
            id: value.id,
            brand: value.brand,
            last_four: value.last_four,
            exp_month: value.exp_month,
            exp_year: value.exp_year,
            is_default: value.is_default,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SetupIntentResponse {
    pub client_secret: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SavePaymentMethodRequest {
    pub setup_intent_id: String,
}