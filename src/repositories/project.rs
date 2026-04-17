/* sentinel-disable-file sqlx-query-sin-macro sqlx-query-as-sin-macro: projects usa runtime queries
 * con COALESCE para partial updates y tipos FromRow genéricos. */
/* [074A-12] Repositorio para proyectos/portfolio.
 * Mismo patrón que BlogRepository: CRUD completo, partial update con COALESCE. */

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::Project;

/// Parámetros para crear un proyecto
pub struct CreateProjectParams<'a> {
    pub title: &'a str,
    pub slug: &'a str,
    pub client: Option<&'a str>,
    pub description: &'a str,
    pub featured_image: Option<&'a str>,
    pub gallery_image: Option<&'a str>,
    pub gallery: &'a serde_json::Value,
    pub categories: &'a serde_json::Value,
    pub technologies: &'a serde_json::Value,
    pub links: &'a serde_json::Value,
    pub skills: &'a serde_json::Value,
    pub status: &'a str,
    pub sort_order: i32,
    pub is_featured: bool,
    pub in_carousel: bool,
    pub showcase_category: Option<&'a str>,
    pub detail_title: Option<&'a str>,
    pub use_first_gallery_image: bool,
    pub meta_title: Option<&'a str>,
    pub meta_description: Option<&'a str>,
}

/// Parámetros para actualizar un proyecto (parcial)
pub struct UpdateProjectParams<'a> {
    pub title: Option<&'a str>,
    pub slug: Option<&'a str>,
    pub client: Option<&'a str>,
    pub description: Option<&'a str>,
    pub featured_image: Option<&'a str>,
    pub gallery_image: Option<&'a str>,
    pub gallery: Option<&'a serde_json::Value>,
    pub categories: Option<&'a serde_json::Value>,
    pub technologies: Option<&'a serde_json::Value>,
    pub links: Option<&'a serde_json::Value>,
    pub skills: Option<&'a serde_json::Value>,
    pub status: Option<&'a str>,
    pub sort_order: Option<i32>,
    pub is_featured: Option<bool>,
    pub in_carousel: Option<bool>,
    pub showcase_category: Option<&'a str>,
    pub detail_title: Option<&'a str>,
    pub use_first_gallery_image: Option<bool>,
    pub meta_title: Option<&'a str>,
    pub meta_description: Option<&'a str>,
}

pub struct ProjectRepository;

impl ProjectRepository {
    /// Lista proyectos publicados (público), ordenados por `sort_order` ASC, `created_at` DESC
    pub async fn list_published(pool: &PgPool) -> Result<Vec<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, title, slug, client, description, featured_image, gallery_image,
                    gallery, categories, technologies, links, skills,
                    status, sort_order, is_featured, in_carousel, showcase_category, detail_title, use_first_gallery_image, meta_title, meta_description,
                    created_at, updated_at
             FROM projects
             WHERE status = 'published'
             ORDER BY sort_order ASC, created_at DESC"
        )
        .fetch_all(pool)
        .await
    }

    /// Buscar proyecto publicado por slug (público)
    pub async fn find_published_by_slug(
        pool: &PgPool,
        slug: &str,
    ) -> Result<Option<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, title, slug, client, description, featured_image, gallery_image,
                    gallery, categories, technologies, links, skills,
                    status, sort_order, is_featured, in_carousel, showcase_category, detail_title, use_first_gallery_image, meta_title, meta_description,
                    created_at, updated_at
             FROM projects
             WHERE slug = $1 AND status = 'published'"
        )
        .bind(slug)
        .fetch_optional(pool)
        .await
    }

    /// Lista todos los proyectos (admin), sin filtro de status
    pub async fn list_all(pool: &PgPool) -> Result<Vec<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, title, slug, client, description, featured_image, gallery_image,
                    gallery, categories, technologies, links, skills,
                    status, sort_order, is_featured, in_carousel, showcase_category, detail_title, use_first_gallery_image, meta_title, meta_description,
                    created_at, updated_at
             FROM projects
             ORDER BY sort_order ASC, updated_at DESC"
        )
        .fetch_all(pool)
        .await
    }

    /// Buscar proyecto por ID (admin)
    pub async fn find_by_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, title, slug, client, description, featured_image, gallery_image,
                    gallery, categories, technologies, links, skills,
                    status, sort_order, is_featured, in_carousel, showcase_category, detail_title, use_first_gallery_image, meta_title, meta_description,
                    created_at, updated_at
             FROM projects
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// Crear un nuevo proyecto
    pub async fn create(
        pool: &PgPool,
        params: &CreateProjectParams<'_>,
    ) -> Result<Project, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "INSERT INTO projects
                (title, slug, client, description, featured_image, gallery_image,
                    gallery, categories, technologies, links, skills,
                 status, sort_order, is_featured, in_carousel, showcase_category,
                 detail_title, use_first_gallery_image, meta_title, meta_description)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
             RETURNING id, title, slug, client, description, featured_image, gallery_image,
                    gallery, categories, technologies, links, skills,
                       status, sort_order, is_featured, in_carousel, showcase_category, detail_title, use_first_gallery_image, meta_title, meta_description,
                       created_at, updated_at"
        )
        .bind(params.title)
        .bind(params.slug)
        .bind(params.client)
        .bind(params.description)
        .bind(params.featured_image)
        .bind(params.gallery_image)
        .bind(params.gallery)
        .bind(params.categories)
        .bind(params.technologies)
        .bind(params.links)
        .bind(params.skills)
        .bind(params.status)
        .bind(params.sort_order)
        .bind(params.is_featured)
        .bind(params.in_carousel)
        .bind(params.showcase_category)
        .bind(params.detail_title)
        .bind(params.use_first_gallery_image)
        .bind(params.meta_title)
        .bind(params.meta_description)
        .fetch_one(pool)
        .await
    }

    /// Actualizar un proyecto (parcial, COALESCE)
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        params: &UpdateProjectParams<'_>,
    ) -> Result<Project, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "UPDATE projects SET
                title = COALESCE($2, title),
                slug = COALESCE($3, slug),
                client = COALESCE($4, client),
                description = COALESCE($5, description),
                featured_image = COALESCE($6, featured_image),
                gallery_image = COALESCE($7, gallery_image),
                gallery = COALESCE($8, gallery),
                categories = COALESCE($9, categories),
                technologies = COALESCE($10, technologies),
                links = COALESCE($11, links),
                skills = COALESCE($12, skills),
                status = COALESCE($13, status),
                sort_order = COALESCE($14, sort_order),
                is_featured = COALESCE($15, is_featured),
                in_carousel = COALESCE($16, in_carousel),
                showcase_category = COALESCE($17, showcase_category),
                detail_title = COALESCE($18, detail_title),
                use_first_gallery_image = COALESCE($19, use_first_gallery_image),
                meta_title = COALESCE($20, meta_title),
                meta_description = COALESCE($21, meta_description),
                updated_at = NOW()
             WHERE id = $1
             RETURNING id, title, slug, client, description, featured_image, gallery_image,
                    gallery, categories, technologies, links, skills,
                       status, sort_order, is_featured, in_carousel, showcase_category, detail_title, use_first_gallery_image, meta_title, meta_description,
                       created_at, updated_at"
        )
        .bind(id)
        .bind(params.title)
        .bind(params.slug)
        .bind(params.client)
        .bind(params.description)
        .bind(params.featured_image)
        .bind(params.gallery_image)
        .bind(params.gallery)
        .bind(params.categories)
        .bind(params.technologies)
        .bind(params.links)
        .bind(params.skills)
        .bind(params.status)
        .bind(params.sort_order)
        .bind(params.is_featured)
        .bind(params.in_carousel)
        .bind(params.showcase_category)
        .bind(params.detail_title)
        .bind(params.use_first_gallery_image)
        .bind(params.meta_title)
        .bind(params.meta_description)
        .fetch_one(pool)
        .await
    }

    /// Archivar un proyecto (soft delete: status = 'archived')
    pub async fn archive(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE projects SET status = 'archived', updated_at = NOW() WHERE id = $1"
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /* [084A-10] Hard delete: elimina permanentemente el proyecto */
    pub async fn hard_delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM projects WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /* [124A-CMS3] Reordenar proyectos en batch.
     * Recibe pares (id, sort_order) y actualiza todos en una sola transacción.
     * Usa CTE con UNNEST para evitar N+1 queries. */
    pub async fn reorder(
        pool: &PgPool,
        items: &[(Uuid, i32)],
    ) -> Result<(), sqlx::Error> {
        if items.is_empty() {
            return Ok(());
        }

        let ids: Vec<Uuid> = items.iter().map(|(id, _)| *id).collect();
        let orders: Vec<i32> = items.iter().map(|(_, order)| *order).collect();

        sqlx::query(
            "UPDATE projects AS p SET
                sort_order = v.new_order,
                updated_at = NOW()
             FROM (SELECT UNNEST($1::uuid[]) AS id, UNNEST($2::int[]) AS new_order) AS v
             WHERE p.id = v.id"
        )
        .bind(&ids)
        .bind(&orders)
        .execute(pool)
        .await?;

        Ok(())
    }

    /* [124A-SENT-R1] Slugs de proyectos públicos para sitemap.xml.
     * runtime query (sin macro) para no requerir sqlx prepare contra BD en vivo. */
    pub async fn public_slugs(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT slug FROM projects WHERE slug IS NOT NULL AND slug != ''"
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

}
