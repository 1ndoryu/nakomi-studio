/* sentinel-disable-file sqlx-query-sin-macro sqlx-query-as-sin-macro: dominio nuevo en construcción.
 * [164A-17] Las queries siguen siendo prepared statements con bind, pero evitamos
 * sqlx::query! mientras el esquema VPS se introduce en el mismo ciclo. */
/* [164A-17] Repositorio de reventa VPS.
 * Runtime queries tipadas para no bloquear la tarea con prepare offline mientras crece el dominio.
 * Toda la lógica de estado y auditoría vive aquí para mantener handlers delgados. */

use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{RejectVpsRequest, VpsEvent, VpsPlanConfig, VpsSubscription};

pub struct VpsRepository;

pub struct CreateVpsSubscriptionParams<'a> {
    pub user_id: Option<Uuid>,
    pub client_name: &'a str,
    pub client_email: &'a str,
    pub tier_name: &'a str,
    pub requested_hostname: Option<&'a str>,
    pub client_notes: Option<&'a str>,
    pub monthly_price_cents: i32,
}

pub struct ProvisionedVpsInfo<'a> {
    pub contabo_instance_id: i64,
    pub provisioning_ip: Option<&'a str>,
    pub access_username: &'a str,
    pub requested_hostname: Option<&'a str>,
}

impl VpsRepository {
    pub async fn list_all(pool: &PgPool) -> Result<Vec<VpsSubscription>, AppError> {
        sqlx::query_as::<_, VpsSubscription>(
            r"SELECT id, user_id, client_name, client_email, tier_name, requested_hostname,
                      status, stripe_subscription_id, monthly_price_cents, contabo_instance_id,
                      provisioning_ip, access_username, approved_by, approved_at, provisioned_at,
                      rejected_reason, client_notes, created_at, updated_at
               FROM vps_subscriptions
               ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn list_by_user_id(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<VpsSubscription>, AppError> {
        sqlx::query_as::<_, VpsSubscription>(
            r"SELECT id, user_id, client_name, client_email, tier_name, requested_hostname,
                      status, stripe_subscription_id, monthly_price_cents, contabo_instance_id,
                      provisioning_ip, access_username, approved_by, approved_at, provisioned_at,
                      rejected_reason, client_notes, created_at, updated_at
               FROM vps_subscriptions
               WHERE user_id = $1
               ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<VpsSubscription>, AppError> {
        sqlx::query_as::<_, VpsSubscription>(
            r"SELECT id, user_id, client_name, client_email, tier_name, requested_hostname,
                      status, stripe_subscription_id, monthly_price_cents, contabo_instance_id,
                      provisioning_ip, access_username, approved_by, approved_at, provisioned_at,
                      rejected_reason, client_notes, created_at, updated_at
               FROM vps_subscriptions
               WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn create(
        pool: &PgPool,
        params: CreateVpsSubscriptionParams<'_>,
    ) -> Result<VpsSubscription, AppError> {
        sqlx::query_as::<_, VpsSubscription>(
            r"INSERT INTO vps_subscriptions
                    (user_id, client_name, client_email, tier_name, requested_hostname, client_notes, monthly_price_cents)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING id, user_id, client_name, client_email, tier_name, requested_hostname,
                         status, stripe_subscription_id, monthly_price_cents, contabo_instance_id,
                         provisioning_ip, access_username, approved_by, approved_at, provisioned_at,
                         rejected_reason, client_notes, created_at, updated_at",
        )
        .bind(params.user_id)
        .bind(params.client_name)
        .bind(params.client_email)
        .bind(params.tier_name)
        .bind(params.requested_hostname)
        .bind(params.client_notes)
        .bind(params.monthly_price_cents)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn update_status(pool: &PgPool, id: Uuid, status: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE vps_subscriptions SET status = $1, updated_at = NOW() WHERE id = $2")
            .bind(status)
            .bind(id)
            .execute(pool)
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    pub async fn add_event(
        pool: &PgPool,
        subscription_id: Uuid,
        event_type: &str,
        details: Option<serde_json::Value>,
    ) -> Result<VpsEvent, AppError> {
        sqlx::query_as::<_, VpsEvent>(
            r"INSERT INTO vps_events (subscription_id, event_type, details)
               VALUES ($1, $2, $3)
               RETURNING id, subscription_id, event_type, details, created_at",
        )
        .bind(subscription_id)
        .bind(event_type)
        .bind(details)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn list_events(
        pool: &PgPool,
        subscription_id: Uuid,
        limit: i64,
    ) -> Result<Vec<VpsEvent>, AppError> {
        sqlx::query_as::<_, VpsEvent>(
            r"SELECT id, subscription_id, event_type, details, created_at
               FROM vps_events
               WHERE subscription_id = $1
               ORDER BY created_at DESC
               LIMIT $2",
        )
        .bind(subscription_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn get_plan_config(
        pool: &PgPool,
        tier_name: &str,
    ) -> Result<Option<VpsPlanConfig>, AppError> {
        sqlx::query_as::<_, VpsPlanConfig>(
            r"SELECT id, tier_name, display_name, description, contabo_product_id,
                      base_cost_cents, monthly_price_cents, cpu_cores, ram_mb, disk_mb,
                      region, is_active, approval_required, created_at, updated_at
               FROM vps_plan_configs
               WHERE tier_name = $1",
        )
        .bind(tier_name)
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn list_plan_configs(pool: &PgPool) -> Result<Vec<VpsPlanConfig>, AppError> {
        sqlx::query_as::<_, VpsPlanConfig>(
            r"SELECT id, tier_name, display_name, description, contabo_product_id,
                      base_cost_cents, monthly_price_cents, cpu_cores, ram_mb, disk_mb,
                      region, is_active, approval_required, created_at, updated_at
               FROM vps_plan_configs
               WHERE is_active = TRUE
               ORDER BY monthly_price_cents ASC",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn set_stripe_subscription_id(
        pool: &PgPool,
        id: Uuid,
        stripe_subscription_id: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE vps_subscriptions SET stripe_subscription_id = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(stripe_subscription_id)
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }

    pub async fn find_by_stripe_subscription(
        pool: &PgPool,
        stripe_subscription_id: &str,
    ) -> Result<Option<VpsSubscription>, AppError> {
        sqlx::query_as::<_, VpsSubscription>(
            r"SELECT id, user_id, client_name, client_email, tier_name, requested_hostname,
                      status, stripe_subscription_id, monthly_price_cents, contabo_instance_id,
                      provisioning_ip, access_username, approved_by, approved_at, provisioned_at,
                      rejected_reason, client_notes, created_at, updated_at
               FROM vps_subscriptions
               WHERE stripe_subscription_id = $1",
        )
        .bind(stripe_subscription_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn mark_approved(pool: &PgPool, id: Uuid, admin_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r"UPDATE vps_subscriptions
               SET status = 'provisioning', approved_by = $1, approved_at = NOW(), updated_at = NOW()
               WHERE id = $2",
        )
        .bind(admin_id)
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }

    pub async fn mark_rejected(
        pool: &PgPool,
        id: Uuid,
        req: &RejectVpsRequest,
    ) -> Result<(), AppError> {
        sqlx::query(
            r"UPDATE vps_subscriptions
               SET status = 'rejected', rejected_reason = $1, updated_at = NOW()
               WHERE id = $2",
        )
        .bind(&req.reason)
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }

    pub async fn mark_provisioned(
        pool: &PgPool,
        id: Uuid,
        info: &ProvisionedVpsInfo<'_>,
    ) -> Result<(), AppError> {
        sqlx::query(
            r"UPDATE vps_subscriptions
               SET status = 'active', contabo_instance_id = $1, provisioning_ip = $2,
                   access_username = $3, requested_hostname = COALESCE($4, requested_hostname),
                   provisioned_at = NOW(), updated_at = NOW()
               WHERE id = $5",
        )
        .bind(info.contabo_instance_id)
        .bind(info.provisioning_ip)
        .bind(info.access_username)
        .bind(info.requested_hostname)
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }
}
