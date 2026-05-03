/* [174A-2] Modelos de servicios del marketplace. Extraídos de order.rs para SRP.
 * Mapean a las tablas services, service_plans, service_plan_phases. */

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/* ============================================================
MODELOS DE BD — Servicios
============================================================ */

#[derive(Debug, Clone, FromRow)]
pub struct ServiceRecord {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub base_price_cents: i32,
    pub currency: String,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    /* [074A-8] Campos CMS */
    pub image_url: Option<String>,
    pub gallery: serde_json::Value,
    pub skills: serde_json::Value,
    pub content: Option<String>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub status: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ServicePlan {
    pub id: Uuid,
    pub service_id: Uuid,
    pub slug: String,
    pub name: String,
    pub price_cents: i32,
    pub description: Option<String>,
    pub features: serde_json::Value,
    pub is_highlighted: bool,
    pub is_custom: bool,
    pub stripe_price_id: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct ServicePlanPhase {
    pub id: Uuid,
    pub plan_id: Uuid,
    pub phase_number: i32,
    pub title: String,
    pub description: Option<String>,
    pub percentage_of_total: i32,
    pub estimated_days: i32,
    pub max_revisions: i32,
}

/* ============================================================
RESPONSES — Servicios
============================================================ */

/// Detalle completo de un servicio con sus planes
/* [074A-21] Ampliado con image_url y base_price_cents para que el frontend público
 * pueda mostrar la misma info que el CMS.
 * [084A-6] Añadidos content, gallery, meta_title, meta_description para que
 * la API pública devuelva todo lo que el admin edita en el CMS. */
#[derive(Debug, Serialize, ToSchema)]
pub struct ServiceDetailResponse {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub base_price_cents: i32,
    pub skills: serde_json::Value,
    pub content: Option<String>,
    pub gallery: serde_json::Value,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub plans: Vec<ServicePlanResponse>,
}

/* [074A-8] Response admin con todos los campos CMS */
#[derive(Debug, Serialize, ToSchema)]
pub struct AdminServiceResponse {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub base_price_cents: i32,
    pub currency: String,
    pub is_active: bool,
    pub sort_order: i32,
    pub image_url: Option<String>,
    pub gallery: serde_json::Value,
    pub skills: serde_json::Value,
    pub content: Option<String>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub plans: Vec<ServicePlanResponse>,
}

/* [074A-8] Request para crear servicio */
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateServiceRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(length(min = 1, max = 100))]
    pub slug: String,
    pub description: Option<String>,
    pub base_price_cents: Option<i32>,
    pub currency: Option<String>,
    pub image_url: Option<String>,
    pub gallery: Option<serde_json::Value>,
    pub skills: Option<serde_json::Value>,
    pub content: Option<String>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub status: Option<String>,
    pub sort_order: Option<i32>,
}

/* [074A-8] Request para actualizar servicio (todos opcionales) */
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateServiceRequest {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub base_price_cents: Option<i32>,
    pub currency: Option<String>,
    pub is_active: Option<bool>,
    pub image_url: Option<String>,
    pub gallery: Option<serde_json::Value>,
    pub skills: Option<serde_json::Value>,
    pub content: Option<String>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub status: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ServicePlanResponse {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub price_cents: i32,
    pub description: Option<String>,
    pub features: serde_json::Value,
    pub is_highlighted: bool,
    pub is_custom: bool,
    pub phases: Vec<ServicePlanPhaseResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ServicePlanPhaseResponse {
    pub phase_number: i32,
    pub title: String,
    pub description: Option<String>,
    pub percentage_of_total: i32,
    pub estimated_days: i32,
    pub max_revisions: i32,
}

impl From<ServicePlanPhase> for ServicePlanPhaseResponse {
    fn from(p: ServicePlanPhase) -> Self {
        Self {
            phase_number: p.phase_number,
            title: p.title,
            description: p.description,
            percentage_of_total: p.percentage_of_total,
            estimated_days: p.estimated_days,
            max_revisions: p.max_revisions,
        }
    }
}

/* [074A-66] Request para guardar planes de un servicio (batch replace).
 * Se eliminan todos los planes existentes y se insertan los nuevos en una transacción. */
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SaveServicePlansRequest {
    #[validate(nested)]
    pub plans: Vec<SavePlanItem>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SavePlanItem {
    #[validate(length(min = 1, max = 50))]
    pub slug: String,
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub price_cents: i32,
    pub description: Option<String>,
    pub features: serde_json::Value,
    pub is_highlighted: bool,
    pub is_custom: bool,
    pub sort_order: i32,
    #[validate(nested)]
    pub phases: Vec<SavePhaseItem>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SavePhaseItem {
    pub phase_number: i32,
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    pub description: Option<String>,
    pub percentage_of_total: i32,
    pub estimated_days: i32,
    pub max_revisions: i32,
}
