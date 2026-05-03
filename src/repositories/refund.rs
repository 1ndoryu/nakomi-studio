/* [044A-38 Fase 7] Repositorio de reembolsos.
 * [044A-44] Migrado a query_as! con verificación en compilación.
 * CRUD sobre order_refunds: crear solicitud, listar, aprobar/rechazar/completar.
 * Solo un reembolso activo por orden (constraint en handler). */

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{OrderRefund, RefundStatus};

pub struct RefundRepository;

impl RefundRepository {
    /// Crear solicitud de reembolso
    pub async fn create(
        pool: &PgPool,
        order_id: Uuid,
        payment_id: Uuid,
        requested_by: Uuid,
        amount_cents: i32,
        reason: &str,
    ) -> Result<OrderRefund, sqlx::Error> {
        sqlx::query_as!(
            OrderRefund,
            r#"INSERT INTO order_refunds (order_id, payment_id, requested_by, amount_cents, reason)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, order_id, payment_id, requested_by, reviewed_by,
               amount_cents, reason, admin_response,
               status as "status: RefundStatus",
               stripe_refund_id, requested_at, reviewed_at, completed_at"#,
            order_id,
            payment_id,
            requested_by,
            amount_cents,
            reason,
        )
        .fetch_one(pool)
        .await
    }

    /// Buscar reembolso por ID
    pub async fn find_by_id(
        pool: &PgPool,
        refund_id: Uuid,
    ) -> Result<Option<OrderRefund>, sqlx::Error> {
        sqlx::query_as!(
            OrderRefund,
            r#"SELECT id, order_id, payment_id, requested_by, reviewed_by,
               amount_cents, reason, admin_response,
               status as "status: RefundStatus",
               stripe_refund_id, requested_at, reviewed_at, completed_at
             FROM order_refunds WHERE id = $1"#,
            refund_id,
        )
        .fetch_optional(pool)
        .await
    }

    /// Buscar reembolso activo (no rejected/completed) para una orden
    pub async fn find_active_for_order(
        pool: &PgPool,
        order_id: Uuid,
    ) -> Result<Option<OrderRefund>, sqlx::Error> {
        sqlx::query_as!(
            OrderRefund,
            r#"SELECT id, order_id, payment_id, requested_by, reviewed_by,
               amount_cents, reason, admin_response,
               status as "status: RefundStatus",
               stripe_refund_id, requested_at, reviewed_at, completed_at
             FROM order_refunds
             WHERE order_id = $1
               AND status NOT IN ('rejected', 'completed')
             ORDER BY requested_at DESC
             LIMIT 1"#,
            order_id,
        )
        .fetch_optional(pool)
        .await
    }

    /// Listar todos los reembolsos pendientes (admin)
    pub async fn list_pending(pool: &PgPool) -> Result<Vec<OrderRefund>, sqlx::Error> {
        sqlx::query_as!(
            OrderRefund,
            r#"SELECT id, order_id, payment_id, requested_by, reviewed_by,
               amount_cents, reason, admin_response,
               status as "status: RefundStatus",
               stripe_refund_id, requested_at, reviewed_at, completed_at
             FROM order_refunds
             WHERE status IN ('requested', 'under_review', 'approved')
             ORDER BY requested_at ASC"#,
        )
        .fetch_all(pool)
        .await
    }

    /// Listar reembolsos de un cliente específico
    pub async fn list_for_user(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<OrderRefund>, sqlx::Error> {
        sqlx::query_as!(
            OrderRefund,
            r#"SELECT id, order_id, payment_id, requested_by, reviewed_by,
               amount_cents, reason, admin_response,
               status as "status: RefundStatus",
               stripe_refund_id, requested_at, reviewed_at, completed_at
             FROM order_refunds
             WHERE requested_by = $1
             ORDER BY requested_at DESC"#,
            user_id,
        )
        .fetch_all(pool)
        .await
    }

    /// Buscar reembolso de una orden para un cliente
    pub async fn find_for_order(
        pool: &PgPool,
        order_id: Uuid,
    ) -> Result<Option<OrderRefund>, sqlx::Error> {
        sqlx::query_as!(
            OrderRefund,
            r#"SELECT id, order_id, payment_id, requested_by, reviewed_by,
               amount_cents, reason, admin_response,
               status as "status: RefundStatus",
               stripe_refund_id, requested_at, reviewed_at, completed_at
             FROM order_refunds
             WHERE order_id = $1
             ORDER BY requested_at DESC
             LIMIT 1"#,
            order_id,
        )
        .fetch_optional(pool)
        .await
    }

    /// Aprobar reembolso (admin)
    pub async fn approve(
        pool: &PgPool,
        refund_id: Uuid,
        admin_id: Uuid,
        admin_response: Option<&str>,
    ) -> Result<OrderRefund, sqlx::Error> {
        sqlx::query_as!(
            OrderRefund,
            r#"UPDATE order_refunds
             SET status = 'approved',
                 reviewed_by = $2,
                 admin_response = $3,
                 reviewed_at = NOW()
             WHERE id = $1
             RETURNING id, order_id, payment_id, requested_by, reviewed_by,
               amount_cents, reason, admin_response,
               status as "status: RefundStatus",
               stripe_refund_id, requested_at, reviewed_at, completed_at"#,
            refund_id,
            admin_id,
            admin_response,
        )
        .fetch_one(pool)
        .await
    }

    /// Rechazar reembolso (admin)
    pub async fn reject(
        pool: &PgPool,
        refund_id: Uuid,
        admin_id: Uuid,
        admin_response: Option<&str>,
    ) -> Result<OrderRefund, sqlx::Error> {
        sqlx::query_as!(
            OrderRefund,
            r#"UPDATE order_refunds
             SET status = 'rejected',
                 reviewed_by = $2,
                 admin_response = $3,
                 reviewed_at = NOW()
             WHERE id = $1
             RETURNING id, order_id, payment_id, requested_by, reviewed_by,
               amount_cents, reason, admin_response,
               status as "status: RefundStatus",
               stripe_refund_id, requested_at, reviewed_at, completed_at"#,
            refund_id,
            admin_id,
            admin_response,
        )
        .fetch_one(pool)
        .await
    }

    /// Marcar reembolso como completado (Stripe refund exitoso)
    pub async fn mark_completed(
        pool: &PgPool,
        refund_id: Uuid,
        stripe_refund_id: &str,
    ) -> Result<OrderRefund, sqlx::Error> {
        sqlx::query_as!(
            OrderRefund,
            r#"UPDATE order_refunds
             SET status = 'completed',
                 stripe_refund_id = $2,
                 completed_at = NOW()
             WHERE id = $1
             RETURNING id, order_id, payment_id, requested_by, reviewed_by,
               amount_cents, reason, admin_response,
               status as "status: RefundStatus",
               stripe_refund_id, requested_at, reviewed_at, completed_at"#,
            refund_id,
            stripe_refund_id,
        )
        .fetch_one(pool)
        .await
    }

    /// Actualizar status a `under_review`
    pub async fn set_under_review(
        pool: &PgPool,
        refund_id: Uuid,
    ) -> Result<OrderRefund, sqlx::Error> {
        sqlx::query_as!(
            OrderRefund,
            r#"UPDATE order_refunds
             SET status = 'under_review'
             WHERE id = $1
             RETURNING id, order_id, payment_id, requested_by, reviewed_by,
               amount_cents, reason, admin_response,
               status as "status: RefundStatus",
               stripe_refund_id, requested_at, reviewed_at, completed_at"#,
            refund_id,
        )
        .fetch_one(pool)
        .await
    }
}
