/* [154A-2] Handler admin para gestión de fixtures de contenido.
 * Expone endpoints para ver el estado de los registros rastreados por el fixture system
 * y para disparar una sincronización de los archivos TOML de content/ desde el panel.
 * Solo accesible por admins. El fixture_manager vive en AppState (Arc).
 * Gotcha: ContentManager sin password_hasher — si users.toml tiene plain:xxx y el admin
 * dispara sync, el hash falla silenciosamente (se registra en errors del report).
 * Para re-seedear usuarios con passwords, usar FIXTURES_SYNC=true + reinicio. */

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::UserRole;
use crate::repositories::{FixtureRepository, FixtureTableStat};
use crate::AppState;

#[derive(Serialize, utoipa::ToSchema)]
pub struct FixtureTableSummary {
    pub table_name: String,
    pub record_count: i64,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct FixtureStatusResponse {
    pub tracked_records: i64,
    pub tables: Vec<FixtureTableSummary>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct FixtureSyncResult {
    pub inserted: u64,
    pub updated: u64,
    pub deleted: u64,
    pub skipped: u64,
    pub errors: Vec<String>,
    pub summary: String,
}

/// Estado actual de los fixtures rastreados en `_glory_fixtures`
#[utoipa::path(
    get,
    path = "/api/admin/fixtures",
    responses(
        (status = 200, description = "Estado de fixtures por tabla", body = FixtureStatusResponse),
        (status = 403, description = "Sin permisos"),
        (status = 500, description = "Error de BD"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_fixture_status(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<FixtureStatusResponse>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    if !FixtureRepository::table_exists(&state.pool).await? {
        return Ok(Json(FixtureStatusResponse {
            tracked_records: 0,
            tables: vec![],
        }));
    }

    let table_stats: Vec<FixtureTableStat> = FixtureRepository::list_table_stats(&state.pool).await?;
    let tracked_records: i64 = table_stats.iter().map(|s| s.record_count).sum();
    let tables = table_stats
        .into_iter()
        .map(|s| FixtureTableSummary {
            table_name: s.table_name,
            record_count: s.record_count,
        })
        .collect();

    Ok(Json(FixtureStatusResponse {
        tracked_records,
        tables,
    }))
}

/// Dispara una sincronización completa de los archivos content/*.toml con la BD
#[utoipa::path(
    post,
    path = "/api/admin/fixtures/sync",
    responses(
        (status = 200, description = "Resultado de la sincronización", body = FixtureSyncResult),
        (status = 403, description = "Sin permisos"),
        (status = 503, description = "Fixture manager no disponible"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn trigger_sync(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<FixtureSyncResult>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let manager = state.fixture_manager.as_ref().ok_or_else(|| {
        AppError::Internal("Fixture manager no disponible — directorio content/ no encontrado".into())
    })?;

    let report = manager
        .sync_all()
        .await
        .map_err(|e| AppError::Internal(format!("Error en fixture sync: {e}")))?;

    Ok(Json(FixtureSyncResult {
        inserted: report.inserted,
        updated: report.updated,
        deleted: report.deleted,
        skipped: report.skipped,
        summary: report.summary(),
        errors: report.errors,
    }))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/admin/fixtures", get(get_fixture_status))
        .route("/admin/fixtures/sync", post(trigger_sync))
}
