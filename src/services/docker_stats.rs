/* [114A-15+] Docker container stats via SSH.
 * Conecta al VPS por SSH (tokio::process::Command) y ejecuta `docker stats --no-stream`.
 * Cache en memoria con TTL de 30s para evitar SSH en cada request.
 * Requiere: ssh binario disponible + COOLIFY_SSH_KEY_PATH apuntando a la clave SSH del servidor. */

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

const CACHE_TTL: Duration = Duration::from_secs(30);

/* ── Tipos ──────────────────────────────── */

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct ContainerStats {
    pub name: String,
    pub cpu_percent: f64,
    pub mem_used_mb: f64,
    pub mem_limit_mb: f64,
    pub net_input_mb: f64,
    pub net_output_mb: f64,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct HostingResourceStats {
    /// Stats por contenedor (wordpress, mariadb, ssh)
    pub containers: Vec<ContainerStats>,
    /// CPU total (suma de todos los contenedores)
    pub total_cpu_percent: f64,
    /// RAM usada total en MB
    pub total_ram_used_mb: f64,
    /// RAM límite total en MB
    pub total_ram_limit_mb: f64,
}

/* ── Cache ──────────────────────────────── */

struct CacheEntry {
    stats: HostingResourceStats,
    fetched_at: Instant,
}

#[derive(Clone)]
pub struct DockerStatsCache {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

impl Default for DockerStatsCache {
    fn default() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl DockerStatsCache {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get(&self, key: &str) -> Option<HostingResourceStats> {
        let entries = self.entries.read().await;
        entries.get(key).and_then(|entry| {
            if entry.fetched_at.elapsed() < CACHE_TTL {
                Some(entry.stats.clone())
            } else {
                None
            }
        })
    }

    pub async fn set(&self, key: String, stats: HostingResourceStats) {
        let mut entries = self.entries.write().await;
        entries.insert(key, CacheEntry {
            stats,
            fetched_at: Instant::now(),
        });

        /* Limpieza periódica: eliminar entradas expiradas si hay >50 */
        if entries.len() > 50 {
            entries.retain(|_, e| e.fetched_at.elapsed() < CACHE_TTL * 3);
        }
    }
}

/* ── Fetch via SSH ─────────────────────── */

/// Obtiene stats de contenedores Docker de un hosting específico via SSH.
/// `server_ip`: IP del VPS donde corre el hosting.
/// `ssh_key_path`: Ruta a la clave SSH privada.
/// `container_prefix`: Prefijo de los contenedores (ej: "hosting-midominio").
pub async fn fetch_docker_stats(
    server_ip: &str,
    ssh_key_path: &str,
    container_prefix: &str,
) -> Result<HostingResourceStats, String> {
    /* docker stats con formato parseable (tab-separated) */
    let docker_cmd = format!(
        "docker stats --no-stream --format '{{{{.Name}}}}\\t{{{{.CPUPerc}}}}\\t{{{{.MemUsage}}}}\\t{{{{.NetIO}}}}' 2>/dev/null | grep '^{container_prefix}'"
    );

    let output = tokio::process::Command::new("ssh")
        .args([
            "-i", ssh_key_path,
            "-o", "StrictHostKeyChecking=accept-new",
            "-o", "ConnectTimeout=5",
            "-o", "BatchMode=yes",
            &format!("root@{server_ip}"),
            &docker_cmd,
        ])
        .output()
        .await
        .map_err(|e| format!("SSH ejecutar fallo: {e}"))?;

    if !output.status.success() && output.stdout.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        /* grep retorna exit 1 si no hay matches — no es error real */
        if output.status.code() == Some(1) && stderr.is_empty() {
            return Ok(HostingResourceStats {
                containers: vec![],
                total_cpu_percent: 0.0,
                total_ram_used_mb: 0.0,
                total_ram_limit_mb: 0.0,
            });
        }
        return Err(format!("SSH fallo (exit {}): {stderr}", output.status));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let containers = parse_docker_stats(&stdout);

    let total_cpu = containers.iter().map(|c| c.cpu_percent).sum();
    let total_ram_used = containers.iter().map(|c| c.mem_used_mb).sum();
    let total_ram_limit = containers.iter().map(|c| c.mem_limit_mb).sum();

    Ok(HostingResourceStats {
        containers,
        total_cpu_percent: total_cpu,
        total_ram_used_mb: total_ram_used,
        total_ram_limit_mb: total_ram_limit,
    })
}

/* ── Parser ────────────────────────────── */

/// Parsea la salida de `docker stats --no-stream` en formato tab-separated.
/// Formato esperado: `NAME\tCPU%\tMEM_USAGE / MEM_LIMIT\tNET_I / NET_O`
fn parse_docker_stats(output: &str) -> Vec<ContainerStats> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 4 {
                return None;
            }

            let name = parts[0].trim().to_string();
            let cpu = parse_percent(parts[1]);
            let (mem_used, mem_limit) = parse_mem_usage(parts[2]);
            let (net_in, net_out) = parse_net_io(parts[3]);

            Some(ContainerStats {
                name,
                cpu_percent: cpu,
                mem_used_mb: mem_used,
                mem_limit_mb: mem_limit,
                net_input_mb: net_in,
                net_output_mb: net_out,
            })
        })
        .collect()
}

/// "12.34%" → 12.34
fn parse_percent(s: &str) -> f64 {
    s.trim().trim_end_matches('%').parse().unwrap_or(0.0)
}

/// "123.4MiB / 512MiB" → (123.4, 512.0) en MB
fn parse_mem_usage(s: &str) -> (f64, f64) {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 2 {
        return (0.0, 0.0);
    }
    (parse_size_to_mb(parts[0]), parse_size_to_mb(parts[1]))
}

/// "1.23MB / 4.56MB" → (1.23, 4.56)
fn parse_net_io(s: &str) -> (f64, f64) {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 2 {
        return (0.0, 0.0);
    }
    (parse_size_to_mb(parts[0]), parse_size_to_mb(parts[1]))
}

/// Convierte strings como "123.4MiB", "1.5GiB", "500kB" a megabytes
fn parse_size_to_mb(s: &str) -> f64 {
    let s = s.trim();
    if let Some(val) = s.strip_suffix("GiB") {
        val.trim().parse::<f64>().unwrap_or(0.0) * 1024.0
    } else if let Some(val) = s.strip_suffix("MiB") {
        val.trim().parse::<f64>().unwrap_or(0.0)
    } else if let Some(val) = s.strip_suffix("KiB") {
        val.trim().parse::<f64>().unwrap_or(0.0) / 1024.0
    } else if let Some(val) = s.strip_suffix("GB") {
        val.trim().parse::<f64>().unwrap_or(0.0) * 1000.0
    } else if let Some(val) = s.strip_suffix("MB") {
        val.trim().parse::<f64>().unwrap_or(0.0)
    } else if let Some(val) = s.strip_suffix("kB") {
        val.trim().parse::<f64>().unwrap_or(0.0) / 1000.0
    } else if let Some(val) = s.strip_suffix('B') {
        val.trim().parse::<f64>().unwrap_or(0.0) / 1_000_000.0
    } else {
        0.0
    }
}

/* [154A-3] Obtener uso de almacenamiento (MB) del contenedor WordPress via SSH + du.
 * Ejecuta `docker exec {prefix}-wordpress-1 du -sm /var/www/html` en el servidor. */
pub async fn fetch_storage_usage(
    server_ip: &str,
    ssh_key_path: &str,
    container_prefix: &str,
) -> Result<i64, String> {
    let docker_cmd = format!(
        "docker exec {container_prefix}-wordpress-1 du -sm /var/www/html 2>/dev/null | awk '{{print $1}}'"
    );

    let output = tokio::process::Command::new("ssh")
        .args([
            "-i", ssh_key_path,
            "-o", "StrictHostKeyChecking=accept-new",
            "-o", "ConnectTimeout=5",
            "-o", "BatchMode=yes",
            &format!("root@{server_ip}"),
            &docker_cmd,
        ])
        .output()
        .await
        .map_err(|e| format!("SSH storage check failed: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if stdout.is_empty() {
        return Err("Empty du output — container may not be running".into());
    }

    stdout
        .parse::<i64>()
        .map_err(|e| format!("Failed to parse du output '{stdout}': {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_percent_standard() {
        assert!((parse_percent("12.34%") - 12.34).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_percent_zero() {
        assert!((parse_percent("0.00%") - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_mem_mib() {
        let (used, limit) = parse_mem_usage("123.4MiB / 512MiB");
        assert!((used - 123.4).abs() < 0.01);
        assert!((limit - 512.0).abs() < 0.01);
    }

    #[test]
    fn parse_mem_gib() {
        let (used, limit) = parse_mem_usage("1.5GiB / 4GiB");
        assert!((used - 1536.0).abs() < 0.01);
        assert!((limit - 4096.0).abs() < 0.01);
    }

    #[test]
    fn parse_net_mb() {
        let (input, output) = parse_net_io("1.23MB / 4.56MB");
        assert!((input - 1.23).abs() < 0.01);
        assert!((output - 4.56).abs() < 0.01);
    }

    #[test]
    fn parse_net_kb() {
        let (input, output) = parse_net_io("500kB / 1.2MB");
        assert!((input - 0.5).abs() < 0.01);
        assert!((output - 1.2).abs() < 0.01);
    }

    #[test]
    fn parse_full_docker_stats_output() {
        let output = "hosting-test-wordpress-1\t0.15%\t128.5MiB / 512MiB\t1.2MB / 500kB\n\
                       hosting-test-mariadb-1\t0.30%\t256MiB / 512MiB\t800kB / 200kB\n\
                       hosting-test-ssh-1\t0.01%\t15.2MiB / 256MiB\t100kB / 50kB\n";

        let stats = parse_docker_stats(output);
        assert_eq!(stats.len(), 3);

        assert_eq!(stats[0].name, "hosting-test-wordpress-1");
        assert!((stats[0].cpu_percent - 0.15).abs() < 0.01);
        assert!((stats[0].mem_used_mb - 128.5).abs() < 0.1);
        assert!((stats[0].mem_limit_mb - 512.0).abs() < 0.1);

        assert_eq!(stats[1].name, "hosting-test-mariadb-1");
        assert!((stats[1].cpu_percent - 0.30).abs() < 0.01);
    }

    #[test]
    fn parse_empty_output() {
        let stats = parse_docker_stats("");
        assert!(stats.is_empty());
    }

    #[test]
    fn parse_size_edge_cases() {
        assert!((parse_size_to_mb("0B") - 0.0).abs() < f64::EPSILON);
        assert!((parse_size_to_mb("1024KiB") - 1.0).abs() < 0.01);
        assert!((parse_size_to_mb("1GiB") - 1024.0).abs() < 0.01);
        assert!((parse_size_to_mb("invalid") - 0.0).abs() < f64::EPSILON);
    }
}
