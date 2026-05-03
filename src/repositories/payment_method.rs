use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::models::UserPaymentMethod;

pub struct UpsertPaymentMethodParams<'a> {
    pub user_id: Uuid,
    pub stripe_payment_method_id: &'a str,
    pub card_fingerprint: &'a str,
    pub brand: &'a str,
    pub last_four: &'a str,
    pub exp_month: i32,
    pub exp_year: i32,
    pub is_default: bool,
}

pub struct PaymentMethodRepository;

impl PaymentMethodRepository {
    pub async fn find_owned(
        pool: &PgPool,
        user_id: Uuid,
        payment_method_id: Uuid,
    ) -> Result<Option<UserPaymentMethod>, sqlx::Error> {
        sqlx::query_as!(
            UserPaymentMethod,
            r#"SELECT id, user_id, stripe_payment_method_id, card_fingerprint, brand,
                      last_four, exp_month, exp_year, is_default, created_at, updated_at
               FROM user_payment_methods
               WHERE user_id = $1 AND id = $2"#,
            user_id,
            payment_method_id,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn list_for_user(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<UserPaymentMethod>, sqlx::Error> {
        sqlx::query_as!(
            UserPaymentMethod,
            r#"SELECT id, user_id, stripe_payment_method_id, card_fingerprint, brand,
                      last_four, exp_month, exp_year, is_default, created_at, updated_at
               FROM user_payment_methods
               WHERE user_id = $1
               ORDER BY is_default DESC, created_at DESC"#,
            user_id,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn has_default_for_user(pool: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM user_payment_methods WHERE user_id = $1 AND is_default = TRUE) AS \"exists!\"",
            user_id,
        )
        .fetch_one(pool)
        .await?;

        Ok(row.exists)
    }

    pub async fn upsert_from_stripe(
        pool: &PgPool,
        params: UpsertPaymentMethodParams<'_>,
    ) -> Result<UserPaymentMethod, sqlx::Error> {
        let mut tx = pool.begin().await?;

        if params.is_default {
            Self::clear_default(&mut tx, params.user_id).await?;
        }

        let payment_method = sqlx::query_as!(
            UserPaymentMethod,
            r#"INSERT INTO user_payment_methods (
                    user_id,
                    stripe_payment_method_id,
                    card_fingerprint,
                    brand,
                    last_four,
                    exp_month,
                    exp_year,
                    is_default
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (user_id, card_fingerprint) DO UPDATE SET
                    stripe_payment_method_id = EXCLUDED.stripe_payment_method_id,
                    brand = EXCLUDED.brand,
                    last_four = EXCLUDED.last_four,
                    exp_month = EXCLUDED.exp_month,
                    exp_year = EXCLUDED.exp_year,
                    is_default = CASE
                        WHEN EXCLUDED.is_default THEN TRUE
                        ELSE user_payment_methods.is_default
                    END,
                    updated_at = NOW()
                RETURNING id, user_id, stripe_payment_method_id, card_fingerprint, brand,
                          last_four, exp_month, exp_year, is_default, created_at, updated_at"#,
            params.user_id,
            params.stripe_payment_method_id,
            params.card_fingerprint,
            params.brand,
            params.last_four,
            params.exp_month,
            params.exp_year,
            params.is_default,
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(payment_method)
    }

    pub async fn delete_owned(
        pool: &PgPool,
        user_id: Uuid,
        payment_method_id: Uuid,
    ) -> Result<Option<UserPaymentMethod>, sqlx::Error> {
        let mut tx = pool.begin().await?;
        let deleted_method = sqlx::query_as!(
            UserPaymentMethod,
            r#"DELETE FROM user_payment_methods
               WHERE id = $1 AND user_id = $2
               RETURNING id, user_id, stripe_payment_method_id, card_fingerprint, brand,
                         last_four, exp_month, exp_year, is_default, created_at, updated_at"#,
            payment_method_id,
            user_id,
        )
        .fetch_optional(&mut *tx)
        .await?;

        if deleted_method
            .as_ref()
            .is_some_and(|method| method.is_default)
        {
            sqlx::query!(
                r#"UPDATE user_payment_methods
                   SET is_default = TRUE, updated_at = NOW()
                   WHERE id = (
                       SELECT id
                       FROM user_payment_methods
                       WHERE user_id = $1
                       ORDER BY created_at DESC
                       LIMIT 1
                   )"#,
                user_id,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(deleted_method)
    }

    async fn clear_default(
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE user_payment_methods SET is_default = FALSE, updated_at = NOW() WHERE user_id = $1 AND is_default = TRUE",
            user_id,
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}
