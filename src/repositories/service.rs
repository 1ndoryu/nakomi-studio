/* [174A-2] Repositorio de servicios: CRUD completo sobre services, service_plans, service_plan_phases.
 * Migrado desde OrderRepository para cumplir SRP. */
/* sentinel-disable-file sqlx-query-sin-macro sqlx-query-as-sin-macro: 3 queries dinámicas
 * legacy (DELETE plans, archive, check exists) que no usan macros. */

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{SavePlanItem, ServicePlan, ServicePlanPhase, ServiceRecord};

pub struct ServiceRepository;

impl ServiceRepository {
    /* Slugs de servicios públicos para sitemap.xml.
     * Equivalente a la query directa que vivía en seo.rs. */
    pub async fn public_slugs(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT slug FROM services WHERE slug IS NOT NULL AND slug != ''",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    /* ============================================================
    SERVICIOS (catálogo — solo lectura)
    ============================================================ */

    pub async fn list_services(pool: &PgPool) -> Result<Vec<ServiceRecord>, sqlx::Error> {
        sqlx::query_as!(
            ServiceRecord,
            r#"SELECT id, slug, title, description, base_price_cents, currency, is_active, sort_order, created_at,
             image_url, gallery, skills, content, meta_title, meta_description, status, updated_at
             FROM services WHERE is_active = true ORDER BY sort_order"#,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_service_by_slug(
        pool: &PgPool,
        slug: &str,
    ) -> Result<Option<ServiceRecord>, sqlx::Error> {
        sqlx::query_as!(
            ServiceRecord,
            r#"SELECT id, slug, title, description, base_price_cents, currency, is_active, sort_order, created_at,
             image_url, gallery, skills, content, meta_title, meta_description, status, updated_at
             FROM services WHERE slug = $1 AND is_active = true"#,
            slug,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn list_plans_for_service(
        pool: &PgPool,
        service_id: Uuid,
    ) -> Result<Vec<ServicePlan>, sqlx::Error> {
        sqlx::query_as!(
            ServicePlan,
            r#"SELECT id, service_id, slug, name, price_cents, description, features,
             is_highlighted, is_custom, stripe_price_id, sort_order
             FROM service_plans WHERE service_id = $1 ORDER BY sort_order"#,
            service_id,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_plan_by_slug(
        pool: &PgPool,
        service_id: Uuid,
        plan_slug: &str,
    ) -> Result<Option<ServicePlan>, sqlx::Error> {
        sqlx::query_as!(
            ServicePlan,
            r#"SELECT id, service_id, slug, name, price_cents, description, features,
             is_highlighted, is_custom, stripe_price_id, sort_order
             FROM service_plans WHERE service_id = $1 AND slug = $2"#,
            service_id,
            plan_slug,
        )
        .fetch_all(pool)
        .await
        .map(|mut v| v.pop())
    }

    pub async fn list_plan_phases(
        pool: &PgPool,
        plan_id: Uuid,
    ) -> Result<Vec<ServicePlanPhase>, sqlx::Error> {
        sqlx::query_as!(
            ServicePlanPhase,
            r#"SELECT id, plan_id, phase_number, title, description,
             percentage_of_total, estimated_days, max_revisions
             FROM service_plan_phases WHERE plan_id = $1 ORDER BY phase_number"#,
            plan_id,
        )
        .fetch_all(pool)
        .await
    }

    /* [074A-66] Batch replace de planes para un servicio.
     * DELETE CASCADE elimina planes y fases existentes, luego inserta los nuevos. */
    pub async fn save_plans_for_service(
        pool: &PgPool,
        service_id: Uuid,
        plans: &[SavePlanItem],
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        sqlx::query!(
            "DELETE FROM service_plans WHERE service_id = $1",
            service_id,
        )
        .execute(&mut *tx)
        .await?;

        for plan in plans {
            let plan_id = sqlx::query_scalar!(
                r#"INSERT INTO service_plans
                   (service_id, slug, name, price_cents, description, features,
                    is_highlighted, is_custom, sort_order)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                 RETURNING id"#,
                service_id,
                plan.slug,
                plan.name,
                plan.price_cents,
                plan.description,
                plan.features,
                plan.is_highlighted,
                plan.is_custom,
                plan.sort_order,
            )
            .fetch_one(&mut *tx)
            .await?;

            for phase in &plan.phases {
                sqlx::query!(
                    r#"INSERT INTO service_plan_phases
                       (plan_id, phase_number, title, description,
                        percentage_of_total, estimated_days, max_revisions)
                     VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
                    plan_id,
                    phase.phase_number,
                    phase.title,
                    phase.description,
                    phase.percentage_of_total,
                    phase.estimated_days,
                    phase.max_revisions,
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    /* ============================================================
    SERVICIOS — Admin CRUD (074A-8)
    ============================================================ */

    /// Lista TODOS los servicios (incluyendo inactivos/draft) para el panel admin
    pub async fn list_all_services(pool: &PgPool) -> Result<Vec<ServiceRecord>, sqlx::Error> {
        sqlx::query_as!(
            ServiceRecord,
            r#"SELECT id, slug, title, description, base_price_cents, currency, is_active, sort_order, created_at,
             image_url, gallery, skills, content, meta_title, meta_description, status, updated_at
             FROM services ORDER BY sort_order, created_at DESC"#,
        )
        .fetch_all(pool)
        .await
    }

    /// Busca un servicio por ID (sin filtrar por `is_active`)
    pub async fn find_service_by_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<ServiceRecord>, sqlx::Error> {
        sqlx::query_as!(
            ServiceRecord,
            r#"SELECT id, slug, title, description, base_price_cents, currency, is_active, sort_order, created_at,
             image_url, gallery, skills, content, meta_title, meta_description, status, updated_at
             FROM services WHERE id = $1"#,
            id,
        )
        .fetch_optional(pool)
        .await
    }

    /// Crea un servicio nuevo
    pub async fn create_service(
        pool: &PgPool,
        params: &crate::models::CreateServiceRequest,
    ) -> Result<ServiceRecord, sqlx::Error> {
        sqlx::query_as!(
            ServiceRecord,
            r#"INSERT INTO services (title, slug, description, base_price_cents, currency,
             image_url, gallery, skills, content, meta_title, meta_description, status, sort_order)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             RETURNING id, slug, title, description, base_price_cents, currency, is_active, sort_order, created_at,
             image_url, gallery, skills, content, meta_title, meta_description, status, updated_at"#,
            params.title,
            params.slug,
            params.description,
            params.base_price_cents.unwrap_or(0),
            params.currency.as_deref().unwrap_or("USD"),
            params.image_url,
            params.gallery.clone().unwrap_or_else(|| serde_json::json!([])),
            params.skills.clone().unwrap_or_else(|| serde_json::json!([])),
            params.content,
            params.meta_title,
            params.meta_description,
            params.status.as_deref().unwrap_or("draft"),
            params.sort_order.unwrap_or(0),
        )
        .fetch_one(pool)
        .await
    }

    /// Actualiza un servicio existente (solo campos proporcionados)
    pub async fn update_service(
        pool: &PgPool,
        id: Uuid,
        params: &crate::models::UpdateServiceRequest,
    ) -> Result<ServiceRecord, sqlx::Error> {
        sqlx::query_as!(
            ServiceRecord,
            r#"UPDATE services SET
             title = COALESCE($2, title),
             slug = COALESCE($3, slug),
             description = COALESCE($4, description),
             base_price_cents = COALESCE($5, base_price_cents),
             currency = COALESCE($6, currency),
             is_active = COALESCE($7, is_active),
             image_url = COALESCE($8, image_url),
             gallery = COALESCE($9, gallery),
             skills = COALESCE($10, skills),
             content = COALESCE($11, content),
             meta_title = COALESCE($12, meta_title),
             meta_description = COALESCE($13, meta_description),
             status = COALESCE($14, status),
             sort_order = COALESCE($15, sort_order),
             updated_at = NOW()
             WHERE id = $1
             RETURNING id, slug, title, description, base_price_cents, currency, is_active, sort_order, created_at,
             image_url, gallery, skills, content, meta_title, meta_description, status, updated_at"#,
            id,
            params.title,
            params.slug,
            params.description,
            params.base_price_cents,
            params.currency,
            params.is_active,
            params.image_url,
            params.gallery.clone(),
            params.skills.clone(),
            params.content,
            params.meta_title,
            params.meta_description,
            params.status,
            params.sort_order,
        )
        .fetch_one(pool)
        .await
    }

    /// Archiva un servicio (soft delete: `is_active`=false, status=archived)
    pub async fn archive_service(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE services SET is_active = false, status = 'archived', updated_at = NOW() WHERE id = $1"
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /* [084A-10] Hard delete: elimina permanentemente el servicio y sus planes (CASCADE).
     * Previamente verificar que no existan órdenes referenciando este servicio. */
    pub async fn hard_delete_service(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM services WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /* [124A-CMS10] Batch reorder de servicios — mismo patrón que ProjectRepository::reorder */
    pub async fn reorder_services(pool: &PgPool, items: &[(Uuid, i32)]) -> Result<(), sqlx::Error> {
        if items.is_empty() {
            return Ok(());
        }
        let ids: Vec<Uuid> = items.iter().map(|(id, _)| *id).collect();
        let orders: Vec<i32> = items.iter().map(|(_, order)| *order).collect();

        sqlx::query(
            "UPDATE services AS s SET
                sort_order = v.new_order
             FROM (SELECT UNNEST($1::uuid[]) AS id, UNNEST($2::int[]) AS new_order) AS v
             WHERE s.id = v.id",
        )
        .bind(&ids)
        .bind(&orders)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Verifica si existen órdenes que referencien un servicio
    pub async fn service_has_orders(pool: &PgPool, service_id: Uuid) -> Result<bool, sqlx::Error> {
        let row: (bool,) =
            sqlx::query_as("SELECT EXISTS(SELECT 1 FROM orders WHERE service_id = $1)")
                .bind(service_id)
                .fetch_one(pool)
                .await?;
        Ok(row.0)
    }
}
