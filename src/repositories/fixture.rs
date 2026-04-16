/* sentinel-disable-file sqlx-query-sin-macro sqlx-query-as-sin-macro: las queries de
 * _glory_fixtures usan esquema dinámico sin verificación en compilación, ya que la tabla
 * es genérica y su contenido depende de los fixtures cargados en runtime. */
/* [154A-2] Repositorio para consultas sobre la tabla de tracking de fixtures (_glory_fixtures).
 * Centraliza el acceso a datos del sistema de fixtures desde el handler admin. */

use sqlx::PgPool;

use crate::errors::AppError;

#[derive(Debug)]
pub struct FixtureTableStat {
    pub table_name: String,
    pub record_count: i64,
}

pub struct FixtureRepository;

impl FixtureRepository {
    /// Verifica si la tabla _glory_fixtures existe en la BD
    pub async fn table_exists(pool: &PgPool) -> Result<bool, AppError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = '_glory_fixtures')",
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error verificando tabla fixtures: {e}")))?;
        Ok(exists)
    }

    /// Lista el conteo de registros rastreados por tabla
    pub async fn list_table_stats(pool: &PgPool) -> Result<Vec<FixtureTableStat>, AppError> {
        let rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT table_name, COUNT(*)::bigint FROM _glory_fixtures GROUP BY table_name ORDER BY table_name",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Error consultando fixtures: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|(table_name, record_count)| FixtureTableStat {
                table_name,
                record_count,
            })
            .collect())
    }
}
