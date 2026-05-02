/* [154A-15a] Wallet repository: queries para saldo virtual, transacciones
 * y solicitudes de cancelación. Todas las operaciones de saldo usan
 * transacciones SQL para garantizar consistencia atómica. */

use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{CancellationRequest, UserWallet, WalletTransaction};

pub struct WalletRepository;

impl WalletRepository {
    /* Obtener o crear wallet del usuario (on-demand).
     * Si el user_id no existe en users (FK 23503), el usuario fue eliminado — retorna Unauthorized. */
    pub async fn get_or_create(pool: &PgPool, user_id: Uuid) -> Result<UserWallet, AppError> {
        let result = sqlx::query_as!(
            UserWallet,
            r#"
            INSERT INTO user_wallets (user_id)
            VALUES ($1)
            ON CONFLICT (user_id) DO UPDATE SET updated_at = user_wallets.updated_at
            RETURNING id, user_id, balance_cents, currency, created_at, updated_at
            "#,
            user_id
        )
        .fetch_one(pool)
        .await;

        match result {
            Ok(wallet) => Ok(wallet),
            Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23503") => {
                /* FK violation: el user_id no existe en users — token de usuario eliminado */
                Err(AppError::Unauthorized)
            }
            Err(e) => Err(AppError::Database(e)),
        }
    }

    /* Obtener wallet existente (sin crear) */
    pub async fn find_by_user(pool: &PgPool, user_id: Uuid) -> Result<Option<UserWallet>, AppError> {
        let wallet = sqlx::query_as!(
            UserWallet,
            "SELECT id, user_id, balance_cents, currency, created_at, updated_at FROM user_wallets WHERE user_id = $1",
            user_id
        )
        .fetch_optional(pool)
        .await?;
        Ok(wallet)
    }

    /* [204A-11] Saldo disponible para retiro: solo créditos con más de 7 días.
     * Fórmula: SUM(créditos con created_at <= NOW()-7d) + SUM(débitos).
     * Esto garantiza que las ganancias recientes no se puedan retirar. */
    pub async fn get_withdrawable_balance(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<i32, AppError> {
        /* Retorna tupla (i64,) con COALESCE de subqueries — query_as! macro
         * no soporta tuplas anonimas como tipo destino. */
        // sentinel-disable-next-line sqlx-query-as-sin-macro
        let result: (i64,) = sqlx::query_as(
            r"
            SELECT COALESCE(
                (SELECT COALESCE(SUM(amount_cents), 0)
                 FROM wallet_transactions
                 WHERE user_id = $1 AND amount_cents > 0
                   AND created_at <= NOW() - INTERVAL '7 days')
                +
                (SELECT COALESCE(SUM(amount_cents), 0)
                 FROM wallet_transactions
                 WHERE user_id = $1 AND amount_cents < 0),
                0
            )
            ",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;
        /* No permitir negativo */
        #[allow(clippy::cast_possible_truncation)]
        Ok(result.0.max(0) as i32)
    }

    /* Creditar saldo (ingreso). Retorna la transacción creada.
     * Usa subconsulta atómica para evitar race conditions. */
    pub async fn credit(
        pool: &PgPool,
        user_id: Uuid,
        amount_cents: i32,
        transaction_type: &str,
        reference_type: Option<&str>,
        reference_id: Option<Uuid>,
        description: Option<&str>,
    ) -> Result<WalletTransaction, AppError> {
        if amount_cents <= 0 {
            return Err(AppError::BadRequest("El monto debe ser positivo".into()));
        }

        let mut tx = pool.begin().await?;

        /* Asegurar que el wallet existe */
        sqlx::query!(
            "INSERT INTO user_wallets (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING",
            user_id
        )
        .execute(&mut *tx)
        .await?;

        /* Actualizar balance atómicamente y obtener el wallet actualizado */
        let wallet = sqlx::query_as!(
            UserWallet,
            r#"
            UPDATE user_wallets
            SET balance_cents = balance_cents + $1, updated_at = NOW()
            WHERE user_id = $2
            RETURNING id, user_id, balance_cents, currency, created_at, updated_at
            "#,
            amount_cents,
            user_id
        )
        .fetch_one(&mut *tx)
        .await?;

        /* Insertar transacción con snapshot del balance */
        let transaction = sqlx::query_as!(
            WalletTransaction,
            r#"
            INSERT INTO wallet_transactions
                (wallet_id, user_id, amount_cents, transaction_type, reference_type, reference_id, description, balance_after_cents)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, wallet_id, user_id, amount_cents, transaction_type, reference_type, reference_id, description, balance_after_cents, created_at
            "#,
            wallet.id,
            user_id,
            amount_cents,
            transaction_type,
            reference_type,
            reference_id,
            description,
            wallet.balance_cents
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(transaction)
    }

    /* Debitar saldo (gasto). Verifica que haya fondos suficientes. */
    pub async fn debit(
        pool: &PgPool,
        user_id: Uuid,
        amount_cents: i32,
        transaction_type: &str,
        reference_type: Option<&str>,
        reference_id: Option<Uuid>,
        description: Option<&str>,
    ) -> Result<WalletTransaction, AppError> {
        if amount_cents <= 0 {
            return Err(AppError::BadRequest("El monto debe ser positivo".into()));
        }

        let mut tx = pool.begin().await?;

        /* Obtener wallet con lock FOR UPDATE para evitar race conditions */
        let wallet = sqlx::query_as!(
            UserWallet,
            "SELECT id, user_id, balance_cents, currency, created_at, updated_at FROM user_wallets WHERE user_id = $1 FOR UPDATE",
            user_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::BadRequest("No tienes wallet activo".into()))?;

        if wallet.balance_cents < amount_cents {
            return Err(AppError::BadRequest("Fondos insuficientes".into()));
        }

        let new_balance = wallet.balance_cents - amount_cents;

        sqlx::query!(
            "UPDATE user_wallets SET balance_cents = $1, updated_at = NOW() WHERE id = $2",
            new_balance,
            wallet.id
        )
        .execute(&mut *tx)
        .await?;

        let transaction = sqlx::query_as!(
            WalletTransaction,
            r#"
            INSERT INTO wallet_transactions
                (wallet_id, user_id, amount_cents, transaction_type, reference_type, reference_id, description, balance_after_cents)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, wallet_id, user_id, amount_cents, transaction_type, reference_type, reference_id, description, balance_after_cents, created_at
            "#,
            wallet.id,
            user_id,
            -amount_cents,
            transaction_type,
            reference_type,
            reference_id,
            description,
            new_balance
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(transaction)
    }

    /* Listar transacciones paginadas */
    pub async fn list_transactions(
        pool: &PgPool,
        user_id: Uuid,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<WalletTransaction>, i64), AppError> {
        let offset = (page - 1) * per_page;

        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM wallet_transactions WHERE user_id = $1",
            user_id
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            WalletTransaction,
            r#"
            SELECT id, wallet_id, user_id, amount_cents, transaction_type,
                   reference_type, reference_id, description, balance_after_cents, created_at
            FROM wallet_transactions
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            per_page,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok((rows, total))
    }
}

/* ============================================================
   CANCELLATION REQUESTS
   ============================================================ */

pub struct CancellationRequestRepository;

impl CancellationRequestRepository {
    pub async fn create(
        pool: &PgPool,
        order_id: Uuid,
        requested_by: Uuid,
        reason: &str,
    ) -> Result<CancellationRequest, AppError> {
        let row = sqlx::query_as!(
            CancellationRequest,
            r#"
            INSERT INTO cancellation_requests (order_id, requested_by, reason)
            VALUES ($1, $2, $3)
            RETURNING id, order_id, requested_by, reason, status, resolved_by, resolved_at, created_at
            "#,
            order_id,
            requested_by,
            reason
        )
        .fetch_one(pool)
        .await?;
        Ok(row)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<CancellationRequest>, AppError> {
        let row = sqlx::query_as!(
            CancellationRequest,
            "SELECT id, order_id, requested_by, reason, status, resolved_by, resolved_at, created_at FROM cancellation_requests WHERE id = $1",
            id
        )
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }

    pub async fn find_pending_by_order(pool: &PgPool, order_id: Uuid) -> Result<Option<CancellationRequest>, AppError> {
        let row = sqlx::query_as!(
            CancellationRequest,
            "SELECT id, order_id, requested_by, reason, status, resolved_by, resolved_at, created_at FROM cancellation_requests WHERE order_id = $1 AND status = 'pending' ORDER BY created_at DESC LIMIT 1",
            order_id
        )
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }

    pub async fn resolve(
        pool: &PgPool,
        id: Uuid,
        resolved_by: Uuid,
        accept: bool,
    ) -> Result<CancellationRequest, AppError> {
        let status = if accept { "accepted" } else { "rejected" };
        let row = sqlx::query_as!(
            CancellationRequest,
            r#"
            UPDATE cancellation_requests
            SET status = $1, resolved_by = $2, resolved_at = NOW()
            WHERE id = $3
            RETURNING id, order_id, requested_by, reason, status, resolved_by, resolved_at, created_at
            "#,
            status,
            resolved_by,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(row)
    }

    pub async fn list_by_order(pool: &PgPool, order_id: Uuid) -> Result<Vec<CancellationRequest>, AppError> {
        let rows = sqlx::query_as!(
            CancellationRequest,
            "SELECT id, order_id, requested_by, reason, status, resolved_by, resolved_at, created_at FROM cancellation_requests WHERE order_id = $1 ORDER BY created_at DESC",
            order_id
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}

/* ============================================================
   WITHDRAWAL REQUESTS
   [184A-1] Solicitudes de retiro de fondos del wallet.
   ============================================================ */

use crate::models::WithdrawalRequest;

pub struct WithdrawalRequestRepository;

impl WithdrawalRequestRepository {
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        amount_cents: i32,
        payment_method: Option<&str>,
        payment_details: Option<&str>,
    ) -> Result<WithdrawalRequest, AppError> {
        let row = sqlx::query_as!(
            WithdrawalRequest,
            r#"
            INSERT INTO withdrawal_requests (user_id, amount_cents, payment_method, payment_details)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, amount_cents, status, payment_method, payment_details,
                      admin_notes, resolved_by, created_at, resolved_at
            "#,
            user_id,
            amount_cents,
            payment_method,
            payment_details
        )
        .fetch_one(pool)
        .await?;
        Ok(row)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<WithdrawalRequest>, AppError> {
        let row = sqlx::query_as!(
            WithdrawalRequest,
            r#"
            SELECT id, user_id, amount_cents, status, payment_method, payment_details,
                   admin_notes, resolved_by, created_at, resolved_at
            FROM withdrawal_requests WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }

    pub async fn has_pending(pool: &PgPool, user_id: Uuid) -> Result<bool, AppError> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM withdrawal_requests WHERE user_id = $1 AND status = 'pending'",
            user_id
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);
        Ok(count > 0)
    }

    pub async fn resolve(
        pool: &PgPool,
        id: Uuid,
        resolved_by: Uuid,
        approve: bool,
        admin_notes: Option<&str>,
    ) -> Result<WithdrawalRequest, AppError> {
        let status = if approve { "approved" } else { "rejected" };
        let row = sqlx::query_as!(
            WithdrawalRequest,
            r#"
            UPDATE withdrawal_requests
            SET status = $1, resolved_by = $2, admin_notes = $3, resolved_at = NOW()
            WHERE id = $4
            RETURNING id, user_id, amount_cents, status, payment_method, payment_details,
                      admin_notes, resolved_by, created_at, resolved_at
            "#,
            status,
            resolved_by,
            admin_notes,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(row)
    }

    pub async fn list_by_user(
        pool: &PgPool,
        user_id: Uuid,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<WithdrawalRequest>, i64), AppError> {
        let offset = (page - 1) * per_page;
        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM withdrawal_requests WHERE user_id = $1",
            user_id
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            WithdrawalRequest,
            r#"
            SELECT id, user_id, amount_cents, status, payment_method, payment_details,
                   admin_notes, resolved_by, created_at, resolved_at
            FROM withdrawal_requests WHERE user_id = $1
            ORDER BY created_at DESC LIMIT $2 OFFSET $3
            "#,
            user_id, per_page, offset
        )
        .fetch_all(pool)
        .await?;
        Ok((rows, total))
    }

    pub async fn list_pending(
        pool: &PgPool,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<WithdrawalRequest>, i64), AppError> {
        let offset = (page - 1) * per_page;
        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM withdrawal_requests WHERE status = 'pending'"
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            WithdrawalRequest,
            r#"
            SELECT id, user_id, amount_cents, status, payment_method, payment_details,
                   admin_notes, resolved_by, created_at, resolved_at
            FROM withdrawal_requests WHERE status = 'pending'
            ORDER BY created_at ASC LIMIT $1 OFFSET $2
            "#,
            per_page, offset
        )
        .fetch_all(pool)
        .await?;
        Ok((rows, total))
    }
}
