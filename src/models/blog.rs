/* [074A-10] Modelos para el CMS de blog.
 * BlogPost: registro BD completo. BlogPostResponse: respuesta API pública.
 * CreateBlogPostRequest/UpdateBlogPostRequest: payloads admin. */

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, FromRow)]
pub struct BlogPost {
    pub id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub slug: String,
    pub excerpt: Option<String>,
    pub content: String,
    pub featured_image: Option<String>,
    pub status: String,
    pub tags: serde_json::Value,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub sort_order: i32,
    pub is_featured: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BlogPostResponse {
    pub id: Uuid,
    pub author_id: Uuid,
    pub author_name: Option<String>,
    pub title: String,
    pub slug: String,
    pub excerpt: Option<String>,
    pub content: String,
    pub featured_image: Option<String>,
    pub status: String,
    pub tags: Vec<String>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub sort_order: i32,
    pub is_featured: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedBlogPosts {
    pub posts: Vec<BlogPostResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateBlogPostRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(min = 1, max = 255))]
    pub slug: String,
    pub excerpt: Option<String>,
    #[validate(length(min = 1))]
    pub content: String,
    pub featured_image: Option<String>,
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateBlogPostRequest {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub excerpt: Option<String>,
    pub content: Option<String>,
    pub featured_image: Option<String>,
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_featured: Option<bool>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
}

impl BlogPost {
    /// Convierte a respuesta API, requiere el nombre del autor por separado
    #[must_use]
    pub fn into_response(self, author_name: Option<String>) -> BlogPostResponse {
        let tags: Vec<String> = serde_json::from_value(self.tags.clone()).unwrap_or_default();
        BlogPostResponse {
            id: self.id,
            author_id: self.author_id,
            author_name,
            title: self.title,
            slug: self.slug,
            excerpt: self.excerpt,
            content: self.content,
            featured_image: self.featured_image,
            status: self.status,
            tags,
            meta_title: self.meta_title,
            meta_description: self.meta_description,
            sort_order: self.sort_order,
            is_featured: self.is_featured,
            published_at: self.published_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
