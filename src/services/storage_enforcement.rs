/* [195A-1] Enforcement de límites de almacenamiento para hosting.
 * Ejecuta cada 6h: mide uso real via SSH+du y, si supera storage_limit_mb,
 * detiene el contenedor SSH (bloquea subidas) mientras el sitio web sigue activo.
 * Cuando el cliente libera espacio, el contenedor SSH se reinicia automáticamente.
 *
 * Estrategia:
 * - Enforcement: docker stop {site}-ssh-1 → evento storage_limit_exceeded → notificación
 * - Restore: docker start {site}-ssh-1  → evento storage_limit_restored → notificación
 * - Idempotencia: se revisa el último evento de storage antes de actuar.
 * - No bloquea contenedor web → sitio sigue accesible para visitantes.
 * - Si fetch_storage_usage falla (contenedor no running, SSH error), se salta sin bloquear. */

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    CreateNotification, NOTIF_HOSTING_STORAGE_EXCEEDED, NOTIF_HOSTING_STORAGE_RESTORED,
};
use crate::repositories::{HostingRepository, NotificationRepository};
use crate::services::{
    coolify::CoolifyConfig,
    docker_stats::{fetch_storage_usage, start_ssh_container, stop_ssh_container},
};

const EVENT_EXCEEDED: &str = "storage_limit_exceeded";
const EVENT_RESTORED: &str = "storage_limit_restored";
const CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_hours(6);

struct EnforcementTarget<'a> {
    hosting_id: Uuid,
    user_id: Uuid,
    plan: &'a str,
    site_name: &'a str,
    service_uuid: Option<&'a str>,
    server_ip: &'a str,
    storage_limit_mb: i32,
}

/* Determina el último estado de enforcement para este hosting.
 * Retorna Some(true) si el último evento de storage es "exceeded",
 * Some(false) si es "restored", None si nunca hubo evento de storage. */
async fn last_storage_enforcement_state(pool: &PgPool, id: Uuid) -> Option<bool> {
    let events = HostingRepository::list_events(pool, id, 50).await.ok()?;
    events
        .iter()
        .find(|e| e.event_type == EVENT_EXCEEDED || e.event_type == EVENT_RESTORED)
        .map(|e| e.event_type == EVENT_EXCEEDED)
}

async fn notify_user(
    pool: &PgPool,
    user_id: Uuid,
    hosting_id: Uuid,
    plan: &str,
    used_mb: i64,
    limit_mb: i32,
    exceeded: bool,
) {
    let (notif_type, title, body) = if exceeded {
        (
            NOTIF_HOSTING_STORAGE_EXCEEDED,
            "Límite de almacenamiento superado",
            format!(
                "Tu plan {plan} tiene {limit_mb} MB de almacenamiento. \
                 Estás usando {used_mb} MB. El acceso SSH/SFTP ha sido bloqueado \
                 temporalmente. Libera espacio para restaurarlo automáticamente."
            ),
        )
    } else {
        (
            NOTIF_HOSTING_STORAGE_RESTORED,
            "Acceso SSH/SFTP restaurado",
            format!(
                "Tu almacenamiento bajó a {used_mb} MB (límite: {limit_mb} MB). \
                 El acceso SSH/SFTP ha sido restaurado."
            ),
        )
    };

    let notif = CreateNotification {
        user_id,
        notification_type: notif_type.to_string(),
        title: title.to_string(),
        body: Some(body),
        link: Some("/panel/hosting".to_string()),
        reference_type: Some("hosting".to_string()),
        reference_id: Some(hosting_id),
    };

    if let Err(e) = NotificationRepository::create(pool, &notif).await {
        tracing::warn!("[storage-enforcement] Error creando notificación para {hosting_id}: {e}");
    }
}

async fn handle_storage_exceeded(
    pool: &PgPool,
    target: &EnforcementTarget<'_>,
    ssh_key: &str,
    used_mb: i64,
) {
    tracing::info!(
        "[storage-enforcement] {}: {used_mb}MB > {}MB — bloqueando SSH",
        target.site_name,
        target.storage_limit_mb
    );
    if let Err(e) = stop_ssh_container(target.server_ip, ssh_key, target.site_name).await {
        tracing::warn!(
            "[storage-enforcement] No se pudo detener SSH de {}: {e}",
            target.site_name
        );
    }
    if let Err(e) = HostingRepository::add_event(
        pool,
        target.hosting_id,
        EVENT_EXCEEDED,
        Some(serde_json::json!({
            "used_mb": used_mb,
            "limit_mb": target.storage_limit_mb,
        })),
    )
    .await
    {
        tracing::warn!(
            "[storage-enforcement] Error registrando evento exceeded para {}: {e}",
            target.hosting_id
        );
    }
    notify_user(
        pool,
        target.user_id,
        target.hosting_id,
        target.plan,
        used_mb,
        target.storage_limit_mb,
        true,
    )
    .await;
}

async fn handle_storage_restored(
    pool: &PgPool,
    target: &EnforcementTarget<'_>,
    ssh_key: &str,
    used_mb: i64,
) {
    tracing::info!(
        "[storage-enforcement] {}: {used_mb}MB <= {}MB — restaurando SSH",
        target.site_name,
        target.storage_limit_mb
    );
    if let Err(e) = start_ssh_container(target.server_ip, ssh_key, target.site_name).await {
        tracing::warn!(
            "[storage-enforcement] No se pudo iniciar SSH de {}: {e}",
            target.site_name
        );
    }
    if let Err(e) = HostingRepository::add_event(
        pool,
        target.hosting_id,
        EVENT_RESTORED,
        Some(serde_json::json!({
            "used_mb": used_mb,
            "limit_mb": target.storage_limit_mb,
        })),
    )
    .await
    {
        tracing::warn!(
            "[storage-enforcement] Error registrando evento restored para {}: {e}",
            target.hosting_id
        );
    }
    notify_user(
        pool,
        target.user_id,
        target.hosting_id,
        target.plan,
        used_mb,
        target.storage_limit_mb,
        false,
    )
    .await;
}

async fn check_and_enforce_one(
    pool: &PgPool,
    coolify_config: &CoolifyConfig,
    target: EnforcementTarget<'_>,
) {
    let Some(ssh_key) = &coolify_config.ssh_key_path else {
        tracing::warn!(
            "[storage-enforcement] Sin SSH key configurada, omitiendo {}",
            target.hosting_id
        );
        return;
    };

    let used_mb = match fetch_storage_usage(
        target.server_ip,
        ssh_key,
        target.site_name,
        target.service_uuid,
        target.plan,
    )
    .await
    {
        Ok(mb) => mb,
        Err(e) => {
            tracing::warn!(
                "[storage-enforcement] No se pudo medir almacenamiento de {}@{}: {e}",
                target.site_name,
                target.server_ip
            );
            return;
        }
    };

    let over_limit = used_mb > i64::from(target.storage_limit_mb);
    let last_state = last_storage_enforcement_state(pool, target.hosting_id).await;

    match (over_limit, last_state) {
        /* Nuevo exceso: bloquear SSH */
        (true, None | Some(false)) => {
            handle_storage_exceeded(pool, &target, ssh_key, used_mb).await;
        }

        /* Recuperación: cliente liberó espacio, restaurar SSH */
        (false, Some(true)) => handle_storage_restored(pool, &target, ssh_key, used_mb).await,

        /* Ya bloqueado y sigue excedido, o dentro de límite y sin bloqueo → sin acción */
        _ => {
            tracing::debug!(
                "[storage-enforcement] {}: {used_mb}MB, límite {}MB — sin cambio",
                target.site_name,
                target.storage_limit_mb
            );
        }
    }
}

pub async fn run_storage_check(pool: &PgPool, coolify_config: &CoolifyConfig) {
    let all = match HostingRepository::list_all(pool).await {
        Ok(list) => list,
        Err(e) => {
            tracing::warn!("[storage-enforcement] Error listando hostings: {e}");
            return;
        }
    };

    let candidates: Vec<_> = all
        .into_iter()
        .filter(|h| {
            h.status == "active"
                && h.server_ip.is_some()
                && h.coolify_site_name.is_some()
                && h.user_id.is_some()
        })
        .collect();

    tracing::info!(
        "[storage-enforcement] Revisando {} hostings activos con servidor",
        candidates.len()
    );

    for h in candidates {
        let (Some(site_name), Some(server_ip), Some(user_id)) = (
            h.coolify_site_name.as_deref(),
            h.server_ip.as_deref(),
            h.user_id,
        ) else {
            continue;
        };
        let service_uuid = h.server_uuid.as_deref();

        check_and_enforce_one(
            pool,
            coolify_config,
            EnforcementTarget {
                hosting_id: h.id,
                user_id,
                plan: &h.plan,
                site_name,
                service_uuid,
                server_ip,
                storage_limit_mb: h.storage_limit_mb,
            },
        )
        .await;
    }
}

pub async fn storage_enforcement_loop(pool: PgPool, coolify_config: CoolifyConfig) {
    loop {
        tokio::time::sleep(CHECK_INTERVAL).await;
        run_storage_check(&pool, &coolify_config).await;
    }
}
