use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct HostingSubscription {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub client_name: String,
    pub client_email: String,
    pub plan: String,
    pub domain: Option<String>,
    #[sqlx(default)]
    pub domain_verification_status: String,
    #[sqlx(default)]
    pub domain_verification_token: Option<String>,
    #[sqlx(default)]
    pub domain_verified_at: Option<DateTime<Utc>>,
    /* [245A-6] Identidad del runtime persistida para no depender del fallback
     * a columnas legacy al operar sobre despliegues existentes. */
    pub runtime_kind: String,
    pub deployment_id: Option<String>,
    pub coolify_site_name: Option<String>,
    pub status: String,
    pub stripe_subscription_id: Option<String>,
    pub monthly_price_cents: i32,
    pub storage_limit_mb: i32,
    /* [104A-42] Campos de servidor Coolify: UUID del servicio y IP del VPS */
    #[sqlx(default)]
    pub server_uuid: Option<String>,
    #[sqlx(default)]
    pub server_ip: Option<String>,
    /* [104A-18] Credenciales SFTP generadas al provisionar (contenedor atmoz/sftp) */
    #[sqlx(default)]
    pub sftp_user: Option<String>,
    #[sqlx(default)]
    pub sftp_password: Option<String>,
    /* [104A-18] Puerto SFTP único por hosting (range 10000-65000), mapeado en compose */
    #[sqlx(default)]
    pub sftp_port: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl HostingSubscription {
    #[must_use]
    pub fn deployment_id_or_legacy(&self) -> Option<&str> {
        self.deployment_id.as_deref().or(self.server_uuid.as_deref())
    }

    #[must_use]
    pub fn is_coolify_runtime(&self) -> bool {
        let runtime_kind = self.runtime_kind.trim();
        runtime_kind.is_empty() || runtime_kind.eq_ignore_ascii_case("coolify")
    }
}

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct HostingEvent {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub event_type: String,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/* [114A-3] Configuración de recursos por plan de hosting.
 * Centraliza precios y límites de CPU/RAM/storage/bandwidth.
 * Millicores: 1000 = 1.0 CPU. Admin modifica vía API; compose los usa al provisionar. */
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct HostingPlanConfig {
    pub id: Uuid,
    pub plan_name: String,
    pub monthly_price_cents: i32,
    pub wp_cpu_millicores: i32,
    pub wp_memory_mb: i32,
    pub db_cpu_millicores: i32,
    pub db_memory_mb: i32,
    pub ssh_cpu_millicores: i32,
    pub ssh_memory_mb: i32,
    pub storage_limit_mb: i32,
    pub bandwidth_limit_gb: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PublicHostingPlan {
    pub plan_name: String,
    pub label: String,
    pub description: String,
    pub monthly_price_cents: i32,
    pub wp_cpu_millicores: i32,
    pub wp_memory_mb: i32,
    pub db_cpu_millicores: i32,
    pub db_memory_mb: i32,
    pub ssh_cpu_millicores: i32,
    pub ssh_memory_mb: i32,
    pub storage_limit_mb: i32,
    pub bandwidth_limit_gb: i32,
    pub features: Vec<String>,
    pub recommended: bool,
}
