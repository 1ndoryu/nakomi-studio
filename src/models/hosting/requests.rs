use super::validation::{DOMAIN_REGEX, HOSTING_USERNAME_REGEX};
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

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
 * Solo necesita plan y dominio opcional; nombre/email se toman del perfil del usuario. */
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SelfSubscribeRequest {
    #[validate(length(min = 1, max = 20))]
    pub plan: String,
    #[validate(
        length(max = 253),
        regex(path = "*DOMAIN_REGEX", message = "Dominio inválido")
    )]
    pub domain: Option<String>,
    pub billing_cycle_months: Option<i32>,
    #[validate(
        length(min = 3, max = 60),
        regex(
            path = "*HOSTING_USERNAME_REGEX",
            message = "Usuario wp-admin inválido"
        )
    )]
    pub wp_admin_username: Option<String>,
    #[validate(length(min = 8, max = 128))]
    pub wp_admin_password: Option<String>,
    #[validate(length(max = 20))]
    pub wp_language: Option<String>,
    #[validate(
        length(min = 3, max = 60),
        regex(path = "*HOSTING_USERNAME_REGEX", message = "Usuario SFTP inválido")
    )]
    pub sftp_user: Option<String>,
    #[validate(length(min = 12, max = 128))]
    pub sftp_password: Option<String>,
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
