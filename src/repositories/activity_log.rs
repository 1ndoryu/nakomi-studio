/* [124A-R1] Repositorio de activity_log — centraliza inserts de auditoría.
 * Antes cada handler hacía sqlx::query directamente; ahora delega aquí. */

use sqlx::PgPool;
use uuid::Uuid;

pub struct ActivityLogRepository;

/// Fila cruda de `activity_log` para queries de lectura.
#[derive(sqlx::FromRow)]
pub struct ActivityRow {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub details: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ActivityLogRepository {
    /// Registra una acción en `activity_log`.
    /// `details` es un JSON opcional con contexto adicional.
    pub async fn log(
        pool: &PgPool,
        user_id: Uuid,
        action: &str,
        entity_type: &str,
        entity_id: Uuid,
        details: Option<serde_json::Value>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO activity_log (user_id, action, entity_type, entity_id, details)
               VALUES ($1, $2, $3, $4, $5)"#,
            user_id,
            action,
            entity_type,
            entity_id,
            details,
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Lista entradas de `activity_log` para una entidad (e.g. order).
    pub async fn list_by_entity(
        pool: &PgPool,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Vec<ActivityRow>, sqlx::Error> {
        sqlx::query_as!(
            ActivityRow,
            r#"SELECT id, user_id, action, details, created_at
               FROM activity_log
               WHERE entity_type = $1 AND entity_id = $2
               ORDER BY created_at ASC"#,
            entity_type,
            entity_id,
        )
        .fetch_all(pool)
        .await
    }
}
