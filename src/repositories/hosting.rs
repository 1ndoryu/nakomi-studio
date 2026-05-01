/* [054A-2] Repositorio de hosting: CRUD para suscripciones y eventos.
 * [114A-3] Plan configs: CRUD para configuración de recursos por plan.
 * Queries verificadas con sqlx offline. */

use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{HostingEvent, HostingPlanConfig, HostingSubscription, UpdatePlanConfigRequest};

/* [164A-6] Struct para agrupar datos del servidor tras provisioning.
 * Evita pasar 8 argumentos sueltos a update_server_info (clippy::too_many_arguments). */
pub struct ServerInfo<'a> {
    pub coolify_site_name: &'a str,
    pub server_uuid: &'a str,
    pub server_ip: &'a str,
    pub sftp_user: &'a str,
    pub sftp_password: &'a str,
    pub sftp_port: i32,
}

pub struct HostingRepository;

/* [054A-2] Parámetros agrupados para crear suscripción (evita clippy::too_many_arguments) */
pub struct CreateHostingParams<'a> {
    pub user_id: Option<Uuid>,
    pub client_name: &'a str,
    pub client_email: &'a str,
    pub plan: &'a str,
    pub domain: Option<&'a str>,
    pub monthly_price_cents: i32,
    pub storage_limit_mb: i32,
}

impl HostingRepository {
    pub async fn list_all(pool: &PgPool) -> Result<Vec<HostingSubscription>, AppError> {
        let rows = sqlx::query_as!(
            HostingSubscription,
            "SELECT id, user_id, client_name, client_email, plan, domain,
                    coolify_site_name, status, stripe_subscription_id,
                    monthly_price_cents, storage_limit_mb,
                    server_uuid, server_ip, sftp_user, sftp_password, sftp_port, created_at, updated_at
             FROM hosting_subscriptions
             ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    /* [T-9] Listar suscripciones de hosting de un usuario específico */
    pub async fn list_by_user_id(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<HostingSubscription>, AppError> {
        let rows = sqlx::query_as!(
            HostingSubscription,
            "SELECT id, user_id, client_name, client_email, plan, domain,
                    coolify_site_name, status, stripe_subscription_id,
                    monthly_price_cents, storage_limit_mb,
                    server_uuid, server_ip, sftp_user, sftp_password, sftp_port, created_at, updated_at
             FROM hosting_subscriptions
             WHERE user_id = $1
             ORDER BY created_at DESC",
            user_id
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn find_by_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<HostingSubscription>, AppError> {
        let row = sqlx::query_as!(
            HostingSubscription,
            "SELECT id, user_id, client_name, client_email, plan, domain,
                    coolify_site_name, status, stripe_subscription_id,
                    monthly_price_cents, storage_limit_mb,
                    server_uuid, server_ip, sftp_user, sftp_password, sftp_port, created_at, updated_at
             FROM hosting_subscriptions
             WHERE id = $1",
            id
        )
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }

    pub async fn create(
        pool: &PgPool,
        params: CreateHostingParams<'_>,
    ) -> Result<HostingSubscription, AppError> {
        let row = sqlx::query_as!(
            HostingSubscription,
            "INSERT INTO hosting_subscriptions (user_id, client_name, client_email, plan, domain, monthly_price_cents, storage_limit_mb)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, user_id, client_name, client_email, plan, domain,
                       coolify_site_name, status, stripe_subscription_id,
                       monthly_price_cents, storage_limit_mb,
                       server_uuid, server_ip, sftp_user, sftp_password, sftp_port, created_at, updated_at",
            params.user_id,
            params.client_name,
            params.client_email,
            params.plan,
            params.domain,
            params.monthly_price_cents,
            params.storage_limit_mb
        )
        .fetch_one(pool)
        .await?;
        Ok(row)
    }

    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: &str,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE hosting_subscriptions SET status = $1, updated_at = NOW() WHERE id = $2",
            status,
            id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /* [074A-65] Actualizar campos editables de una suscripción (plan, dominio, precio, almacenamiento) */
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        plan: &str,
        domain: Option<&str>,
        monthly_price_cents: i32,
        storage_limit_mb: i32,
    ) -> Result<HostingSubscription, AppError> {
        let row = sqlx::query_as!(
            HostingSubscription,
            "UPDATE hosting_subscriptions
             SET plan = $1, domain = $2, monthly_price_cents = $3, storage_limit_mb = $4, updated_at = NOW()
             WHERE id = $5
             RETURNING id, user_id, client_name, client_email, plan, domain,
                       coolify_site_name, status, stripe_subscription_id,
                       monthly_price_cents, storage_limit_mb,
                       server_uuid, server_ip, sftp_user, sftp_password, sftp_port, created_at, updated_at",
            plan,
            domain,
            monthly_price_cents,
            storage_limit_mb,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(row)
    }

    /* [074A-65] Eliminar suscripción de hosting */
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM hosting_subscriptions WHERE id = $1",
            id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn add_event(
        pool: &PgPool,
        subscription_id: Uuid,
        event_type: &str,
        details: Option<serde_json::Value>,
    ) -> Result<HostingEvent, AppError> {
        let row = sqlx::query_as!(
            HostingEvent,
            "INSERT INTO hosting_events (subscription_id, event_type, details)
             VALUES ($1, $2, $3)
             RETURNING id, subscription_id, event_type, details, created_at",
            subscription_id,
            event_type,
            details
        )
        .fetch_one(pool)
        .await?;
        Ok(row)
    }

    pub async fn list_events(
        pool: &PgPool,
        subscription_id: Uuid,
        limit: i64,
    ) -> Result<Vec<HostingEvent>, AppError> {
        let rows = sqlx::query_as!(
            HostingEvent,
            "SELECT id, subscription_id, event_type, details, created_at
             FROM hosting_events
             WHERE subscription_id = $1
             ORDER BY created_at DESC
             LIMIT $2",
            subscription_id,
            limit
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    /* [084A-24] Buscar suscripción por stripe_subscription_id (para webhooks) */
    pub async fn find_by_stripe_subscription(
        pool: &PgPool,
        stripe_sub_id: &str,
    ) -> Result<Option<HostingSubscription>, AppError> {
        let row = sqlx::query_as!(
            HostingSubscription,
            "SELECT id, user_id, client_name, client_email, plan, domain,
                    coolify_site_name, status, stripe_subscription_id,
                    monthly_price_cents, storage_limit_mb,
                    server_uuid, server_ip, sftp_user, sftp_password, sftp_port, created_at, updated_at
             FROM hosting_subscriptions
             WHERE stripe_subscription_id = $1",
            stripe_sub_id
        )
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }

    /* [304A-3] Asignar (o desasignar) una suscripción de hosting a un usuario.
     * Admin only. Permite vincular hostings creados manualmente a cuentas de clientes. */
    pub async fn assign_user(
        pool: &PgPool,
        id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<HostingSubscription, AppError> {
        let row = sqlx::query_as!(
            HostingSubscription,
            "UPDATE hosting_subscriptions
             SET user_id = $1, updated_at = NOW()
             WHERE id = $2
             RETURNING id, user_id, client_name, client_email, plan, domain,
                       coolify_site_name, status, stripe_subscription_id,
                       monthly_price_cents, storage_limit_mb,
                       server_uuid, server_ip, sftp_user, sftp_password, sftp_port, created_at, updated_at",
            user_id,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(row)
    }

    /* [084A-24] Asignar stripe_subscription_id a una suscripción de hosting */
    pub async fn set_stripe_subscription_id(
        pool: &PgPool,
        id: Uuid,
        stripe_sub_id: &str,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE hosting_subscriptions SET stripe_subscription_id = $1, updated_at = NOW() WHERE id = $2",
            stripe_sub_id,
            id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /* [104A-42] Guardar datos del servidor Coolify tras provisioning exitoso.
     * [104A-18] También guarda credenciales SFTP generadas al provisionar. */
    pub async fn update_server_info(
        pool: &PgPool,
        id: Uuid,
        info: &ServerInfo<'_>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE hosting_subscriptions
             SET coolify_site_name = $1, server_uuid = $2, server_ip = $3,
                 sftp_user = $4, sftp_password = $5, sftp_port = $6, updated_at = NOW()
             WHERE id = $7",
            info.coolify_site_name,
            info.server_uuid,
            info.server_ip,
            info.sftp_user,
            info.sftp_password,
            info.sftp_port,
            id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /* [164A-16] Genera un puerto SFTP único en rango 10000-49151 (evita efímeros 49152-65535).
     * Verifica unicidad en BD antes de retornar. Reintenta hasta 10 veces.
     * Requiere UNIQUE constraint en sftp_port (migration 20260417000000). */
    pub async fn find_available_sftp_port(pool: &PgPool) -> Result<i32, AppError> {
        for _ in 0..10 {
            let port: i32 = rand::thread_rng().gen_range(10000..49152);
            let taken = sqlx::query_scalar!(
                "SELECT EXISTS(SELECT 1 FROM hosting_subscriptions WHERE sftp_port = $1) AS \"exists!\"",
                port
            )
            .fetch_one(pool)
            .await?;
            if !taken {
                return Ok(port);
            }
        }
        Err(AppError::Internal(
            "No se pudo encontrar un puerto SFTP disponible tras 10 intentos".into(),
        ))
    }

    /* [114A-1] Rotación de credenciales SFTP: actualiza contraseña y timestamp.
     * No toca el usuario ni el puerto — solo la contraseña.
     * El caller debe también actualizar el compose en Coolify para que surta efecto. */
    pub async fn update_sftp_password(
        pool: &PgPool,
        id: Uuid,
        new_password: &str,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE hosting_subscriptions
             SET sftp_password = $1, sftp_credentials_rotated_at = NOW(), updated_at = NOW()
             WHERE id = $2",
            new_password,
            id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /* [114A-3] Obtener configuración de recursos para un plan específico */
    pub async fn get_plan_config(
        pool: &PgPool,
        plan_name: &str,
    ) -> Result<Option<HostingPlanConfig>, AppError> {
        let row = sqlx::query_as!(
            HostingPlanConfig,
            "SELECT id, plan_name, monthly_price_cents,
                    wp_cpu_millicores, wp_memory_mb, db_cpu_millicores, db_memory_mb,
                    ssh_cpu_millicores, ssh_memory_mb, storage_limit_mb, bandwidth_limit_gb,
                    created_at, updated_at
             FROM hosting_plan_configs
             WHERE plan_name = $1",
            plan_name
        )
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }

    /* [114A-3] Listar todas las configuraciones de planes */
    pub async fn list_plan_configs(
        pool: &PgPool,
    ) -> Result<Vec<HostingPlanConfig>, AppError> {
        let rows = sqlx::query_as!(
            HostingPlanConfig,
            "SELECT id, plan_name, monthly_price_cents,
                    wp_cpu_millicores, wp_memory_mb, db_cpu_millicores, db_memory_mb,
                    ssh_cpu_millicores, ssh_memory_mb, storage_limit_mb, bandwidth_limit_gb,
                    created_at, updated_at
             FROM hosting_plan_configs
             ORDER BY monthly_price_cents ASC"
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    /* [114A-3] Actualizar configuración de un plan (PATCH semántico: solo campos presentes).
     * Usa COALESCE para mantener valores existentes cuando el campo es NULL en el request. */
    pub async fn update_plan_config(
        pool: &PgPool,
        plan_name: &str,
        req: &UpdatePlanConfigRequest,
    ) -> Result<HostingPlanConfig, AppError> {
        let row = sqlx::query_as!(
            HostingPlanConfig,
            "UPDATE hosting_plan_configs SET
                monthly_price_cents = COALESCE($1, monthly_price_cents),
                wp_cpu_millicores = COALESCE($2, wp_cpu_millicores),
                wp_memory_mb = COALESCE($3, wp_memory_mb),
                db_cpu_millicores = COALESCE($4, db_cpu_millicores),
                db_memory_mb = COALESCE($5, db_memory_mb),
                ssh_cpu_millicores = COALESCE($6, ssh_cpu_millicores),
                ssh_memory_mb = COALESCE($7, ssh_memory_mb),
                storage_limit_mb = COALESCE($8, storage_limit_mb),
                bandwidth_limit_gb = COALESCE($9, bandwidth_limit_gb),
                updated_at = NOW()
             WHERE plan_name = $10
             RETURNING id, plan_name, monthly_price_cents,
                       wp_cpu_millicores, wp_memory_mb, db_cpu_millicores, db_memory_mb,
                       ssh_cpu_millicores, ssh_memory_mb, storage_limit_mb, bandwidth_limit_gb,
                       created_at, updated_at",
            req.monthly_price_cents,
            req.wp_cpu_millicores,
            req.wp_memory_mb,
            req.db_cpu_millicores,
            req.db_memory_mb,
            req.ssh_cpu_millicores,
            req.ssh_memory_mb,
            req.storage_limit_mb,
            req.bandwidth_limit_gb,
            plan_name
        )
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound(format!("Plan '{plan_name}' no encontrado")),
            other => AppError::from(other),
        })?;
        Ok(row)
    }
}
