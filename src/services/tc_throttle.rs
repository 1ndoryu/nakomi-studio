/* [225A-4+F7] Throttle anti-abuso via tc qdisc sobre veth del contenedor.
 * SSH al servidor, encuentra el veth del contenedor por nombre del proyecto
 * Docker Compose, y aplica/remueve un qdisc TBF para limitar velocidad.
 * Sin notificaciones al cliente — solo registros internos y eventos admin. */

use std::process::Stdio;
use std::time::Duration;

const TC_SSH_TIMEOUT: Duration = Duration::from_secs(15);

fn find_veth_script(site_name: &str) -> String {
    format!(
        r#"CID=$(docker compose -p {site_name} ps -q 2>/dev/null | head -1)
if [ -z "$CID" ]; then
  CID=$(docker compose -p {site_name} ps -q nginx 2>/dev/null || docker compose -p {site_name} ps -q wp 2>/dev/null)
fi
if [ -n "$CID" ]; then
  PID=$(docker inspect -f '{{{{.State.Pid}}}}' "$CID" 2>/dev/null)
  IFACE=$(nsenter -t "$PID" -n ip -o route show to default 2>/dev/null | awk '{{print $5}}')
  IDX=$(nsenter -t "$PID" -n cat /sys/class/net/"$IFACE"/iflink 2>/dev/null)
  if [ -n "$IDX" ]; then
    ip link show 2>/dev/null | awk -F': ' -v idx="$IDX" '$1 == idx {{print $2}}' | cut -d@ -f1
  fi
fi"#,
    )
}

fn set_rate_limit_script(veth: &str, rate_mbps: u32) -> String {
    format!(
        "tc qdisc replace dev {veth} root tbf rate {rate_mbps}mbit burst 32kbit latency 400ms 2>/dev/null || tc qdisc add dev {veth} root tbf rate {rate_mbps}mbit burst 32kbit latency 400ms",
    )
}

fn remove_rate_limit_script(veth: &str) -> String {
    format!("tc qdisc del dev {veth} root 2>/dev/null || true")
}

static LIST_THROTTLED_CMD: &str = "tc qdisc show | grep -E 'tbf|htb' | awk '{print $3}' | sort -u";

async fn run_ssh(server_ip: &str, ssh_key_path: &str, cmd: &str) -> Result<String, String> {
    let mut command = tokio::process::Command::new("ssh");
    command
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

    let output = tokio::time::timeout(TC_SSH_TIMEOUT, command.output())
        .await
        .map_err(|_| format!("tc ssh timeout {server_ip}"))?
        .map_err(|e| format!("tc ssh error {server_ip}: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("tc ssh exit {}: {stderr}", output.status));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub async fn find_veth(
    server_ip: &str,
    ssh_key_path: &str,
    site_name: &str,
) -> Result<String, String> {
    let script = find_veth_script(site_name);
    run_ssh(server_ip, ssh_key_path, &script).await
}

pub async fn set_rate_limit(
    server_ip: &str,
    ssh_key_path: &str,
    veth: &str,
    rate_mbps: u32,
) -> Result<(), String> {
    let script = set_rate_limit_script(veth, rate_mbps);
    let output = run_ssh(server_ip, ssh_key_path, &script).await?;
    if output.contains("RTNETLINK") || output.contains("Error") {
        return Err(format!("tc set_rate_limit failed: {output}"));
    }
    Ok(())
}

pub async fn remove_rate_limit(
    server_ip: &str,
    ssh_key_path: &str,
    veth: &str,
) -> Result<(), String> {
    let script = remove_rate_limit_script(veth);
    run_ssh(server_ip, ssh_key_path, &script).await?;
    Ok(())
}

pub async fn list_throttled(server_ip: &str, ssh_key_path: &str) -> Result<Vec<String>, String> {
    let output = run_ssh(server_ip, ssh_key_path, LIST_THROTTLED_CMD).await?;
    if output.is_empty() {
        return Ok(Vec::new());
    }
    Ok(output.lines().map(ToString::to_string).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_veth_script_contains_site_name() {
        let script = find_veth_script("mi-sitio-test");
        assert!(script.contains("mi-sitio-test"));
    }

    #[test]
    fn find_veth_script_contains_docker_compose_ps() {
        let script = find_veth_script("test");
        assert!(script.contains("docker compose -p"));
    }

    #[test]
    fn find_veth_script_contains_nsenter() {
        let script = find_veth_script("test");
        assert!(script.contains("nsenter -t"));
        assert!(script.contains("iflink"));
    }

    #[test]
    fn find_veth_script_multiple_sites_produce_different_scripts() {
        let s1 = find_veth_script("site-a");
        let s2 = find_veth_script("site-b");
        assert_ne!(s1, s2);
    }

    #[test]
    fn set_rate_limit_script_contains_veth_and_rate() {
        let script = set_rate_limit_script("veth1234", 10);
        assert!(script.contains("veth1234"));
        assert!(script.contains("10mbit"));
    }

    #[test]
    fn set_rate_limit_script_has_tbf_parameters() {
        let script = set_rate_limit_script("veth0", 5);
        assert!(script.contains("tbf"));
        assert!(script.contains("burst 32kbit"));
        assert!(script.contains("latency 400ms"));
    }

    #[test]
    fn set_rate_limit_script_has_replace_and_add_fallback() {
        let script = set_rate_limit_script("vethX", 20);
        assert!(script.contains("replace"));
        assert!(script.contains("add"));
    }

    #[test]
    fn set_rate_limit_script_different_rates_produce_different_scripts() {
        let s1 = set_rate_limit_script("veth", 5);
        let s2 = set_rate_limit_script("veth", 10);
        assert_ne!(s1, s2);
    }

    #[test]
    fn remove_rate_limit_script_contains_veth() {
        let script = remove_rate_limit_script("veth-test");
        assert!(script.contains("veth-test"));
    }

    #[test]
    fn remove_rate_limit_script_has_delete_and_ignore_error() {
        let script = remove_rate_limit_script("veth0");
        assert!(script.contains("del dev"));
        assert!(script.contains("|| true"));
    }

    #[test]
    fn list_throttled_command_contains_grep_tbf() {
        assert!(LIST_THROTTLED_CMD.contains("tbf|htb"));
    }

    #[test]
    fn list_throttled_command_contains_sort() {
        assert!(LIST_THROTTLED_CMD.contains("sort -u"));
    }

    #[test]
    fn scripts_are_valid_shell_no_newlines_in_simple_cmds() {
        let srl = set_rate_limit_script("v", 10);
        assert!(!srl.contains('\n'), "set_rate_limit debe ser una linea");
        let rrl = remove_rate_limit_script("v");
        assert!(!rrl.contains('\n'), "remove_rate_limit debe ser una linea");
        assert!(
            !LIST_THROTTLED_CMD.contains('\n'),
            "LIST_THROTTLED_CMD debe ser una linea"
        );
    }
}
