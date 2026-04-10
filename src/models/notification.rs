/* [044A-38 Fase 9] Modelos de notificaciones.
 * Tabla notifications ya existe en migración marketplace.
 * notification_type identifica la clase de evento (new_order, payment, assignment, etc.).
 * reference_type + reference_id permiten vincular a la entidad origen. */

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/* Tipos de notificación soportados.
 * [104A-38] Todos integrados en sus respectivos handlers.
 * NOTIF_PAYMENT_RELEASED y NOTIF_PHASE_APPROVED pendientes de integrar. */
pub const NOTIF_NEW_ORDER: &str = "new_order";
pub const NOTIF_ORDER_ASSIGNED: &str = "order_assigned";
pub const NOTIF_ORDER_COMPLETED: &str = "order_completed";
pub const NOTIF_ORDER_CANCELLED: &str = "order_cancelled";
pub const NOTIF_PAYMENT_RECEIVED: &str = "payment_received";
#[allow(dead_code)]
pub const NOTIF_PAYMENT_RELEASED: &str = "payment_released";
pub const NOTIF_PHASE_DELIVERED: &str = "phase_delivered";
#[allow(dead_code)]
pub const NOTIF_PHASE_APPROVED: &str = "phase_approved";
pub const NOTIF_REVISION_REQUESTED: &str = "revision_requested";
pub const NOTIF_REFUND_REQUESTED: &str = "refund_requested";
pub const NOTIF_REFUND_RESOLVED: &str = "refund_resolved";
pub const NOTIF_NEW_REVIEW: &str = "new_review";
pub const NOTIF_REVIEW_RESPONSE: &str = "review_response";
pub const NOTIF_DELEGATION_RECEIVED: &str = "delegation_received";
pub const NOTIF_DELEGATION_RESOLVED: &str = "delegation_resolved";
pub const NOTIF_NEW_MESSAGE: &str = "new_message";
/* [T-6] Notificación de escalación: la IA detectó que se necesita intervención humana */
pub const NOTIF_ESCALATION_NEEDED: &str = "escalation_needed";

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: String,
    pub title: String,
    pub body: Option<String>,
    pub link: Option<String>,
    pub read: bool,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/* Respuesta para el frontend con campos como string */
#[derive(Debug, Serialize, ToSchema)]
pub struct NotificationResponse {
    pub id: String,
    pub user_id: String,
    pub notification_type: String,
    pub title: String,
    pub body: Option<String>,
    pub link: Option<String>,
    pub read: bool,
    pub reference_type: Option<String>,
    pub reference_id: Option<String>,
    pub created_at: String,
}

impl From<Notification> for NotificationResponse {
    fn from(n: Notification) -> Self {
        Self {
            id: n.id.to_string(),
            user_id: n.user_id.to_string(),
            notification_type: n.notification_type,
            title: n.title,
            body: n.body,
            link: n.link,
            read: n.read,
            reference_type: n.reference_type,
            reference_id: n.reference_id.map(|id| id.to_string()),
            created_at: n.created_at.to_rfc3339(),
        }
    }
}

/* Mensaje WebSocket que se envía en tiempo real al usuario conectado */
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsNotification {
    /* Nueva notificación push */
    Notification {
        id: String,
        notification_type: String,
        title: String,
        body: Option<String>,
        link: Option<String>,
        reference_type: Option<String>,
        reference_id: Option<String>,
        created_at: String,
    },
    /* Contador actualizado de no leídas */
    UnreadCount {
        count: i64,
    },
}

impl From<Notification> for WsNotification {
    fn from(n: Notification) -> Self {
        Self::Notification {
            id: n.id.to_string(),
            notification_type: n.notification_type,
            title: n.title,
            body: n.body,
            link: n.link,
            reference_type: n.reference_type,
            reference_id: n.reference_id.map(|id| id.to_string()),
            created_at: n.created_at.to_rfc3339(),
        }
    }
}

/* Body para marcar notificaciones como leídas */
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct MarkReadBody {
    #[validate(length(min = 1, max = 50))]
    pub ids: Vec<String>,
}

/* Body para crear notificación (uso interno, no expuesto en API pública) */
#[derive(Debug)]
pub struct CreateNotification {
    pub user_id: Uuid,
    pub notification_type: String,
    pub title: String,
    pub body: Option<String>,
    pub link: Option<String>,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
}

/* Respuesta con solo el conteo de no leídas */
#[derive(Debug, Serialize, ToSchema)]
pub struct UnreadCountResponse {
    pub count: i64,
}
