/* [265A-1] Burst dinamico de CPU para hostings Coolify.
 * Lee snapshots del sampler y aplica `docker update --cpus` al contenedor
 * principal (`site` o `wordpress`) solo cuando el host tiene holgura sostenida.
 * Si el host se tensiona o el sitio deja de saturar el cap actual, restaura el
 * baseline comercial del plan. */

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::errors::AppError;
use crate::repositories::{CpuBurstCandidate, HostingRepository, InfrastructureRepository};
use crate::services::coolify::CoolifyConfig;
use crate::services::infrastructure::coolify_server_targets;

const CPU_BURST_INTERVAL: Duration = Duration::from_mins(5);
const CPU_BURST_RAISE_WINDOW: Duration = Duration::from_mins(10);
const CPU_BURST_LOWER_WINDOW: Duration = Duration::from_mins(10);
const CPU_BURST_MAX_SAMPLE_AGE_MINUTES: i64 = 20;
const CPU_ACTIVATION_UTILIZATION: f64 = 0.85;
const CPU_DEACTIVATION_UTILIZATION: f64 = 0.35;
const SERVER_LOW_PRESSURE_PCT: f64 = 45.0;
const SERVER_HIGH_PRESSURE_PCT: f64 = 70.0;
const SERVER_RESERVE_RATIO: f64 = 0.25;
const SERVER_RESERVE_MIN_CORES: f64 = 1.0;
const CPU_STEP_CORES: f64 = 0.25;
const CPU_EPSILON: f64 = 0.01;
/* [265A-4] En VPS cargados, `docker compose ps` + `docker update` puede tardar
 * mas de 15s. Si el future de `output()` vence sin matar el proceso, el SSH
 * sigue vivo y aplica el cambio fuera del control del loop, dejando un falso
 * negativo en logs/eventos. */
const CPU_SSH_TIMEOUT: Duration = Duration::from_secs(45);

#[derive(Debug, Clone, Default)]
struct CpuBurstState {
    raise_since: Option<Instant>,
    lower_since: Option<Instant>,
    last_requested_target: Option<f64>,
}

static CPU_STATE_MAP: OnceLock<RwLock<HashMap<Uuid, CpuBurstState>>> = OnceLock::new();

fn state_map() -> &'static RwLock<HashMap<Uuid, CpuBurstState>> {
    CPU_STATE_MAP.get_or_init(|| RwLock::new(HashMap::new()))
}

fn ssh_key_for(
    server_ip: &str,
    vps1: Option<&CoolifyConfig>,
    default: Option<&CoolifyConfig>,
) -> Option<String> {
    coolify_server_targets(vps1, default)
        .into_iter()
        .find(|target| server_ip == target.config.server_ip)
        .and_then(|target| target.config.ssh_key_path.clone())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/* [265A-3] Coolify usa el deployment UUID como `com.docker.compose.project` en
 * stacks runtime/legacy. El burst intenta primero ese project y solo cae al
 * slug del sitio cuando realmente coincide con el compose project. */
fn compose_project_candidates(deployment_uuid: &str, site_name: &str) -> Vec<String> {
    let mut candidates = Vec::new();

    for value in [deployment_uuid, site_name] {
        let value = value.trim();
        if value.is_empty() || candidates.iter().any(|existing| existing == value) {
            continue;
        }
        candidates.push(value.to_string());
    }

    candidates
}

fn usize_to_f64(value: usize) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(1.0)
}

fn round_cpu_target(value: f64) -> f64 {
    ((value / CPU_STEP_CORES).floor() * CPU_STEP_CORES).max(CPU_STEP_CORES)
}

fn reserve_cores(total_cores: Option<f64>) -> f64 {
    let Some(total_cores) = total_cores.filter(|value| *value > 0.0) else {
        return 0.0;
    };

    (total_cores * SERVER_RESERVE_RATIO).max(SERVER_RESERVE_MIN_CORES)
}

fn available_cores(total_cores: Option<f64>) -> Option<f64> {
    total_cores
        .filter(|value| *value > 0.0)
        .map(|value| (value - reserve_cores(Some(value))).max(0.0))
}

fn normalized_cpu_utilization(cpu_percent: Option<f64>, limit_cores: f64) -> Option<f64> {
    if limit_cores <= 0.0 {
        return None;
    }

    cpu_percent
        .filter(|value| *value >= 0.0)
        .map(|value| value / (limit_cores * 100.0))
}

fn sample_is_fresh(sampled_at: Option<DateTime<Utc>>) -> bool {
    sampled_at.is_some_and(|value| {
        value >= Utc::now() - chrono::Duration::minutes(CPU_BURST_MAX_SAMPLE_AGE_MINUTES)
    })
}

fn server_low_pressure(candidate: &CpuBurstCandidate) -> bool {
    sample_is_fresh(candidate.server_sampled_at)
        && candidate
            .server_cpu_percent
            .is_some_and(|value| value < SERVER_LOW_PRESSURE_PCT)
}

fn server_high_pressure(candidate: &CpuBurstCandidate) -> bool {
    !sample_is_fresh(candidate.server_sampled_at)
        || candidate
            .server_cpu_percent
            .is_some_and(|value| value >= SERVER_HIGH_PRESSURE_PCT)
}

fn current_site_limit(candidate: &CpuBurstCandidate) -> Option<f64> {
    candidate
        .current_site_cpu_limit_cores
        .filter(|value| *value > 0.0)
        .map(round_cpu_target)
}

fn baseline_site_limit(candidate: &CpuBurstCandidate) -> f64 {
    round_cpu_target(candidate.baseline_site_cpu_cores.max(CPU_STEP_CORES))
}

fn is_cpu_demander(candidate: &CpuBurstCandidate) -> bool {
    let Some(current_limit) = current_site_limit(candidate) else {
        return false;
    };

    if !sample_is_fresh(candidate.deployment_sampled_at) || !server_low_pressure(candidate) {
        return false;
    }

    normalized_cpu_utilization(candidate.deployment_cpu_percent, current_limit)
        .is_some_and(|value| value >= CPU_ACTIVATION_UTILIZATION)
}

fn demanders_by_server(candidates: &[CpuBurstCandidate]) -> HashMap<Uuid, usize> {
    let mut counts = HashMap::new();
    for candidate in candidates {
        if is_cpu_demander(candidate) {
            *counts.entry(candidate.server_id).or_insert(0) += 1;
        }
    }
    counts
}

fn compute_burst_target(
    baseline_cores: f64,
    server_cpu_cores: Option<f64>,
    demander_count: usize,
) -> f64 {
    let demander_count = demander_count.max(1);

    if let Some(available) = available_cores(server_cpu_cores) {
        let fair_share = (available / usize_to_f64(demander_count)).max(baseline_cores);
        return round_cpu_target(fair_share.min(available.max(baseline_cores)));
    }

    round_cpu_target((baseline_cores * 2.0).max(baseline_cores))
}

fn desired_cpu_limit(
    candidate: &CpuBurstCandidate,
    demanders: &HashMap<Uuid, usize>,
) -> Option<f64> {
    let baseline = baseline_site_limit(candidate);
    let current_limit = current_site_limit(candidate)?;
    let boosted = current_limit > baseline + CPU_EPSILON;

    if !sample_is_fresh(candidate.deployment_sampled_at) {
        return None;
    }

    let utilization = normalized_cpu_utilization(candidate.deployment_cpu_percent, current_limit)?;

    if server_high_pressure(candidate) || utilization <= CPU_DEACTIVATION_UTILIZATION {
        return boosted.then_some(baseline);
    }

    if !server_low_pressure(candidate) {
        return None;
    }

    if utilization < CPU_ACTIVATION_UTILIZATION {
        return boosted.then_some(baseline);
    }

    let demander_count = demanders.get(&candidate.server_id).copied().unwrap_or(1).max(1);
    let target = compute_burst_target(baseline, candidate.server_cpu_cores, demander_count);
    ((target - current_limit).abs() > CPU_EPSILON).then_some(target)
}

async fn run_ssh(server_ip: &str, ssh_key_path: &str, cmd: &str) -> Result<String, String> {
    let mut command = tokio::process::Command::new("ssh");
    command
        .kill_on_drop(true)
        .args([
            "-i",
            ssh_key_path,
            "-o",
            "StrictHostKeyChecking=accept-new",
            "-o",
            "ConnectTimeout=5",
            "-o",
            "BatchMode=yes",
            &format!("root@{server_ip}"),
            cmd,
        ])
        .stdin(Stdio::null());

    let output = tokio::time::timeout(CPU_SSH_TIMEOUT, command.output())
        .await
        .map_err(|_| format!("cpu burst ssh timeout {server_ip}"))?
        .map_err(|e| format!("cpu burst ssh error {server_ip}: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("cpu burst ssh exit {}: {stderr}", output.status));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn cpu_update_script(deployment_uuid: &str, site_name: &str, cpu_cores: f64) -> String {
        let projects = compose_project_candidates(deployment_uuid, site_name)
                .into_iter()
                .map(|value| shell_quote(&value))
                .collect::<Vec<_>>()
                .join(" ");
    let cpu_cores = format!("{cpu_cores:.2}");
    format!(
                r#"CID=""
for PROJECT in {projects}; do
    [ -n "$PROJECT" ] || continue
    CID=$(docker compose -p "$PROJECT" ps -q site 2>/dev/null | head -1)
    if [ -z "$CID" ]; then
        CID=$(docker compose -p "$PROJECT" ps -q wordpress 2>/dev/null | head -1)
    fi
    if [ -z "$CID" ]; then
        CID=$(docker compose -p "$PROJECT" ps -q app 2>/dev/null | head -1)
    fi
    if [ -n "$CID" ]; then
        break
    fi
done
if [ -z "$CID" ]; then
  echo "container_not_found"
  exit 10
fi
docker update --cpus {cpu_cores} "$CID" >/dev/null
docker inspect -f '{{{{.Name}}}}' "$CID" 2>/dev/null | sed 's#^/##'"#,
    )
}

async fn apply_site_cpu_limit(
    server_ip: &str,
    ssh_key_path: &str,
    deployment_uuid: &str,
    site_name: &str,
    cpu_cores: f64,
) -> Result<String, String> {
    let script = cpu_update_script(deployment_uuid, site_name, cpu_cores);
    run_ssh(server_ip, ssh_key_path, &script).await
}

async fn apply_target(
    pool: &PgPool,
    candidate: &CpuBurstCandidate,
    ssh_key: &str,
    target_cores: f64,
) -> bool {
    match apply_site_cpu_limit(
        &candidate.server_ip,
        ssh_key,
        &candidate.deployment_uuid,
        &candidate.coolify_site_name,
        target_cores,
    )
    .await
    {
        Ok(container_name) => {
            let baseline = baseline_site_limit(candidate);
            let event_type = if target_cores <= baseline + CPU_EPSILON {
                "cpu_burst_restored"
            } else {
                "cpu_burst_applied"
            };
            let details = serde_json::json!({
                "target_cpu_cores": target_cores,
                "baseline_cpu_cores": baseline,
                "observed_site_cpu_limit_cores": candidate.current_site_cpu_limit_cores,
                "deployment_cpu_percent": candidate.deployment_cpu_percent,
                "server_cpu_percent": candidate.server_cpu_percent,
                "container_name": container_name,
                "site_name": candidate.coolify_site_name,
            });
            let _ = HostingRepository::add_event(pool, candidate.subscription_id, event_type, Some(details)).await;
            tracing::info!(
                "[cpu-burst] {} -> {:.2} cores en {}",
                candidate.subscription_id,
                target_cores,
                container_name
            );
            true
        }
        Err(error) => {
            tracing::warn!(
                "[cpu-burst] {} {}: {error}",
                candidate.subscription_id,
                candidate.coolify_site_name
            );
            false
        }
    }
}

async fn evaluate_cpu_burst(
    pool: &PgPool,
    vps1_config: Option<&CoolifyConfig>,
    default_config: Option<&CoolifyConfig>,
) -> Result<(), AppError> {
    let candidates = InfrastructureRepository::cpu_burst_candidates(pool).await?;
    if candidates.is_empty() {
        return Ok(());
    }

    let demanders = demanders_by_server(&candidates);
    let now = Instant::now();
    let mut state_map = state_map().write().await;
    let mut ssh_cache: HashMap<String, String> = HashMap::new();

    for candidate in &candidates {
        let Some(current_limit) = current_site_limit(candidate) else {
            continue;
        };

        let state = state_map.entry(candidate.subscription_id).or_default();
        if let Some(last_requested) = state.last_requested_target {
            if (last_requested - current_limit).abs() <= CPU_EPSILON {
                state.last_requested_target = None;
            }
        }

        let Some(desired_limit) = desired_cpu_limit(candidate, &demanders) else {
            state.raise_since = None;
            state.lower_since = None;
            continue;
        };

        if state
            .last_requested_target
            .is_some_and(|value| (value - desired_limit).abs() <= CPU_EPSILON)
        {
            state.raise_since = None;
            state.lower_since = None;
            continue;
        }

        let is_raise = desired_limit > current_limit + CPU_EPSILON;
        let timer = if is_raise {
            state.lower_since = None;
            &mut state.raise_since
        } else {
            state.raise_since = None;
            &mut state.lower_since
        };

        let start = timer.get_or_insert(now);
        let elapsed = now.duration_since(*start);
        let required = if is_raise {
            CPU_BURST_RAISE_WINDOW
        } else {
            CPU_BURST_LOWER_WINDOW
        };

        if elapsed < required {
            continue;
        }

        let ssh_key = if let Some(existing) = ssh_cache.get(&candidate.server_ip) {
            existing.clone()
        } else {
            let Some(ssh_key) = ssh_key_for(&candidate.server_ip, vps1_config, default_config) else {
                tracing::warn!("[cpu-burst] Sin SSH key para {}", candidate.server_ip);
                continue;
            };
            ssh_cache.insert(candidate.server_ip.clone(), ssh_key.clone());
            ssh_key
        };

        if apply_target(pool, candidate, &ssh_key, desired_limit).await {
            state.last_requested_target = Some(desired_limit);
            state.raise_since = None;
            state.lower_since = None;
        }
    }

    Ok(())
}

pub async fn cpu_burst_loop(
    pool: PgPool,
    vps1_config: Option<CoolifyConfig>,
    default_config: Option<CoolifyConfig>,
) {
    loop {
        if let Err(error) =
            evaluate_cpu_burst(&pool, vps1_config.as_ref(), default_config.as_ref()).await
        {
            tracing::warn!("[cpu-burst] ciclo: {error}");
        }

        tokio::time::sleep(CPU_BURST_INTERVAL).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate() -> CpuBurstCandidate {
        CpuBurstCandidate {
            subscription_id: Uuid::nil(),
            deployment_uuid: "dep".into(),
            coolify_site_name: "hosting-test".into(),
            server_ip: "127.0.0.1".into(),
            server_id: Uuid::nil(),
            server_cpu_cores: Some(8.0),
            server_cpu_percent: Some(20.0),
            server_sampled_at: Some(Utc::now()),
            baseline_site_cpu_cores: 0.5,
            current_site_cpu_limit_cores: Some(0.5),
            deployment_cpu_percent: Some(50.0),
            deployment_sampled_at: Some(Utc::now()),
        }
    }

    #[test]
    fn normalized_utilization_respects_cpu_limit() {
        let utilization = normalized_cpu_utilization(Some(50.0), 0.5).unwrap_or_default();
        assert!((utilization - 1.0).abs() < 0.001);
    }

    #[test]
    fn compute_burst_target_keeps_host_reserve() {
        let target = compute_burst_target(0.5, Some(8.0), 1);
        assert_eq!(target, 6.0);
    }

    #[test]
    fn desired_cpu_limit_restores_to_baseline_when_pressure_rises() {
        let mut row = candidate();
        row.current_site_cpu_limit_cores = Some(2.0);
        row.server_cpu_percent = Some(85.0);
        row.deployment_cpu_percent = Some(30.0);
        let desired = desired_cpu_limit(&row, &HashMap::new());
        assert_eq!(desired, Some(0.5));
    }

    #[test]
    fn desired_cpu_limit_requests_burst_when_site_hits_cap() {
        let row = candidate();
        let demanders = HashMap::from([(row.server_id, 1_usize)]);
        let desired = desired_cpu_limit(&row, &demanders).unwrap_or_default();
        assert_eq!(desired, 6.0);
    }

    #[test]
    fn compose_project_candidates_prefers_deployment_uuid() {
        let candidates = compose_project_candidates("v77j8dfkb8rat8mlhzoid2eh", "hosting-0fa1d5da");
        assert_eq!(candidates, vec!["v77j8dfkb8rat8mlhzoid2eh", "hosting-0fa1d5da"]);
    }

    #[test]
    fn compose_project_candidates_deduplicates_identifiers() {
        let candidates = compose_project_candidates("hosting-0fa1d5da", "hosting-0fa1d5da");
        assert_eq!(candidates, vec!["hosting-0fa1d5da"]);
    }
}