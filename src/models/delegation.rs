/* [044A-38 Fase 4] Modelos para asignación, delegación y perfiles de empleados.
 * Mapean a order_delegations y employee_profiles de la migración marketplace.
 * DelegationStatus es tipo PG enum; EmployeeProfile usa CAST para average_rating. */

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/* ============================================================
ENUMS
============================================================ */

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "delegation_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DelegationStatus {
    Requested,
    Accepted,
    Rejected,
    Completed,
}

/* ============================================================
MODELOS DE BD
============================================================ */

#[derive(Debug, Clone, FromRow)]
pub struct Delegation {
    pub id: Uuid,
    pub order_id: Uuid,
    pub from_employee_id: Uuid,
    pub to_employee_id: Option<Uuid>,
    pub reason: String,
    pub delegation_type: String,
    pub status: DelegationStatus,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Perfil de empleado — `average_rating` se castea desde NUMERIC(3,2) a f64 en las queries
#[derive(Debug, Clone, FromRow)]
pub struct EmployeeProfile {
    pub user_id: Uuid,
    pub specialties: Vec<String>,
    pub availability: String,
    pub max_concurrent_orders: i32,
    pub last_activity_at: DateTime<Utc>,
    pub total_completed_orders: i32,
    pub average_rating: Option<f64>,
}

/* ============================================================
REQUESTS
============================================================ */

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateDelegationRequest {
    #[validate(length(min = 1, max = 1000))]
    pub reason: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RespondDelegationRequest {
    /// true = aceptar, false = rechazar
    pub accept: bool,
}

/* ============================================================
RESPONSES
============================================================ */

#[derive(Debug, Serialize, ToSchema)]
pub struct DelegationResponse {
    pub id: Uuid,
    pub order_id: Uuid,
    pub order_number: i32,
    pub service_title: String,
    pub from_employee_id: Uuid,
    pub to_employee_id: Option<Uuid>,
    pub reason: String,
    pub delegation_type: String,
    pub status: DelegationStatus,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EmployeeListItem {
    pub user_id: Uuid,
    pub email: String,
    pub specialties: Vec<String>,
    pub availability: String,
    pub current_orders: i64,
    pub max_concurrent_orders: i32,
    pub total_completed_orders: i32,
    pub average_rating: Option<f64>,
}
