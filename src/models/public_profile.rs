/* [084A-7] Modelos para la página pública de perfil de usuario.
 * Incluye perfil público (sin datos sensibles) y reviews enriquecidas
 * con nombre del autor y servicio para mostrarse en la página. */

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// Perfil público de un usuario — datos visibles sin autenticación
#[derive(Debug, Serialize, ToSchema)]
pub struct PublicUserProfile {
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub specialties: Option<Vec<String>>,
    pub average_rating: Option<f64>,
    pub total_completed_orders: Option<i32>,
    pub linkedin: Option<String>,
    pub twitter: Option<String>,
    pub website: Option<String>,
    pub member_since: DateTime<Utc>,
}

/// Row intermedia para construir `PublicUserProfile` desde JOIN
#[derive(Debug, FromRow)]
pub struct PublicProfileRow {
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub specialties: Option<Vec<String>>,
    pub average_rating: Option<f64>,
    pub total_completed_orders: Option<i32>,
    pub linkedin: Option<String>,
    pub twitter: Option<String>,
    pub website: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<PublicProfileRow> for PublicUserProfile {
    fn from(row: PublicProfileRow) -> Self {
        Self {
            username: row.username,
            display_name: row.display_name,
            avatar_url: row.avatar_url,
            bio: row.bio,
            specialties: row.specialties,
            average_rating: row.average_rating,
            total_completed_orders: row.total_completed_orders,
            linkedin: row.linkedin,
            twitter: row.twitter,
            website: row.website,
            member_since: row.created_at,
        }
    }
}

/// Review pública enriquecida con nombre del autor y servicio
#[derive(Debug, Serialize, ToSchema)]
pub struct PublicReviewItem {
    pub id: String,
    pub rating: i32,
    pub comment: Option<String>,
    pub employee_response: Option<String>,
    pub author_name: Option<String>,
    pub author_avatar: Option<String>,
    pub author_username: Option<String>,
    pub service_title: Option<String>,
    pub created_at: String,
}

/// Row intermedia para reviews recibidas (como empleado)
#[derive(Debug, FromRow)]
pub struct ReceivedReviewRow {
    pub id: Uuid,
    pub rating: i32,
    pub comment: Option<String>,
    pub employee_response: Option<String>,
    pub client_name: Option<String>,
    pub client_avatar: Option<String>,
    pub client_username: Option<String>,
    pub service_title: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Row intermedia para reviews dadas (como cliente)
#[derive(Debug, FromRow)]
pub struct GivenReviewRow {
    pub id: Uuid,
    pub rating: i32,
    pub comment: Option<String>,
    pub employee_response: Option<String>,
    pub employee_name: Option<String>,
    pub employee_avatar: Option<String>,
    pub employee_username: Option<String>,
    pub service_title: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Respuesta paginada de reviews públicas
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedPublicReviews {
    pub reviews: Vec<PublicReviewItem>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

/// Distribución de ratings (1-5 estrellas)
#[derive(Debug, Serialize, ToSchema)]
pub struct RatingDistribution {
    pub stars_5: i64,
    pub stars_4: i64,
    pub stars_3: i64,
    pub stars_2: i64,
    pub stars_1: i64,
    pub total: i64,
    pub average: f64,
}
