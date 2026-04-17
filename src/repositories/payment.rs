/* [044A-38 Fase 3] Repositorio de pagos: CRUD sobre order_payments.
 * [044A-44] Migrado a query_as! con verificación en compilación. */

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{OrderPayment, PaymentMode, PaymentStatus};

pub struct CreatePaymentParams<'a> {
    pub order_id: Uuid,
    pub phase_id: Option<Uuid>,
    pub amount_cents: i32,
    pub currency: &'a str,
    pub payment_mode: PaymentMode,
    pub stripe_payment_intent_id: &'a str,
    pub description: Option<&'a str>,
}

pub struct PaymentRepository;

impl PaymentRepository {
    pub async fn create_payment(
        pool: &PgPool,
        params: CreatePaymentParams<'_>,
    ) -> Result<OrderPayment, sqlx::Error> {
        sqlx::query_as!(
            OrderPayment,
            r#"INSERT INTO order_payments (order_id, phase_id, amount_cents, currency, status,
             payment_mode, stripe_payment_intent_id, description)
             VALUES ($1, $2, $3, $4, 'pending', $5, $6, $7)
             RETURNING id, order_id, phase_id, amount_cents, currency,
                       status as "status: PaymentStatus", payment_mode as "payment_mode: PaymentMode",
                       stripe_payment_intent_id, stripe_charge_id, held_at, released_at,
                       description, created_at, updated_at"#,
            params.order_id,
            params.phase_id,
            params.amount_cents,
            params.currency,
            params.payment_mode as PaymentMode,
            params.stripe_payment_intent_id,
            params.description,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_stripe_intent(
        pool: &PgPool,
        stripe_pi_id: &str,
    ) -> Result<Option<OrderPayment>, sqlx::Error> {
        sqlx::query_as!(
            OrderPayment,
            r#"SELECT id, order_id, phase_id, amount_cents, currency,
                      status as "status: PaymentStatus", payment_mode as "payment_mode: PaymentMode",
                      stripe_payment_intent_id, stripe_charge_id, held_at, released_at,
                      description, created_at, updated_at
             FROM order_payments WHERE stripe_payment_intent_id = $1"#,
            stripe_pi_id,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn update_status_held(
        pool: &PgPool,
        payment_id: Uuid,
    ) -> Result<OrderPayment, sqlx::Error> {
        sqlx::query_as!(
            OrderPayment,
            r#"UPDATE order_payments SET status = 'held', held_at = NOW(), updated_at = NOW()
             WHERE id = $1
             RETURNING id, order_id, phase_id, amount_cents, currency,
                       status as "status: PaymentStatus", payment_mode as "payment_mode: PaymentMode",
                       stripe_payment_intent_id, stripe_charge_id, held_at, released_at,
                       description, created_at, updated_at"#,
            payment_id,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn update_status_released(
        pool: &PgPool,
        payment_id: Uuid,
    ) -> Result<OrderPayment, sqlx::Error> {
        sqlx::query_as!(
            OrderPayment,
            r#"UPDATE order_payments SET status = 'released', released_at = NOW(), updated_at = NOW()
             WHERE id = $1
             RETURNING id, order_id, phase_id, amount_cents, currency,
                       status as "status: PaymentStatus", payment_mode as "payment_mode: PaymentMode",
                       stripe_payment_intent_id, stripe_charge_id, held_at, released_at,
                       description, created_at, updated_at"#,
            payment_id,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn update_status(
        pool: &PgPool,
        payment_id: Uuid,
        status: PaymentStatus,
    ) -> Result<OrderPayment, sqlx::Error> {
        sqlx::query_as!(
            OrderPayment,
            r#"UPDATE order_payments SET status = $2, updated_at = NOW()
             WHERE id = $1
             RETURNING id, order_id, phase_id, amount_cents, currency,
                       status as "status: PaymentStatus", payment_mode as "payment_mode: PaymentMode",
                       stripe_payment_intent_id, stripe_charge_id, held_at, released_at,
                       description, created_at, updated_at"#,
            payment_id,
            status as PaymentStatus,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn update_charge_id(
        pool: &PgPool,
        payment_id: Uuid,
        charge_id: &str,
    ) -> Result<OrderPayment, sqlx::Error> {
        sqlx::query_as!(
            OrderPayment,
            r#"UPDATE order_payments SET stripe_charge_id = $2, updated_at = NOW()
             WHERE id = $1
             RETURNING id, order_id, phase_id, amount_cents, currency,
                       status as "status: PaymentStatus", payment_mode as "payment_mode: PaymentMode",
                       stripe_payment_intent_id, stripe_charge_id, held_at, released_at,
                       description, created_at, updated_at"#,
            payment_id,
            charge_id,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn list_for_order(
        pool: &PgPool,
        order_id: Uuid,
    ) -> Result<Vec<OrderPayment>, sqlx::Error> {
        sqlx::query_as!(
            OrderPayment,
            r#"SELECT id, order_id, phase_id, amount_cents, currency,
                      status as "status: PaymentStatus", payment_mode as "payment_mode: PaymentMode",
                      stripe_payment_intent_id, stripe_charge_id, held_at, released_at,
                      description, created_at, updated_at
             FROM order_payments WHERE order_id = $1 ORDER BY created_at DESC"#,
            order_id,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_held_for_order(
        pool: &PgPool,
        order_id: Uuid,
    ) -> Result<Vec<OrderPayment>, sqlx::Error> {
        sqlx::query_as!(
            OrderPayment,
            r#"SELECT id, order_id, phase_id, amount_cents, currency,
                      status as "status: PaymentStatus", payment_mode as "payment_mode: PaymentMode",
                      stripe_payment_intent_id, stripe_charge_id, held_at, released_at,
                      description, created_at, updated_at
             FROM order_payments WHERE order_id = $1 AND status = 'held'
             ORDER BY created_at"#,
            order_id,
        )
        .fetch_all(pool)
        .await
    }

    /* [124A-SENT-R1] Comprueba si un evento Stripe ya fue procesado (deduplicación idempotente).
     * runtime query (sin macro). Retorna false ante cualquier error de BD. */
    pub async fn is_event_processed(pool: &PgPool, event_id: &str) -> bool {
        sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM stripe_processed_events WHERE event_id = $1)"
        )
        .bind(event_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false)
    }

    /* [124A-SENT-R1] Marca un evento Stripe como procesado.
     * ON CONFLICT DO NOTHING garantiza idempotencia sin panic. */
    pub async fn mark_event_processed(
        pool: &PgPool,
        event_id: &str,
        event_type: &str,
    ) -> Result<(), sqlx::Error> {
        // sentinel-disable-next-line sqlx-query-sin-macro
        sqlx::query(
            "INSERT INTO stripe_processed_events (event_id, event_type) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(event_id)
        .bind(event_type)
        .execute(pool)
        .await?;
        Ok(())
    }
}
