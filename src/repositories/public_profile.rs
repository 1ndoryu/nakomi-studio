/* [084A-7] Repositorio para perfiles públicos de usuario.
 * Queries optimizadas con JOINs para obtener perfil completo y reviews
 * enriquecidas en una sola consulta. */

use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{
    GivenReviewRow, PublicProfileRow, RatingDistribution, ReceivedReviewRow,
};

pub struct PublicProfileRepository;

impl PublicProfileRepository {
    /// Obtiene perfil público por username (JOIN `users` + `user_profiles` + `employee_profiles`)
    pub async fn find_by_username(
        pool: &PgPool,
        username: &str,
    ) -> Result<Option<PublicProfileRow>, AppError> {
        let row = sqlx::query_as!(
            PublicProfileRow,
            r#"SELECT
                u.username,
                u.display_name,
                u.avatar_url,
                up.bio as "bio?",
                ep.specialties as "specialties?: Vec<String>",
                ep.average_rating::float8 as "average_rating?: f64",
                ep.total_completed_orders as "total_completed_orders?",
                up.linkedin as "linkedin?",
                up.twitter as "twitter?",
                up.website as "website?",
                u.created_at
            FROM users u
            LEFT JOIN user_profiles up ON up.user_id = u.id
            LEFT JOIN employee_profiles ep ON ep.user_id = u.id
            WHERE u.username = $1 AND u.status = 'active'"#,
            username,
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error buscando perfil público: {e}")))?;

        Ok(row)
    }

    /// Reviews recibidas como empleado (paginado)
    pub async fn list_reviews_received(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ReceivedReviewRow>, i64), AppError> {
        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM order_reviews WHERE employee_id = $1",
            user_id,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error contando reviews: {e}")))?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            ReceivedReviewRow,
            r#"SELECT
                r.id, r.rating, r.comment, r.employee_response,
                c.display_name as client_name,
                c.avatar_url as client_avatar,
                c.username as client_username,
                s.title as service_title,
                r.created_at
            FROM order_reviews r
            JOIN users c ON c.id = r.client_id
            JOIN orders o ON o.id = r.order_id
            JOIN services s ON s.id = o.service_id
            WHERE r.employee_id = $1
            ORDER BY r.created_at DESC
            LIMIT $2 OFFSET $3"#,
            user_id,
            limit,
            offset,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error listando reviews recibidas: {e}")))?;

        Ok((rows, total))
    }

    /// Reviews dadas como cliente (paginado)
    pub async fn list_reviews_given(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<GivenReviewRow>, i64), AppError> {
        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM order_reviews WHERE client_id = $1",
            user_id,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error contando reviews dadas: {e}")))?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            GivenReviewRow,
            r#"SELECT
                r.id, r.rating, r.comment, r.employee_response,
                e.display_name as employee_name,
                e.avatar_url as employee_avatar,
                e.username as employee_username,
                s.title as service_title,
                r.created_at
            FROM order_reviews r
            JOIN users e ON e.id = r.employee_id
            JOIN orders o ON o.id = r.order_id
            JOIN services s ON s.id = o.service_id
            WHERE r.client_id = $1
            ORDER BY r.created_at DESC
            LIMIT $2 OFFSET $3"#,
            user_id,
            limit,
            offset,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error listando reviews dadas: {e}")))?;

        Ok((rows, total))
    }

    /// Distribución de ratings para un usuario (como empleado)
    pub async fn rating_distribution(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<RatingDistribution, AppError> {
        struct CountRow {
            stars_5: Option<i64>,
            stars_4: Option<i64>,
            stars_3: Option<i64>,
            stars_2: Option<i64>,
            stars_1: Option<i64>,
            total: Option<i64>,
            average: Option<f64>,
        }

        let row = sqlx::query_as!(
            CountRow,
            r#"SELECT
                COUNT(*) FILTER (WHERE rating = 5) as stars_5,
                COUNT(*) FILTER (WHERE rating = 4) as stars_4,
                COUNT(*) FILTER (WHERE rating = 3) as stars_3,
                COUNT(*) FILTER (WHERE rating = 2) as stars_2,
                COUNT(*) FILTER (WHERE rating = 1) as stars_1,
                COUNT(*) as total,
                AVG(rating)::float8 as average
            FROM order_reviews
            WHERE employee_id = $1"#,
            user_id,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error obteniendo distribución de ratings: {e}")))?;

        Ok(RatingDistribution {
            stars_5: row.stars_5.unwrap_or(0),
            stars_4: row.stars_4.unwrap_or(0),
            stars_3: row.stars_3.unwrap_or(0),
            stars_2: row.stars_2.unwrap_or(0),
            stars_1: row.stars_1.unwrap_or(0),
            total: row.total.unwrap_or(0),
            average: row.average.unwrap_or(0.0),
        })
    }

    /// Obtiene el ID de un usuario por username (para queries posteriores)
    pub async fn get_user_id_by_username(
        pool: &PgPool,
        username: &str,
    ) -> Result<Option<Uuid>, AppError> {
        let id = sqlx::query_scalar!(
            "SELECT id FROM users WHERE username = $1 AND status = 'active'",
            username,
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error buscando usuario: {e}")))?;

        Ok(id)
    }
}
