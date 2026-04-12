/* [124A-SENT-R1] Repositorio mínimo para services: solo public_slugs para sitemap SEO.
 * runtime query (sin macro) para no requerir sqlx prepare contra BD en vivo.
 * Si se necesita CRUD completo, expandir este archivo siguiendo el patrón de ProjectRepository. */

use sqlx::PgPool;

pub struct ServiceRepository;

impl ServiceRepository {
    /* Slugs de servicios públicos para sitemap.xml.
     * Equivalente a la query directa que vivía en seo.rs. */
    pub async fn public_slugs(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT slug FROM services WHERE slug IS NOT NULL AND slug != ''"
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}
