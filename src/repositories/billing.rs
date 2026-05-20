/* sentinel-disable-file sqlx-query-sin-macro sqlx-query-as-sin-macro: repositorio nuevo con filtros opcionales.
 * [205A-1] Queries preparadas con bind para cobros pendientes del panel. */
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::BillingItem;

pub struct BillingRepository;

impl BillingRepository {
    pub async fn list_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<BillingItem>, AppError> {
        sqlx::query_as::<_, BillingItem>(
            r"SELECT id, user_id, resource_type, resource_id, title, description,
                      amount_cents, currency, billing_period, status, due_at,
                      grace_period_ends_at, paid_at, stripe_session_id, metadata,
                      created_at, updated_at
              FROM billing_items
              WHERE user_id = $1
              ORDER BY CASE WHEN status = 'pending' THEN 0 ELSE 1 END, due_at ASC, created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn pending_for_checkout(
        pool: &PgPool,
        user_id: Uuid,
        item_ids: Option<&[Uuid]>,
    ) -> Result<Vec<BillingItem>, AppError> {
        let ids: Option<Vec<Uuid>> = item_ids.map(<[Uuid]>::to_vec);
        sqlx::query_as::<_, BillingItem>(
            r"SELECT id, user_id, resource_type, resource_id, title, description,
                      amount_cents, currency, billing_period, status, due_at,
                      grace_period_ends_at, paid_at, stripe_session_id, metadata,
                      created_at, updated_at
              FROM billing_items
              WHERE user_id = $1
                AND status = 'pending'
                AND ($2::uuid[] IS NULL OR id = ANY($2))
              ORDER BY due_at ASC, created_at ASC",
        )
        .bind(user_id)
        .bind(ids)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn set_checkout_session(
        pool: &PgPool,
        item_ids: &[Uuid],
        stripe_session_id: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r"UPDATE billing_items
              SET stripe_session_id = $1, updated_at = NOW()
              WHERE id = ANY($2) AND status = 'pending'",
        )
        .bind(stripe_session_id)
        .bind(item_ids)
        .execute(pool)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }

    pub async fn mark_paid_by_session(
        pool: &PgPool,
        stripe_session_id: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r"UPDATE billing_items
              SET status = 'paid', paid_at = COALESCE(paid_at, NOW()), updated_at = NOW()
              WHERE stripe_session_id = $1 AND status = 'pending'",
        )
        .bind(stripe_session_id)
        .execute(pool)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }
}
