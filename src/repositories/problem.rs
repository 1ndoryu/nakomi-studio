/* [104A-28] Repository de problemas reportados en órdenes.
 * CRUD con queries tipados para la tabla order_problems. */

use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{OrderProblem, ProblemStatus};

pub struct ProblemRepository;

impl ProblemRepository {
    pub async fn create(
        pool: &PgPool,
        order_id: Uuid,
        reporter_id: Uuid,
        reporter_role: &str,
        reason: &str,
    ) -> Result<OrderProblem, AppError> {
        let row = sqlx::query_as!(
            OrderProblem,
            r#"INSERT INTO order_problems (order_id, reporter_id, reporter_role, reason)
               VALUES ($1, $2, $3, $4)
               RETURNING id, order_id, reporter_id, reporter_role, reason,
                         status as "status: ProblemStatus", admin_response, resolved_by,
                         resolved_at, created_at, updated_at"#,
            order_id,
            reporter_id,
            reporter_role,
            reason,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error creando problema: {e}")))?;
        Ok(row)
    }

    pub async fn list_all(pool: &PgPool) -> Result<Vec<ProblemWithContext>, AppError> {
        let rows = sqlx::query_as!(
            ProblemWithContext,
            r#"SELECT op.id, op.order_id, op.reporter_id, op.reporter_role, op.reason,
                      op.status as "status: ProblemStatus", op.admin_response, op.resolved_by,
                      op.resolved_at, op.created_at, op.updated_at,
                      o.order_number,
                      COALESCE(u.display_name, u.email) AS "reporter_name!"
               FROM order_problems op
               JOIN orders o ON o.id = op.order_id
               JOIN users u ON u.id = op.reporter_id
               ORDER BY
                 CASE op.status WHEN 'open' THEN 0 WHEN 'in_review' THEN 1 ELSE 2 END,
                 op.created_at DESC"#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error listando problemas: {e}")))?;
        Ok(rows)
    }

    pub async fn list_by_order(
        pool: &PgPool,
        order_id: Uuid,
    ) -> Result<Vec<ProblemWithContext>, AppError> {
        let rows = sqlx::query_as!(
            ProblemWithContext,
            r#"SELECT op.id, op.order_id, op.reporter_id, op.reporter_role, op.reason,
                      op.status as "status: ProblemStatus", op.admin_response, op.resolved_by,
                      op.resolved_at, op.created_at, op.updated_at,
                      o.order_number,
                      COALESCE(u.display_name, u.email) AS "reporter_name!"
               FROM order_problems op
               JOIN orders o ON o.id = op.order_id
               JOIN users u ON u.id = op.reporter_id
               WHERE op.order_id = $1
               ORDER BY op.created_at DESC"#,
            order_id,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error listando problemas de orden: {e}")))?;
        Ok(rows)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<OrderProblem>, AppError> {
        let row = sqlx::query_as!(
            OrderProblem,
            r#"SELECT id, order_id, reporter_id, reporter_role, reason,
                      status as "status: ProblemStatus", admin_response, resolved_by,
                      resolved_at, created_at, updated_at
               FROM order_problems WHERE id = $1"#,
            id,
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error buscando problema: {e}")))?;
        Ok(row)
    }

    pub async fn resolve(
        pool: &PgPool,
        id: Uuid,
        status: ProblemStatus,
        admin_id: Uuid,
        response: Option<&str>,
    ) -> Result<OrderProblem, AppError> {
        let row = sqlx::query_as!(
            OrderProblem,
            r#"UPDATE order_problems
               SET status = $1, resolved_by = $2, admin_response = $3,
                   resolved_at = now(), updated_at = now()
               WHERE id = $4
               RETURNING id, order_id, reporter_id, reporter_role, reason,
                         status as "status: ProblemStatus", admin_response, resolved_by,
                         resolved_at, created_at, updated_at"#,
            status as ProblemStatus,
            admin_id,
            response,
            id,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error resolviendo problema: {e}")))?;
        Ok(row)
    }
}

/* Row extendido que incluye datos de contexto para la lista */
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProblemWithContext {
    /* Campos de order_problems */
    pub id: Uuid,
    pub order_id: Uuid,
    pub reporter_id: Uuid,
    pub reporter_role: String,
    pub reason: String,
    pub status: ProblemStatus,
    pub admin_response: Option<String>,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /* JOINs */
    pub order_number: i32,
    pub reporter_name: String,
}
