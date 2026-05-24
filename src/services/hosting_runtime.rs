/* [245A-3] Fachada de runtime de hosting para desacoplar el dominio de la API
 * directa de Coolify. En este bloque el provider `lightweight` queda declarado
 * pero no implementado para no mezclar la abstracción con el runtime nuevo.
 * [245A-6] `runtime_kind` y `deployment_id` ya se persisten en la suscripción,
 * así que control, borrado y refresh pueden despacharse por runtime guardado. */

use reqwest::Client;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::HostingPlanConfig;

pub use super::hosting_runtime_backups::{
    HostingRuntimeBackupEntry, HostingRuntimeBackupReport, HostingRuntimeRestoreReport,
};
use super::hosting_runtime_backups::{
    LightweightManagerBackupListReport, LightweightManagerBackupReport,
    LightweightManagerRestoreReport,
};
use super::hosting_runtime_lightweight_types::{
    LightweightManagerInventoryReport, LightweightManagerProvisionStaticReport,
};
use super::coolify::{
    CoolifyConfig, CoolifyProvisionResult, CoolifyService, CoolifyServiceSummary,
    HostingComposeUpdate, HostingProvisionPreferences,
};

const HOSTING_RUNTIME_PROVIDER_ENV: &str = "HOSTING_RUNTIME_PROVIDER";
const HOSTING_RUNTIME_COOLIFY: &str = "coolify";
const HOSTING_RUNTIME_LIGHTWEIGHT: &str = "lightweight";
const HOSTING_LIGHTWEIGHT_MANAGER_BIN_ENV: &str = "HOSTING_LIGHTWEIGHT_MANAGER_BIN";
const HOSTING_LIGHTWEIGHT_MANAGER_CONFIG_ENV: &str = "HOSTING_LIGHTWEIGHT_MANAGER_CONFIG";
const HOSTING_LIGHTWEIGHT_TARGET_ENV: &str = "HOSTING_LIGHTWEIGHT_TARGET";

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

#[derive(Debug, Clone)]
struct LightweightManagerConfig {
    bin_path: String,
    config_path: Option<String>,
    target: String,
}

impl LightweightManagerConfig {
    fn configured() -> bool {
        std::env::var(HOSTING_LIGHTWEIGHT_TARGET_ENV)
            .ok()
            .is_some_and(|value| !value.trim().is_empty())
    }

    fn from_env() -> Result<Self, AppError> {
        let target = std::env::var(HOSTING_LIGHTWEIGHT_TARGET_ENV)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                AppError::ServiceUnavailable(format!(
                    "El runtime de hosting 'lightweight' requiere {HOSTING_LIGHTWEIGHT_TARGET_ENV} para ubicar el target."
                ))
            })?;

        let bin_path = std::env::var(HOSTING_LIGHTWEIGHT_MANAGER_BIN_ENV)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "coolify-manager".to_string());

        let config_path = std::env::var(HOSTING_LIGHTWEIGHT_MANAGER_CONFIG_ENV)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        Ok(Self {
            bin_path,
            config_path,
            target,
        })
    }
}

pub struct HostingRuntimeService;

impl HostingRuntimeService {
    #[must_use]
    pub fn current_kind() -> HostingRuntimeKind {
        HostingRuntimeKind::from_env()
    }

    /* [245A-10] Las compras ya no pueden depender del provider global.
     * Gotcha: el corte actual mezcla `normal-*` en lightweight y WordPress en Coolify,
     * así que decidir por env rompería uno de los dos flujos comerciales. */
    #[must_use]
    pub fn runtime_kind_for_plan(plan: &str) -> HostingRuntimeKind {
        let normalized = plan.trim().to_ascii_lowercase();
        if normalized.starts_with("normal-") && Self::lightweight_manager_configured() {
            return HostingRuntimeKind::Lightweight;
        }

        HostingRuntimeKind::Coolify
    }

    #[must_use]
    pub fn lightweight_manager_configured() -> bool {
        LightweightManagerConfig::configured()
    }

    /* [245A-7] El runtime persistido debe gobernar las operaciones legacy.
     * Gotcha: si se resuelve por env global, cambiar a `lightweight` rompe
     * listados y controles sobre despliegues viejos aún gestionados por Coolify. */
    #[must_use]
    fn resolved_kind(runtime_kind: Option<HostingRuntimeKind>) -> HostingRuntimeKind {
        runtime_kind.unwrap_or_else(Self::current_kind)
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
        Self::require_target_config_for(Self::current_kind(), coolify_config, operation)
    }

    pub fn require_target_config_for<'a>(
        runtime_kind: HostingRuntimeKind,
        coolify_config: Option<&'a CoolifyConfig>,
        operation: &str,
    ) -> Result<&'a CoolifyConfig, AppError> {
        match runtime_kind {
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

    pub fn optional_target_config_for<'a>(
        runtime_kind: HostingRuntimeKind,
        coolify_config: Option<&'a CoolifyConfig>,
        operation: &str,
    ) -> Result<Option<&'a CoolifyConfig>, AppError> {
        match runtime_kind {
            HostingRuntimeKind::Coolify => {
                Ok(Some(Self::require_target_config_for(
                    runtime_kind,
                    coolify_config,
                    operation,
                )?))
            }
            HostingRuntimeKind::Lightweight => Ok(None),
        }
    }

    pub async fn list_deployments(
        http_client: &Client,
        coolify_config: Option<&CoolifyConfig>,
        runtime_kind: Option<HostingRuntimeKind>,
    ) -> Result<Vec<HostingRuntimeDeploymentSummary>, AppError> {
        let runtime_kind = Self::resolved_kind(runtime_kind);
        match runtime_kind {
            HostingRuntimeKind::Coolify => {
                let config =
                    Self::require_target_config_for(runtime_kind, coolify_config, "listar despliegues")?;
                Ok(CoolifyService::list_services(http_client, config)
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect())
            }
            HostingRuntimeKind::Lightweight => Self::list_lightweight_deployments().await,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn provision_hosting(
        http_client: &Client,
        coolify_config: Option<&CoolifyConfig>,
        runtime_kind: Option<HostingRuntimeKind>,
        deployment_name: &str,
        access_port: i32,
        plan_config: &HostingPlanConfig,
        client_name: &str,
        client_email: &str,
        preferences: Option<&HostingProvisionPreferences>,
    ) -> Result<HostingRuntimeProvisionResult, AppError> {
        let runtime_kind = Self::resolved_kind(runtime_kind);
        match runtime_kind {
            HostingRuntimeKind::Coolify => {
                let config = Self::require_target_config_for(
                    runtime_kind,
                    coolify_config,
                    "provisionar hostings",
                )?;
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
            HostingRuntimeKind::Lightweight => {
                if !plan_config.plan_name.starts_with("normal-") {
                    return Err(AppError::ServiceUnavailable(
                        "El runtime de hosting 'lightweight' aún solo soporta planes normal-*; los planes WordPress siguen en el runtime legacy.".into(),
                    ));
                }

                let mut args = vec![
                    "--site".to_string(),
                    deployment_name.to_string(),
                    "--json".to_string(),
                ];

                if let Some(access_user) = preferences
                    .and_then(|prefs| prefs.sftp_user.as_deref())
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                {
                    args.push("--access-user".to_string());
                    args.push(access_user.to_string());
                }

                if let Some(access_password) = preferences
                    .and_then(|prefs| prefs.sftp_password.as_deref())
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                {
                    args.push("--access-password".to_string());
                    args.push(access_password.to_string());
                }

                let report: LightweightManagerProvisionStaticReport =
                    Self::run_lightweight_manager_json(
                        "provisionar hostings",
                        "provision-static",
                        &args,
                    )
                    .await?;

                Ok(HostingRuntimeProvisionResult {
                    runtime_kind,
                    deployment_id: report.deployment_id,
                    public_url: report.public_url,
                    server_ip: report.target_ip,
                    access_user: report.access_user,
                    access_password: report.access_password,
                    access_port: i32::from(report.access_port),
                    wordpress_ready: false,
                    wordpress_install_error: None,
                })
            }
        }
    }

    pub async fn update_deployment(
        http_client: &Client,
        coolify_config: Option<&CoolifyConfig>,
        runtime_kind: Option<HostingRuntimeKind>,
        update: HostingRuntimeUpdate<'_>,
    ) -> Result<(), AppError> {
        let runtime_kind = Self::resolved_kind(runtime_kind);
        match runtime_kind {
            HostingRuntimeKind::Coolify => {
                let config = Self::require_target_config_for(
                    runtime_kind,
                    coolify_config,
                    "actualizar despliegues",
                )?;
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
                /* [245A-8] El runtime lightweight deja de ser un stub en refresh/domain/rotate.
                 * Gotcha: si estos updates siguieran exigiendo CoolifyConfig, el sitio podria provisionarse
                 * pero no cambiar dominio ni credenciales una vez creado. */
                let mut args = vec![
                    "--site".to_string(),
                    update.deployment_id.to_string(),
                    "--action".to_string(),
                    "reconfigure".to_string(),
                    "--access-user".to_string(),
                    update.access_user.to_string(),
                    "--access-password".to_string(),
                    update.access_password.to_string(),
                    "--json".to_string(),
                ];
                if let Some(custom_domain) = update.custom_domain {
                    args.push("--fqdn".to_string());
                    args.push(custom_domain.to_string());
                }
                let _: serde_json::Value = Self::run_lightweight_manager_json(
                    "actualizar despliegues",
                    "light-site",
                    &args,
                )
                .await?;
                Ok(())
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
        let runtime_kind = Self::resolved_kind(runtime_kind);
        match runtime_kind {
            HostingRuntimeKind::Coolify => {
                let config = Self::require_target_config_for(
                    runtime_kind,
                    coolify_config,
                    "eliminar despliegues",
                )?;
                CoolifyService::delete_service(http_client, config, deployment_id, delete_volumes)
                    .await
            }
            HostingRuntimeKind::Lightweight => {
                Self::run_lightweight_site_action("eliminar despliegues", deployment_id, "delete", delete_volumes).await
            }
        }
    }

    pub async fn list_backups(
        runtime_kind: Option<HostingRuntimeKind>,
        deployment_id: &str,
    ) -> Result<Vec<HostingRuntimeBackupEntry>, AppError> {
        let runtime_kind = Self::resolved_kind(runtime_kind);
        match runtime_kind {
            HostingRuntimeKind::Coolify => {
                Err(Self::unsupported(runtime_kind, "listar backups"))
            }
            HostingRuntimeKind::Lightweight => {
                let report: LightweightManagerBackupListReport = Self::run_lightweight_manager_json(
                    "listar backups",
                    "light-backup",
                    &[
                        "--site".to_string(),
                        deployment_id.to_string(),
                        "--list".to_string(),
                        "--json".to_string(),
                    ],
                )
                .await?;

                Ok(report
                    .entries
                    .into_iter()
                    .map(|entry| HostingRuntimeBackupEntry {
                        backup_id: entry.backup_id,
                        tier: entry.tier,
                        file_id: entry.file_id,
                        file_name: entry.file_name,
                    })
                    .collect())
            }
        }
    }

    pub async fn create_backup(
        runtime_kind: Option<HostingRuntimeKind>,
        deployment_id: &str,
        tier: &str,
        label: Option<&str>,
    ) -> Result<HostingRuntimeBackupReport, AppError> {
        let runtime_kind = Self::resolved_kind(runtime_kind);
        match runtime_kind {
            HostingRuntimeKind::Coolify => {
                Err(Self::unsupported(runtime_kind, "crear backups"))
            }
            HostingRuntimeKind::Lightweight => {
                let mut args = vec![
                    "--site".to_string(),
                    deployment_id.to_string(),
                    "--tier".to_string(),
                    tier.to_string(),
                    "--json".to_string(),
                ];
                if let Some(label) = label.map(str::trim).filter(|value| !value.is_empty()) {
                    args.push("--label".to_string());
                    args.push(label.to_string());
                }

                let report: LightweightManagerBackupReport = Self::run_lightweight_manager_json(
                    "crear backups",
                    "light-backup",
                    &args,
                )
                .await?;

                Ok(HostingRuntimeBackupReport {
                    backup_id: report.backup_id,
                    tier: report.tier,
                    status: report.status,
                    notes: report.notes,
                })
            }
        }
    }

    pub async fn restore_backup(
        runtime_kind: Option<HostingRuntimeKind>,
        deployment_id: &str,
        backup_id: &str,
        access_password: Option<&str>,
        skip_safety_snapshot: bool,
    ) -> Result<HostingRuntimeRestoreReport, AppError> {
        let runtime_kind = Self::resolved_kind(runtime_kind);
        match runtime_kind {
            HostingRuntimeKind::Coolify => {
                Err(Self::unsupported(runtime_kind, "restaurar backups"))
            }
            HostingRuntimeKind::Lightweight => {
                let mut args = vec![
                    "--site".to_string(),
                    deployment_id.to_string(),
                    "--backup-id".to_string(),
                    backup_id.to_string(),
                    "--json".to_string(),
                ];
                if let Some(access_password) = access_password
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                {
                    args.push("--access-password".to_string());
                    args.push(access_password.to_string());
                }
                if skip_safety_snapshot {
                    args.push("--skip-safety-snapshot".to_string());
                }

                let report: LightweightManagerRestoreReport = Self::run_lightweight_manager_json(
                    "restaurar backups",
                    "light-restore",
                    &args,
                )
                .await?;

                Ok(HostingRuntimeRestoreReport {
                    backup_id: report.backup_id,
                    status: report.status,
                    fqdn: report.fqdn,
                    access_user: report.access_user,
                    access_password: report.access_password,
                    notes: report.notes,
                })
            }
        }
    }

    pub async fn stop_deployment(
        http_client: &Client,
        coolify_config: Option<&CoolifyConfig>,
        runtime_kind: Option<HostingRuntimeKind>,
        deployment_id: &str,
    ) -> Result<(), AppError> {
        let runtime_kind = Self::resolved_kind(runtime_kind);
        match runtime_kind {
            HostingRuntimeKind::Coolify => {
                let config = Self::require_target_config_for(
                    runtime_kind,
                    coolify_config,
                    "detener despliegues",
                )?;
                CoolifyService::stop_service(http_client, config, deployment_id).await
            }
            HostingRuntimeKind::Lightweight => {
                Self::run_lightweight_site_action("detener despliegues", deployment_id, "stop", false).await
            }
        }
    }

    pub async fn start_deployment(
        http_client: &Client,
        coolify_config: Option<&CoolifyConfig>,
        runtime_kind: Option<HostingRuntimeKind>,
        deployment_id: &str,
    ) -> Result<(), AppError> {
        let runtime_kind = Self::resolved_kind(runtime_kind);
        match runtime_kind {
            HostingRuntimeKind::Coolify => {
                let config = Self::require_target_config_for(
                    runtime_kind,
                    coolify_config,
                    "iniciar despliegues",
                )?;
                CoolifyService::start_service(http_client, config, deployment_id).await
            }
            HostingRuntimeKind::Lightweight => {
                Self::run_lightweight_site_action("iniciar despliegues", deployment_id, "start", false).await
            }
        }
    }

    pub async fn restart_deployment(
        http_client: &Client,
        coolify_config: Option<&CoolifyConfig>,
        runtime_kind: Option<HostingRuntimeKind>,
        deployment_id: &str,
    ) -> Result<(), AppError> {
        let runtime_kind = Self::resolved_kind(runtime_kind);
        match runtime_kind {
            HostingRuntimeKind::Coolify => {
                let config = Self::require_target_config_for(
                    runtime_kind,
                    coolify_config,
                    "reiniciar despliegues",
                )?;
                CoolifyService::restart_service(http_client, config, deployment_id).await
            }
            HostingRuntimeKind::Lightweight => {
                Self::run_lightweight_site_action("reiniciar despliegues", deployment_id, "restart", false).await
            }
        }
    }

    async fn list_lightweight_deployments(
    ) -> Result<Vec<HostingRuntimeDeploymentSummary>, AppError> {
        let report: LightweightManagerInventoryReport = Self::run_lightweight_manager_json(
            "listar despliegues",
            "inventory-light",
            &["--json".to_string()],
        )
        .await?;

        Ok(report
            .sites
            .into_iter()
            .map(|site| HostingRuntimeDeploymentSummary {
                runtime_kind: HostingRuntimeKind::Lightweight,
                deployment_id: site.deployment_id,
                name: site.name,
                status: site.status,
                fqdn: site.fqdn,
                target_id: Some(report.target_ip.clone()),
                target_name: Some(report.target.clone()),
                project_id: None,
                environment_name: None,
            })
            .collect())
    }

    async fn run_lightweight_site_action(
        operation: &str,
        deployment_id: &str,
        action: &str,
        delete_volumes: bool,
    ) -> Result<(), AppError> {
        let mut args = vec![
            "--site".to_string(),
            deployment_id.to_string(),
            "--action".to_string(),
            action.to_string(),
            "--json".to_string(),
        ];
        if delete_volumes {
            args.push("--delete-volumes".to_string());
        }
        let _: serde_json::Value = Self::run_lightweight_manager_json(
            operation,
            "light-site",
            &args,
        )
        .await?;
        Ok(())
    }

    async fn run_lightweight_manager_json<T: DeserializeOwned>(
        operation: &str,
        command_name: &str,
        extra_args: &[String],
    ) -> Result<T, AppError> {
        let stdout = Self::run_lightweight_manager(operation, command_name, extra_args).await?;
        serde_json::from_slice(&stdout).map_err(|error| {
            AppError::ServiceUnavailable(format!(
                "El runtime de hosting 'lightweight' devolvió JSON inválido para {operation}: {error}"
            ))
        })
    }

    async fn run_lightweight_manager(
        operation: &str,
        command_name: &str,
        extra_args: &[String],
    ) -> Result<Vec<u8>, AppError> {
        let config = LightweightManagerConfig::from_env()?;
        let mut command = tokio::process::Command::new(&config.bin_path);
        command.arg("--log-level").arg("error");
        if let Some(config_path) = &config.config_path {
            command.arg("--config").arg(config_path);
        }
        command
            .arg(command_name)
            .arg("--target")
            .arg(&config.target);
        for arg in extra_args {
            command.arg(arg);
        }

        let output = command.output().await.map_err(|error| {
            AppError::ServiceUnavailable(format!(
                "No se pudo ejecutar {} del runtime de hosting 'lightweight' con '{}': {}",
                operation, config.bin_path, error
            ))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(AppError::ServiceUnavailable(format!(
                "El runtime de hosting 'lightweight' falló al {}: {}{}{}",
                operation,
                stderr.trim(),
                if stderr.trim().is_empty() || stdout.trim().is_empty() {
                    ""
                } else {
                    " | stdout: "
                },
                if stdout.trim().is_empty() {
                    ""
                } else {
                    stdout.trim()
                }
            )));
        }

        Ok(output.stdout)
    }

    fn unsupported(runtime_kind: HostingRuntimeKind, operation: &str) -> AppError {
        AppError::ServiceUnavailable(format!(
            "El runtime de hosting '{}' aún no implementa {}.",
            runtime_kind.as_str(),
            operation
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{HostingRuntimeKind, HostingRuntimeService, HOSTING_LIGHTWEIGHT_TARGET_ENV};

    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn runtime_kind_for_plan_keeps_wordpress_on_coolify() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        std::env::set_var(HOSTING_LIGHTWEIGHT_TARGET_ENV, "vps2");

        assert_eq!(
            HostingRuntimeService::runtime_kind_for_plan("basico"),
            HostingRuntimeKind::Coolify
        );

        std::env::remove_var(HOSTING_LIGHTWEIGHT_TARGET_ENV);
    }

    #[test]
    fn runtime_kind_for_plan_uses_lightweight_for_normal_when_configured() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        std::env::set_var(HOSTING_LIGHTWEIGHT_TARGET_ENV, "vps2");

        assert_eq!(
            HostingRuntimeService::runtime_kind_for_plan("normal-basico"),
            HostingRuntimeKind::Lightweight
        );

        std::env::remove_var(HOSTING_LIGHTWEIGHT_TARGET_ENV);
    }

    #[test]
    fn runtime_kind_for_plan_falls_back_to_coolify_without_lightweight_target() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        std::env::remove_var(HOSTING_LIGHTWEIGHT_TARGET_ENV);

        assert_eq!(
            HostingRuntimeService::runtime_kind_for_plan("normal-basico"),
            HostingRuntimeKind::Coolify
        );
    }
}