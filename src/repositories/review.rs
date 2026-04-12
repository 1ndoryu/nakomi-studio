/* [044A-38 Fase 8] Repository de reviews.
 * CRUD: crear, buscar por orden, responder, listar por empleado.
 * UNIQUE en order_id garantiza una review por orden. */

use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::OrderReview;

pub struct ReviewRepository;

impl ReviewRepository {
    pub async fn create(
        pool: &PgPool,
        order_id: Uuid,
        client_id: Uuid,
        employee_id: Uuid,
        rating: i32,
        comment: Option<&str>,
    ) -> Result<OrderReview, AppError> {
        let review = sqlx::query_as!(
            OrderReview,
            r#"INSERT INTO order_reviews (order_id, client_id, employee_id, rating, comment)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING id, order_id, client_id, employee_id, rating, comment,
                         employee_response, employee_responded_at, created_at"#,
            order_id,
            client_id,
            employee_id,
            rating,
            comment,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.constraint() == Some("order_reviews_order_id_key") {
                    return AppError::BadRequest("Ya existe una review para esta orden".into());
                }
            }
            AppError::Internal(format!("Error creando review: {e}"))
        })?;

        Ok(review)
    }

    pub async fn find_by_order(
        pool: &PgPool,
        order_id: Uuid,
    ) -> Result<Option<OrderReview>, AppError> {
        let review = sqlx::query_as!(
            OrderReview,
            r#"SELECT id, order_id, client_id, employee_id, rating, comment,
                      employee_response, employee_responded_at, created_at
               FROM order_reviews WHERE order_id = $1"#,
            order_id,
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error buscando review: {e}")))?;

        Ok(review)
    }

    pub async fn respond(
        pool: &PgPool,
        review_id: Uuid,
        response: &str,
    ) -> Result<OrderReview, AppError> {
        let review = sqlx::query_as!(
            OrderReview,
            r#"UPDATE order_reviews
               SET employee_response = $2, employee_responded_at = NOW()
               WHERE id = $1
               RETURNING id, order_id, client_id, employee_id, rating, comment,
                         employee_response, employee_responded_at, created_at"#,
            review_id,
            response,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error respondiendo review: {e}")))?;

        Ok(review)
    }

    pub async fn list_by_employee(
        pool: &PgPool,
        employee_id: Uuid,
    ) -> Result<Vec<OrderReview>, AppError> {
        let reviews = sqlx::query_as!(
            OrderReview,
            r#"SELECT id, order_id, client_id, employee_id, rating, comment,
                      employee_response, employee_responded_at, created_at
               FROM order_reviews WHERE employee_id = $1
               ORDER BY created_at DESC"#,
            employee_id,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error listando reviews: {e}")))?;

        Ok(reviews)
    }

    /* Actualiza el average_rating en employee_profiles basado en reviews */
    pub async fn update_employee_average(
        pool: &PgPool,
        employee_id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"UPDATE employee_profiles
               SET average_rating = (
                   SELECT CAST(AVG(rating) AS DOUBLE PRECISION)
                   FROM order_reviews WHERE employee_id = $1
               )
               WHERE user_id = $1"#,
            employee_id,
        )
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error actualizando average: {e}")))?;

        Ok(())
    }

    pub async fn list_all(
        pool: &PgPool,
    ) -> Result<Vec<OrderReview>, AppError> {
        let reviews = sqlx::query_as!(
            OrderReview,
            r#"SELECT id, order_id, client_id, employee_id, rating, comment,
                      employee_response, employee_responded_at, created_at
               FROM order_reviews
               ORDER BY created_at DESC"#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error listando todas las reviews: {e}")))?;

        Ok(reviews)
    }

    /* [124A-SENT-R1] Buscar review por ID.
     * runtime query (sin macro) para no requerir sqlx prepare contra BD en vivo. */
    pub async fn find_by_id(
        pool: &PgPool,
        review_id: Uuid,
    ) -> Result<Option<OrderReview>, AppError> {
        sqlx::query_as::<_, OrderReview>(
            r#"SELECT id, order_id, client_id, employee_id, rating, comment,
                      employee_response, employee_responded_at, created_at
               FROM order_reviews WHERE id = $1"#
        )
        .bind(review_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error buscando review: {e}")))
    }
}
