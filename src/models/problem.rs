/* [104A-28] Modelo de problemas reportados en órdenes.
 * Empleados y clientes pueden reportar con razón escrita.
 * Admin gestiona desde su panel (resolver / descartar). */

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "problem_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ProblemStatus {
    Open,
    InReview,
    Resolved,
    Dismissed,
}

#[derive(Debug, Clone, FromRow)]
pub struct OrderProblem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub reporter_id: Uuid,
    pub reporter_role: String,
    pub reason: String,
    pub status: ProblemStatus,
    pub admin_response: Option<String>,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/* ============================================================
   REQUESTS
   ============================================================ */

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ReportProblemRequest {
    #[validate(length(min = 10, max = 2000, message = "La razón debe tener entre 10 y 2000 caracteres"))]
    pub reason: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ResolveProblemRequest {
    pub action: ProblemAction,
    #[validate(length(max = 2000))]
    pub response: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProblemAction {
    Resolve,
    Dismiss,
}

/* ============================================================
   REQUESTS — cancel con razón
   ============================================================ */

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CancelOrderRequest {
    #[validate(length(max = 2000))]
    pub reason: Option<String>,
}

/* ============================================================
   RESPONSES
   ============================================================ */

#[derive(Debug, Serialize, ToSchema)]
pub struct ProblemResponse {
    pub id: Uuid,
    pub order_id: Uuid,
    pub order_number: i32,
    pub reporter_id: Uuid,
    pub reporter_name: String,
    pub reporter_role: String,
    pub reason: String,
    pub status: ProblemStatus,
    pub admin_response: Option<String>,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<String>,
    pub created_at: String,
}
