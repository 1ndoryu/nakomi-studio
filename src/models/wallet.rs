/* [154A-15a] Wallet system: modelos para saldo virtual de usuarios.
 * UserWallet: saldo en cents por usuario (uno a uno).
 * WalletTransaction: cada movimiento de saldo con snapshot del balance.
 * CancellationRequest: solicitud de cancelación con flujo de aprobación.
 * [184A-1] WithdrawalRequest: solicitudes de retiro de fondos del wallet. */

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct UserWallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub balance_cents: i32,
    pub currency: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct WalletTransaction {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub user_id: Uuid,
    pub amount_cents: i32,
    pub transaction_type: String,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub description: Option<String>,
    pub balance_after_cents: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct CancellationRequest {
    pub id: Uuid,
    pub order_id: Uuid,
    pub requested_by: Uuid,
    pub reason: String,
    pub status: String,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/* ============================================================
   RESPONSES
   ============================================================ */

/* [184A-1] Withdrawal request: solicitud de retiro de fondos */
#[derive(Debug, Clone, FromRow)]
pub struct WithdrawalRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount_cents: i32,
    pub status: String,
    pub payment_method: Option<String>,
    pub payment_details: Option<String>,
    pub admin_notes: Option<String>,
    pub resolved_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WithdrawalRequestResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount_cents: i32,
    pub status: String,
    pub payment_method: Option<String>,
    pub payment_details: Option<String>,
    pub admin_notes: Option<String>,
    pub resolved_by: Option<Uuid>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WithdrawalRequestsPage {
    pub requests: Vec<WithdrawalRequestResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateWithdrawalRequest {
    pub amount_cents: i32,
    pub payment_method: Option<String>,
    pub payment_details: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ResolveWithdrawalRequest {
    pub approve: bool,
    pub admin_notes: Option<String>,
}

impl From<&WithdrawalRequest> for WithdrawalRequestResponse {
    fn from(w: &WithdrawalRequest) -> Self {
        Self {
            id: w.id,
            user_id: w.user_id,
            amount_cents: w.amount_cents,
            status: w.status.clone(),
            payment_method: w.payment_method.clone(),
            payment_details: w.payment_details.clone(),
            admin_notes: w.admin_notes.clone(),
            resolved_by: w.resolved_by,
            created_at: w.created_at.to_rfc3339(),
            resolved_at: w.resolved_at.map(|t| t.to_rfc3339()),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WalletResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub balance_cents: i32,
    pub currency: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WalletTransactionResponse {
    pub id: Uuid,
    pub amount_cents: i32,
    pub transaction_type: String,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub description: Option<String>,
    pub balance_after_cents: i32,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WalletTransactionsPage {
    pub transactions: Vec<WalletTransactionResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CancellationRequestResponse {
    pub id: Uuid,
    pub order_id: Uuid,
    pub order_number: Option<i32>,
    pub requested_by: Uuid,
    pub requester_name: Option<String>,
    pub reason: String,
    pub status: String,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<String>,
    pub created_at: String,
}

/* ============================================================
   REQUESTS
   ============================================================ */

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCancellationRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RespondCancellationRequest {
    pub accept: bool,
}

/* ============================================================
   CONVERSIONS
   ============================================================ */

impl From<&UserWallet> for WalletResponse {
    fn from(w: &UserWallet) -> Self {
        Self {
            id: w.id,
            user_id: w.user_id,
            balance_cents: w.balance_cents,
            currency: w.currency.clone(),
            created_at: w.created_at.to_rfc3339(),
            updated_at: w.updated_at.to_rfc3339(),
        }
    }
}

impl From<&WalletTransaction> for WalletTransactionResponse {
    fn from(t: &WalletTransaction) -> Self {
        Self {
            id: t.id,
            amount_cents: t.amount_cents,
            transaction_type: t.transaction_type.clone(),
            reference_type: t.reference_type.clone(),
            reference_id: t.reference_id,
            description: t.description.clone(),
            balance_after_cents: t.balance_after_cents,
            created_at: t.created_at.to_rfc3339(),
        }
    }
}
