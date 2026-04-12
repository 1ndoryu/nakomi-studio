/* sentinel-disable-file sqlx-query-sin-macro sqlx-query-as-sin-macro: order tiene 3 queries
 * dinámicas legacy (DELETE services, check exists) que no usan macros. */
/* [044A-38] Repositorio de órdenes: CRUD sobre orders, order_phases, services, service_plans.
 * [044A-44] Migrado a query_as! con verificación en compilación. */

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    Order, OrderPhase, OrderStatus, PaymentMode, PhaseStatus,
    SavePlanItem, ServicePlan, ServicePlanPhase, ServiceRecord,
};

/* Structs de parámetros para evitar too_many_arguments (clippy) */

pub struct CreateOrderParams<'a> {
    pub client_id: Uuid,
    pub service_id: Uuid,
    pub plan_id: Uuid,
    pub payment_mode: PaymentMode,
    pub base_price_cents: i32,
    pub discount_percent: i32,
    pub final_price_cents: i32,
    pub project_description: Option<&'a str>,
    pub client_notes: Option<&'a str>,
}

pub struct CreatePhaseParams<'a> {
    pub order_id: Uuid,
    pub phase_number: i32,
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub price_cents: i32,
    pub status: PhaseStatus,
    pub max_revisions: i32,
    pub estimated_days: i32,
}

pub struct OrderRepository;

impl OrderRepository {
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
    pub async fn reorder_services(
        pool: &PgPool,
        items: &[(Uuid, i32)],
    ) -> Result<(), sqlx::Error> {
        if items.is_empty() {
            return Ok(());
        }
        let ids: Vec<Uuid> = items.iter().map(|(id, _)| *id).collect();
        let orders: Vec<i32> = items.iter().map(|(_, order)| *order).collect();

        sqlx::query(
            "UPDATE services AS s SET
                sort_order = v.new_order
             FROM (SELECT UNNEST($1::uuid[]) AS id, UNNEST($2::int[]) AS new_order) AS v
             WHERE s.id = v.id"
        )
        .bind(&ids)
        .bind(&orders)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Verifica si existen órdenes que referencien un servicio
    pub async fn service_has_orders(pool: &PgPool, service_id: Uuid) -> Result<bool, sqlx::Error> {
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM orders WHERE service_id = $1)"
        )
        .bind(service_id)
        .fetch_one(pool)
        .await?;
        Ok(row.0)
    }

    /* ============================================================
       ÓRDENES
       ============================================================ */

    pub async fn create_order(
        pool: &PgPool,
        params: CreateOrderParams<'_>,
    ) -> Result<Order, sqlx::Error> {
        sqlx::query_as!(
            Order,
            r#"INSERT INTO orders (client_id, service_id, plan_id, payment_mode,
             base_price_cents, discount_percent, final_price_cents, project_description, client_notes)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING id, order_number, client_id, service_id, plan_id,
               payment_mode as "payment_mode: PaymentMode",
               base_price_cents, discount_percent, final_price_cents, currency,
               status as "status: OrderStatus",
               assigned_employee_id, assigned_at, auto_assign_deadline, current_phase,
               started_at, completed_at, cancelled_at, project_description, client_notes,
               internal_notes,
               created_at, updated_at, ai_intermediary_enabled, ai_summary, open_to_employees"#,
            params.client_id,
            params.service_id,
            params.plan_id,
            params.payment_mode as PaymentMode,
            params.base_price_cents,
            params.discount_percent,
            params.final_price_cents,
            params.project_description,
            params.client_notes,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn find_order_by_id(
        pool: &PgPool,
        order_id: Uuid,
    ) -> Result<Option<Order>, sqlx::Error> {
        sqlx::query_as!(
            Order,
            r#"SELECT id, order_number, client_id, service_id, plan_id,
               payment_mode as "payment_mode: PaymentMode",
               base_price_cents, discount_percent, final_price_cents, currency,
               status as "status: OrderStatus",
               assigned_employee_id, assigned_at, auto_assign_deadline, current_phase,
                    started_at, completed_at, cancelled_at, project_description, client_notes,
                    internal_notes,
               created_at, updated_at, ai_intermediary_enabled, ai_summary, open_to_employees
             FROM orders WHERE id = $1"#,
            order_id,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn list_orders_for_client(
        pool: &PgPool,
        client_id: Uuid,
    ) -> Result<Vec<Order>, sqlx::Error> {
        sqlx::query_as!(
            Order,
            r#"SELECT id, order_number, client_id, service_id, plan_id,
               payment_mode as "payment_mode: PaymentMode",
               base_price_cents, discount_percent, final_price_cents, currency,
               status as "status: OrderStatus",
               assigned_employee_id, assigned_at, auto_assign_deadline, current_phase,
                    started_at, completed_at, cancelled_at, project_description, client_notes,
                    internal_notes,
               created_at, updated_at, ai_intermediary_enabled, ai_summary, open_to_employees
             FROM orders WHERE client_id = $1 ORDER BY created_at DESC"#,
            client_id,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn list_orders_for_employee(
        pool: &PgPool,
        employee_id: Uuid,
    ) -> Result<Vec<Order>, sqlx::Error> {
        sqlx::query_as!(
            Order,
            r#"SELECT id, order_number, client_id, service_id, plan_id,
               payment_mode as "payment_mode: PaymentMode",
               base_price_cents, discount_percent, final_price_cents, currency,
               status as "status: OrderStatus",
               assigned_employee_id, assigned_at, auto_assign_deadline, current_phase,
                    started_at, completed_at, cancelled_at, project_description, client_notes,
                    internal_notes,
               created_at, updated_at, ai_intermediary_enabled, ai_summary, open_to_employees
             FROM orders WHERE assigned_employee_id = $1 ORDER BY created_at DESC"#,
            employee_id,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn list_all_orders(pool: &PgPool) -> Result<Vec<Order>, sqlx::Error> {
        sqlx::query_as!(
            Order,
            r#"SELECT id, order_number, client_id, service_id, plan_id,
               payment_mode as "payment_mode: PaymentMode",
               base_price_cents, discount_percent, final_price_cents, currency,
               status as "status: OrderStatus",
               assigned_employee_id, assigned_at, auto_assign_deadline, current_phase,
                    started_at, completed_at, cancelled_at, project_description, client_notes,
                    internal_notes,
               created_at, updated_at, ai_intermediary_enabled, ai_summary, open_to_employees
             FROM orders ORDER BY created_at DESC"#,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn list_unassigned_orders(pool: &PgPool) -> Result<Vec<Order>, sqlx::Error> {
        sqlx::query_as!(
            Order,
            r#"SELECT id, order_number, client_id, service_id, plan_id,
               payment_mode as "payment_mode: PaymentMode",
               base_price_cents, discount_percent, final_price_cents, currency,
               status as "status: OrderStatus",
               assigned_employee_id, assigned_at, auto_assign_deadline, current_phase,
                    started_at, completed_at, cancelled_at, project_description, client_notes,
                    internal_notes,
               created_at, updated_at, ai_intermediary_enabled, ai_summary, open_to_employees
             FROM orders WHERE status = 'awaiting_assignment' ORDER BY created_at ASC"#,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn update_order_status(
        pool: &PgPool,
        order_id: Uuid,
        status: OrderStatus,
    ) -> Result<Order, sqlx::Error> {
        sqlx::query_as!(
            Order,
            r#"UPDATE orders SET status = $2, updated_at = NOW() WHERE id = $1
             RETURNING id, order_number, client_id, service_id, plan_id,
               payment_mode as "payment_mode: PaymentMode",
               base_price_cents, discount_percent, final_price_cents, currency,
               status as "status: OrderStatus",
               assigned_employee_id, assigned_at, auto_assign_deadline, current_phase,
                    started_at, completed_at, cancelled_at, project_description, client_notes,
                    internal_notes,
               created_at, updated_at, ai_intermediary_enabled, ai_summary, open_to_employees"#,
            order_id,
            status as OrderStatus,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn assign_order(
        pool: &PgPool,
        order_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Order, sqlx::Error> {
        sqlx::query_as!(
            Order,
            r#"UPDATE orders SET assigned_employee_id = $2, assigned_at = NOW(),
             status = 'in_progress', started_at = COALESCE(started_at, NOW()), updated_at = NOW()
             WHERE id = $1
             RETURNING id, order_number, client_id, service_id, plan_id,
               payment_mode as "payment_mode: PaymentMode",
               base_price_cents, discount_percent, final_price_cents, currency,
               status as "status: OrderStatus",
               assigned_employee_id, assigned_at, auto_assign_deadline, current_phase,
                    started_at, completed_at, cancelled_at, project_description, client_notes,
                    internal_notes,
               created_at, updated_at, ai_intermediary_enabled, ai_summary, open_to_employees"#,
            order_id,
            employee_id,
        )
        .fetch_one(pool)
        .await
    }

    /* ============================================================
       FASES DE ORDEN
       ============================================================ */

    pub async fn create_order_phase(
        pool: &PgPool,
        params: CreatePhaseParams<'_>,
    ) -> Result<OrderPhase, sqlx::Error> {
        sqlx::query_as!(
            OrderPhase,
            r#"INSERT INTO order_phases (order_id, phase_number, title, description,
             price_cents, status, max_revisions, estimated_days)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, order_id, phase_number, title, description, price_cents,
               status as "status: PhaseStatus",
               max_revisions, revisions_used, estimated_days,
               started_at, delivered_at, approved_at, deadline, created_at, updated_at"#,
            params.order_id,
            params.phase_number,
            params.title,
            params.description,
            params.price_cents,
            params.status as PhaseStatus,
            params.max_revisions,
            params.estimated_days,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn list_order_phases(
        pool: &PgPool,
        order_id: Uuid,
    ) -> Result<Vec<OrderPhase>, sqlx::Error> {
        sqlx::query_as!(
            OrderPhase,
            r#"SELECT id, order_id, phase_number, title, description, price_cents,
               status as "status: PhaseStatus",
               max_revisions, revisions_used, estimated_days,
               started_at, delivered_at, approved_at, deadline, created_at, updated_at
             FROM order_phases WHERE order_id = $1 ORDER BY phase_number"#,
            order_id,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn update_phase_status(
        pool: &PgPool,
        phase_id: Uuid,
        status: PhaseStatus,
    ) -> Result<OrderPhase, sqlx::Error> {
        sqlx::query_as!(
            OrderPhase,
            r#"UPDATE order_phases SET status = $2, updated_at = NOW() WHERE id = $1
             RETURNING id, order_id, phase_number, title, description, price_cents,
               status as "status: PhaseStatus",
               max_revisions, revisions_used, estimated_days,
               started_at, delivered_at, approved_at, deadline, created_at, updated_at"#,
            phase_id,
            status as PhaseStatus,
        )
        .fetch_one(pool)
        .await
    }

    /* [044A-38 Fase 2] Busca una fase por order_id + phase_number */
    pub async fn find_phase_by_number(
        pool: &PgPool,
        order_id: Uuid,
        phase_number: i32,
    ) -> Result<Option<OrderPhase>, sqlx::Error> {
        sqlx::query_as!(
            OrderPhase,
            r#"SELECT id, order_id, phase_number, title, description, price_cents,
               status as "status: PhaseStatus",
               max_revisions, revisions_used, estimated_days,
               started_at, delivered_at, approved_at, deadline, created_at, updated_at
             FROM order_phases WHERE order_id = $1 AND phase_number = $2"#,
            order_id,
            phase_number,
        )
        .fetch_optional(pool)
        .await
    }

    /* [044A-38 Fase 2] Marca fase como entregada */
    pub async fn deliver_phase(
        pool: &PgPool,
        phase_id: Uuid,
    ) -> Result<OrderPhase, sqlx::Error> {
        sqlx::query_as!(
            OrderPhase,
            r#"UPDATE order_phases SET status = 'delivered', delivered_at = NOW(), updated_at = NOW()
             WHERE id = $1
             RETURNING id, order_id, phase_number, title, description, price_cents,
               status as "status: PhaseStatus",
               max_revisions, revisions_used, estimated_days,
               started_at, delivered_at, approved_at, deadline, created_at, updated_at"#,
            phase_id,
        )
        .fetch_one(pool)
        .await
    }

    /* [044A-38 Fase 2] Marca fase como aprobada */
    pub async fn approve_phase(
        pool: &PgPool,
        phase_id: Uuid,
    ) -> Result<OrderPhase, sqlx::Error> {
        sqlx::query_as!(
            OrderPhase,
            r#"UPDATE order_phases SET status = 'approved', approved_at = NOW(), updated_at = NOW()
             WHERE id = $1
             RETURNING id, order_id, phase_number, title, description, price_cents,
               status as "status: PhaseStatus",
               max_revisions, revisions_used, estimated_days,
               started_at, delivered_at, approved_at, deadline, created_at, updated_at"#,
            phase_id,
        )
        .fetch_one(pool)
        .await
    }

    /* [044A-38 Fase 2] Incrementa revisiones y pone status revision_requested */
    pub async fn request_revision(
        pool: &PgPool,
        phase_id: Uuid,
    ) -> Result<OrderPhase, sqlx::Error> {
        sqlx::query_as!(
            OrderPhase,
            r#"UPDATE order_phases SET status = 'revision_requested',
             revisions_used = revisions_used + 1, updated_at = NOW()
             WHERE id = $1
             RETURNING id, order_id, phase_number, title, description, price_cents,
               status as "status: PhaseStatus",
               max_revisions, revisions_used, estimated_days,
               started_at, delivered_at, approved_at, deadline, created_at, updated_at"#,
            phase_id,
        )
        .fetch_one(pool)
        .await
    }

    /* [044A-38 Fase 2] Cancela una orden: marca cancelled + timestamp */
    pub async fn cancel_order(
        pool: &PgPool,
        order_id: Uuid,
    ) -> Result<Order, sqlx::Error> {
        sqlx::query_as!(
            Order,
            r#"UPDATE orders SET status = 'cancelled', cancelled_at = NOW(), updated_at = NOW()
             WHERE id = $1
             RETURNING id, order_number, client_id, service_id, plan_id,
               payment_mode as "payment_mode: PaymentMode",
               base_price_cents, discount_percent, final_price_cents, currency,
               status as "status: OrderStatus",
               assigned_employee_id, assigned_at, auto_assign_deadline, current_phase,
               started_at, completed_at, cancelled_at, project_description, client_notes,
               internal_notes,
               created_at, updated_at, ai_intermediary_enabled, ai_summary, open_to_employees"#,
            order_id,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn update_project_description(
        pool: &PgPool,
        order_id: Uuid,
        project_description: &str,
    ) -> Result<Order, sqlx::Error> {
        sqlx::query_as!(
            Order,
            r#"UPDATE orders
               SET project_description = $2, updated_at = NOW()
             WHERE id = $1
             RETURNING id, order_number, client_id, service_id, plan_id,
               payment_mode as "payment_mode: PaymentMode",
               base_price_cents, discount_percent, final_price_cents, currency,
               status as "status: OrderStatus",
               assigned_employee_id, assigned_at, auto_assign_deadline, current_phase,
               started_at, completed_at, cancelled_at, project_description, client_notes,
               internal_notes,
               created_at, updated_at, ai_intermediary_enabled, ai_summary, open_to_employees"#,
            order_id,
            project_description,
        )
        .fetch_one(pool)
        .await
    }

    /* [044A-38 Fase 2] Actualiza current_phase de una orden */
    pub async fn update_current_phase(
        pool: &PgPool,
        order_id: Uuid,
        phase_number: i32,
    ) -> Result<Order, sqlx::Error> {
        sqlx::query_as!(
            Order,
            r#"UPDATE orders SET current_phase = $2, updated_at = NOW()
             WHERE id = $1
             RETURNING id, order_number, client_id, service_id, plan_id,
               payment_mode as "payment_mode: PaymentMode",
               base_price_cents, discount_percent, final_price_cents, currency,
               status as "status: OrderStatus",
               assigned_employee_id, assigned_at, auto_assign_deadline, current_phase,
                    started_at, completed_at, cancelled_at, project_description, client_notes,
                    internal_notes,
               created_at, updated_at, ai_intermediary_enabled, ai_summary, open_to_employees"#,
            order_id,
            phase_number,
        )
        .fetch_one(pool)
        .await
    }

    /* [044A-38 Fase 2] Marca orden como completada */
    pub async fn complete_order(
        pool: &PgPool,
        order_id: Uuid,
    ) -> Result<Order, sqlx::Error> {
        sqlx::query_as!(
            Order,
            r#"UPDATE orders SET status = 'completed', completed_at = NOW(), updated_at = NOW()
             WHERE id = $1
             RETURNING id, order_number, client_id, service_id, plan_id,
               payment_mode as "payment_mode: PaymentMode",
               base_price_cents, discount_percent, final_price_cents, currency,
               status as "status: OrderStatus",
               assigned_employee_id, assigned_at, auto_assign_deadline, current_phase,
                    started_at, completed_at, cancelled_at, project_description, client_notes,
                    internal_notes,
               created_at, updated_at, ai_intermediary_enabled, ai_summary, open_to_employees"#,
            order_id,
        )
        .fetch_one(pool)
        .await
    }

    /// Obtiene título, slug del servicio y nombre del plan para un order
    pub async fn get_order_display_info(
        pool: &PgPool,
        service_id: Uuid,
        plan_id: Uuid,
    ) -> Result<(String, String, String), sqlx::Error> {
        let (service_title, service_slug): (String, String) = {
            let row = sqlx::query!(
                r#"SELECT title, slug FROM services WHERE id = $1"#,
                service_id,
            )
            .fetch_one(pool)
            .await?;
            (row.title, row.slug)
        };

        let plan_name: String = sqlx::query_scalar!(
            r#"SELECT name FROM service_plans WHERE id = $1"#,
            plan_id,
        )
        .fetch_one(pool)
        .await?;

        Ok((service_title, service_slug, plan_name))
    }

    /* [064A-30] Obtiene display_name del empleado asignado a una orden.
     * Retorna None si employee_id es None o si el usuario no tiene display_name.
     * Usa query_scalar sin macro para no depender del cache offline. */
    pub async fn get_employee_display_name(
        pool: &PgPool,
        employee_id: Option<Uuid>,
    ) -> Result<Option<String>, sqlx::Error> {
        let Some(eid) = employee_id else {
            return Ok(None);
        };
        let name: Option<String> = sqlx::query_scalar(
            "SELECT display_name FROM users WHERE id = $1",
        )
        .bind(eid)
        .fetch_optional(pool)
        .await?
        .flatten();
        Ok(name)
    }

    /* [074A-53] Obtener nombre del cliente para la respuesta de órdenes */
    pub async fn get_client_display_name(
        pool: &PgPool,
        client_id: Uuid,
    ) -> Result<Option<String>, sqlx::Error> {
        let name: Option<String> = sqlx::query_scalar(
            "SELECT display_name FROM users WHERE id = $1",
        )
        .bind(client_id)
        .fetch_optional(pool)
        .await?
        .flatten();
        Ok(name)
    }

    /* ============================================================
       [044A-38 Fase 4] ASIGNACIÓN Y AUTO-ASIGNACIÓN
       ============================================================ */

    /// Transiciona orden a `awaiting_assignment` y establece deadline de 48h para que empleados la tomen
    pub async fn set_awaiting_assignment(
        pool: &PgPool,
        order_id: Uuid,
    ) -> Result<Order, sqlx::Error> {
        sqlx::query_as!(
            Order,
            r#"UPDATE orders SET status = 'awaiting_assignment',
             auto_assign_deadline = NOW() + INTERVAL '48 hours',
             updated_at = NOW() WHERE id = $1
             RETURNING id, order_number, client_id, service_id, plan_id,
               payment_mode as "payment_mode: PaymentMode",
               base_price_cents, discount_percent, final_price_cents, currency,
               status as "status: OrderStatus",
               assigned_employee_id, assigned_at, auto_assign_deadline, current_phase,
               started_at, completed_at, cancelled_at, project_description, client_notes,
               internal_notes,
               created_at, updated_at, ai_intermediary_enabled, ai_summary, open_to_employees"#,
            order_id,
        )
        .fetch_one(pool)
        .await
    }

    /* [124A-SENT-R1] Reabre una orden rechazada al pool de disponibles.
     * Difiere de set_awaiting_assignment: también unasigna al empleado y
     * no establece deadline de auto-asignación (queda null para no urgir).
     * runtime query (sin macro) para no requerir sqlx prepare. */
    pub async fn reopen_after_rejection(
        pool: &PgPool,
        order_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE orders \
             SET status = 'awaiting_assignment', \
                 assigned_employee_id = NULL, \
                 assigned_at = NULL, \
                 open_to_employees = true, \
                 updated_at = NOW() \
             WHERE id = $1",
        )
        .bind(order_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Órdenes que pasaron su deadline de auto-asignación (24h en `awaiting_assignment`)
    pub async fn list_overdue_unassigned(pool: &PgPool) -> Result<Vec<Order>, sqlx::Error> {
        sqlx::query_as!(
            Order,
            r#"SELECT id, order_number, client_id, service_id, plan_id,
               payment_mode as "payment_mode: PaymentMode",
               base_price_cents, discount_percent, final_price_cents, currency,
               status as "status: OrderStatus",
               assigned_employee_id, assigned_at, auto_assign_deadline, current_phase,
               started_at, completed_at, cancelled_at, project_description, client_notes,
               internal_notes,
               created_at, updated_at, ai_intermediary_enabled, ai_summary, open_to_employees
             FROM orders WHERE status = 'awaiting_assignment'
             AND auto_assign_deadline IS NOT NULL AND auto_assign_deadline < NOW()
             ORDER BY auto_assign_deadline ASC"#,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn update_order_phase_definition(
        pool: &PgPool,
        phase_id: Uuid,
        title: Option<&str>,
        description: Option<&str>,
        price_cents: Option<i32>,
        estimated_days: Option<i32>,
        max_revisions: Option<i32>,
    ) -> Result<OrderPhase, sqlx::Error> {
        sqlx::query_as!(
            OrderPhase,
            r#"UPDATE order_phases
               SET title = COALESCE($2, title),
                   description = COALESCE($3, description),
                   price_cents = COALESCE($4, price_cents),
                   estimated_days = COALESCE($5, estimated_days),
                   max_revisions = COALESCE($6, max_revisions),
                   updated_at = NOW()
             WHERE id = $1
             RETURNING id, order_id, phase_number, title, description, price_cents,
               status as "status: PhaseStatus",
               max_revisions, revisions_used, estimated_days,
               started_at, delivered_at, approved_at, deadline, created_at, updated_at"#,
            phase_id,
            title,
            description,
            price_cents,
            estimated_days,
            max_revisions,
        )
        .fetch_one(pool)
        .await
    }

    /* [T-10] Activa/desactiva IA intermediaria para una orden */
    pub async fn toggle_ai_intermediary(
        pool: &PgPool,
        order_id: Uuid,
        enabled: bool,
    ) -> Result<Order, sqlx::Error> {
        sqlx::query_as::<_, Order>(
            "UPDATE orders SET ai_intermediary_enabled = $2, updated_at = NOW() \
             WHERE id = $1 \
             RETURNING id, order_number, client_id, service_id, plan_id, \
             payment_mode, base_price_cents, discount_percent, final_price_cents, currency, \
             status, assigned_employee_id, assigned_at, auto_assign_deadline, current_phase, \
               started_at, completed_at, cancelled_at, project_description, client_notes, internal_notes, \
             created_at, updated_at, ai_intermediary_enabled, ai_summary, open_to_employees",
        )
        .bind(order_id)
        .bind(enabled)
        .fetch_one(pool)
        .await
    }

    /* [T-10] Actualiza el resumen IA de una orden (reemplaza, no acumula) */
    pub async fn update_ai_summary(
        pool: &PgPool,
        order_id: Uuid,
        summary: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query_as::<_, Order>(
            "UPDATE orders SET ai_summary = $2, updated_at = NOW() WHERE id = $1 \
             RETURNING id, order_number, client_id, service_id, plan_id, \
             payment_mode, base_price_cents, discount_percent, final_price_cents, currency, \
             status, assigned_employee_id, assigned_at, auto_assign_deadline, current_phase, \
               started_at, completed_at, cancelled_at, project_description, client_notes, internal_notes, \
             created_at, updated_at, ai_intermediary_enabled, ai_summary, open_to_employees",
        )
        .bind(order_id)
        .bind(summary)
        .fetch_one(pool)
        .await?;
        Ok(())
    }

}
