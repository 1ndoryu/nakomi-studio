/* [044A-38 Fase 6] Repositorio de entregables (phase_deliverables).
 * [044A-44] Migrado a query_as!/query_scalar! con verificación en compilación.
 * CRUD sobre archivos asociados a fases de órdenes. */

use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::PhaseDeliverable;

/// Parámetros para crear un entregable
pub struct CreateDeliverableParams<'a> {
    pub phase_id: Uuid,
    pub uploaded_by: Uuid,
    pub file_name: &'a str,
    pub file_url: &'a str,
    pub file_size_bytes: Option<i64>,
    pub mime_type: Option<&'a str>,
    pub revision_number: i32,
    pub notes: Option<&'a str>,
}

pub struct DeliverableRepository;

impl DeliverableRepository {
    /// Guarda un registro de entregable en BD
    pub async fn create(
        pool: &PgPool,
        params: CreateDeliverableParams<'_>,
    ) -> Result<PhaseDeliverable, AppError> {
        let row = sqlx::query_as!(
            PhaseDeliverable,
            r#"INSERT INTO phase_deliverables
              (phase_id, uploaded_by, file_name, file_url, file_size_bytes, mime_type, revision_number, notes)
              VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
              RETURNING id, phase_id, uploaded_by, file_name, file_url,
                file_size_bytes, mime_type, revision_number, notes, created_at"#,
            params.phase_id,
            params.uploaded_by,
            params.file_name,
            params.file_url,
            params.file_size_bytes,
            params.mime_type,
            params.revision_number,
            params.notes,
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(row)
    }

    /// Lista entregables de una fase, ordenados por revision + fecha
    pub async fn list_by_phase(
        pool: &PgPool,
        phase_id: Uuid,
    ) -> Result<Vec<PhaseDeliverable>, AppError> {
        let rows = sqlx::query_as!(
            PhaseDeliverable,
            r#"SELECT id, phase_id, uploaded_by, file_name, file_url,
              file_size_bytes, mime_type, revision_number, notes, created_at
              FROM phase_deliverables
              WHERE phase_id = $1
              ORDER BY revision_number DESC, created_at DESC"#,
            phase_id,
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(rows)
    }

    /// Lista entregables de una revisión específica
    pub async fn list_by_revision(
        pool: &PgPool,
        phase_id: Uuid,
        revision_number: i32,
    ) -> Result<Vec<PhaseDeliverable>, AppError> {
        let rows = sqlx::query_as!(
            PhaseDeliverable,
            r#"SELECT id, phase_id, uploaded_by, file_name, file_url,
              file_size_bytes, mime_type, revision_number, notes, created_at
              FROM phase_deliverables
              WHERE phase_id = $1 AND revision_number = $2
              ORDER BY created_at ASC"#,
            phase_id,
            revision_number,
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(rows)
    }

    /// Busca un entregable por ID
    pub async fn find_by_id(
        pool: &PgPool,
        deliverable_id: Uuid,
    ) -> Result<Option<PhaseDeliverable>, AppError> {
        let row = sqlx::query_as!(
            PhaseDeliverable,
            r#"SELECT id, phase_id, uploaded_by, file_name, file_url,
              file_size_bytes, mime_type, revision_number, notes, created_at
              FROM phase_deliverables WHERE id = $1"#,
            deliverable_id,
        )
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(row)
    }

    /// Obtiene el número de revisión actual (max `revision_number`) de una fase
    pub async fn current_revision_number(pool: &PgPool, phase_id: Uuid) -> Result<i32, AppError> {
        let val: Option<i32> = sqlx::query_scalar!(
            r#"SELECT MAX(revision_number) FROM phase_deliverables WHERE phase_id = $1"#,
            phase_id,
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(val.unwrap_or(0))
    }
}
