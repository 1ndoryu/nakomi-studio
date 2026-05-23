/* [225A-3] La infraestructura visible nace de servidores Coolify configurados y se enriquece con Contabo por IP.
 * Gotcha: Contabo puede devolver solo una VPS, pero los despliegues reales viven en todos los Coolify configurados. */

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::Serialize;

use super::coolify::CoolifyConfig;

#[derive(Debug, Clone)]
pub struct CoolifyServerTarget<'a> {
    pub label: String,
    pub config: &'a CoolifyConfig,
}

#[derive(Debug, Clone, Serialize)]
pub struct InfrastructureServerSummary {
    pub inventory_id: String,
    pub instance_id: Option<i64>,
    pub name: String,
    pub label: String,
    pub ip: String,
    pub status: String,
    pub region: String,
    pub cpu_cores: Option<i32>,
    pub ram_mb: Option<i64>,
    pub disk_mb: Option<i64>,
    pub provider: String,
    pub source: String,
    pub is_configured: bool,
    pub coolify_server_uuid: Option<String>,
    pub ssh_available: bool,
    pub cpu_avg_1h: Option<f64>,
    pub ram_used_mb: Option<f64>,
    pub ram_limit_mb: Option<f64>,
    pub disk_used_mb: Option<f64>,
    pub disk_limit_mb: Option<f64>,
    pub sampled_at: Option<DateTime<Utc>>,
}

impl InfrastructureServerSummary {
    #[must_use]
    pub fn from_contabo(instance: crate::services::contabo::VpsSummary) -> Self {
        let name = if instance.name.trim().is_empty() {
            format!("VPS #{}", instance.instance_id)
        } else {
            instance.name
        };

        Self {
            inventory_id: format!("contabo:{}", instance.instance_id),
            instance_id: Some(instance.instance_id),
            name: name.clone(),
            label: name,
            ip: instance.ip,
            status: instance.status,
            region: instance.region,
            cpu_cores: Some(instance.cpu_cores),
            ram_mb: Some(instance.ram_mb),
            disk_mb: Some(instance.disk_mb),
            provider: "contabo".to_string(),
            source: "contabo".to_string(),
            is_configured: false,
            coolify_server_uuid: None,
            ssh_available: false,
            cpu_avg_1h: None,
            ram_used_mb: None,
            ram_limit_mb: None,
            disk_used_mb: None,
            disk_limit_mb: None,
            sampled_at: None,
        }
    }
}

fn read_label(env_key: &str, fallback: &str) -> String {
    std::env::var(env_key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

fn default_coolify_label(config: &CoolifyConfig) -> &'static str {
    match config.server_ip.as_str() {
        "66.94.100.241" => "VPS Principal",
        "173.249.50.44" => "VPS2",
        _ => "VPS Coolify",
    }
}

fn coolify_dedupe_key(config: &CoolifyConfig) -> String {
    if !config.server_uuid.trim().is_empty() {
        return format!("server:{}", config.server_uuid);
    }

    format!("ip:{}:{}", config.server_ip, config.base_url)
}

fn push_target<'a>(
    targets: &mut Vec<CoolifyServerTarget<'a>>,
    seen: &mut HashSet<String>,
    label: String,
    config: &'a CoolifyConfig,
) {
    let key = coolify_dedupe_key(config);
    if seen.insert(key) {
        targets.push(CoolifyServerTarget { label, config });
    }
}

#[must_use]
pub fn coolify_server_targets<'a>(
    vps1_config: Option<&'a CoolifyConfig>,
    default_config: Option<&'a CoolifyConfig>,
) -> Vec<CoolifyServerTarget<'a>> {
    let mut targets = Vec::new();
    let mut seen = HashSet::new();

    if let Some(config) = vps1_config {
        push_target(
            &mut targets,
            &mut seen,
            read_label("COOLIFY_VPS1_LABEL", "VPS Principal"),
            config,
        );
    }

    if let Some(config) = default_config {
        push_target(
            &mut targets,
            &mut seen,
            read_label("COOLIFY_LABEL", default_coolify_label(config)),
            config,
        );
    }

    targets
}

#[must_use]
pub fn configured_server_summaries(
    vps1_config: Option<&CoolifyConfig>,
    default_config: Option<&CoolifyConfig>,
) -> Vec<InfrastructureServerSummary> {
    coolify_server_targets(vps1_config, default_config)
        .into_iter()
        .map(|target| InfrastructureServerSummary {
            inventory_id: format!("coolify:{}", target.config.server_uuid),
            instance_id: None,
            name: target.label.clone(),
            label: target.label,
            ip: target.config.server_ip.clone(),
            status: "configured".to_string(),
            region: "Coolify".to_string(),
            cpu_cores: None,
            ram_mb: None,
            disk_mb: None,
            provider: "coolify".to_string(),
            source: "coolify_config".to_string(),
            is_configured: true,
            coolify_server_uuid: Some(target.config.server_uuid.clone()),
            ssh_available: target.config.ssh_key_path.is_some(),
            cpu_avg_1h: None,
            ram_used_mb: None,
            ram_limit_mb: None,
            disk_used_mb: None,
            disk_limit_mb: None,
            sampled_at: None,
        })
        .collect()
}
