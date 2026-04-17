/* [044A-38] Modelos del marketplace: órdenes, fases, servicios, pagos.
 * Mapean directamente a las tablas de la migración 20260404. */

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::UserRole;

/* ============================================================
   ENUMS — mapean a tipos PostgreSQL
   ============================================================ */

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "order_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    /* [104A-29] PendingPayment ya no se usa en órdenes nuevas (DB default = payment_held).
     * Se mantiene en el enum para deserializar registros legacy de PostgreSQL. */
    PendingPayment,
    PaymentHeld,
    AwaitingAssignment,
    InProgress,
    UnderReview,
    Completed,
    Cancelled,
    Disputed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "payment_mode", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PaymentMode {
    Full,
    HalfHalf,
    Phased,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "phase_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PhaseStatus {
    Locked,
    PendingPayment,
    Paid,
    InProgress,
    Delivered,
    RevisionRequested,
    Approved,
    Skipped,
}

/* ============================================================
   MODELOS DE BD
   ============================================================ */

#[derive(Debug, Clone, FromRow)]
pub struct Order {
    pub id: Uuid,
    pub order_number: i32,
    pub client_id: Uuid,
    pub service_id: Uuid,
    pub plan_id: Uuid,
    pub payment_mode: PaymentMode,
    pub base_price_cents: i32,
    pub discount_percent: i32,
    pub final_price_cents: i32,
    pub currency: String,
    pub status: OrderStatus,
    pub assigned_employee_id: Option<Uuid>,
    pub assigned_at: Option<DateTime<Utc>>,
    pub auto_assign_deadline: Option<DateTime<Utc>>,
    pub current_phase: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub project_description: Option<String>,
    pub client_notes: Option<String>,
    pub internal_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /* [P-2] IA intermediaria en chat de pedidos */
    #[sqlx(default)]
    pub ai_intermediary_enabled: Option<bool>,
    #[sqlx(default)]
    pub ai_summary: Option<String>,
    /* [154A-15b] Indica si la orden está abierta para que empleados la tomen */
    #[sqlx(default)]
    pub open_to_employees: Option<bool>,
}

#[derive(Debug, Clone, FromRow)]
pub struct OrderPhase {
    pub id: Uuid,
    pub order_id: Uuid,
    pub phase_number: i32,
    pub title: String,
    pub description: Option<String>,
    pub price_cents: i32,
    pub status: PhaseStatus,
    pub max_revisions: i32,
    pub revisions_used: i32,
    pub estimated_days: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
    pub deadline: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/* ============================================================
   REQUESTS
   ============================================================ */

/// Request para crear una orden (contratar servicio)
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateOrderRequest {
    pub service_slug: String,
    pub plan_slug: String,
    pub payment_mode: PaymentMode,
    #[validate(length(max = 4000))]
    pub project_description: Option<String>,
    #[validate(length(max = 2000))]
    pub client_notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateOrderProjectDescriptionRequest {
    #[validate(length(min = 10, max = 4000))]
    pub project_description: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateOrderPhaseDefinitionRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub price_cents: Option<i32>,
    pub estimated_days: Option<i32>,
    pub max_revisions: Option<i32>,
}

/// Request para cambiar el `active_role` del admin
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SwitchRoleRequest {
    pub role: UserRole,
}

/* [T-10] Request para activar/desactivar IA intermediaria en una orden */
#[derive(Debug, Deserialize, ToSchema)]
pub struct ToggleAiIntermediaryRequest {
    pub enabled: bool,
}

/* ============================================================
   RESPONSES
   ============================================================ */

#[derive(Debug, Serialize, ToSchema)]
pub struct OrderResponse {
    pub id: Uuid,
    pub order_number: i32,
    pub client_id: Uuid,
    pub client_name: Option<String>,
    pub service_title: String,
    pub service_slug: String,
    pub plan_name: String,
    pub payment_mode: PaymentMode,
    pub base_price_cents: i32,
    pub discount_percent: i32,
    pub final_price_cents: i32,
    pub currency: String,
    pub status: OrderStatus,
    pub assigned_employee_id: Option<Uuid>,
    pub assigned_employee_name: Option<String>,
    pub current_phase: i32,
    pub total_phases: i32,
    pub project_description: Option<String>,
    pub client_notes: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    /* [T-10] Campos de IA intermediaria */
    pub ai_intermediary_enabled: bool,
    pub ai_summary: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrderPhaseResponse {
    pub phase_number: i32,
    pub title: String,
    pub description: Option<String>,
    pub price_cents: i32,
    pub status: PhaseStatus,
    pub max_revisions: i32,
    pub revisions_used: i32,
    pub estimated_days: i32,
    pub deadline: Option<DateTime<Utc>>,
}

impl From<OrderPhase> for OrderPhaseResponse {
    fn from(p: OrderPhase) -> Self {
        Self {
            phase_number: p.phase_number,
            title: p.title,
            description: p.description,
            price_cents: p.price_cents,
            status: p.status,
            max_revisions: p.max_revisions,
            revisions_used: p.revisions_used,
            estimated_days: p.estimated_days,
            deadline: p.deadline,
        }
    }
}