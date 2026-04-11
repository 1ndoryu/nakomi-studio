/* [154A-15a] Wallet service: lógica de negocio para saldo virtual.
 * Métodos credit/debit delegan al repositorio.
 * get_or_create: obtiene wallet existente o crea uno nuevo.
 * Transacciones: historial paginado de movimientos. */

use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{
    WalletResponse, WalletTransactionResponse, WalletTransactionsPage,
};
use crate::repositories::WalletRepository;

pub struct WalletService;

impl WalletService {
    pub async fn get_balance(pool: &PgPool, user_id: Uuid) -> Result<WalletResponse, AppError> {
        let wallet = WalletRepository::get_or_create(pool, user_id).await?;
        /* [204A-11] Incluir saldo retirable (créditos >7 días) */
        let withdrawable = WalletRepository::get_withdrawable_balance(pool, user_id).await?;
        Ok(WalletResponse::from_wallet(&wallet, withdrawable))
    }

    pub async fn credit(
        pool: &PgPool,
        user_id: Uuid,
        amount_cents: i32,
        transaction_type: &str,
        reference_type: Option<&str>,
        reference_id: Option<Uuid>,
        description: Option<&str>,
    ) -> Result<WalletTransactionResponse, AppError> {
        let tx = WalletRepository::credit(
            pool,
            user_id,
            amount_cents,
            transaction_type,
            reference_type,
            reference_id,
            description,
        )
        .await?;
        Ok(WalletTransactionResponse::from(&tx))
    }

    pub async fn debit(
        pool: &PgPool,
        user_id: Uuid,
        amount_cents: i32,
        transaction_type: &str,
        reference_type: Option<&str>,
        reference_id: Option<Uuid>,
        description: Option<&str>,
    ) -> Result<WalletTransactionResponse, AppError> {
        let tx = WalletRepository::debit(
            pool,
            user_id,
            amount_cents,
            transaction_type,
            reference_type,
            reference_id,
            description,
        )
        .await?;
        Ok(WalletTransactionResponse::from(&tx))
    }

    pub async fn list_transactions(
        pool: &PgPool,
        user_id: Uuid,
        page: i64,
        per_page: i64,
    ) -> Result<WalletTransactionsPage, AppError> {
        let per_page = per_page.clamp(1, 100);
        let page = page.max(1);

        let (rows, total) =
            WalletRepository::list_transactions(pool, user_id, page, per_page).await?;

        let transactions = rows.iter().map(WalletTransactionResponse::from).collect();

        Ok(WalletTransactionsPage {
            transactions,
            total,
            page,
            per_page,
        })
    }
}
