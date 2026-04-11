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
        r"^(?i)(?:[a-z0-9](?:[a-z0-9\-]{0,61}[a-z0-9])?\.)*[a-z0-9](?:[a-z0-9\-]{0,61}[a-z0-9])?$"
    ).expect("regex de dominio válida")
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
    #[validate(length(max = 253), regex(path = "*DOMAIN_REGEX", message = "Dominio inválido"))]
    pub domain: Option<String>,
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
    #[validate(length(max = 253), regex(path = "*DOMAIN_REGEX", message = "Dominio inválido"))]
    pub domain: Option<String>,
}

/* [074A-65] Request para editar suscripción (plan, dominio) */
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateHostingRequest {
    #[validate(length(min = 1, max = 20))]
    pub plan: String,
    #[validate(length(max = 253), regex(path = "*DOMAIN_REGEX", message = "Dominio inválido"))]
    pub domain: Option<String>,
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

/* [094A-8] Estadísticas reales de una suscripción de hosting.
 * Uptime se calcula desde el historial de eventos (status_change).
 * Storage/bandwidth son límites del plan; uso real requiere monitoreo futuro (Coolify/cAdvisor). */
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
}

/* ============================================================
   TESTS — [094A-10] Validación de modelos hosting
   ============================================================ */

#[cfg(test)]
mod tests {
    use super::*;

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

    /* --- Validator integration: domain field --- */

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
}
