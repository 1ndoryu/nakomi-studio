/* sentinel-disable-file limite-lineas: servicio central de integracion Coolify.
 * Agrupa create/list/start/stop/restart/update para que el dominio de hosting no
 * disperse llamadas HTTP y parsing de la API en varios handlers.
 */
/* [104A-42] Servicio de provisioning Coolify para hosting administrado.
 * [155A-13] Soporta hosting WordPress legacy y hosting normal con Nginx + SFTP.
 * Lee configuración desde variables de entorno COOLIFY_*.
 * Idempotencia: el service_name incluye el UUID del hosting para evitar duplicados.
 * Diseño no-fatal: los errores de provisioning se loguean pero no bloquean el pago. */

use base64::Engine;
use rand::distributions::Alphanumeric;
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing;

use crate::errors::AppError;
use crate::models::HostingPlanConfig;

/* ============================================================
CONFIGURACIÓN
============================================================ */

/// Configuración de Coolify cargada desde variables de entorno
#[derive(Debug, Clone)]
pub struct CoolifyConfig {
    pub base_url: String,
    pub api_token: String,
    pub server_uuid: String,
    pub project_uuid: String,
    /// IP pública del servidor Coolify (usada como `server_ip` de las suscripciones)
    pub server_ip: String,
    /// [114A-15+] Ruta a la clave SSH para acceso al VPS (docker stats, monitoreo)
    pub ssh_key_path: Option<String>,
}

impl CoolifyConfig {
    /// Carga config desde variables de entorno COOLIFY_*.
    /// Retorna None si alguna variable requerida está ausente.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var("COOLIFY_BASE_URL").ok()?;
        let api_token = std::env::var("COOLIFY_API_TOKEN").ok()?;
        let server_uuid = std::env::var("COOLIFY_SERVER_UUID").ok()?;
        let project_uuid = std::env::var("COOLIFY_PROJECT_UUID").ok()?;
        let server_ip = std::env::var("COOLIFY_SERVER_IP").ok()?;

        if base_url.is_empty()
            || api_token.is_empty()
            || server_uuid.is_empty()
            || project_uuid.is_empty()
            || server_ip.is_empty()
        {
            return None;
        }

        /* [114A-15+] SSH key opcional para docker stats / monitoreo */
        let ssh_key_path = std::env::var("COOLIFY_SSH_KEY_PATH")
            .ok()
            .filter(|s| !s.is_empty());

        Some(Self {
            base_url,
            api_token,
            server_uuid,
            project_uuid,
            server_ip,
            ssh_key_path,
        })
    }

    /* [VPS1-support] Carga config desde env vars con prefijo arbitrario
     * (`COOLIFY_VPS1_`, `COOLIFY_VPS2_`, etc.) para permitir varios destinos. */
    #[must_use]
    pub fn from_env_with_prefix(prefix: &str) -> Option<Self> {
        let base_url = std::env::var(format!("{prefix}BASE_URL")).ok()?;
        let api_token = std::env::var(format!("{prefix}API_TOKEN")).ok()?;
        let server_uuid = std::env::var(format!("{prefix}SERVER_UUID")).ok()?;
        let project_uuid = std::env::var(format!("{prefix}PROJECT_UUID")).ok()?;
        let server_ip = std::env::var(format!("{prefix}SERVER_IP")).ok()?;

        if base_url.is_empty()
            || api_token.is_empty()
            || server_uuid.is_empty()
            || project_uuid.is_empty()
            || server_ip.is_empty()
        {
            return None;
        }

        let ssh_key_path = std::env::var(format!("{prefix}SSH_KEY_PATH"))
            .ok()
            .filter(|s| !s.is_empty());

        Some(Self {
            base_url,
            api_token,
            server_uuid,
            project_uuid,
            server_ip,
            ssh_key_path,
        })
    }
}

/* ============================================================
TIPOS DE API
============================================================ */

/// Body de la llamada POST /api/v1/services (`docker_compose_raw` en base64).
/// `instant_deploy` evita un 500 opaco en Coolify al crear stacks compose.
#[derive(Debug, Serialize)]
struct CreateServiceBody<'a> {
    name: &'a str,
    project_uuid: &'a str,
    environment_name: &'static str,
    server_uuid: &'a str,
    docker_compose_raw: String,
    instant_deploy: bool,
}

/// Respuesta de POST /api/v1/services (solo campos que usamos)
#[derive(Debug, Deserialize)]
struct CreateServiceResponse {
    uuid: String,
    domains: Vec<String>,
}

/// Resultado de provisionar un hosting
#[derive(Debug, Clone)]
pub struct CoolifyProvisionResult {
    /// UUID del servicio en Coolify (para gestión posterior: suspend, delete)
    pub service_uuid: String,
    /// Dominio sslip.io asignado por Coolify
    pub domain: String,
    /// IP del servidor VPS (vendrá del `CoolifyConfig`)
    pub server_ip: String,
    /// Usuario SFTP generado para acceso a archivos del sitio
    pub sftp_user: String,
    /// Contraseña SFTP generada aleatoriamente
    pub sftp_password: String,
    /// Puerto del host mapeado al contenedor SFTP (único por hosting)
    pub sftp_port: i32,
}

pub struct HostingComposeUpdate<'a> {
    pub service_uuid: &'a str,
    pub service_name: &'a str,
    pub custom_domain: Option<&'a str>,
    pub sftp_user: &'a str,
    pub sftp_password: &'a str,
    pub sftp_port: i32,
    pub plan_config: &'a HostingPlanConfig,
}

/* [164A-19] Resumen de servicios reales devueltos por Coolify.
 * Se usa para poblar el panel admin con despliegues de la VPS2, no con la lista
 * de instancias del proveedor. Los campos opcionales vienen de la API y no siempre
 * están presentes según la versión de Coolify o el tipo de servicio. */
#[derive(Debug, Clone)]
pub struct CoolifyServiceSummary {
    pub uuid: String,
    pub name: String,
    pub status: String,
    pub fqdn: Option<String>,
    pub server_uuid: Option<String>,
    pub server_name: Option<String>,
    pub project_uuid: Option<String>,
    pub environment_name: Option<String>,
}

/* ============================================================
SERVICIO
============================================================ */

pub struct CoolifyService;

fn read_nested_string(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }

    current
        .as_str()
        .map(str::trim)
        .filter(|candidate| !candidate.is_empty())
        .map(ToOwned::to_owned)
}

fn read_first_string(value: &Value, paths: &[&[&str]]) -> Option<String> {
    paths
        .iter()
        .find_map(|path| read_nested_string(value, path))
}

fn extract_service_fqdn(value: &Value) -> Option<String> {
    read_first_string(
        value,
        &[&["fqdn"], &["destination", "fqdn"], &["service", "fqdn"]],
    )
    .or_else(|| {
        value
            .get("domains")
            .and_then(Value::as_array)
            .and_then(|domains| {
                domains.iter().find_map(|domain| {
                    domain
                        .as_str()
                        .map(str::trim)
                        .filter(|candidate| !candidate.is_empty())
                        .map(ToOwned::to_owned)
                        .or_else(|| {
                            domain
                                .get("domain")
                                .and_then(Value::as_str)
                                .map(str::trim)
                                .filter(|candidate| !candidate.is_empty())
                                .map(ToOwned::to_owned)
                        })
                        .or_else(|| {
                            domain
                                .get("fqdn")
                                .and_then(Value::as_str)
                                .map(str::trim)
                                .filter(|candidate| !candidate.is_empty())
                                .map(ToOwned::to_owned)
                        })
                })
            })
    })
}

fn parse_service_summary(value: &Value) -> Option<CoolifyServiceSummary> {
    let uuid = read_first_string(value, &[&["uuid"]])?;
    let name = read_first_string(value, &[&["name"]])?;

    Some(CoolifyServiceSummary {
        uuid,
        name,
        status: read_first_string(value, &[&["status"], &["deployment_status"]])
            .unwrap_or_else(|| "unknown".to_string()),
        fqdn: extract_service_fqdn(value),
        server_uuid: read_first_string(
            value,
            &[
                &["server_uuid"],
                &["server", "uuid"],
                &["server", "server_uuid"],
            ],
        ),
        server_name: read_first_string(value, &[&["server_name"], &["server", "name"]]),
        project_uuid: read_first_string(value, &[&["project_uuid"], &["project", "uuid"]]),
        environment_name: read_first_string(
            value,
            &[&["environment_name"], &["environment", "name"]],
        ),
    })
}

fn service_matches_target(service: &CoolifyServiceSummary, config: &CoolifyConfig) -> bool {
    let matches_server = service
        .server_uuid
        .as_deref()
        .is_none_or(|server_uuid| server_uuid == config.server_uuid);
    let matches_project = service
        .project_uuid
        .as_deref()
        .is_none_or(|project_uuid| project_uuid == config.project_uuid);

    matches_server && matches_project
}

fn is_normal_hosting_plan(plan_name: &str) -> bool {
    plan_name.starts_with("normal-")
}

fn millicores_to_cpu(millicores: i32) -> String {
        format!("{:.2}", f64::from(millicores) / 1000.0)
}

fn clean_route_host(value: &str) -> String {
    value
        .trim()
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_end_matches('/')
        .split('/')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

fn route_host_slug(host: &str) -> String {
    let slug = clean_route_host(host)
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();

    slug
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn hosting_bootstrap_host(plan_name: &str, service_name: &str, server_ip: &str) -> String {
    let service_prefix = if is_normal_hosting_plan(plan_name) {
        "site"
    } else {
        "wordpress"
    };

    format!("{service_prefix}-{service_name}.{server_ip}.sslip.io")
}

fn hosting_route_hosts(
    plan_name: &str,
    service_name: &str,
    server_ip: &str,
    custom_domain: Option<&str>,
) -> Vec<String> {
    let mut hosts = vec![hosting_bootstrap_host(plan_name, service_name, server_ip)];

    if let Some(domain) = custom_domain {
        let cleaned = clean_route_host(domain);
        if !cleaned.is_empty() && !hosts.iter().any(|host| host == &cleaned) {
            hosts.push(cleaned);
        }
    }

    hosts
}

fn build_traefik_labels(route_hosts: &[String]) -> String {
    if route_hosts.is_empty() {
        return String::new();
    }

    let mut lines = vec![
        "        labels:".to_string(),
        "            - 'traefik.enable=true'".to_string(),
    ];

    for host in route_hosts {
        let cleaned_host = clean_route_host(host);
        if cleaned_host.is_empty() {
            continue;
        }

        let slug = route_host_slug(&cleaned_host);
        if slug.is_empty() {
            continue;
        }

        lines.push(format!(
            "            - 'traefik.http.services.{slug}-svc.loadbalancer.server.port=80'"
        ));
        lines.push(format!(
            "            - 'traefik.http.routers.{slug}-http.rule=Host(`{cleaned_host}`)'"
        ));
        lines.push(format!(
            "            - 'traefik.http.routers.{slug}-http.entrypoints=http'"
        ));
        lines.push(format!(
            "            - 'traefik.http.routers.{slug}-http.service={slug}-svc'"
        ));
        lines.push(format!(
            "            - 'traefik.http.routers.{slug}-https.rule=Host(`{cleaned_host}`)'"
        ));
        lines.push(format!(
            "            - 'traefik.http.routers.{slug}-https.entrypoints=https'"
        ));
        lines.push(format!(
            "            - 'traefik.http.routers.{slug}-https.tls=true'"
        ));
        lines.push(format!(
            "            - 'traefik.http.routers.{slug}-https.tls.certresolver=letsencrypt'"
        ));
        lines.push(format!(
            "            - 'traefik.http.routers.{slug}-https.service={slug}-svc'"
        ));
    }

    format!("{}\n", lines.join("\n"))
}

fn generate_sftp_credentials() -> (String, String) {
    let sftp_user: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect::<String>()
        .to_lowercase();
    let sftp_password: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();

    (format!("wp_{sftp_user}"), sftp_password)
}

/* [114A-3] Límites dinámicos por plan. WP reserva 25% de su límite, DB reserva 25% también. */
fn build_compose_wp_db(
    wp_cpu: &str,
    wp_mem: &str,
    db_cpu: &str,
    db_mem: &str,
    route_hosts: &[String],
) -> String {
        let traefik_labels = build_traefik_labels(route_hosts);
        format!(
                "  wordpress:\n    image: 'wordpress:6.7-php8.3-apache'\n    environment:\n      - SERVICE_FQDN_WORDPRESS=\n      - WORDPRESS_DB_HOST=mariadb\n      - WORDPRESS_DB_USER=wordpress\n      - WORDPRESS_DB_PASSWORD=SERVICE_PASSWORD_DB\n      - WORDPRESS_DB_NAME=wordpress\n      - WORDPRESS_CONFIG_EXTRA=define('DISALLOW_FILE_EDIT', true);\n    volumes:\n      - 'wordpress-data:/var/www/html'\n    depends_on:\n      - mariadb\n    restart: unless-stopped\n    networks:\n      - frontend_net\n      - backend_net\n{traefik_labels}    cap_drop:\n      - ALL\n    cap_add:\n      - CHOWN\n      - SETUID\n      - SETGID\n      - DAC_OVERRIDE\n      - NET_BIND_SERVICE\n    security_opt:\n      - no-new-privileges:true\n    pids_limit: 200\n    deploy:\n      resources:\n        limits:\n          cpus: '{wp_cpu}'\n          memory: {wp_mem}\n        reservations:\n          memory: 128M\n  mariadb:\n    image: 'mariadb:11.4'\n    environment:\n      - MYSQL_ROOT_PASSWORD=SERVICE_PASSWORD_ROOT\n      - MYSQL_DATABASE=wordpress\n      - MYSQL_USER=wordpress\n      - MYSQL_PASSWORD=SERVICE_PASSWORD_DB\n    volumes:\n      - 'mariadb-data:/var/lib/mysql'\n    restart: unless-stopped\n    networks:\n      - backend_net\n    cap_drop:\n      - ALL\n    cap_add:\n      - CHOWN\n      - SETUID\n      - SETGID\n      - DAC_OVERRIDE\n    security_opt:\n      - no-new-privileges:true\n    pids_limit: 150\n    deploy:\n      resources:\n        limits:\n          cpus: '{db_cpu}'\n          memory: {db_mem}\n        reservations:\n          memory: 128M\n"
        )
}

/* [155A-13] Servicio web para hosting normal: Nginx sirve el volumen editable por SFTP. */
fn build_compose_static_site(site_cpu: &str, site_mem: &str, route_hosts: &[String]) -> String {
        let traefik_labels = build_traefik_labels(route_hosts);
        format!(
                r#"  site:
        image: 'nginx:1.27-alpine'
        environment:
            - SERVICE_FQDN_SITE=
        command:
            - sh
            - -c
            - 'mkdir -p /usr/share/nginx/html; test -f /usr/share/nginx/html/index.html || printf "%s\n" "<h1>Hosting activo</h1>" > /usr/share/nginx/html/index.html; chown -R nginx:nginx /usr/share/nginx/html; nginx -g "daemon off;"'
        volumes:
            - 'site-data:/usr/share/nginx/html'
        restart: unless-stopped
        networks:
            - frontend_net
{traefik_labels}        cap_drop:
            - ALL
        cap_add:
            - CHOWN
            - SETUID
            - SETGID
            - DAC_OVERRIDE
            - NET_BIND_SERVICE
        security_opt:
            - no-new-privileges:true
        pids_limit: 160
        deploy:
            resources:
                limits:
                    cpus: '{site_cpu}'
                    memory: {site_mem}
                reservations:
                    memory: 64M
"#
        )
}

/* [164A-6][155A-13] Genera el compose YAML para hosting administrado.
 * Los slugs `normal-*` usan Nginx + SFTP; los slugs legacy usan WordPress + MariaDB + SSH/SFTP.
 * [164A-16] Hardening: imágenes pineadas, network isolation, cap_drop ALL,
 * no-new-privileges, pids_limit, WP file editing deshabilitado.
 * [174A-17] Plan ecommerce incluye sidecar de backup automático (3 daily + 2 weekly).
 * [114A-3] Límites de CPU/RAM dinámicos desde HostingPlanConfig (admin-configurable). */
#[cfg(test)]
fn build_hosting_compose(
    sftp_user: &str,
    sftp_password: &str,
    sftp_port: i32,
    config: &HostingPlanConfig,
) -> String {
    build_hosting_compose_with_routes(&[], sftp_user, sftp_password, sftp_port, config)
}

fn build_hosting_compose_for_service(
    service_name: &str,
    server_ip: &str,
    custom_domain: Option<&str>,
    sftp_user: &str,
    sftp_password: &str,
    sftp_port: i32,
    config: &HostingPlanConfig,
) -> String {
    let route_hosts = hosting_route_hosts(&config.plan_name, service_name, server_ip, custom_domain);
    build_hosting_compose_with_routes(&route_hosts, sftp_user, sftp_password, sftp_port, config)
}

fn build_hosting_compose_with_routes(
    route_hosts: &[String],
    sftp_user: &str,
    sftp_password: &str,
    sftp_port: i32,
    config: &HostingPlanConfig,
) -> String {
    if is_normal_hosting_plan(&config.plan_name) {
        return build_normal_hosting_compose(route_hosts, sftp_user, sftp_password, sftp_port, config);
    }

    build_wordpress_hosting_compose(route_hosts, sftp_user, sftp_password, sftp_port, config)
}

fn build_wordpress_hosting_compose(
    route_hosts: &[String],
    sftp_user: &str,
    sftp_password: &str,
    sftp_port: i32,
    config: &HostingPlanConfig,
) -> String {
    let wp_cpu = millicores_to_cpu(config.wp_cpu_millicores);
    let wp_mem = format!("{}M", config.wp_memory_mb);
    let db_cpu = millicores_to_cpu(config.db_cpu_millicores);
    let db_mem = format!("{}M", config.db_memory_mb);
    let ssh_cpu = millicores_to_cpu(config.ssh_cpu_millicores);
    let ssh_mem = format!("{}M", config.ssh_memory_mb);

    let wp_db = build_compose_wp_db(&wp_cpu, &wp_mem, &db_cpu, &db_mem, route_hosts);
    let ssh = build_compose_ssh(sftp_user, sftp_password, sftp_port, &ssh_cpu, &ssh_mem);
    let backup = if config.plan_name == "ecommerce" {
        build_compose_backup()
    } else {
        String::new()
    };
    let backup_vol = if config.plan_name == "ecommerce" {
        "  backup-data:\n"
    } else {
        ""
    };
    format!("services:\n{wp_db}{ssh}{backup}\nnetworks:\n  frontend_net:\n  backend_net:\n    internal: true\n  ssh_net:\nvolumes:\n  wordpress-data:\n  mariadb-data:\n{backup_vol}")
}

fn build_normal_hosting_compose(
    route_hosts: &[String],
    sftp_user: &str,
    sftp_password: &str,
    sftp_port: i32,
    config: &HostingPlanConfig,
) -> String {
    let site_cpu = millicores_to_cpu(config.wp_cpu_millicores);
    let site_mem = format!("{}M", config.wp_memory_mb);
    let ssh_cpu = millicores_to_cpu(config.ssh_cpu_millicores);
    let ssh_mem = format!("{}M", config.ssh_memory_mb);
    let site = build_compose_static_site(&site_cpu, &site_mem, route_hosts);
    let ssh = build_compose_static_ssh(sftp_user, sftp_password, sftp_port, &ssh_cpu, &ssh_mem);

    format!(
        "services:\n{site}{ssh}\nnetworks:\n  frontend_net:\n  ssh_net:\nvolumes:\n  site-data:\n"
    )
}

/* [114A-3] SSH container con wp-cli vía dockerfile_inline + hardening sshd.
 * - PHP + wp-cli instalados para gestión WordPress vía shell.
 * - backend_net añadida para que wp-cli pueda conectar a MariaDB.
 * - sshd hardening: AllowTcpForwarding=no, X11Forwarding=no, PermitTunnel=no, GatewayPorts=no
 *   aplicados idempotentemente vía custom-cont-init.d script.
 * - Límites de CPU/RAM dinámicos desde plan config. */
fn build_compose_ssh(
    sftp_user: &str,
    sftp_password: &str,
    sftp_port: i32,
    ssh_cpu: &str,
    ssh_mem: &str,
) -> String {
        format!(
            r#"  ssh:
        build:
            dockerfile_inline: |
                FROM lscr.io/linuxserver/openssh-server:9.9_p2-r0-ls190
                RUN apk add --no-cache php83-cli php83-phar php83-json php83-mbstring php83-curl php83-mysqli php83-xml php83-tokenizer bash coreutils \
                        && ln -sf /usr/bin/php83 /usr/bin/php \
                        && curl -sSL https://raw.githubusercontent.com/wp-cli/builds/gh-pages/phar/wp-cli.phar -o /usr/local/bin/wp \
                        && chmod +x /usr/local/bin/wp
                RUN mkdir -p /custom-cont-init.d && \
                        echo '#!/bin/bash' > /custom-cont-init.d/10-harden-ssh && \
                        echo 'grep -q "AllowTcpForwarding no" /config/sshd/sshd_config 2>/dev/null || {{' >> /custom-cont-init.d/10-harden-ssh && \
                        echo '  echo "AllowTcpForwarding no" >> /config/sshd/sshd_config' >> /custom-cont-init.d/10-harden-ssh && \
                        echo '  echo "X11Forwarding no" >> /config/sshd/sshd_config' >> /custom-cont-init.d/10-harden-ssh && \
                        echo '  echo "PermitTunnel no" >> /config/sshd/sshd_config' >> /custom-cont-init.d/10-harden-ssh && \
                        echo '  echo "GatewayPorts no" >> /config/sshd/sshd_config' >> /custom-cont-init.d/10-harden-ssh && \
                        echo '}}' >> /custom-cont-init.d/10-harden-ssh && \
                        chmod +x /custom-cont-init.d/10-harden-ssh
        environment:
            - PUID=33
            - PGID=33
            - TZ=UTC
            - USER_NAME={sftp_user}
            - USER_PASSWORD={sftp_password}
            - PASSWORD_ACCESS=true
            - SUDO_ACCESS=false
            - LOG_STDOUT=true
        volumes:
            - 'wordpress-data:/home/{sftp_user}/html'
        ports:
            - '{sftp_port}:2222'
        restart: unless-stopped
        networks:
            - ssh_net
            - backend_net
        cap_drop:
            - ALL
        cap_add:
            - CHOWN
            - SETUID
            - SETGID
            - DAC_OVERRIDE
            - NET_BIND_SERVICE
        security_opt:
            - no-new-privileges:true
        pids_limit: 100
        deploy:
            resources:
                limits:
                    cpus: '{ssh_cpu}'
                    memory: {ssh_mem}
                reservations:
                    memory: 64M
"#
        )
}

/* [155A-13] SSH/SFTP para hosting normal sin PHP/WP-CLI ni red backend. */
fn build_compose_static_ssh(
    sftp_user: &str,
    sftp_password: &str,
    sftp_port: i32,
    ssh_cpu: &str,
    ssh_mem: &str,
) -> String {
        format!(
            r#"  ssh:
        build:
            dockerfile_inline: |
                FROM lscr.io/linuxserver/openssh-server:9.9_p2-r0-ls190
                RUN apk add --no-cache bash coreutils
                RUN mkdir -p /custom-cont-init.d && \
                        echo '#!/bin/bash' > /custom-cont-init.d/10-harden-ssh && \
                        echo 'grep -q "AllowTcpForwarding no" /config/sshd/sshd_config 2>/dev/null || {{' >> /custom-cont-init.d/10-harden-ssh && \
                        echo '  echo "AllowTcpForwarding no" >> /config/sshd/sshd_config' >> /custom-cont-init.d/10-harden-ssh && \
                        echo '  echo "X11Forwarding no" >> /config/sshd/sshd_config' >> /custom-cont-init.d/10-harden-ssh && \
                        echo '  echo "PermitTunnel no" >> /config/sshd/sshd_config' >> /custom-cont-init.d/10-harden-ssh && \
                        echo '  echo "GatewayPorts no" >> /config/sshd/sshd_config' >> /custom-cont-init.d/10-harden-ssh && \
                        echo '}}' >> /custom-cont-init.d/10-harden-ssh && \
                        chmod +x /custom-cont-init.d/10-harden-ssh
        environment:
            - PUID=101
            - PGID=101
            - TZ=UTC
            - USER_NAME={sftp_user}
            - USER_PASSWORD={sftp_password}
            - PASSWORD_ACCESS=true
            - SUDO_ACCESS=false
            - LOG_STDOUT=true
        volumes:
            - 'site-data:/home/{sftp_user}/html'
        ports:
            - '{sftp_port}:2222'
        restart: unless-stopped
        networks:
            - ssh_net
        cap_drop:
            - ALL
        cap_add:
            - CHOWN
            - SETUID
            - SETGID
            - DAC_OVERRIDE
            - NET_BIND_SERVICE
        security_opt:
            - no-new-privileges:true
        pids_limit: 100
        deploy:
            resources:
                limits:
                    cpus: '{ssh_cpu}'
                    memory: {ssh_mem}
                reservations:
                    memory: 64M
"#
        )
}

/* [174A-17] Sidecar de backup automático para plan ecommerce.
 * Retention: 3 copias diarias + 2 semanales (domingos).
 * MYSQL_PWD es leído automáticamente por mysqldump — no se pasa en CLI.
 * Sleep inicial de 60s da tiempo a MariaDB para arrancar completamente.
 * Solo se incluye en compose cuando plan == "ecommerce". */
fn build_compose_backup() -> String {
    "  backup:\n    image: 'mariadb:11.4'\n    environment:\n      - MYSQL_PWD=SERVICE_PASSWORD_DB\n    command:\n      - sh\n      - -c\n      - 'sleep 60; while true; do DT=$$(date +%Y%m%d_%H%M%S); DOW=$$(date +%u); mysqldump -h mariadb -u wordpress wordpress > /backups/daily_$$DT.sql 2>&1; tar czf /backups/daily_wp_$$DT.tar.gz -C /wp-html . 2>&1; if [ $$DOW = 7 ]; then cp /backups/daily_$$DT.sql /backups/weekly_$$DT.sql; cp /backups/daily_wp_$$DT.tar.gz /backups/weekly_wp_$$DT.tar.gz; fi; find /backups -maxdepth 1 -name \"daily_*\" -mtime +3 -delete; find /backups -maxdepth 1 -name \"weekly_*\" -mtime +14 -delete; sleep 86400; done'\n    volumes:\n      - 'wordpress-data:/wp-html:ro'\n      - 'backup-data:/backups'\n    networks:\n      - backend_net\n    depends_on:\n      - mariadb\n    restart: unless-stopped\n    cap_drop:\n      - ALL\n    cap_add:\n      - CHOWN\n      - SETUID\n      - SETGID\n      - DAC_OVERRIDE\n    security_opt:\n      - no-new-privileges:true\n    pids_limit: 50\n    deploy:\n      resources:\n        limits:\n          cpus: '0.25'\n          memory: 256M\n        reservations:\n          memory: 64M\n".to_string()
}

impl CoolifyService {
    /* [164A-19] Lista despliegues reales visibles en Coolify para la VPS2 configurada.
     * La tarea original confundía "instancia VPS" con "servicio desplegado"; este método
     * corrige esa frontera leyendo directamente `/api/v1/services` y filtrando por target. */
    pub async fn list_services(
        http_client: &Client,
        config: &CoolifyConfig,
    ) -> Result<Vec<CoolifyServiceSummary>, AppError> {
        let url = format!("{}/api/v1/services", config.base_url);
        let resp = http_client
            .get(&url)
            .bearer_auth(&config.api_token)
            .send()
            .await
            .map_err(|e| {
                AppError::Internal(format!("Coolify list services request failed: {e}"))
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::error!("[Coolify] Error listando servicios: {} — {}", status, body);
            return Err(AppError::Internal(format!(
                "Coolify list services failed: {status}"
            )));
        }

        let payload: Value = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Coolify list services parse error: {e}")))?;

        let services_raw = payload.as_array().ok_or_else(|| {
            AppError::Internal("Coolify list services parse error: expected array".into())
        })?;

        let mut services = Vec::new();
        let mut skipped = 0usize;

        for service_value in services_raw {
            let Some(service) = parse_service_summary(service_value) else {
                skipped += 1;
                continue;
            };

            if service_matches_target(&service, config) {
                services.push(service);
            }
        }

        if skipped > 0 {
            tracing::warn!(
                "[Coolify] {} servicio(s) omitidos del panel por respuesta incompleta.",
                skipped
            );
        }

        services.sort_by(|left, right| left.name.cmp(&right.name).then(left.uuid.cmp(&right.uuid)));

        Ok(services)
    }

    /// Provision completo: crea servicio `WordPress` en Coolify y lo arranca.
    /// Usa `docker_compose_raw` con `WordPress` + `MariaDB` + SSH/SFTP.
    /// El nombre de servicio garantiza idempotencia (Coolify rechaza duplicados).
    /// `sftp_port` debe generarse externamente con `HostingRepository::find_available_sftp_port`.
    /// `plan_config` determina límites de recursos y features extra (backup sidecar para ecommerce).
    ///
    /// # Errors
    /// Retorna `AppError` si la API falla o el JSON no parsea.
    pub async fn provision_hosting(
        http_client: &Client,
        config: &CoolifyConfig,
        service_name: &str,
        sftp_port: i32,
        plan_config: &HostingPlanConfig,
    ) -> Result<CoolifyProvisionResult, AppError> {
        let (sftp_user, sftp_password) = generate_sftp_credentials();

        let bootstrap_domain = hosting_bootstrap_host(
            &plan_config.plan_name,
            service_name,
            &config.server_ip,
        );
        let compose_yaml = build_hosting_compose_for_service(
            service_name,
            &config.server_ip,
            None,
            &sftp_user,
            &sftp_password,
            sftp_port,
            plan_config,
        );
        let compose_b64 = base64::engine::general_purpose::STANDARD.encode(&compose_yaml);

        /* Paso 1: Crear el servicio */
        let create_body = CreateServiceBody {
            name: service_name,
            project_uuid: &config.project_uuid,
            environment_name: "production",
            server_uuid: &config.server_uuid,
            docker_compose_raw: compose_b64,
            instant_deploy: true,
        };

        let create_url = format!("{}/api/v1/services", config.base_url);
        let create_resp = http_client
            .post(&create_url)
            .bearer_auth(&config.api_token)
            .json(&create_body)
            .send()
            .await
            .map_err(|e| {
                AppError::Internal(format!("Coolify create service request failed: {e}"))
            })?;

        if !create_resp.status().is_success() {
            let status = create_resp.status();
            let body = create_resp.text().await.unwrap_or_default();
            tracing::error!(
                "[Coolify] Error creando servicio '{}': {} — {}",
                service_name,
                status,
                body
            );
            return Err(AppError::Internal(format!(
                "Coolify create service failed: {status} — {body}"
            )));
        }

        let created: CreateServiceResponse = create_resp
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Coolify create service parse error: {e}")))?;

        let coolify_domain = created.domains.into_iter().next();
        let domain = format!("http://{bootstrap_domain}");

        tracing::info!(
            "[Coolify] Servicio '{}' creado: uuid={}, bootstrap_domain={}, coolify_domain={:?}",
            service_name,
            created.uuid,
            domain,
            coolify_domain
        );

        /* Paso 2: Arrancar el servicio */
        let start_url = format!("{}/api/v1/services/{}/start", config.base_url, created.uuid);
        let start_resp = http_client
            .post(&start_url)
            .bearer_auth(&config.api_token)
            .send()
            .await
            .map_err(|e| {
                AppError::Internal(format!("Coolify start service request failed: {e}"))
            })?;

        if start_resp.status().is_success() {
            tracing::info!(
                "[Coolify] Servicio '{}' iniciado correctamente.",
                service_name
            );
        } else {
            let status = start_resp.status();
            let body = start_resp.text().await.unwrap_or_default();
            /* No-fatal: el servicio fue creado, aunque no pudo iniciarse automáticamente.
             * El admin puede iniciarlo manualmente desde el panel. */
            tracing::warn!(
                "[Coolify] Error iniciando servicio '{}' (uuid={}): {} — {}. Servicio creado pero no iniciado.",
                service_name,
                created.uuid,
                status,
                body
            );
        }

        Ok(CoolifyProvisionResult {
            service_uuid: created.uuid,
            domain,
            server_ip: config.server_ip.clone(),
            sftp_user,
            sftp_password,
            sftp_port,
        })
    }

    /// Detiene y elimina un servicio de Coolify.
    /// Usado cuando se cancela o suspende definitivamente un hosting.
    ///
    /// # Errors
    /// Retorna `AppError` si la API falla. El caller decide si es fatal.
    pub async fn delete_service(
        http_client: &Client,
        config: &CoolifyConfig,
        service_uuid: &str,
        delete_volumes: bool,
    ) -> Result<(), AppError> {
        /* Detener primero para liberar recursos gradualmente */
        let stop_url = format!("{}/api/v1/services/{}/stop", config.base_url, service_uuid);
        let stop_resp = http_client
            .post(&stop_url)
            .bearer_auth(&config.api_token)
            .send()
            .await;

        match stop_resp {
            Ok(r) if r.status().is_success() => {
                tracing::info!("[Coolify] Servicio {} detenido.", service_uuid);
            }
            Ok(r) => {
                tracing::warn!(
                    "[Coolify] No se pudo detener servicio {}: {}",
                    service_uuid,
                    r.status()
                );
            }
            Err(e) => {
                tracing::warn!(
                    "[Coolify] Error de red al detener servicio {}: {}",
                    service_uuid,
                    e
                );
            }
        }

        /* Eliminar el servicio (y volúmenes si se indica) */
        let delete_url = format!(
            "{}/api/v1/services/{}?delete_volumes={}&delete_networks=true&docker_cleanup=false",
            config.base_url,
            service_uuid,
            if delete_volumes { "true" } else { "false" }
        );

        let del_resp = http_client
            .delete(&delete_url)
            .bearer_auth(&config.api_token)
            .send()
            .await
            .map_err(|e| {
                AppError::Internal(format!("Coolify delete service request failed: {e}"))
            })?;

        if del_resp.status().is_success() {
            tracing::info!(
                "[Coolify] Servicio {} eliminado (delete_volumes={}).",
                service_uuid,
                delete_volumes
            );
        } else {
            let status = del_resp.status();
            let body = del_resp.text().await.unwrap_or_default();
            tracing::error!(
                "[Coolify] Error eliminando servicio {}: {} — {}",
                service_uuid,
                status,
                body
            );
            return Err(AppError::Internal(format!(
                "Coolify delete service failed: {status}"
            )));
        }

        Ok(())
    }

    /// Genera un nombre de servicio Coolify a partir del hosting subscription UUID.
    /// Usa los primeros 8 chars del UUID para ser corto pero único.
    #[must_use]
    pub fn service_name_for(subscription_id: &uuid::Uuid) -> String {
        let id_str = subscription_id.to_string();
        let short = &id_str[..8];
        format!("hosting-{short}")
    }

    /* [114A-1] Actualiza el compose YAML de un servicio y lo reinicia.
     * Usado para rotación de credenciales SFTP: el compose se regenera con la nueva
     * contraseña, se sube vía PATCH y se reinicia el servicio (stop+start).
     * [114A-3] Ahora usa HostingPlanConfig para límites dinámicos. */
    pub async fn update_compose_and_restart(
        http_client: &Client,
        config: &CoolifyConfig,
        update: HostingComposeUpdate<'_>,
    ) -> Result<(), AppError> {
        let compose = build_hosting_compose_for_service(
            update.service_name,
            &config.server_ip,
            update.custom_domain,
            update.sftp_user,
            update.sftp_password,
            update.sftp_port,
            update.plan_config,
        );
        let compose_b64 = base64::engine::general_purpose::STANDARD.encode(&compose);

        /* PATCH: actualizar docker_compose_raw en Coolify */
        let patch_url = format!("{}/api/v1/services/{}", config.base_url, update.service_uuid);
        let patch_resp = http_client
            .patch(&patch_url)
            .bearer_auth(&config.api_token)
            .json(&serde_json::json!({ "docker_compose_raw": compose_b64 }))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Coolify PATCH compose failed: {e}")))?;

        if !patch_resp.status().is_success() {
            let status = patch_resp.status();
            let body = patch_resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Coolify PATCH compose failed: {status} — {body}"
            )));
        }

        /* Restart: stop + start para aplicar nuevo compose */
        let stop_url = format!("{}/api/v1/services/{}/stop", config.base_url, update.service_uuid);
        if let Err(e) = http_client
            .post(&stop_url)
            .bearer_auth(&config.api_token)
            .send()
            .await
        {
            tracing::warn!("[Coolify] Error deteniendo servicio {}: {e}", update.service_uuid);
        }
        let start_url = format!("{}/api/v1/services/{}/start", config.base_url, update.service_uuid);
        let start_resp = http_client
            .post(&start_url)
            .bearer_auth(&config.api_token)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Coolify restart failed: {e}")))?;

        if start_resp.status().is_success() {
            tracing::info!(
                "[Coolify] Servicio {} reiniciado con credenciales actualizadas.",
                update.service_uuid
            );
        } else {
            let status = start_resp.status();
            tracing::warn!(
                "[Coolify] Compose actualizado pero restart falló para {}: {status}",
                update.service_uuid
            );
        }

        Ok(())
    }

    /* [154A-9] Control de servicio: stop / start / restart */

    pub async fn stop_service(
        http_client: &Client,
        config: &CoolifyConfig,
        service_uuid: &str,
    ) -> Result<(), AppError> {
        let url = format!("{}/api/v1/services/{}/stop", config.base_url, service_uuid);
        let resp = http_client
            .post(&url)
            .bearer_auth(&config.api_token)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Coolify stop failed: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Coolify stop failed: {status} — {body}"
            )));
        }
        tracing::info!("[Coolify] Servicio {service_uuid} detenido.");
        Ok(())
    }

    pub async fn start_service(
        http_client: &Client,
        config: &CoolifyConfig,
        service_uuid: &str,
    ) -> Result<(), AppError> {
        let url = format!("{}/api/v1/services/{}/start", config.base_url, service_uuid);
        let resp = http_client
            .post(&url)
            .bearer_auth(&config.api_token)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Coolify start failed: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Coolify start failed: {status} — {body}"
            )));
        }
        tracing::info!("[Coolify] Servicio {service_uuid} iniciado.");
        Ok(())
    }

    pub async fn restart_service(
        http_client: &Client,
        config: &CoolifyConfig,
        service_uuid: &str,
    ) -> Result<(), AppError> {
        let url = format!(
            "{}/api/v1/services/{}/restart",
            config.base_url, service_uuid
        );
        let resp = http_client
            .post(&url)
            .bearer_auth(&config.api_token)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Coolify restart failed: {e}")))?;
        if resp.status().is_success() {
            tracing::info!("[Coolify] Servicio {service_uuid} reiniciado.");
        } else {
            /* Fallback: stop + start si restart endpoint no existe */
            tracing::warn!("[Coolify] Restart endpoint falló, intentando stop+start.");
            Self::stop_service(http_client, config, service_uuid).await?;
            Self::start_service(http_client, config, service_uuid).await?;
        }
        Ok(())
    }

    /* [154A-4] Actualizar el FQDN de un servicio Coolify cuando cambia el dominio.
     * Coolify v4 congela el FQDN al crear el servicio, así que intentamos:
     * 1. PATCH al endpoint de servicio con la nueva FQDN
     * 2. Si no funciona, se necesitan labels Traefik explícitas en el compose.
     * En cualquier caso, se hace redeploy para aplicar cambios. */
    pub async fn update_service_domain(
        http_client: &Client,
        config: &CoolifyConfig,
        service_uuid: &str,
        new_domain: &str,
    ) -> Result<(), AppError> {
        let url = format!("{}/api/v1/services/{service_uuid}", config.base_url);

        let fqdn = if new_domain.starts_with("http") {
            new_domain.to_string()
        } else {
            format!("https://{new_domain}")
        };

        let body = serde_json::json!({
            "domains": [fqdn],
        });

        let resp = http_client
            .patch(&url)
            .bearer_auth(&config.api_token)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Coolify FQDN update failed: {e}")))?;

        if resp.status().is_success() {
            tracing::info!("[Coolify] FQDN actualizado a {fqdn} para servicio {service_uuid}");
        } else {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            tracing::warn!(
                "[Coolify] FQDN update retornó {status}: {body_text}. \
                 Puede requerir labels Traefik manuales en el compose."
            );
        }

        /* Redeploy para que Coolify aplique la nueva config */
        Self::restart_service(http_client, config, service_uuid).await?;

        Ok(())
    }
}

/* ============================================================
TESTS — [114A-14] Tests completos del servicio de hosting
============================================================ */

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;
    use serde_yaml::Value as YamlValue;
    use uuid::Uuid;

    fn test_plan_config(plan_name: &str) -> HostingPlanConfig {
        HostingPlanConfig {
            id: Uuid::new_v4(),
            plan_name: plan_name.to_string(),
            monthly_price_cents: 500,
            wp_cpu_millicores: 500,
            wp_memory_mb: 256,
            db_cpu_millicores: 500,
            db_memory_mb: 512,
            ssh_cpu_millicores: 250,
            ssh_memory_mb: 128,
            storage_limit_mb: 5120,
            bandwidth_limit_gb: 50,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /* --- millicores_to_cpu --- */

    #[test]
    fn millicores_to_cpu_standard_values() {
        assert_eq!(millicores_to_cpu(1000), "1.00");
        assert_eq!(millicores_to_cpu(500), "0.50");
        assert_eq!(millicores_to_cpu(250), "0.25");
        assert_eq!(millicores_to_cpu(1500), "1.50");
    }

    #[test]
    fn millicores_to_cpu_edge_cases() {
        assert_eq!(millicores_to_cpu(0), "0.00");
        assert_eq!(millicores_to_cpu(1), "0.00");
        assert_eq!(millicores_to_cpu(10), "0.01");
        assert_eq!(millicores_to_cpu(100), "0.10");
        assert_eq!(millicores_to_cpu(4000), "4.00");
    }

    #[test]
    fn parse_service_summary_reads_flat_service_payload() {
        let payload = json!({
            "uuid": "svc-123",
            "name": "hosting-demo",
            "status": "running",
            "server_uuid": "srv-vps2",
            "server_name": "vps2",
            "project_uuid": "project-1",
            "environment_name": "production",
            "domains": ["https://demo.example.com"]
        });

        let service = parse_service_summary(&payload).expect("service parse");

        assert_eq!(service.uuid, "svc-123");
        assert_eq!(service.name, "hosting-demo");
        assert_eq!(service.status, "running");
        assert_eq!(service.fqdn.as_deref(), Some("https://demo.example.com"));
        assert_eq!(service.server_uuid.as_deref(), Some("srv-vps2"));
        assert_eq!(service.server_name.as_deref(), Some("vps2"));
        assert_eq!(service.project_uuid.as_deref(), Some("project-1"));
        assert_eq!(service.environment_name.as_deref(), Some("production"));
    }

    #[test]
    fn create_service_body_enables_instant_deploy() {
        let body = CreateServiceBody {
            name: "hosting-abcdef01",
            project_uuid: "project-1",
            environment_name: "production",
            server_uuid: "srv-1",
            docker_compose_raw: "ZHVtbXk=".to_string(),
            instant_deploy: true,
        };

        let payload = serde_json::to_value(&body).expect("serialize create body");

        assert_eq!(payload["instant_deploy"], Value::Bool(true));
        assert_eq!(payload["environment_name"], Value::String("production".to_string()));
    }

    #[test]
    fn parse_service_summary_reads_nested_service_payload() {
        let payload = json!({
            "uuid": "svc-456",
            "name": "wordpress-demo",
            "deployment_status": "healthy",
            "fqdn": "https://wp.example.com",
            "server": {
                "uuid": "srv-vps2",
                "name": "vps2"
            },
            "project": {
                "uuid": "project-1"
            },
            "environment": {
                "name": "production"
            }
        });

        let service = parse_service_summary(&payload).expect("service parse");

        assert_eq!(service.status, "healthy");
        assert_eq!(service.fqdn.as_deref(), Some("https://wp.example.com"));
        assert_eq!(service.server_uuid.as_deref(), Some("srv-vps2"));
        assert_eq!(service.project_uuid.as_deref(), Some("project-1"));
        assert_eq!(service.environment_name.as_deref(), Some("production"));
    }

    #[test]
    fn service_matches_target_rejects_other_server_or_project() {
        let config = CoolifyConfig {
            base_url: "https://coolify.example.com".to_string(),
            api_token: "token".to_string(),
            server_uuid: "srv-vps2".to_string(),
            project_uuid: "project-vps2".to_string(),
            server_ip: "10.0.0.2".to_string(),
            ssh_key_path: None,
        };

        let matching = CoolifyServiceSummary {
            uuid: "svc-1".to_string(),
            name: "hosting-a".to_string(),
            status: "running".to_string(),
            fqdn: None,
            server_uuid: Some("srv-vps2".to_string()),
            server_name: None,
            project_uuid: Some("project-vps2".to_string()),
            environment_name: Some("production".to_string()),
        };

        let other_server = CoolifyServiceSummary {
            server_uuid: Some("srv-vps1".to_string()),
            ..matching.clone()
        };

        let other_project = CoolifyServiceSummary {
            project_uuid: Some("project-vps1".to_string()),
            ..matching.clone()
        };

        assert!(service_matches_target(&matching, &config));
        assert!(!service_matches_target(&other_server, &config));
        assert!(!service_matches_target(&other_project, &config));
    }

    /* --- service_name_for --- */

    #[test]
    fn service_name_for_uses_first_8_chars() {
        let id = Uuid::parse_str("abcdef01-2345-6789-abcd-ef0123456789").unwrap();
        assert_eq!(CoolifyService::service_name_for(&id), "hosting-abcdef01");
    }

    #[test]
    fn service_name_for_different_uuids_are_unique() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        assert_ne!(
            CoolifyService::service_name_for(&id1),
            CoolifyService::service_name_for(&id2),
        );
    }

    #[test]
    fn service_name_for_starts_with_hosting_prefix() {
        let id = Uuid::new_v4();
        let name = CoolifyService::service_name_for(&id);
        assert!(name.starts_with("hosting-"));
        assert_eq!(name.len(), 16); /* "hosting-" (8) + 8 chars UUID */
    }

    /* --- build_hosting_compose (estructura general) --- */

    #[test]
    fn compose_contains_all_required_services() {
        let config = test_plan_config("basico");
        let compose = build_hosting_compose("testuser", "testpass", 10001, &config);

        assert!(
            compose.contains("wordpress:"),
            "Debe contener servicio WordPress"
        );
        assert!(
            compose.contains("mariadb:"),
            "Debe contener servicio MariaDB"
        );
        assert!(compose.contains("ssh:"), "Debe contener servicio SSH");
    }

    #[test]
    fn normal_hosting_compose_uses_nginx_without_database() {
        let config = test_plan_config("normal-basico");
        let compose = build_hosting_compose("testuser", "testpass", 10001, &config);

        assert!(
            compose.contains("site:"),
            "Debe contener servicio web normal"
        );
        assert!(compose.contains("nginx:1.27-alpine"), "Debe usar Nginx");
        assert!(compose.contains("SERVICE_FQDN_SITE"));
        assert!(!compose.contains("wordpress:"), "No debe crear WordPress");
        assert!(!compose.contains("mariadb:"), "No debe crear base de datos");
        assert!(compose.contains("site-data:/home/testuser/html"));
    }

    #[test]
    fn compose_for_service_adds_bootstrap_route_labels() {
        let config = test_plan_config("normal-basico");
        let compose = build_hosting_compose_for_service(
            "hosting-6d746a75",
            "173.249.50.44",
            None,
            "testuser",
            "testpass",
            10001,
            &config,
        );

        assert!(compose.contains("site-hosting-6d746a75.173.249.50.44.sslip.io"));
        assert!(compose.contains("traefik.http.routers.site-hosting-6d746a75-173-249-50-44-sslip-io-http.rule"));
    }

    #[test]
    fn compose_for_service_keeps_custom_domain_route() {
        let config = test_plan_config("basico");
        let compose = build_hosting_compose_for_service(
            "hosting-6d746a75",
            "173.249.50.44",
            Some("cliente.example.com"),
            "testuser",
            "testpass",
            10001,
            &config,
        );

        assert!(compose.contains("Host(`cliente.example.com`)"));
        assert!(compose.contains("traefik.http.routers.cliente-example-com-https.rule"));
    }

    #[test]
    fn compose_contains_required_networks() {
        let config = test_plan_config("basico");
        let compose = build_hosting_compose("testuser", "testpass", 10001, &config);

        assert!(compose.contains("frontend_net:"));
        assert!(compose.contains("backend_net:"));
        assert!(compose.contains("ssh_net:"));
        assert!(
            compose.contains("internal: true"),
            "backend_net debe ser internal"
        );
    }

    #[test]
    fn compose_contains_required_volumes() {
        let config = test_plan_config("basico");
        let compose = build_hosting_compose("testuser", "testpass", 10001, &config);

        assert!(compose.contains("wordpress-data:"));
        assert!(compose.contains("mariadb-data:"));
    }

    /* --- Compose: credenciales SSH/SFTP --- */

    #[test]
    fn compose_uses_provided_sftp_credentials() {
        let config = test_plan_config("basico");
        let compose = build_hosting_compose("myuser", "s3cur3P@ss", 12345, &config);

        assert!(compose.contains("USER_NAME=myuser"));
        assert!(compose.contains("USER_PASSWORD=s3cur3P@ss"));
        assert!(
            compose.contains("12345:2222"),
            "Puerto SSH debe mapearse correctamente"
        );
    }

    #[test]
    fn compose_ssh_volume_maps_to_user_home() {
        let config = test_plan_config("basico");
        let compose = build_hosting_compose("webmaster", "pass", 10001, &config);

        assert!(compose.contains("wordpress-data:/home/webmaster/html"));
    }

    /* --- Compose: seguridad (hardening) --- */

    #[test]
    fn compose_has_security_restrictions() {
        let config = test_plan_config("basico");
        let compose = build_hosting_compose("user", "pass", 10001, &config);

        /* cap_drop: ALL en todos los contenedores */
        let cap_drop_count = compose.matches("cap_drop:").count();
        assert!(
            cap_drop_count >= 3,
            "wp, mariadb y ssh deben tener cap_drop: {cap_drop_count}"
        );

        /* no-new-privileges */
        let nnp_count = compose.matches("no-new-privileges:true").count();
        assert!(
            nnp_count >= 3,
            "wp, mariadb y ssh deben tener no-new-privileges: {nnp_count}"
        );

        /* pids_limit */
        assert!(compose.contains("pids_limit: 200"), "WordPress pids_limit");
        assert!(compose.contains("pids_limit: 150"), "MariaDB pids_limit");
        assert!(compose.contains("pids_limit: 100"), "SSH pids_limit");
    }

    #[test]
    fn compose_ssh_has_sudo_disabled() {
        let config = test_plan_config("basico");
        let compose = build_hosting_compose("user", "pass", 10001, &config);

        assert!(compose.contains("SUDO_ACCESS=false"));
    }

    #[test]
    fn compose_ssh_has_sshd_hardening_script() {
        let config = test_plan_config("basico");
        let compose = build_hosting_compose("user", "pass", 10001, &config);

        assert!(compose.contains("AllowTcpForwarding no"));
        assert!(compose.contains("X11Forwarding no"));
        assert!(compose.contains("PermitTunnel no"));
        assert!(compose.contains("GatewayPorts no"));
    }

    #[test]
    fn compose_wordpress_yaml_parses() {
        let config = test_plan_config("basico");
        let compose = build_hosting_compose("user", "pass", 10001, &config);

        serde_yaml::from_str::<YamlValue>(&compose).expect("wordpress compose debe parsear");
    }

    #[test]
    fn compose_normal_yaml_parses() {
        let config = test_plan_config("normal-basico");
        let compose = build_hosting_compose("user", "pass", 10001, &config);

        serde_yaml::from_str::<YamlValue>(&compose).expect("normal compose debe parsear");
    }

    #[test]
    fn compose_wp_disables_file_editing() {
        let config = test_plan_config("basico");
        let compose = build_hosting_compose("user", "pass", 10001, &config);

        assert!(compose.contains("DISALLOW_FILE_EDIT"));
    }

    /* --- Compose: recursos dinámicos --- */

    #[test]
    fn compose_applies_dynamic_resource_limits() {
        let mut config = test_plan_config("pro");
        config.wp_cpu_millicores = 1000;
        config.wp_memory_mb = 512;
        config.db_cpu_millicores = 750;
        config.db_memory_mb = 256;
        config.ssh_cpu_millicores = 500;
        config.ssh_memory_mb = 128;

        let compose = build_hosting_compose("user", "pass", 10001, &config);

        /* WordPress: 1.0 CPU, 512M */
        assert!(compose.contains("cpus: '1.00'"), "WP CPU debe ser 1.00");
        assert!(compose.contains("memory: 512M"), "WP memory debe ser 512M");
        /* DB: 0.75 CPU, 256M */
        assert!(compose.contains("cpus: '0.75'"), "DB CPU debe ser 0.75");
        assert!(compose.contains("memory: 256M"), "DB memory debe ser 256M");
        /* SSH: 0.50 CPU, 128M */
        assert!(compose.contains("cpus: '0.50'"), "SSH CPU debe ser 0.50");
        assert!(compose.contains("memory: 128M"), "SSH memory debe ser 128M");
    }

    /* --- Compose: backup sidecar (solo ecommerce) --- */

    #[test]
    fn compose_ecommerce_includes_backup_sidecar() {
        let config = test_plan_config("ecommerce");
        let compose = build_hosting_compose("user", "pass", 10001, &config);

        assert!(
            compose.contains("backup:"),
            "Ecommerce debe incluir sidecar backup"
        );
        assert!(
            compose.contains("backup-data:"),
            "Ecommerce debe incluir volumen backup"
        );
        assert!(compose.contains("mysqldump"), "Backup debe usar mysqldump");
    }

    #[test]
    fn compose_basico_excludes_backup_sidecar() {
        let config = test_plan_config("basico");
        let compose = build_hosting_compose("user", "pass", 10001, &config);

        assert!(
            !compose.contains("backup:"),
            "Basico no debe incluir sidecar backup"
        );
        assert!(
            !compose.contains("backup-data:"),
            "Basico no debe incluir volumen backup"
        );
    }

    #[test]
    fn compose_pro_excludes_backup_sidecar() {
        let config = test_plan_config("pro");
        let compose = build_hosting_compose("user", "pass", 10001, &config);

        assert!(
            !compose.contains("backup:"),
            "Pro no debe incluir sidecar backup"
        );
    }

    #[test]
    fn compose_backup_retention_policy() {
        let config = test_plan_config("ecommerce");
        let compose = build_hosting_compose("user", "pass", 10001, &config);

        /* 3 días de backups diarios */
        assert!(
            compose.contains("-mtime +3 -delete"),
            "Retención diaria: 3 días"
        );
        /* 14 días (2 semanas) de backups semanales */
        assert!(
            compose.contains("-mtime +14 -delete"),
            "Retención semanal: 14 días"
        );
    }

    #[test]
    fn compose_backup_runs_on_sundays_only() {
        let config = test_plan_config("ecommerce");
        let compose = build_hosting_compose("user", "pass", 10001, &config);

        /* DOW = 7 es domingo */
        assert!(compose.contains("DOW = 7"), "Backup semanal solo domingos");
    }

    /* --- Compose: isolation de red --- */

    #[test]
    fn compose_wordpress_connects_to_frontend_and_backend() {
        let compose = build_compose_wp_db("0.50", "256M", "0.50", "512M", &[]);

        /* WordPress debe estar en frontend (sirve tráfico) y backend (habla con DB) */
        assert!(compose.contains("frontend_net"));
        assert!(compose.contains("backend_net"));
    }

    #[test]
    fn compose_mariadb_only_on_backend() {
        let compose = build_compose_wp_db("0.50", "256M", "0.50", "512M", &[]);

        /* MariaDB aparece como servicio "  mariadb:\n". Su sección debe contener
         * backend_net pero NO frontend_net. Extraer desde la definición del servicio. */
        let mariadb_start = compose.find("  mariadb:\n").expect("mariadb service");
        let mariadb_section = &compose[mariadb_start..];
        assert!(mariadb_section.contains("backend_net"));
        assert!(
            !mariadb_section.contains("frontend_net"),
            "MariaDB no debe estar en frontend_net"
        );
    }

    #[test]
    fn compose_ssh_on_ssh_net_and_backend() {
        let compose = build_compose_ssh("user", "pass", 10001, "0.25", "128M");

        assert!(compose.contains("ssh_net"));
        assert!(compose.contains("backend_net"));
        /* SSH no debe estar en frontend_net */
        assert!(
            !compose.contains("frontend_net"),
            "SSH no debe estar en frontend_net"
        );
    }

    #[test]
    fn normal_compose_ssh_avoids_backend_network() {
        let config = test_plan_config("normal-pro");
        let compose = build_hosting_compose("user", "pass", 10001, &config);

        assert!(compose.contains("ssh_net"));
        assert!(!compose.contains("backend_net"));
        assert!(!compose.contains("WP-CLI"));
    }

    /* --- CoolifyConfig::from_env --- */

    #[test]
    fn coolify_config_requires_all_env_vars() {
        /* Sin ninguna variable = None */
        std::env::remove_var("COOLIFY_BASE_URL");
        std::env::remove_var("COOLIFY_API_TOKEN");
        std::env::remove_var("COOLIFY_SERVER_UUID");
        std::env::remove_var("COOLIFY_PROJECT_UUID");
        std::env::remove_var("COOLIFY_SERVER_IP");
        let config = CoolifyConfig::from_env();
        assert!(config.is_none());
    }
}
