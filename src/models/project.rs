/* [074A-12] Modelos para CMS de proyectos/portfolio.
 * Project: registro BD completo. ProjectResponse: respuesta API.
 * Sin author_id porque los proyectos no tienen autor individual. */

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, FromRow)]
pub struct Project {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub client: Option<String>,
    pub description: String,
    pub featured_image: Option<String>,
    pub gallery: serde_json::Value,
    pub categories: serde_json::Value,
    pub technologies: serde_json::Value,
    pub links: serde_json::Value,
    pub skills: serde_json::Value,
    pub status: String,
    pub sort_order: i32,
    pub is_featured: bool,
    pub in_carousel: bool,
    pub showcase_category: Option<String>,
    pub detail_title: Option<String>,
    pub use_first_gallery_image: bool,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub client: Option<String>,
    pub description: String,
    pub featured_image: Option<String>,
    pub gallery: Vec<GalleryImage>,
    pub categories: Vec<String>,
    pub technologies: Vec<String>,
    pub links: Vec<ProjectLink>,
    pub skills: Vec<ProjectSkill>,
    pub status: String,
    pub sort_order: i32,
    pub is_featured: bool,
    pub in_carousel: bool,
    pub showcase_category: Option<String>,
    pub detail_title: Option<String>,
    pub use_first_gallery_image: bool,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ProjectLink {
    pub tipo: String,
    pub url: String,
    pub etiqueta: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ProjectSkill {
    pub titulo: String,
    pub descripcion: Option<String>,
}

/* [124A-PROJ1] Imagen de galería con layout (full=100% ancho, half=50% ancho) */
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct GalleryImage {
    pub url: String,
    pub layout: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateProjectRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(min = 1, max = 255))]
    pub slug: String,
    pub client: Option<String>,
    pub description: Option<String>,
    pub featured_image: Option<String>,
    pub gallery: Option<Vec<GalleryImage>>,
    pub categories: Option<Vec<String>>,
    pub technologies: Option<Vec<String>>,
    pub links: Option<Vec<ProjectLink>>,
    pub skills: Option<Vec<ProjectSkill>>,
    pub status: Option<String>,
    pub sort_order: Option<i32>,
    pub is_featured: Option<bool>,
    pub in_carousel: Option<bool>,
    pub showcase_category: Option<String>,
    pub detail_title: Option<String>,
    pub use_first_gallery_image: Option<bool>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProjectRequest {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub client: Option<String>,
    pub description: Option<String>,
    pub featured_image: Option<String>,
    pub gallery: Option<Vec<GalleryImage>>,
    pub categories: Option<Vec<String>>,
    pub technologies: Option<Vec<String>>,
    pub links: Option<Vec<ProjectLink>>,
    pub skills: Option<Vec<ProjectSkill>>,
    pub status: Option<String>,
    pub sort_order: Option<i32>,
    pub is_featured: Option<bool>,
    pub in_carousel: Option<bool>,
    pub showcase_category: Option<String>,
    pub detail_title: Option<String>,
    pub use_first_gallery_image: Option<bool>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
}

/* [124A-CMS3] Request para reordenar proyectos en batch */
#[derive(Debug, Deserialize, ToSchema)]
pub struct ReorderItem {
    pub id: uuid::Uuid,
    pub sort_order: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReorderProjectsRequest {
    pub items: Vec<ReorderItem>,
}

/* [124A-CMS10] Request genérico de reorder, reutilizable por servicios/blog/cualquier entidad */
#[derive(Debug, Deserialize, ToSchema)]
pub struct ReorderRequest {
    pub items: Vec<ReorderItem>,
}

impl Project {
    /// Convierte a respuesta API deserializando JSONB a tipos tipados
    #[must_use]
    pub fn into_response(self) -> ProjectResponse {
        /* [124A-PROJ1] Gallery: intenta formato nuevo {url, layout} primero,
         * si falla intenta legacy ["url"] y convierte a GalleryImage con layout "full" */
        let gallery: Vec<GalleryImage> = serde_json::from_value::<Vec<GalleryImage>>(self.gallery.clone())
            .unwrap_or_else(|_| {
                serde_json::from_value::<Vec<String>>(self.gallery)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|url| GalleryImage { url, layout: "full".to_string() })
                    .collect()
            });
        let categories: Vec<String> = serde_json::from_value(self.categories).unwrap_or_default();
        let technologies: Vec<String> =
            serde_json::from_value(self.technologies).unwrap_or_default();
        let links: Vec<ProjectLink> = serde_json::from_value(self.links).unwrap_or_default();
        let skills: Vec<ProjectSkill> = serde_json::from_value(self.skills).unwrap_or_default();

        ProjectResponse {
            id: self.id,
            title: self.title,
            slug: self.slug,
            client: self.client,
            description: self.description,
            featured_image: self.featured_image,
            gallery,
            categories,
            technologies,
            links,
            skills,
            status: self.status,
            sort_order: self.sort_order,
            is_featured: self.is_featured,
            in_carousel: self.in_carousel,
            showcase_category: self.showcase_category,
            detail_title: self.detail_title,
            use_first_gallery_image: self.use_first_gallery_image,
            meta_title: self.meta_title,
            meta_description: self.meta_description,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
