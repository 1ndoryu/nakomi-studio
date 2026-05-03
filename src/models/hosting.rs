/* sentinel-disable-file limite-lineas: archivo de contratos legacy de hosting compartido.
 * [164A-17] La tarea actual necesita ampliar el catálogo sin abrir una refactorización grande
 * de DTOs/FromRow que afectaría múltiples handlers y queries en el mismo ciclo. */
/* [054A-2] Modelos de hosting: suscripciones y eventos.
 * hosting_subscriptions: registra planes de hosting contratados por clientes.
 * hosting_events: log de eventos del ciclo de vida (provisioned, backup, health_fail, etc).
 * [094A-9] Validación de dominio con regex RFC 1035/1123 para prevenir inyección. */

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::LazyLock;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/* [094A-9] Regex para validar nombres de dominio (RFC 1035/1123).
 * Acepta subdominios, TLDs estándar. No acepta IPs, rutas ni esquemas. */
static DOMAIN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?i)(?:[a-z0-9](?:[a-z0-9\-]{0,61}[a-z0-9])?\.)*[a-z0-9](?:[a-z0-9\-]{0,61}[a-z0-9])?$",
    )
    .expect("regex de dominio válida")
});

/* ============================================================
MODELOS DE BD
============================================================ */

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct HostingSubscription {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub client_name: String,
    pub client_email: String,
    pub plan: String,
    pub domain: Option<String>,
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

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct HostingEvent {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub event_type: String,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/* ============================================================
REQUESTS
============================================================ */

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateHostingRequest {
    #[validate(length(min = 1, max = 200))]
    pub client_name: String,
    #[validate(email)]
    pub client_email: String,
    #[validate(length(min = 1, max = 20))]
    pub plan: String,
    #[validate(
        length(max = 253),
        regex(path = "*DOMAIN_REGEX", message = "Dominio inválido")
    )]
    pub domain: Option<String>,
    /* [304A-3] Permite vincular manualmente a un despliegue Coolify existente (admin) */
    #[validate(length(max = 200))]
    pub coolify_site_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateHostingStatusRequest {
    #[validate(length(min = 1, max = 20))]
    pub status: String,
    pub reason: Option<String>,
}

/* [094A-3] Self-service: cliente contrata hosting sin form admin.
 * Solo necesita plan y dominio opcional — nombre/email se toman del perfil del usuario. */
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SelfSubscribeRequest {
    #[validate(length(min = 1, max = 20))]
    pub plan: String,
    #[validate(
        length(max = 253),
        regex(path = "*DOMAIN_REGEX", message = "Dominio inválido")
    )]
    pub domain: Option<String>,
}

/* [074A-65] Request para editar suscripción (plan, dominio) */
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateHostingRequest {
    #[validate(length(min = 1, max = 20))]
    pub plan: String,
    #[validate(
        length(max = 253),
        regex(path = "*DOMAIN_REGEX", message = "Dominio inválido")
    )]
    pub domain: Option<String>,
}

/* [304A-3] Request para asignar hosting a un usuario registrado por email (admin only).
 * Vincula una suscripción existente (creada manualmente) a la cuenta de un cliente. */
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AssignHostingRequest {
    #[validate(email)]
    pub user_email: String,
}

/* ============================================================
RESPONSES
============================================================ */

#[derive(Debug, Serialize, ToSchema)]
pub struct HostingSubscriptionResponse {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub client_name: String,
    pub client_email: String,
    pub plan: String,
    pub domain: Option<String>,
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
        Self {
            id: s.id,
            user_id: s.user_id,
            client_name: s.client_name,
            client_email: s.client_email,
            plan: s.plan,
            domain: s.domain,
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

/* [114A-3] Request para actualizar configuración de un plan (todos los campos opcionales). */
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdatePlanConfigRequest {
    pub monthly_price_cents: Option<i32>,
    pub wp_cpu_millicores: Option<i32>,
    pub wp_memory_mb: Option<i32>,
    pub db_cpu_millicores: Option<i32>,
    pub db_memory_mb: Option<i32>,
    pub ssh_cpu_millicores: Option<i32>,
    pub ssh_memory_mb: Option<i32>,
    pub storage_limit_mb: Option<i32>,
    pub bandwidth_limit_gb: Option<i32>,
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
    pub bandwidth_used_gb: Option<i64>,
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

/* [164A-19] Despliegues reales de VPS2 en el panel admin.
 * Expone el estado de Coolify enriquecido con el vínculo opcional a suscripciones
 * de hosting guardadas en la BD para detectar drift entre panel e infraestructura. */
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CoolifyDeploymentResponse {
    pub uuid: String,
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
    /// Etiqueta del servidor Coolify de origen, ej: "VPS Principal" o "VPS2".
    pub server_label: String,
}

/* ============================================================
TESTS — [094A-10] Validación de modelos hosting
============================================================ */

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    /* --- Domain regex --- */

    #[test]
    fn domain_regex_valid_domains() {
        let valid = [
            "example.com",
            "sub.example.com",
            "my-site.example.co.uk",
            "a.b.c.d.example.com",
            "x.com",
            "example123.com",
        ];
        for d in valid {
            assert!(DOMAIN_REGEX.is_match(d), "Should be valid: {d}");
        }
    }

    #[test]
    fn domain_regex_rejects_invalid() {
        let invalid = [
            "",
            " ",
            "http://example.com",
            "https://example.com",
            "example.com/path",
            "-example.com",
            "example-.com",
            "example..com",
            "'; DROP TABLE users; --",
            "javascript:alert(1)",
            "../../etc/passwd",
            "example .com",
        ];
        for d in invalid {
            assert!(!DOMAIN_REGEX.is_match(d), "Should be invalid: {d}");
        }
    }

    /* --- Validator integration: SelfSubscribeRequest --- */

    #[test]
    fn self_subscribe_request_valid_domain() {
        let req = SelfSubscribeRequest {
            plan: "basico".to_string(),
            domain: Some("example.com".to_string()),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn self_subscribe_request_invalid_domain_rejected() {
        let req = SelfSubscribeRequest {
            plan: "basico".to_string(),
            domain: Some("'; DROP TABLE hosting_subscriptions; --".to_string()),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn self_subscribe_request_no_domain_ok() {
        let req = SelfSubscribeRequest {
            plan: "basico".to_string(),
            domain: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn self_subscribe_request_empty_plan_rejected() {
        let req = SelfSubscribeRequest {
            plan: String::new(),
            domain: None,
        };
        assert!(req.validate().is_err());
    }

    /* --- [114A-14] Tests adicionales: CreateHostingRequest --- */

    #[test]
    fn create_request_valid_all_fields() {
        let req = CreateHostingRequest {
            client_name: "Juan García".to_string(),
            client_email: "juan@example.com".to_string(),
            plan: "pro".to_string(),
            domain: Some("nakomi.studio".to_string()),
            coolify_site_name: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn create_request_empty_name_rejected() {
        let req = CreateHostingRequest {
            client_name: String::new(),
            client_email: "test@test.com".to_string(),
            plan: "basico".to_string(),
            domain: None,
            coolify_site_name: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_request_invalid_email_rejected() {
        let req = CreateHostingRequest {
            client_name: "Test".to_string(),
            client_email: "not-an-email".to_string(),
            plan: "basico".to_string(),
            domain: None,
            coolify_site_name: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_request_sql_injection_in_domain_rejected() {
        let req = CreateHostingRequest {
            client_name: "Test".to_string(),
            client_email: "test@test.com".to_string(),
            plan: "basico".to_string(),
            domain: Some("' OR 1=1; --".to_string()),
            coolify_site_name: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_request_xss_in_domain_rejected() {
        let req = CreateHostingRequest {
            client_name: "Test".to_string(),
            client_email: "test@test.com".to_string(),
            plan: "basico".to_string(),
            domain: Some("<script>alert(1)</script>.com".to_string()),
            coolify_site_name: None,
        };
        assert!(req.validate().is_err());
    }

    /* --- [114A-14] UpdateHostingRequest --- */

    #[test]
    fn update_request_valid() {
        let req = UpdateHostingRequest {
            plan: "ecommerce".to_string(),
            domain: Some("new-domain.com".to_string()),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn update_request_empty_plan_rejected() {
        let req = UpdateHostingRequest {
            plan: String::new(),
            domain: None,
        };
        assert!(req.validate().is_err());
    }

    /* --- [114A-14] UpdateHostingStatusRequest --- */

    #[test]
    fn status_request_valid() {
        let req = UpdateHostingStatusRequest {
            status: "active".to_string(),
            reason: Some("Payment received".to_string()),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn status_request_empty_status_rejected() {
        let req = UpdateHostingStatusRequest {
            status: String::new(),
            reason: None,
        };
        assert!(req.validate().is_err());
    }

    /* --- [114A-14] Domain edge cases --- */

    #[test]
    fn domain_regex_max_label_length_63() {
        /* Cada label en un dominio puede tener max 63 chars */
        let long_label = "a".repeat(63);
        let domain = format!("{long_label}.com");
        assert!(
            DOMAIN_REGEX.is_match(&domain),
            "63 char label debería ser válido"
        );

        let too_long = "a".repeat(64);
        let domain = format!("{too_long}.com");
        assert!(
            !DOMAIN_REGEX.is_match(&domain),
            "64 char label debería ser inválido"
        );
    }

    #[test]
    fn domain_regex_accepts_numeric_tld() {
        assert!(DOMAIN_REGEX.is_match("12345.678"));
    }

    #[test]
    fn domain_regex_rejects_trailing_dot() {
        assert!(!DOMAIN_REGEX.is_match("example.com."));
    }

    /* --- [114A-14] HostingSubscriptionResponse::from --- */

    #[test]
    fn subscription_response_preserves_all_fields() {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let sub = HostingSubscription {
            id,
            user_id: Some(user_id),
            client_name: "Test Client".to_string(),
            client_email: "test@test.com".to_string(),
            plan: "pro".to_string(),
            domain: Some("test.com".to_string()),
            coolify_site_name: Some("hosting-abc123".to_string()),
            status: "active".to_string(),
            stripe_subscription_id: Some("sub_123".to_string()),
            monthly_price_cents: 1000,
            storage_limit_mb: 20480,
            server_uuid: Some("uuid-123".to_string()),
            server_ip: Some("1.2.3.4".to_string()),
            sftp_user: Some("user".to_string()),
            sftp_password: Some("pass".to_string()),
            sftp_port: Some(10001),
            created_at: now,
            updated_at: now,
        };

        let resp = HostingSubscriptionResponse::from(sub);
        assert_eq!(resp.id, id);
        assert_eq!(resp.user_id, Some(user_id));
        assert_eq!(resp.plan, "pro");
        assert_eq!(resp.domain.as_deref(), Some("test.com"));
        assert_eq!(resp.status, "active");
        assert_eq!(resp.monthly_price_cents, 1000);
        assert_eq!(resp.storage_limit_mb, 20480);
        assert_eq!(resp.server_ip.as_deref(), Some("1.2.3.4"));
        assert_eq!(resp.sftp_user.as_deref(), Some("user"));
        assert_eq!(resp.sftp_port, Some(10001));
    }
}
