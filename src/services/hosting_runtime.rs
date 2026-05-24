/* [245A-3] Fachada de runtime de hosting para desacoplar el dominio de la API
 * directa de Coolify. En este bloque el provider `lightweight` queda declarado
 * pero no implementado para no mezclar la abstracción con el runtime nuevo.
 * [245A-6] `runtime_kind` y `deployment_id` ya se persisten en la suscripción,
 * así que control, borrado y refresh pueden despacharse por runtime guardado. */

use reqwest::Client;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::HostingPlanConfig;

use super::coolify::{
    CoolifyConfig, CoolifyProvisionResult, CoolifyService, CoolifyServiceSummary,
    HostingComposeUpdate, HostingProvisionPreferences,
};

const HOSTING_RUNTIME_PROVIDER_ENV: &str = "HOSTING_RUNTIME_PROVIDER";
const HOSTING_RUNTIME_COOLIFY: &str = "coolify";
const HOSTING_RUNTIME_LIGHTWEIGHT: &str = "lightweight";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostingRuntimeKind {
    Coolify,
    Lightweight,
}

impl HostingRuntimeKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Coolify => HOSTING_RUNTIME_COOLIFY,
            Self::Lightweight => HOSTING_RUNTIME_LIGHTWEIGHT,
        }
    }

    #[must_use]
    pub fn parse(raw_value: &str) -> Option<Self> {
        match raw_value.trim().to_ascii_lowercase().as_str() {
            "" => None,
            HOSTING_RUNTIME_COOLIFY => Some(Self::Coolify),
            HOSTING_RUNTIME_LIGHTWEIGHT => Some(Self::Lightweight),
            _ => None,
        }
    }

    #[must_use]
    pub fn from_persisted(raw_value: &str) -> Self {
        Self::parse(raw_value).unwrap_or(Self::Coolify)
    }

    #[must_use]
    pub fn from_env() -> Self {
        let Ok(raw_value) = std::env::var(HOSTING_RUNTIME_PROVIDER_ENV) else {
            return Self::Coolify;
        };

        if let Some(kind) = Self::parse(&raw_value) {
            kind
        } else {
            tracing::warn!(
                "Runtime de hosting desconocido '{}' en {}. Se usa 'coolify'.",
                raw_value,
                HOSTING_RUNTIME_PROVIDER_ENV
            );
            Self::Coolify
        }
    }
}

#[derive(Debug, Clone)]
pub struct HostingRuntimeProvisionResult {
    pub runtime_kind: HostingRuntimeKind,
    pub deployment_id: String,
    pub public_url: String,
    pub server_ip: String,
    pub access_user: String,
    pub access_password: String,
    pub access_port: i32,
    pub wordpress_ready: bool,
    pub wordpress_install_error: Option<String>,
}

impl From<CoolifyProvisionResult> for HostingRuntimeProvisionResult {
    fn from(value: CoolifyProvisionResult) -> Self {
        Self {
            runtime_kind: HostingRuntimeKind::Coolify,
            deployment_id: value.service_uuid,
            public_url: value.domain,
            server_ip: value.server_ip,
            access_user: value.sftp_user,
            access_password: value.sftp_password,
            access_port: value.sftp_port,
            wordpress_ready: value.wordpress_ready,
            wordpress_install_error: value.wordpress_install_error,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HostingRuntimeDeploymentSummary {
    pub runtime_kind: HostingRuntimeKind,
    pub deployment_id: String,
    pub name: String,
    pub status: String,
    pub fqdn: Option<String>,
    pub target_id: Option<String>,
    pub target_name: Option<String>,
    pub project_id: Option<String>,
    pub environment_name: Option<String>,
}

impl From<CoolifyServiceSummary> for HostingRuntimeDeploymentSummary {
    fn from(value: CoolifyServiceSummary) -> Self {
        Self {
            runtime_kind: HostingRuntimeKind::Coolify,
            deployment_id: value.uuid,
            name: value.name,
            status: value.status,
            fqdn: value.fqdn,
            target_id: value.server_uuid,
            target_name: value.server_name,
            project_id: value.project_uuid,
            environment_name: value.environment_name,
        }
    }
}

pub struct HostingRuntimeUpdate<'a> {
    pub deployment_id: &'a str,
    pub deployment_name: &'a str,
    pub custom_domain: Option<&'a str>,
    pub access_user: &'a str,
    pub access_password: &'a str,
    pub access_port: i32,
    pub plan_config: &'a HostingPlanConfig,
}

pub struct HostingRuntimeService;

impl HostingRuntimeService {
    #[must_use]
    pub fn current_kind() -> HostingRuntimeKind {
        HostingRuntimeKind::from_env()
    }

    #[must_use]
    pub fn deployment_name_for(subscription_id: &Uuid) -> String {
        let id_str = subscription_id.to_string();
        let short = &id_str[..8];
        format!("hosting-{short}")
    }

    pub fn require_target_config<'a>(
        coolify_config: Option<&'a CoolifyConfig>,
        operation: &str,
    ) -> Result<&'a CoolifyConfig, AppError> {
        match Self::current_kind() {
            HostingRuntimeKind::Coolify => {
                coolify_config.ok_or_else(|| {
                    AppError::ServiceUnavailable(format!(
                        "El runtime de hosting 'coolify' no está configurado para {operation}. Variables COOLIFY_* ausentes."
                    ))
                })
            }
            HostingRuntimeKind::Lightweight => {
                Err(Self::unsupported(HostingRuntimeKind::Lightweight, operation))
            }
        }
    }

    pub async fn list_deployments(
        http_client: &Client,
        coolify_config: Option<&CoolifyConfig>,
    ) -> Result<Vec<HostingRuntimeDeploymentSummary>, AppError> {
        match Self::current_kind() {
            HostingRuntimeKind::Coolify => {
                let config = Self::require_target_config(coolify_config, "listar despliegues")?;
                Ok(CoolifyService::list_services(http_client, config)
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect())
            }
            HostingRuntimeKind::Lightweight => Err(Self::unsupported(
                HostingRuntimeKind::Lightweight,
                "listar despliegues",
            )),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn provision_hosting(
        http_client: &Client,
        coolify_config: Option<&CoolifyConfig>,
        deployment_name: &str,
        access_port: i32,
        plan_config: &HostingPlanConfig,
        client_name: &str,
        client_email: &str,
        preferences: Option<&HostingProvisionPreferences>,
    ) -> Result<HostingRuntimeProvisionResult, AppError> {
        match Self::current_kind() {
            HostingRuntimeKind::Coolify => {
                let config = Self::require_target_config(coolify_config, "provisionar hostings")?;
                Ok(CoolifyService::provision_hosting(
                    http_client,
                    config,
                    deployment_name,
                    access_port,
                    plan_config,
                    client_name,
                    client_email,
                    preferences,
                )
                .await?
                .into())
            }
            HostingRuntimeKind::Lightweight => Err(Self::unsupported(
                HostingRuntimeKind::Lightweight,
                "provisionar hostings",
            )),
        }
    }

    pub async fn update_deployment(
        http_client: &Client,
        coolify_config: Option<&CoolifyConfig>,
        runtime_kind: Option<HostingRuntimeKind>,
        update: HostingRuntimeUpdate<'_>,
    ) -> Result<(), AppError> {
        let runtime_kind = runtime_kind.unwrap_or_else(Self::current_kind);
        match runtime_kind {
            HostingRuntimeKind::Coolify => {
                let config =
                    Self::require_target_config(coolify_config, "actualizar despliegues")?;
                CoolifyService::update_compose_and_restart(
                    http_client,
                    config,
                    HostingComposeUpdate {
                        service_uuid: update.deployment_id,
                        service_name: update.deployment_name,
                        custom_domain: update.custom_domain,
                        sftp_user: update.access_user,
                        sftp_password: update.access_password,
                        sftp_port: update.access_port,
                        plan_config: update.plan_config,
                    },
                )
                .await
            }
            HostingRuntimeKind::Lightweight => {
                Err(Self::unsupported(runtime_kind, "actualizar despliegues"))
            }
        }
    }

    pub async fn delete_deployment(
        http_client: &Client,
        coolify_config: Option<&CoolifyConfig>,
        runtime_kind: Option<HostingRuntimeKind>,
        deployment_id: &str,
        delete_volumes: bool,
    ) -> Result<(), AppError> {
        let runtime_kind = runtime_kind.unwrap_or_else(Self::current_kind);
        match runtime_kind {
            HostingRuntimeKind::Coolify => {
                let config = Self::require_target_config(coolify_config, "eliminar despliegues")?;
                CoolifyService::delete_service(http_client, config, deployment_id, delete_volumes)
                    .await
            }
            HostingRuntimeKind::Lightweight => {
                Err(Self::unsupported(runtime_kind, "eliminar despliegues"))
            }
        }
    }

    pub async fn stop_deployment(
        http_client: &Client,
        coolify_config: Option<&CoolifyConfig>,
        runtime_kind: Option<HostingRuntimeKind>,
        deployment_id: &str,
    ) -> Result<(), AppError> {
        let runtime_kind = runtime_kind.unwrap_or_else(Self::current_kind);
        match runtime_kind {
            HostingRuntimeKind::Coolify => {
                let config = Self::require_target_config(coolify_config, "detener despliegues")?;
                CoolifyService::stop_service(http_client, config, deployment_id).await
            }
            HostingRuntimeKind::Lightweight => {
                Err(Self::unsupported(runtime_kind, "detener despliegues"))
            }
        }
    }

    pub async fn start_deployment(
        http_client: &Client,
        coolify_config: Option<&CoolifyConfig>,
        runtime_kind: Option<HostingRuntimeKind>,
        deployment_id: &str,
    ) -> Result<(), AppError> {
        let runtime_kind = runtime_kind.unwrap_or_else(Self::current_kind);
        match runtime_kind {
            HostingRuntimeKind::Coolify => {
                let config = Self::require_target_config(coolify_config, "iniciar despliegues")?;
                CoolifyService::start_service(http_client, config, deployment_id).await
            }
            HostingRuntimeKind::Lightweight => {
                Err(Self::unsupported(runtime_kind, "iniciar despliegues"))
            }
        }
    }

    pub async fn restart_deployment(
        http_client: &Client,
        coolify_config: Option<&CoolifyConfig>,
        runtime_kind: Option<HostingRuntimeKind>,
        deployment_id: &str,
    ) -> Result<(), AppError> {
        let runtime_kind = runtime_kind.unwrap_or_else(Self::current_kind);
        match runtime_kind {
            HostingRuntimeKind::Coolify => {
                let config =
                    Self::require_target_config(coolify_config, "reiniciar despliegues")?;
                CoolifyService::restart_service(http_client, config, deployment_id).await
            }
            HostingRuntimeKind::Lightweight => {
                Err(Self::unsupported(runtime_kind, "reiniciar despliegues"))
            }
        }
    }

    fn unsupported(runtime_kind: HostingRuntimeKind, operation: &str) -> AppError {
        AppError::ServiceUnavailable(format!(
            "El runtime de hosting '{}' aún no implementa {}.",
            runtime_kind.as_str(),
            operation
        ))
    }
}