/* [225A-4] Sampler de recursos de infraestructura.
 * Coolify sigue siendo el inventario autoritativo; este loop toma promedios
 * aproximados por SSH cada 10 minutos y guarda snapshots para que el panel no
 * dispare SSH en cada render. */

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::OnceLock;
use std::time::Duration;

use chrono::Utc;
use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::models::HostingSubscription;
use crate::repositories::{
    BandwidthSnapshotInput, ConfiguredServerInput, HostingRepository, InfrastructureRepository,
    InfrastructureServerRecord, ResourceSampleInput,
};
use crate::services::coolify::{CoolifyConfig, CoolifyServiceSummary};
use crate::services::docker_stats::{parse_docker_stats_public, ContainerStats};
use crate::services::infrastructure::coolify_server_targets;
use crate::services::CoolifyService;

const SAMPLER_INTERVAL: Duration = Duration::from_mins(10);
const SSH_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, Clone, Copy)]
struct CpuCounters {
    total: f64,
    idle: f64,
}

#[derive(Debug, Clone, Default)]
struct ServerSshSnapshot {
    cpu_counters: Option<CpuCounters>,
    cpu_cores: Option<f64>,
    ram_used_mb: Option<f64>,
    ram_limit_mb: Option<f64>,
    disk_used_mb: Option<f64>,
    disk_limit_mb: Option<f64>,
    containers: Vec<ContainerStats>,
    storage_by_uuid: HashMap<String, i64>,
}

static CPU_HISTORY: OnceLock<RwLock<HashMap<String, CpuCounters>>> = OnceLock::new();

#[must_use]
fn secret_ref_for(label: &str) -> &'static str {
    if label.to_ascii_lowercase().contains("principal") || label.contains("VPS1") {
        "COOLIFY_VPS1_API_TOKEN"
    } else {
        "COOLIFY_API_TOKEN"
    }
}

#[must_use]
fn ssh_secret_ref_for(label: &str, config: &CoolifyConfig) -> Option<&'static str> {
    config.ssh_key_path.as_ref()?;
    if label.to_ascii_lowercase().contains("principal") || label.contains("VPS1") {
        Some("COOLIFY_VPS1_SSH_KEY_PATH")
    } else {
        Some("COOLIFY_SSH_KEY_PATH")
    }
}

#[must_use]
fn current_hourly_storage_window() -> bool {
    let ten_minute_window = Utc::now().timestamp() / 600;
    ten_minute_window % 6 == 0
}

fn sampler_command(include_storage: bool) -> String {
    let mut command = concat!(
        "printf '__CPU__\\n'; ",
        "awk '/^cpu /{print $2+$3+$4+$5+$6+$7+$8, $5+$6}' /proc/stat; ",
        "printf '__NPROC__\\n'; nproc 2>/dev/null || true; ",
        "printf '__MEM__\\n'; free -m | awk '/^Mem:/{print $3, $2}'; ",
        "printf '__DISK__\\n'; df -Pm / | awk 'NR==2{print $3, $2}'; ",
        "printf '__DOCKER__\\n'; ",
        "docker stats --no-stream --format '{{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}' 2>/dev/null || true;"
    )
    .to_string();

    if include_storage {
        command.push_str(
            " printf '__STORAGE__\\n'; du -sm /var/lib/docker/volumes/*/_data 2>/dev/null || true;",
        );
    }

    command
}

async fn fetch_server_snapshot(
    server_ip: &str,
    ssh_key_path: &str,
    include_storage: bool,
) -> Result<ServerSshSnapshot, String> {
    let mut command = tokio::process::Command::new("ssh");
    command
        .args([
            "-i",
            ssh_key_path,
            "-o",
            "StrictHostKeyChecking=accept-new",
            "-o",
            "ConnectTimeout=8",
            "-o",
            "BatchMode=yes",
            &format!("root@{server_ip}"),
            &sampler_command(include_storage),
        ])
        .stdin(Stdio::null());

    let output = tokio::time::timeout(SSH_TIMEOUT, command.output())
        .await
        .map_err(|_| format!("Timeout SSH sampler para {server_ip}"))?
        .map_err(|error| format!("SSH sampler fallo para {server_ip}: {error}"))?;

    if !output.status.success() && output.stdout.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("SSH sampler exit {}: {stderr}", output.status));
    }

    Ok(parse_sampler_output(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

fn parse_sampler_output(output: &str) -> ServerSshSnapshot {
    let mut snapshot = ServerSshSnapshot::default();
    let mut section = "";
    let mut docker_lines = Vec::new();

    for raw_line in output.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        match line {
            "__CPU__" | "__NPROC__" | "__MEM__" | "__DISK__" | "__DOCKER__" | "__STORAGE__" => {
                section = line;
                continue;
            }
            _ => {}
        }

        match section {
            "__CPU__" => {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let (Ok(total), Ok(idle)) = (parts[0].parse(), parts[1].parse()) {
                        snapshot.cpu_counters = Some(CpuCounters { total, idle });
                    }
                }
            }
            "__NPROC__" => snapshot.cpu_cores = line.parse::<f64>().ok(),
            "__MEM__" => {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    snapshot.ram_used_mb = parts[0].parse::<f64>().ok();
                    snapshot.ram_limit_mb = parts[1].parse::<f64>().ok();
                }
            }
            "__DISK__" => {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    snapshot.disk_used_mb = parts[0].parse::<f64>().ok();
                    snapshot.disk_limit_mb = parts[1].parse::<f64>().ok();
                }
            }
            "__DOCKER__" => docker_lines.push(line.to_string()),
            "__STORAGE__" => parse_storage_line(line, &mut snapshot.storage_by_uuid),
            _ => {}
        }
    }

    snapshot.containers = parse_docker_stats_public(&docker_lines.join("\n"));
    snapshot
}

fn parse_storage_line(line: &str, storage_by_uuid: &mut HashMap<String, i64>) {
    let cols: Vec<&str> = line.split_whitespace().collect();
    if cols.len() < 2 {
        return;
    }
    let Ok(mb) = cols[0].parse::<i64>() else {
        return;
    };
    let Some(folder_path) = cols[1].strip_suffix("/_data") else {
        return;
    };
    if let Some(volume_name) = folder_path.split('/').next_back() {
        let uuid = volume_name.split('_').next().unwrap_or(volume_name);
        *storage_by_uuid.entry(uuid.to_string()).or_insert(0) += mb;
    }
}

fn f64_to_i64_rounded(value: f64) -> i64 {
    if !value.is_finite() {
        return 0;
    }
    format!("{value:.0}").parse::<i64>().unwrap_or(i64::MAX)
}

fn f64_to_i32_rounded(value: f64) -> i32 {
    if !value.is_finite() {
        return 0;
    }
    format!("{value:.0}").parse::<i32>().unwrap_or(i32::MAX)
}

fn i64_to_f64(value: i64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}

async fn compute_server_cpu_percent(server_ip: &str, counters: Option<CpuCounters>) -> Option<f64> {
    let counters = counters?;
    let history = CPU_HISTORY.get_or_init(|| RwLock::new(HashMap::new()));
    let mut guard = history.write().await;
    let previous = guard.insert(server_ip.to_string(), counters)?;
    let total_delta = counters.total - previous.total;
    let idle_delta = counters.idle - previous.idle;
    if total_delta <= 0.0 || idle_delta < 0.0 {
        return None;
    }
    Some(((1.0 - idle_delta / total_delta) * 100.0).clamp(0.0, 100.0))
}

fn matching_containers<'a>(
    containers: &'a [ContainerStats],
    deployment_uuid: &str,
    service_name: &str,
) -> Vec<&'a ContainerStats> {
    containers
        .iter()
        .filter(|container| {
            container.name.contains(deployment_uuid) || container.name.contains(service_name)
        })
        .collect()
}

async fn record_bandwidth_delta(
    pool: &PgPool,
    server: &InfrastructureServerRecord,
    subscription: &HostingSubscription,
    deployment_uuid: &str,
    sampled_at: chrono::DateTime<Utc>,
    containers: &[&ContainerStats],
) -> Result<(), crate::errors::AppError> {
    let net_input_mb: f64 = containers
        .iter()
        .map(|container| container.net_input_mb)
        .sum();
    let net_output_mb: f64 = containers
        .iter()
        .map(|container| container.net_output_mb)
        .sum();

    if let Some(previous) =
        InfrastructureRepository::find_bandwidth_snapshot(pool, subscription.id, deployment_uuid)
            .await?
    {
        let rx_delta_mb = net_input_mb - previous.net_input_mb;
        let tx_delta_mb = net_output_mb - previous.net_output_mb;
        if rx_delta_mb >= 0.0 || tx_delta_mb >= 0.0 {
            InfrastructureRepository::add_bandwidth_delta(
                pool,
                subscription.id,
                f64_to_i64_rounded(rx_delta_mb.max(0.0) * 1_000_000.0),
                f64_to_i64_rounded(tx_delta_mb.max(0.0) * 1_000_000.0),
            )
            .await?;
        }
    }

    InfrastructureRepository::upsert_bandwidth_snapshot(
        pool,
        BandwidthSnapshotInput {
            subscription_id: subscription.id,
            deployment_uuid,
            server_id: server.id,
            net_input_mb,
            net_output_mb,
            sampled_at,
        },
    )
    .await
}

async fn sample_target(
    pool: &PgPool,
    http_client: &reqwest::Client,
    label: String,
    config: CoolifyConfig,
    subscriptions_by_uuid: &HashMap<String, HostingSubscription>,
    subscriptions_by_name: &HashMap<String, HostingSubscription>,
) -> Result<(), crate::errors::AppError> {
    let server = InfrastructureRepository::upsert_configured_server(
        pool,
        ConfiguredServerInput {
            label: &label,
            config: &config,
            secret_ref: secret_ref_for(&label),
            ssh_secret_ref: ssh_secret_ref_for(&label, &config),
        },
    )
    .await?;
    InfrastructureRepository::ensure_capacity_row(pool, &server).await?;

    let services = CoolifyService::list_services(http_client, &config)
        .await
        .map_err(|error| {
            tracing::warn!("[infra-metrics] Coolify {label} no disponible: {error}");
            crate::errors::AppError::ServiceUnavailable(error.to_string())
        })?;

    let Some(ssh_key_path) = config.ssh_key_path.as_deref() else {
        tracing::warn!(
            "[infra-metrics] {label} sin SSH key; se sincroniza inventario sin muestras"
        );
        return Ok(());
    };
    let sampled_at = Utc::now();
    let snapshot = fetch_server_snapshot(
        &config.server_ip,
        ssh_key_path,
        current_hourly_storage_window(),
    )
    .await
    .map_err(|error| {
        tracing::warn!("[infra-metrics] {error}");
        crate::errors::AppError::ServiceUnavailable(error)
    })?;

    let cpu_percent = compute_server_cpu_percent(&config.server_ip, snapshot.cpu_counters).await;
    InfrastructureRepository::insert_sample(
        pool,
        ResourceSampleInput {
            entity_kind: "server",
            server_id: server.id,
            deployment_uuid: None,
            sampled_at,
            cpu_percent,
            ram_used_mb: snapshot.ram_used_mb,
            ram_limit_mb: snapshot.ram_limit_mb,
            disk_used_mb: snapshot.disk_used_mb,
            disk_limit_mb: snapshot.disk_limit_mb,
        },
    )
    .await?;

    if let Some(server_uuid) = server.coolify_server_uuid.as_deref() {
        InfrastructureRepository::update_capacity_specs(
            pool,
            server_uuid,
            snapshot.cpu_cores,
            snapshot.ram_limit_mb.map(f64_to_i32_rounded),
            snapshot.disk_limit_mb.map(f64_to_i32_rounded),
        )
        .await?;
    }

    for service in services {
        record_deployment_sample(
            pool,
            &server,
            &service,
            &snapshot,
            sampled_at,
            subscriptions_by_uuid,
            subscriptions_by_name,
        )
        .await?;
    }

    Ok(())
}

async fn record_deployment_sample(
    pool: &PgPool,
    server: &InfrastructureServerRecord,
    service: &CoolifyServiceSummary,
    snapshot: &ServerSshSnapshot,
    sampled_at: chrono::DateTime<Utc>,
    subscriptions_by_uuid: &HashMap<String, HostingSubscription>,
    subscriptions_by_name: &HashMap<String, HostingSubscription>,
) -> Result<(), crate::errors::AppError> {
    let containers = matching_containers(&snapshot.containers, &service.uuid, &service.name);
    if containers.is_empty() && !snapshot.storage_by_uuid.contains_key(&service.uuid) {
        return Ok(());
    }

    let subscription = subscriptions_by_uuid
        .get(&service.uuid)
        .or_else(|| subscriptions_by_name.get(&service.name));

    let cpu_percent = containers
        .iter()
        .map(|container| container.cpu_percent)
        .sum();
    let ram_used_mb = containers
        .iter()
        .map(|container| container.mem_used_mb)
        .sum();
    let ram_limit_mb = containers
        .iter()
        .map(|container| container.mem_limit_mb)
        .sum();
    let storage_used_mb = snapshot
        .storage_by_uuid
        .get(&service.uuid)
        .map(|value| i64_to_f64(*value));
    let storage_limit_mb = subscription.map(|sub| f64::from(sub.storage_limit_mb));

    InfrastructureRepository::insert_sample(
        pool,
        ResourceSampleInput {
            entity_kind: "deployment",
            server_id: server.id,
            deployment_uuid: Some(&service.uuid),
            sampled_at,
            cpu_percent: Some(cpu_percent),
            ram_used_mb: Some(ram_used_mb),
            ram_limit_mb: Some(ram_limit_mb),
            disk_used_mb: storage_used_mb,
            disk_limit_mb: storage_limit_mb,
        },
    )
    .await?;

    if let Some(subscription) = subscription {
        record_bandwidth_delta(
            pool,
            server,
            subscription,
            &service.uuid,
            sampled_at,
            &containers,
        )
        .await?;
    }

    Ok(())
}

pub async fn sample_infrastructure_once(
    pool: PgPool,
    http_client: reqwest::Client,
    vps1_config: Option<CoolifyConfig>,
    default_config: Option<CoolifyConfig>,
) -> Result<(), crate::errors::AppError> {
    let targets = coolify_server_targets(vps1_config.as_ref(), default_config.as_ref());
    if targets.is_empty() {
        tracing::warn!("[infra-metrics] Coolify no configurado; sampler desactivado");
        return Ok(());
    }

    let subscriptions = HostingRepository::list_all(&pool).await?;
    let subscriptions_by_uuid: HashMap<String, HostingSubscription> = subscriptions
        .iter()
        .filter_map(|subscription| {
            subscription
                .server_uuid
                .as_ref()
                .map(|server_uuid| (server_uuid.clone(), subscription.clone()))
        })
        .collect();
    let subscriptions_by_name: HashMap<String, HostingSubscription> = subscriptions
        .iter()
        .filter_map(|subscription| {
            subscription
                .coolify_site_name
                .as_ref()
                .map(|site_name| (site_name.clone(), subscription.clone()))
        })
        .collect();

    let futures = targets.into_iter().map(|target| {
        sample_target(
            &pool,
            &http_client,
            target.label,
            target.config.clone(),
            &subscriptions_by_uuid,
            &subscriptions_by_name,
        )
    });

    for result in futures::future::join_all(futures).await {
        if let Err(error) = result {
            tracing::warn!("[infra-metrics] muestra parcial fallida: {error}");
        }
    }

    InfrastructureRepository::purge_old_samples(&pool).await?;
    Ok(())
}

pub async fn infrastructure_metrics_loop(
    pool: PgPool,
    http_client: reqwest::Client,
    vps1_config: Option<CoolifyConfig>,
    default_config: Option<CoolifyConfig>,
) {
    loop {
        if let Err(error) = sample_infrastructure_once(
            pool.clone(),
            http_client.clone(),
            vps1_config.clone(),
            default_config.clone(),
        )
        .await
        {
            tracing::warn!("[infra-metrics] ciclo incompleto: {error}");
        }
        tokio::time::sleep(SAMPLER_INTERVAL).await;
    }
}
