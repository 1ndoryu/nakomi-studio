/* [044A-38 Fase 4] Repositorio de delegaciones y perfiles de empleados.
 * [044A-44] Migrado a query_as!/query_scalar! con verificación en compilación.
 * CRUD sobre order_delegations y employee_profiles.
 * Todas las queries usan prepared statements vía sqlx. */

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Delegation, DelegationStatus, EmployeeProfile};

pub struct DelegationRepository;

impl DelegationRepository {
    /* ============================================================
    DELEGACIONES
    ============================================================ */

    pub async fn create_delegation(
        pool: &PgPool,
        order_id: Uuid,
        from_employee_id: Uuid,
        reason: &str,
        delegation_type: &str,
    ) -> Result<Delegation, sqlx::Error> {
        sqlx::query_as!(
            Delegation,
            r#"INSERT INTO order_delegations (order_id, from_employee_id, reason, delegation_type)
             VALUES ($1, $2, $3, $4)
             RETURNING id, order_id, from_employee_id, to_employee_id, reason,
               delegation_type, status as "status: DelegationStatus",
               created_at, resolved_at"#,
            order_id,
            from_employee_id,
            reason,
            delegation_type,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Delegation>, sqlx::Error> {
        sqlx::query_as!(
            Delegation,
            r#"SELECT id, order_id, from_employee_id, to_employee_id, reason,
               delegation_type, status as "status: DelegationStatus",
               created_at, resolved_at
             FROM order_delegations WHERE id = $1"#,
            id,
        )
        .fetch_optional(pool)
        .await
    }

    /// Delegaciones donde este empleado puede responder (incoming abiertas o dirigidas a él)
    pub async fn list_incoming(
        pool: &PgPool,
        employee_id: Uuid,
    ) -> Result<Vec<Delegation>, sqlx::Error> {
        sqlx::query_as!(
            Delegation,
            r#"SELECT id, order_id, from_employee_id, to_employee_id, reason,
               delegation_type, status as "status: DelegationStatus",
               created_at, resolved_at
             FROM order_delegations
             WHERE (to_employee_id = $1 OR (to_employee_id IS NULL AND status = 'requested'))
             AND from_employee_id != $1
             ORDER BY created_at DESC"#,
            employee_id,
        )
        .fetch_all(pool)
        .await
    }

    /// Delegaciones que el empleado ha creado
    pub async fn list_outgoing(
        pool: &PgPool,
        employee_id: Uuid,
    ) -> Result<Vec<Delegation>, sqlx::Error> {
        sqlx::query_as!(
            Delegation,
            r#"SELECT id, order_id, from_employee_id, to_employee_id, reason,
               delegation_type, status as "status: DelegationStatus",
               created_at, resolved_at
             FROM order_delegations WHERE from_employee_id = $1 ORDER BY created_at DESC"#,
            employee_id,
        )
        .fetch_all(pool)
        .await
    }

    /// Todas las delegaciones (admin)
    pub async fn list_all(pool: &PgPool) -> Result<Vec<Delegation>, sqlx::Error> {
        sqlx::query_as!(
            Delegation,
            r#"SELECT id, order_id, from_employee_id, to_employee_id, reason,
               delegation_type, status as "status: DelegationStatus",
               created_at, resolved_at
             FROM order_delegations ORDER BY created_at DESC"#,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn accept_delegation(
        pool: &PgPool,
        delegation_id: Uuid,
        to_employee_id: Uuid,
    ) -> Result<Delegation, sqlx::Error> {
        sqlx::query_as!(
            Delegation,
            r#"UPDATE order_delegations SET to_employee_id = $2, status = 'accepted',
             resolved_at = NOW() WHERE id = $1
             RETURNING id, order_id, from_employee_id, to_employee_id, reason,
               delegation_type, status as "status: DelegationStatus",
               created_at, resolved_at"#,
            delegation_id,
            to_employee_id,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn reject_delegation(
        pool: &PgPool,
        delegation_id: Uuid,
    ) -> Result<Delegation, sqlx::Error> {
        sqlx::query_as!(
            Delegation,
            r#"UPDATE order_delegations SET status = 'rejected', resolved_at = NOW()
             WHERE id = $1
             RETURNING id, order_id, from_employee_id, to_employee_id, reason,
               delegation_type, status as "status: DelegationStatus",
               created_at, resolved_at"#,
            delegation_id,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn complete_delegation(
        pool: &PgPool,
        delegation_id: Uuid,
    ) -> Result<Delegation, sqlx::Error> {
        sqlx::query_as!(
            Delegation,
            r#"UPDATE order_delegations SET status = 'completed', resolved_at = NOW()
             WHERE id = $1
             RETURNING id, order_id, from_employee_id, to_employee_id, reason,
               delegation_type, status as "status: DelegationStatus",
               created_at, resolved_at"#,
            delegation_id,
        )
        .fetch_one(pool)
        .await
    }

    /* ============================================================
    PERFILES DE EMPLEADOS
    ============================================================ */

    /// Obtiene perfil de empleado (NUMERIC → f64 cast en SQL)
    pub async fn find_employee_profile(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Option<EmployeeProfile>, sqlx::Error> {
        sqlx::query_as!(
            EmployeeProfile,
            r#"SELECT user_id, specialties, availability, max_concurrent_orders,
             last_activity_at, total_completed_orders,
             CAST(average_rating AS DOUBLE PRECISION) AS average_rating
             FROM employee_profiles WHERE user_id = $1"#,
            user_id,
        )
        .fetch_optional(pool)
        .await
    }

    /// Crea perfil por defecto si no existe, actualiza `last_activity_at` si existe
    pub async fn ensure_employee_profile(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<EmployeeProfile, sqlx::Error> {
        sqlx::query_as!(
            EmployeeProfile,
            r#"INSERT INTO employee_profiles (user_id) VALUES ($1)
             ON CONFLICT (user_id) DO UPDATE SET last_activity_at = NOW()
             RETURNING user_id, specialties, availability, max_concurrent_orders,
             last_activity_at, total_completed_orders,
             CAST(average_rating AS DOUBLE PRECISION) AS average_rating"#,
            user_id,
        )
        .fetch_one(pool)
        .await
    }

    /// Cuenta órdenes activas (`in_progress`, `under_review`) del empleado
    pub async fn count_active_orders(pool: &PgPool, employee_id: Uuid) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!: i64" FROM orders
             WHERE assigned_employee_id = $1
             AND status IN ('in_progress', 'under_review')"#,
            employee_id,
        )
        .fetch_one(pool)
        .await
    }

    /// Lista empleados con perfil y conteo de órdenes activas (JOIN users + `employee_profiles`)
    pub async fn list_employees_with_stats(
        pool: &PgPool,
    ) -> Result<Vec<EmployeeListItemRow>, sqlx::Error> {
        sqlx::query_as!(
            EmployeeListItemRow,
            r#"SELECT u.id AS user_id, u.email,
             COALESCE(ep.specialties, ARRAY[]::TEXT[]) AS "specialties!: Vec<String>",
             COALESCE(ep.availability, 'available') AS "availability!: String",
             COALESCE(ep.max_concurrent_orders, 3) AS "max_concurrent_orders!: i32",
             COALESCE(ep.total_completed_orders, 0) AS "total_completed_orders!: i32",
             CAST(ep.average_rating AS DOUBLE PRECISION) AS average_rating,
             (SELECT COUNT(*) FROM orders o WHERE o.assigned_employee_id = u.id
              AND o.status IN ('in_progress', 'under_review')) AS "current_orders!: i64"
             FROM users u
             LEFT JOIN employee_profiles ep ON ep.user_id = u.id
             WHERE u.role = 'employee' OR u.role = 'admin'
             ORDER BY u.email"#,
        )
        .fetch_all(pool)
        .await
    }

    /// Empleados disponibles para auto-asignación: `availability` = 'available',
    /// bajo su máximo de órdenes concurrentes, especialidad coincide o es generalista
    pub async fn find_available_employees(
        pool: &PgPool,
        service_slug: &str,
    ) -> Result<Vec<EmployeeProfile>, sqlx::Error> {
        sqlx::query_as!(
            EmployeeProfile,
            r#"SELECT ep.user_id, ep.specialties, ep.availability, ep.max_concurrent_orders,
             ep.last_activity_at, ep.total_completed_orders,
             CAST(ep.average_rating AS DOUBLE PRECISION) AS average_rating
             FROM employee_profiles ep
             JOIN users u ON u.id = ep.user_id
             WHERE u.role = 'employee'
             AND ep.availability = 'available'
             AND (ep.specialties @> ARRAY[$1]::TEXT[] OR ep.specialties = '{}')
             AND (SELECT COUNT(*) FROM orders o
                  WHERE o.assigned_employee_id = ep.user_id
                  AND o.status IN ('in_progress', 'under_review')) < ep.max_concurrent_orders
             ORDER BY ep.total_completed_orders DESC, ep.last_activity_at DESC"#,
            service_slug,
        )
        .fetch_all(pool)
        .await
    }
}

/// Row type intermedio para `list_employees_with_stats` (JOIN users + `employee_profiles`)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EmployeeListItemRow {
    pub user_id: Uuid,
    pub email: String,
    pub specialties: Vec<String>,
    pub availability: String,
    pub max_concurrent_orders: i32,
    pub total_completed_orders: i32,
    pub average_rating: Option<f64>,
    pub current_orders: i64,
}
