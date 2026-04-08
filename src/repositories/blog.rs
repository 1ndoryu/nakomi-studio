/* sentinel-disable-file sqlx-query-sin-macro sqlx-query-as-sin-macro: blog usa runtime queries
 * con COALESCE para partial updates y tipos FromRow genéricos. */
/* [074A-10] Repositorio para blog posts.
 * Queries SQL directas con sqlx. Usa COALESCE para partial updates. */

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::BlogPost;

/// Parámetros para crear un blog post
pub struct CreateBlogPostParams<'a> {
    pub author_id: Uuid,
    pub title: &'a str,
    pub slug: &'a str,
    pub excerpt: Option<&'a str>,
    pub content: &'a str,
    pub featured_image: Option<&'a str>,
    pub status: &'a str,
    pub tags: &'a serde_json::Value,
    pub meta_title: Option<&'a str>,
    pub meta_description: Option<&'a str>,
}

/// Parámetros para actualizar un blog post (parcial)
pub struct UpdateBlogPostParams<'a> {
    pub title: Option<&'a str>,
    pub slug: Option<&'a str>,
    pub excerpt: Option<&'a str>,
    pub content: Option<&'a str>,
    pub featured_image: Option<&'a str>,
    pub status: Option<&'a str>,
    pub tags: Option<&'a serde_json::Value>,
    pub meta_title: Option<&'a str>,
    pub meta_description: Option<&'a str>,
}

pub struct BlogRepository;

impl BlogRepository {
    /// Lista posts publicados (público), paginados, ordenados por `published_at` DESC
    pub async fn list_published(
        pool: &PgPool,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<BlogPost>, i64), sqlx::Error> {
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM blog_posts WHERE status = 'published'"
        )
        .fetch_one(pool)
        .await?;

        let posts = sqlx::query_as::<_, BlogPost>(
            "SELECT id, author_id, title, slug, excerpt, content, featured_image,
                    status, tags, meta_title, meta_description, published_at,
                    created_at, updated_at
             FROM blog_posts
             WHERE status = 'published'
             ORDER BY published_at DESC NULLS LAST
             LIMIT $1 OFFSET $2"
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok((posts, total.0))
    }

    /// Buscar post publicado por slug (público)
    pub async fn find_published_by_slug(
        pool: &PgPool,
        slug: &str,
    ) -> Result<Option<BlogPost>, sqlx::Error> {
        sqlx::query_as::<_, BlogPost>(
            "SELECT id, author_id, title, slug, excerpt, content, featured_image,
                    status, tags, meta_title, meta_description, published_at,
                    created_at, updated_at
             FROM blog_posts
             WHERE slug = $1 AND status = 'published'"
        )
        .bind(slug)
        .fetch_optional(pool)
        .await
    }

    /// Lista todos los posts (admin), sin filtro de status
    pub async fn list_all(pool: &PgPool) -> Result<Vec<BlogPost>, sqlx::Error> {
        sqlx::query_as::<_, BlogPost>(
            "SELECT id, author_id, title, slug, excerpt, content, featured_image,
                    status, tags, meta_title, meta_description, published_at,
                    created_at, updated_at
             FROM blog_posts
             ORDER BY updated_at DESC"
        )
        .fetch_all(pool)
        .await
    }

    /// Buscar post por ID (admin)
    pub async fn find_by_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<BlogPost>, sqlx::Error> {
        sqlx::query_as::<_, BlogPost>(
            "SELECT id, author_id, title, slug, excerpt, content, featured_image,
                    status, tags, meta_title, meta_description, published_at,
                    created_at, updated_at
             FROM blog_posts
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// Crear un nuevo blog post
    pub async fn create(
        pool: &PgPool,
        params: &CreateBlogPostParams<'_>,
    ) -> Result<BlogPost, sqlx::Error> {
        let published_at = if params.status == "published" {
            Some(chrono::Utc::now())
        } else {
            None
        };

        sqlx::query_as::<_, BlogPost>(
            "INSERT INTO blog_posts
                (author_id, title, slug, excerpt, content, featured_image,
                 status, tags, meta_title, meta_description, published_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             RETURNING id, author_id, title, slug, excerpt, content, featured_image,
                       status, tags, meta_title, meta_description, published_at,
                       created_at, updated_at"
        )
        .bind(params.author_id)
        .bind(params.title)
        .bind(params.slug)
        .bind(params.excerpt)
        .bind(params.content)
        .bind(params.featured_image)
        .bind(params.status)
        .bind(params.tags)
        .bind(params.meta_title)
        .bind(params.meta_description)
        .bind(published_at)
        .fetch_one(pool)
        .await
    }

    /// Actualizar un blog post (parcial, COALESCE)
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        params: &UpdateBlogPostParams<'_>,
    ) -> Result<BlogPost, sqlx::Error> {
        /* Si el status cambia a published y no tenía `published_at`, setearlo ahora */
        sqlx::query_as::<_, BlogPost>(
            "UPDATE blog_posts SET
                title = COALESCE($2, title),
                slug = COALESCE($3, slug),
                excerpt = COALESCE($4, excerpt),
                content = COALESCE($5, content),
                featured_image = COALESCE($6, featured_image),
                status = COALESCE($7, status),
                tags = COALESCE($8, tags),
                meta_title = COALESCE($9, meta_title),
                meta_description = COALESCE($10, meta_description),
                published_at = CASE
                    WHEN COALESCE($7, status) = 'published' AND published_at IS NULL
                    THEN NOW()
                    ELSE published_at
                END,
                updated_at = NOW()
             WHERE id = $1
             RETURNING id, author_id, title, slug, excerpt, content, featured_image,
                       status, tags, meta_title, meta_description, published_at,
                       created_at, updated_at"
        )
        .bind(id)
        .bind(params.title)
        .bind(params.slug)
        .bind(params.excerpt)
        .bind(params.content)
        .bind(params.featured_image)
        .bind(params.status)
        .bind(params.tags)
        .bind(params.meta_title)
        .bind(params.meta_description)
        .fetch_one(pool)
        .await
    }

    /// Archivar un blog post (soft delete: status = 'archived')
    pub async fn archive(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE blog_posts SET status = 'archived', updated_at = NOW() WHERE id = $1"
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /* [084A-10] Hard delete: elimina permanentemente el blog post */
    pub async fn hard_delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM blog_posts WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Obtener nombre del autor por user ID (usa `display_name` o email como fallback)
    pub async fn get_author_name(pool: &PgPool, author_id: Uuid) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(Option<String>, String)> = sqlx::query_as(
            "SELECT display_name, email FROM users WHERE id = $1"
        )
        .bind(author_id)
        .fetch_optional(pool)
        .await?;
        Ok(row.map(|r| r.0.unwrap_or(r.1)))
    }
}
