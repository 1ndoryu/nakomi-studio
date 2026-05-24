use super::entities::HostingSubscription;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct HostingSubscriptionResponse {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub client_name: String,
    pub client_email: String,
    pub plan: String,
    pub domain: Option<String>,
    pub domain_verification_status: String,
    pub domain_verification_token: Option<String>,
    pub domain_verified_at: Option<DateTime<Utc>>,
    pub runtime_kind: Option<String>,
    pub deployment_id: Option<String>,
    pub coolify_site_name: Option<String>,
    pub status: String,
    pub monthly_price_cents: i32,
    pub storage_limit_mb: i32,
    /* [104A-42] IP y UUID de Coolify expuestos al frontend para datos reales del servidor */
    pub server_uuid: Option<String>,
    pub server_ip: Option<String>,
    /* [104A-18] Credenciales SFTP para acceso a archivos WordPress */
    pub sftp_user: Option<String>,
    pub sftp_password: Option<String>,
    pub sftp_port: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<HostingSubscription> for HostingSubscriptionResponse {
    fn from(s: HostingSubscription) -> Self {
        let deployment_id = s.server_uuid.clone();
        let runtime_kind = if deployment_id.is_some() || s.coolify_site_name.is_some() {
            Some("coolify".to_string())
        } else {
            None
        };

        Self {
            id: s.id,
            user_id: s.user_id,
            client_name: s.client_name,
            client_email: s.client_email,
            plan: s.plan,
            domain: s.domain,
            domain_verification_status: s.domain_verification_status,
            domain_verification_token: s.domain_verification_token,
            domain_verified_at: s.domain_verified_at,
            runtime_kind,
            deployment_id,
            coolify_site_name: s.coolify_site_name,
            status: s.status,
            monthly_price_cents: s.monthly_price_cents,
            storage_limit_mb: s.storage_limit_mb,
            server_uuid: s.server_uuid,
            server_ip: s.server_ip,
            sftp_user: s.sftp_user,
            sftp_password: s.sftp_password,
            sftp_port: s.sftp_port,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

/* [094A-3] Respuesta self-service: suscripción creada + URL de Stripe Checkout */
#[derive(Debug, Serialize, ToSchema)]
pub struct SelfSubscribeResponse {
    pub subscription: HostingSubscriptionResponse,
    pub checkout_url: String,
}

/* [094A-8] Estadísticas reales de una suscripción de hosting.
 * Uptime se calcula desde el historial de eventos (status_change).
 * [114A-15+] CPU y RAM reales obtenidos via docker stats SSH. */
#[derive(Debug, Serialize, ToSchema)]
pub struct HostingStatsResponse {
    pub storage_limit_mb: i32,
    /// null = monitoring no disponible aún
    pub storage_used_mb: Option<i64>,
    pub bandwidth_limit_gb: i32,
    /// null = monitoring no disponible aún
    pub bandwidth_used_gb: Option<f64>,
    pub bandwidth_remaining_gb: Option<f64>,
    pub bandwidth_reset_at: DateTime<Utc>,
    /// Calculado desde historial de eventos (tiempo en status "active")
    pub uptime_percent: f64,
    /// Timestamp desde que la suscripción está activa (null si nunca se activó)
    pub active_since: Option<DateTime<Utc>>,
    pub total_events: i64,
    pub last_event_at: Option<DateTime<Utc>>,
    /// true si hay agente de monitoreo configurado (`coolify_site_name` != null)
    pub monitoring_available: bool,
    /// [114A-15+] CPU % combinado de todos los contenedores (null si SSH no disponible)
    pub cpu_percent: Option<f64>,
    /// [114A-15+] RAM usada en MB combinada (null si SSH no disponible)
    pub ram_used_mb: Option<f64>,
    /// [114A-15+] RAM límite en MB combinada (null si SSH no disponible)
    pub ram_limit_mb: Option<f64>,
    /// [114A-15+] Stats por contenedor individual
    pub containers: Option<Vec<crate::services::docker_stats::ContainerStats>>,
}

/* [164A-19] Despliegues reales de infraestructura en el panel admin.
 * Expone el estado de Coolify enriquecido con el vínculo opcional a suscripciones
 * de hosting guardadas en la BD para detectar drift entre panel e infraestructura. */
/* [215A-14] Enriquecido con recursos reales por despliegue (CPU, RAM, disco)
 * y nombre del cliente dueño para mostrar en la tabla de infraestructura. */
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CoolifyDeploymentResponse {
    pub uuid: String,
    pub runtime_kind: String,
    pub deployment_id: String,
    pub name: String,
    pub status: String,
    pub fqdn: Option<String>,
    pub server_uuid: Option<String>,
    pub server_name: Option<String>,
    pub project_uuid: Option<String>,
    pub environment_name: Option<String>,
    pub linked_subscription_id: Option<Uuid>,
    pub linked_subscription_domain: Option<String>,
    pub linked_subscription_status: Option<String>,
    pub linked_subscription_plan: Option<String>,
    /// Etiqueta del servidor Coolify de origen.
    pub server_label: String,
    /// [215A-14] Nombre del cliente dueño de la suscripción vinculada
    pub linked_subscription_client: Option<String>,
    /// [215A-14] CPU % combinado de todos los contenedores del despliegue (null si no disponible)
    pub cpu_percent: Option<f64>,
    /// [215A-14] RAM usada en MB combinada (null si no disponible)
    pub ram_used_mb: Option<f64>,
    /// [215A-14] RAM límite en MB combinada (null si no disponible)
    pub ram_limit_mb: Option<f64>,
    /// [215A-14] Almacenamiento usado en MB (null si no disponible)
    pub storage_used_mb: Option<i64>,
    /// [215A-14] Límite de almacenamiento del plan en MB (null si no vinculado)
    pub storage_limit_mb: Option<i32>,
}
