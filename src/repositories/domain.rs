/* sentinel-disable-file sqlx-query-sin-macro sqlx-query-as-sin-macro: dominio self-service nuevo con queries preparadas dinámicas.
 * [195A-1] Repositorio de órdenes de dominio pagadas por Stripe. */
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::DomainOrder;

pub struct DomainOrderRepository;

pub struct CreateDomainOrderParams<'a> {
    pub user_id: Uuid,
    pub domain: &'a str,
    pub tld: &'a str,
    pub base_cost_cents: i32,
    pub price_cents: i32,
}

impl DomainOrderRepository {
    pub async fn create(
        pool: &PgPool,
        params: CreateDomainOrderParams<'_>,
    ) -> Result<DomainOrder, AppError> {
        sqlx::query_as::<_, DomainOrder>(
            r"INSERT INTO domain_orders (user_id, domain, tld, base_cost_cents, price_cents)
              VALUES ($1, $2, $3, $4, $5)
              RETURNING id, user_id, domain, tld, status, base_cost_cents, price_cents,
                        stripe_session_id, created_at, updated_at",
        )
        .bind(params.user_id)
        .bind(params.domain)
        .bind(params.tld)
        .bind(params.base_cost_cents)
        .bind(params.price_cents)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn list_all(pool: &PgPool) -> Result<Vec<DomainOrder>, AppError> {
        sqlx::query_as::<_, DomainOrder>(
            r"SELECT id, user_id, domain, tld, status, base_cost_cents, price_cents,
                      stripe_session_id, created_at, updated_at
              FROM domain_orders
              ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn list_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<DomainOrder>, AppError> {
        sqlx::query_as::<_, DomainOrder>(
            r"SELECT id, user_id, domain, tld, status, base_cost_cents, price_cents,
                      stripe_session_id, created_at, updated_at
              FROM domain_orders
              WHERE user_id = $1
              ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn set_checkout_session(
        pool: &PgPool,
        id: Uuid,
        stripe_session_id: &str,
    ) -> Result<DomainOrder, AppError> {
        sqlx::query_as::<_, DomainOrder>(
            r"UPDATE domain_orders
              SET stripe_session_id = $1, updated_at = NOW()
              WHERE id = $2
              RETURNING id, user_id, domain, tld, status, base_cost_cents, price_cents,
                        stripe_session_id, created_at, updated_at",
        )
        .bind(stripe_session_id)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn mark_paid_by_session(
        pool: &PgPool,
        stripe_session_id: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r"UPDATE domain_orders
              SET status = 'paid_pending_registration', updated_at = NOW()
              WHERE stripe_session_id = $1 AND status = 'pending_payment'",
        )
        .bind(stripe_session_id)
        .execute(pool)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }
}
