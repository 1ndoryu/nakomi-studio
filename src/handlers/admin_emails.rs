/* [311A-1] Handler admin para consultar trazabilidad de correos enviados (email_logs).
 * Endpoint solo accesible por admins. Lista paginada con filtro opcional por tipo de plantilla.
 * Non-fatal: la tabla email_logs se escribe desde EmailService::send(), no desde aquí. */

use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::UserRole;
use crate::repositories::{EmailLogRepository, EmailLogRow};
use crate::AppState;

#[derive(Deserialize)]
pub struct EmailLogsQuery {
    pub template: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Serialize)]
pub struct EmailLogsResponse {
    pub logs: Vec<EmailLogWithLabel>,
    pub total: i64,
}

#[derive(Serialize)]
pub struct EmailLogWithLabel {
    pub id: Uuid,
    pub to_email: String,
    pub subject: String,
    pub template: String,
    pub template_label: String,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub status: String,
    pub status_color: String,
    pub sent_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<EmailLogRow> for EmailLogWithLabel {
    fn from(row: EmailLogRow) -> Self {
        let template_label = row.template_label().to_string();
        let status_color = row.status_color().to_string();
        Self {
            id: row.id,
            to_email: row.to_email,
            subject: row.subject,
            template: row.template,
            template_label,
            reference_type: row.reference_type,
            reference_id: row.reference_id,
            status: row.status,
            status_color,
            sent_at: row.sent_at,
            created_at: row.created_at,
        }
    }
}

/// Lista paginada de correos enviados, con filtro opcional por plantilla
#[utoipa::path(
    get,
    path = "/api/admin/email-logs",
    params(
        ("template" = Option<String>, Query, description = "Filtrar por tipo de plantilla"),
        ("limit" = Option<i32>, Query, description = "Máximo 100, default 50"),
        ("offset" = Option<i32>, Query, description = "Paginación"),
    ),
    responses(
        (status = 200, description = "Lista de correos enviados", body = EmailLogsResponse),
        (status = 403, description = "Sin permisos"),
        (status = 500, description = "Error de BD"),
    ),
    security(("bearer_auth" = [])),
    tag = "admin"
)]
pub async fn list_email_logs(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<EmailLogsQuery>,
) -> Result<Json<EmailLogsResponse>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let limit = params.limit.map(|v| v.max(1).min(100)).unwrap_or(50);
    let offset = params.offset.unwrap_or(0).max(0);
    let template = params.template.as_deref();

    let logs = EmailLogRepository::list(&state.pool, template, limit, offset)
        .await
        .map_err(|e| AppError::Internal(format!("Error consultando email_logs: {e}")))?;

    let total = EmailLogRepository::count(&state.pool, template)
        .await
        .map_err(|e| AppError::Internal(format!("Error contando email_logs: {e}")))?;

    let logs_with_labels: Vec<EmailLogWithLabel> = logs.into_iter().map(Into::into).collect();

    Ok(Json(EmailLogsResponse {
        logs: logs_with_labels,
        total,
    }))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/admin/email-logs", get(list_email_logs))
}
