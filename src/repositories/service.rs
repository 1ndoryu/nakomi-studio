/* [174A-2] Repositorio de servicios: CRUD completo sobre services, service_plans, service_plan_phases.
 * Migrado desde OrderRepository para cumplir SRP. */
/* sentinel-disable-file sqlx-query-sin-macro sqlx-query-as-sin-macro: 3 queries dinámicas
 * legacy (DELETE plans, archive, check exists) que no usan macros. */

use std::collections::{HashMap, HashSet};

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::models::{SavePhaseItem, SavePlanItem, ServicePlan, ServicePlanPhase, ServiceRecord};

pub struct ServiceRepository;

/* [045A-1] Services update adopta el mismo patrón paramétrico que projects.
 * Reduce la fragilidad del query macro en partial updates y centraliza binds tipados. */
pub struct UpdateServiceParams<'a> {
    pub title: Option<&'a str>,
    pub slug: Option<&'a str>,
    pub description: Option<&'a str>,
    pub base_price_cents: Option<i32>,
    pub currency: Option<&'a str>,
    pub is_active: Option<bool>,
    pub image_url: Option<&'a str>,
    pub gallery: Option<&'a serde_json::Value>,
    pub skills: Option<&'a serde_json::Value>,
    pub content: Option<&'a str>,
    pub meta_title: Option<&'a str>,
    pub meta_description: Option<&'a str>,
    pub status: Option<&'a str>,
    pub sort_order: Option<i32>,
}

async fn fetch_existing_service_plans(
    tx: &mut Transaction<'_, Postgres>,
    service_id: Uuid,
) -> Result<Vec<ServicePlan>, sqlx::Error> {
    sqlx::query_as::<_, ServicePlan>(
        r"SELECT id, service_id, slug, name, price_cents, description, features,
           is_highlighted, is_custom, stripe_price_id, sort_order
           FROM service_plans WHERE service_id = $1 ORDER BY sort_order",
    )
    .bind(service_id)
    .fetch_all(&mut **tx)
    .await
}

fn build_service_plan_lookups(
    existing_plans: &[ServicePlan],
) -> (HashMap<Uuid, Uuid>, HashMap<String, Uuid>) {
    let mut existing_by_id = HashMap::new();
    let mut existing_by_slug = HashMap::new();

    for existing in existing_plans {
        existing_by_id.insert(existing.id, existing.id);
        existing_by_slug.insert(existing.slug.clone(), existing.id);
    }

    (existing_by_id, existing_by_slug)
}

fn resolve_existing_plan_id(
    plan: &SavePlanItem,
    existing_by_id: &HashMap<Uuid, Uuid>,
    existing_by_slug: &HashMap<String, Uuid>,
) -> Option<Uuid> {
    plan.id
        .and_then(|incoming_id| existing_by_id.get(&incoming_id).copied())
        .or_else(|| existing_by_slug.get(&plan.slug).copied())
}

async fn upsert_service_plan(
    tx: &mut Transaction<'_, Postgres>,
    service_id: Uuid,
    plan: &SavePlanItem,
    existing_id: Option<Uuid>,
) -> Result<Uuid, sqlx::Error> {
    if let Some(existing_id) = existing_id {
        sqlx::query(
            r"UPDATE service_plans
               SET slug = $3,
                   name = $4,
                   price_cents = $5,
                   description = $6,
                   features = $7,
                   is_highlighted = $8,
                   is_custom = $9,
                   sort_order = $10
             WHERE id = $1 AND service_id = $2",
        )
        .bind(existing_id)
        .bind(service_id)
        .bind(&plan.slug)
        .bind(&plan.name)
        .bind(plan.price_cents)
        .bind(&plan.description)
        .bind(&plan.features)
        .bind(plan.is_highlighted)
        .bind(plan.is_custom)
        .bind(plan.sort_order)
        .execute(&mut **tx)
        .await?;

        Ok(existing_id)
    } else {
        sqlx::query_scalar::<_, Uuid>(
            r"INSERT INTO service_plans
               (service_id, slug, name, price_cents, description, features,
                is_highlighted, is_custom, sort_order)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING id",
        )
        .bind(service_id)
        .bind(&plan.slug)
        .bind(&plan.name)
        .bind(plan.price_cents)
        .bind(&plan.description)
        .bind(&plan.features)
        .bind(plan.is_highlighted)
        .bind(plan.is_custom)
        .bind(plan.sort_order)
        .fetch_one(&mut **tx)
        .await
    }
}

async fn replace_service_plan_phases(
    tx: &mut Transaction<'_, Postgres>,
    plan_id: Uuid,
    phases: &[SavePhaseItem],
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM service_plan_phases WHERE plan_id = $1")
        .bind(plan_id)
        .execute(&mut **tx)
        .await?;

    for phase in phases {
        sqlx::query(
            r"INSERT INTO service_plan_phases
               (plan_id, phase_number, title, description,
                percentage_of_total, estimated_days, max_revisions)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(plan_id)
        .bind(phase.phase_number)
        .bind(&phase.title)
        .bind(&phase.description)
        .bind(phase.percentage_of_total)
        .bind(phase.estimated_days)
        .bind(phase.max_revisions)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn delete_removed_service_plans(
    tx: &mut Transaction<'_, Postgres>,
    service_id: Uuid,
    existing_plans: Vec<ServicePlan>,
    persisted_plan_ids: &HashSet<Uuid>,
) -> Result<(), sqlx::Error> {
    for existing in existing_plans {
        if persisted_plan_ids.contains(&existing.id) {
            continue;
        }

        sqlx::query("DELETE FROM service_plan_phases WHERE plan_id = $1")
            .bind(existing.id)
            .execute(&mut **tx)
            .await?;

        sqlx::query("DELETE FROM service_plans WHERE id = $1 AND service_id = $2")
            .bind(existing.id)
            .bind(service_id)
            .execute(&mut **tx)
            .await?;
    }

    Ok(())
}

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

    /* [074A-66] [035A-26] Persistencia estable de planes para un servicio.
     * Preserva `service_plans.id` en planes existentes para que órdenes históricas sigan
     * apuntando al mismo plan aunque el CMS edite sus fases o metadatos. */
    pub async fn save_plans_for_service(
        pool: &PgPool,
        service_id: Uuid,
        plans: &[SavePlanItem],
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        let existing_plans = fetch_existing_service_plans(&mut tx, service_id).await?;
        let (existing_by_id, existing_by_slug) = build_service_plan_lookups(&existing_plans);
        let mut persisted_plan_ids = HashSet::new();

        for plan in plans {
            let existing_id = resolve_existing_plan_id(plan, &existing_by_id, &existing_by_slug);
            let plan_id = upsert_service_plan(&mut tx, service_id, plan, existing_id).await?;

            persisted_plan_ids.insert(plan_id);

            replace_service_plan_phases(&mut tx, plan_id, &plan.phases).await?;
        }

        delete_removed_service_plans(&mut tx, service_id, existing_plans, &persisted_plan_ids)
            .await?;

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
        params: &UpdateServiceParams<'_>,
    ) -> Result<ServiceRecord, sqlx::Error> {
        sqlx::query_as::<_, ServiceRecord>(
            r"UPDATE services SET
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
               image_url, gallery, skills, content, meta_title, meta_description, status, updated_at",
        )
        .bind(id)
        .bind(params.title)
        .bind(params.slug)
        .bind(params.description)
        .bind(params.base_price_cents)
        .bind(params.currency)
        .bind(params.is_active)
        .bind(params.image_url)
        .bind(params.gallery)
        .bind(params.skills)
        .bind(params.content)
        .bind(params.meta_title)
        .bind(params.meta_description)
        .bind(params.status)
        .bind(params.sort_order)
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
