/* [104A-42] Servicio de provisioning Coolify para hosting administrado.
 * [154A-11] Cambiado de WordPress+MariaDB a Nginx puro vía docker_compose_raw.
 * Lee configuración desde variables de entorno COOLIFY_*.
 * Idempotencia: el service_name incluye el UUID del hosting para evitar duplicados.
 * Diseño no-fatal: los errores de provisioning se loguean pero no bloquean el pago. */

use base64::Engine;
use rand::distributions::Alphanumeric;
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
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

        if base_url.is_empty() || api_token.is_empty() || server_uuid.is_empty() || project_uuid.is_empty() || server_ip.is_empty() {
            return None;
        }

        Some(Self {
            base_url,
            api_token,
            server_uuid,
            project_uuid,
            server_ip,
        })
    }
}

/* ============================================================
   TIPOS DE API
   ============================================================ */

/// Body de la llamada POST /api/v1/services (`docker_compose_raw` en base64)
#[derive(Debug, Serialize)]
struct CreateServiceBody<'a> {
    name: &'a str,
    project_uuid: &'a str,
    environment_name: &'static str,
    server_uuid: &'a str,
    docker_compose_raw: String,
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
    /// Usuario SFTP generado para acceso a archivos `WordPress`
    pub sftp_user: String,
    /// Contraseña SFTP generada aleatoriamente
    pub sftp_password: String,
    /// Puerto del host mapeado al contenedor SFTP (único por hosting)
    pub sftp_port: i32,
}

/* ============================================================
   SERVICIO
   ============================================================ */

pub struct CoolifyService;

/* [164A-6] Genera el compose YAML para un hosting WordPress + MariaDB + SSH/SFTP.
 * [164A-16] Hardening: imágenes pineadas, network isolation, cap_drop ALL,
 * no-new-privileges, pids_limit, WP file editing deshabilitado.
 * [174A-17] Plan ecommerce incluye sidecar de backup automático (3 daily + 2 weekly).
 * [114A-3] Límites de CPU/RAM dinámicos desde HostingPlanConfig (admin-configurable). */
fn build_hosting_compose(sftp_user: &str, sftp_password: &str, sftp_port: i32, config: &HostingPlanConfig) -> String {
    let wp_cpu = millicores_to_cpu(config.wp_cpu_millicores);
    let wp_mem = format!("{}M", config.wp_memory_mb);
    let db_cpu = millicores_to_cpu(config.db_cpu_millicores);
    let db_mem = format!("{}M", config.db_memory_mb);
    let ssh_cpu = millicores_to_cpu(config.ssh_cpu_millicores);
    let ssh_mem = format!("{}M", config.ssh_memory_mb);

    let wp_db = build_compose_wp_db(&wp_cpu, &wp_mem, &db_cpu, &db_mem);
    let ssh = build_compose_ssh(sftp_user, sftp_password, sftp_port, &ssh_cpu, &ssh_mem);
    let backup = if config.plan_name == "ecommerce" { build_compose_backup() } else { String::new() };
    let backup_vol = if config.plan_name == "ecommerce" { "  backup-data:\n" } else { "" };
    format!("services:\n{wp_db}{ssh}{backup}\nnetworks:\n  frontend_net:\n  backend_net:\n    internal: true\n  ssh_net:\nvolumes:\n  wordpress-data:\n  mariadb-data:\n{backup_vol}")
}

/* [114A-3] Convierte millicores a string de CPU para Docker Compose (ej: 1000 → "1.00") */
#[allow(clippy::cast_precision_loss)]
fn millicores_to_cpu(millicores: i32) -> String {
    format!("{:.2}", f64::from(millicores) / 1000.0)
}

/* [114A-3] Límites dinámicos por plan. WP reserva 25% de su límite, DB reserva 25% también. */
fn build_compose_wp_db(wp_cpu: &str, wp_mem: &str, db_cpu: &str, db_mem: &str) -> String {
    format!("  wordpress:\n    image: 'wordpress:6.7-php8.3-apache'\n    environment:\n      - SERVICE_FQDN_WORDPRESS=\n      - WORDPRESS_DB_HOST=mariadb\n      - WORDPRESS_DB_USER=wordpress\n      - WORDPRESS_DB_PASSWORD=SERVICE_PASSWORD_DB\n      - WORDPRESS_DB_NAME=wordpress\n      - WORDPRESS_CONFIG_EXTRA=define('DISALLOW_FILE_EDIT', true);\n    volumes:\n      - 'wordpress-data:/var/www/html'\n    depends_on:\n      - mariadb\n    restart: unless-stopped\n    networks:\n      - frontend_net\n      - backend_net\n    cap_drop:\n      - ALL\n    cap_add:\n      - CHOWN\n      - SETUID\n      - SETGID\n      - DAC_OVERRIDE\n      - NET_BIND_SERVICE\n    security_opt:\n      - no-new-privileges:true\n    pids_limit: 200\n    deploy:\n      resources:\n        limits:\n          cpus: '{wp_cpu}'\n          memory: {wp_mem}\n        reservations:\n          memory: 128M\n  mariadb:\n    image: 'mariadb:11.4'\n    environment:\n      - MYSQL_ROOT_PASSWORD=SERVICE_PASSWORD_ROOT\n      - MYSQL_DATABASE=wordpress\n      - MYSQL_USER=wordpress\n      - MYSQL_PASSWORD=SERVICE_PASSWORD_DB\n    volumes:\n      - 'mariadb-data:/var/lib/mysql'\n    restart: unless-stopped\n    networks:\n      - backend_net\n    cap_drop:\n      - ALL\n    cap_add:\n      - CHOWN\n      - SETUID\n      - SETGID\n      - DAC_OVERRIDE\n    security_opt:\n      - no-new-privileges:true\n    pids_limit: 150\n    deploy:\n      resources:\n        limits:\n          cpus: '{db_cpu}'\n          memory: {db_mem}\n        reservations:\n          memory: 128M\n")
}

/* [114A-3] SSH container con wp-cli vía dockerfile_inline + hardening sshd.
 * - PHP + wp-cli instalados para gestión WordPress vía shell.
 * - backend_net añadida para que wp-cli pueda conectar a MariaDB.
 * - sshd hardening: AllowTcpForwarding=no, X11Forwarding=no, PermitTunnel=no, GatewayPorts=no
 *   aplicados idempotentemente vía custom-cont-init.d script.
 * - Límites de CPU/RAM dinámicos desde plan config. */
fn build_compose_ssh(sftp_user: &str, sftp_password: &str, sftp_port: i32, ssh_cpu: &str, ssh_mem: &str) -> String {
    format!("\
  ssh:\n\
    build:\n\
      dockerfile_inline: |\n\
        FROM lscr.io/linuxserver/openssh-server:9.9_p2-r0-ls190\n\
        RUN apk add --no-cache php83-cli php83-phar php83-json php83-mbstring php83-curl php83-mysqli php83-xml php83-tokenizer bash coreutils \\\n\
            && ln -sf /usr/bin/php83 /usr/bin/php \\\n\
            && curl -sSL https://raw.githubusercontent.com/wp-cli/builds/gh-pages/phar/wp-cli.phar -o /usr/local/bin/wp \\\n\
            && chmod +x /usr/local/bin/wp\n\
        RUN mkdir -p /custom-cont-init.d && \\\n\
            echo '#!/bin/bash' > /custom-cont-init.d/10-harden-ssh && \\\n\
            echo 'grep -q \"AllowTcpForwarding no\" /config/sshd/sshd_config 2>/dev/null || {{' >> /custom-cont-init.d/10-harden-ssh && \\\n\
            echo '  echo \"AllowTcpForwarding no\" >> /config/sshd/sshd_config' >> /custom-cont-init.d/10-harden-ssh && \\\n\
            echo '  echo \"X11Forwarding no\" >> /config/sshd/sshd_config' >> /custom-cont-init.d/10-harden-ssh && \\\n\
            echo '  echo \"PermitTunnel no\" >> /config/sshd/sshd_config' >> /custom-cont-init.d/10-harden-ssh && \\\n\
            echo '  echo \"GatewayPorts no\" >> /config/sshd/sshd_config' >> /custom-cont-init.d/10-harden-ssh && \\\n\
            echo '}}' >> /custom-cont-init.d/10-harden-ssh && \\\n\
            chmod +x /custom-cont-init.d/10-harden-ssh\n\
    environment:\n\
      - PUID=33\n\
      - PGID=33\n\
      - TZ=UTC\n\
      - USER_NAME={sftp_user}\n\
      - USER_PASSWORD={sftp_password}\n\
      - PASSWORD_ACCESS=true\n\
      - SUDO_ACCESS=false\n\
      - LOG_STDOUT=true\n\
    volumes:\n\
      - 'wordpress-data:/home/{sftp_user}/html'\n\
    ports:\n\
      - '{sftp_port}:2222'\n\
    restart: unless-stopped\n\
    networks:\n\
      - ssh_net\n\
      - backend_net\n\
    cap_drop:\n\
      - ALL\n\
    cap_add:\n\
      - CHOWN\n\
      - SETUID\n\
      - SETGID\n\
      - DAC_OVERRIDE\n\
      - NET_BIND_SERVICE\n\
    security_opt:\n\
      - no-new-privileges:true\n\
    pids_limit: 100\n\
    deploy:\n\
      resources:\n\
        limits:\n\
          cpus: '{ssh_cpu}'\n\
          memory: {ssh_mem}\n\
        reservations:\n\
          memory: 64M\n")
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
        /* Generar credenciales SSH/SFTP únicas para este hosting */
        let sftp_user: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect::<String>()
            .to_lowercase();
        let sftp_user = format!("wp_{sftp_user}");
        let sftp_password: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(20)
            .map(char::from)
            .collect();

        let compose_yaml = build_hosting_compose(&sftp_user, &sftp_password, sftp_port, plan_config);
        let compose_b64 = base64::engine::general_purpose::STANDARD.encode(&compose_yaml);

        /* Paso 1: Crear el servicio */
        let create_body = CreateServiceBody {
            name: service_name,
            project_uuid: &config.project_uuid,
            environment_name: "production",
            server_uuid: &config.server_uuid,
            docker_compose_raw: compose_b64,
        };

        let create_url = format!("{}/api/v1/services", config.base_url);
        let create_resp = http_client
            .post(&create_url)
            .bearer_auth(&config.api_token)
            .json(&create_body)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Coolify create service request failed: {e}")))?;

        if !create_resp.status().is_success() {
            let status = create_resp.status();
            let body = create_resp.text().await.unwrap_or_default();
            tracing::error!("[Coolify] Error creando servicio '{}': {} — {}", service_name, status, body);
            return Err(AppError::Internal(format!(
                "Coolify create service failed: {status}"
            )));
        }

        let created: CreateServiceResponse = create_resp
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Coolify create service parse error: {e}")))?;

        /* Extraer dominio asignado */
        let domain = created
            .domains
            .into_iter()
            .next()
            .unwrap_or_else(|| format!("http://{}.{}.sslip.io", service_name, config.server_ip));

        tracing::info!(
            "[Coolify] Servicio '{}' creado: uuid={}, domain={}",
            service_name,
            created.uuid,
            domain
        );

        /* Paso 2: Arrancar el servicio */
        let start_url = format!("{}/api/v1/services/{}/start", config.base_url, created.uuid);
        let start_resp = http_client
            .post(&start_url)
            .bearer_auth(&config.api_token)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Coolify start service request failed: {e}")))?;

        if start_resp.status().is_success() {
            tracing::info!("[Coolify] Servicio '{}' iniciado correctamente.", service_name);
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
                tracing::warn!("[Coolify] No se pudo detener servicio {}: {}", service_uuid, r.status());
            }
            Err(e) => {
                tracing::warn!("[Coolify] Error de red al detener servicio {}: {}", service_uuid, e);
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
            .map_err(|e| AppError::Internal(format!("Coolify delete service request failed: {e}")))?;

        if del_resp.status().is_success() {
            tracing::info!("[Coolify] Servicio {} eliminado (delete_volumes={}).", service_uuid, delete_volumes);
        } else {
            let status = del_resp.status();
            let body = del_resp.text().await.unwrap_or_default();
            tracing::error!("[Coolify] Error eliminando servicio {}: {} — {}", service_uuid, status, body);
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
        service_uuid: &str,
        sftp_user: &str,
        sftp_password: &str,
        sftp_port: i32,
        plan_config: &HostingPlanConfig,
    ) -> Result<(), AppError> {
        let compose = build_hosting_compose(sftp_user, sftp_password, sftp_port, plan_config);
        let compose_b64 = base64::engine::general_purpose::STANDARD.encode(&compose);

        /* PATCH: actualizar docker_compose_raw en Coolify */
        let patch_url = format!("{}/api/v1/services/{}", config.base_url, service_uuid);
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
        let stop_url = format!("{}/api/v1/services/{}/stop", config.base_url, service_uuid);
        if let Err(e) = http_client.post(&stop_url).bearer_auth(&config.api_token).send().await {
            tracing::warn!("[Coolify] Error deteniendo servicio {service_uuid}: {e}");
        }
        let start_url = format!("{}/api/v1/services/{}/start", config.base_url, service_uuid);
        let start_resp = http_client
            .post(&start_url)
            .bearer_auth(&config.api_token)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Coolify restart failed: {e}")))?;

        if start_resp.status().is_success() {
            tracing::info!("[Coolify] Servicio {} reiniciado con credenciales actualizadas.", service_uuid);
        } else {
            let status = start_resp.status();
            tracing::warn!("[Coolify] Compose actualizado pero restart falló para {service_uuid}: {status}");
        }

        Ok(())
    }
}

/* ============================================================
   TESTS — [114A-14] Tests completos del servicio de hosting
   ============================================================ */

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;

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

        assert!(compose.contains("wordpress:"), "Debe contener servicio WordPress");
        assert!(compose.contains("mariadb:"), "Debe contener servicio MariaDB");
        assert!(compose.contains("ssh:"), "Debe contener servicio SSH");
    }

    #[test]
    fn compose_contains_required_networks() {
        let config = test_plan_config("basico");
        let compose = build_hosting_compose("testuser", "testpass", 10001, &config);

        assert!(compose.contains("frontend_net:"));
        assert!(compose.contains("backend_net:"));
        assert!(compose.contains("ssh_net:"));
        assert!(compose.contains("internal: true"), "backend_net debe ser internal");
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
        assert!(compose.contains("12345:2222"), "Puerto SSH debe mapearse correctamente");
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
        assert!(cap_drop_count >= 3, "wp, mariadb y ssh deben tener cap_drop: {cap_drop_count}");

        /* no-new-privileges */
        let nnp_count = compose.matches("no-new-privileges:true").count();
        assert!(nnp_count >= 3, "wp, mariadb y ssh deben tener no-new-privileges: {nnp_count}");

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

        assert!(compose.contains("backup:"), "Ecommerce debe incluir sidecar backup");
        assert!(compose.contains("backup-data:"), "Ecommerce debe incluir volumen backup");
        assert!(compose.contains("mysqldump"), "Backup debe usar mysqldump");
    }

    #[test]
    fn compose_basico_excludes_backup_sidecar() {
        let config = test_plan_config("basico");
        let compose = build_hosting_compose("user", "pass", 10001, &config);

        assert!(!compose.contains("backup:"), "Basico no debe incluir sidecar backup");
        assert!(!compose.contains("backup-data:"), "Basico no debe incluir volumen backup");
    }

    #[test]
    fn compose_pro_excludes_backup_sidecar() {
        let config = test_plan_config("pro");
        let compose = build_hosting_compose("user", "pass", 10001, &config);

        assert!(!compose.contains("backup:"), "Pro no debe incluir sidecar backup");
    }

    #[test]
    fn compose_backup_retention_policy() {
        let config = test_plan_config("ecommerce");
        let compose = build_hosting_compose("user", "pass", 10001, &config);

        /* 3 días de backups diarios */
        assert!(compose.contains("-mtime +3 -delete"), "Retención diaria: 3 días");
        /* 14 días (2 semanas) de backups semanales */
        assert!(compose.contains("-mtime +14 -delete"), "Retención semanal: 14 días");
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
        let compose = build_compose_wp_db("0.50", "256M", "0.50", "512M");

        /* WordPress debe estar en frontend (sirve tráfico) y backend (habla con DB) */
        assert!(compose.contains("frontend_net"));
        assert!(compose.contains("backend_net"));
    }

    #[test]
    fn compose_mariadb_only_on_backend() {
        let compose = build_compose_wp_db("0.50", "256M", "0.50", "512M");

        /* MariaDB aparece como servicio "  mariadb:\n". Su sección debe contener
         * backend_net pero NO frontend_net. Extraer desde la definición del servicio. */
        let mariadb_start = compose.find("  mariadb:\n").expect("mariadb service");
        let mariadb_section = &compose[mariadb_start..];
        assert!(mariadb_section.contains("backend_net"));
        assert!(!mariadb_section.contains("frontend_net"), "MariaDB no debe estar en frontend_net");
    }

    #[test]
    fn compose_ssh_on_ssh_net_and_backend() {
        let compose = build_compose_ssh("user", "pass", 10001, "0.25", "128M");

        assert!(compose.contains("ssh_net"));
        assert!(compose.contains("backend_net"));
        /* SSH no debe estar en frontend_net */
        assert!(!compose.contains("frontend_net"), "SSH no debe estar en frontend_net");
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
