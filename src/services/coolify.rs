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
    /// Usuario SFTP generado para acceso a archivos WordPress
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

impl CoolifyService {
    /// Provision completo: crea servicio Nginx en Coolify y lo arranca.
    /// Usa `docker_compose_raw` con Nginx puro (sin `WordPress` ni BD).
    /// El nombre de servicio garantiza idempotencia (Coolify rechaza duplicados).
    ///
    /// # Errors
    /// Retorna `AppError` si la API falla o el JSON no parsea.
    pub async fn provision_hosting(
        http_client: &Client,
        config: &CoolifyConfig,
        service_name: &str,
    ) -> Result<CoolifyProvisionResult, AppError> {
        /* Generar credenciales SFTP únicas para este hosting */
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
        /* Puerto único en rango 10000-65000 para el contenedor SFTP de este hosting.
         * Así cada hosting tiene su propio puerto y no colisionan en el mismo VPS. */
        let sftp_port: i32 = rand::thread_rng().gen_range(10000_i32..65000_i32);

        /* [104A-14] Compose WordPress + MariaDB + SFTP (atmoz/sftp).
         * El contenedor sftp comparte el volumen wordpress-data, dando acceso SFTP
         * a los archivos del sitio sin exponer SSH del VPS. Puerto 2222 para no
         * colisionar con el SSH del host (22). Formato usuarios atmoz: user:pass:uid */
        let compose_yaml = format!(r#"services:
  wordpress:
    image: 'wordpress:latest'
    environment:
      - SERVICE_FQDN_WORDPRESS=
      - WORDPRESS_DB_HOST=mariadb
      - WORDPRESS_DB_USER=wordpress
      - WORDPRESS_DB_PASSWORD=SERVICE_PASSWORD_DB
      - WORDPRESS_DB_NAME=wordpress
    volumes:
      - 'wordpress-data:/var/www/html'
    depends_on:
      - mariadb
    restart: unless-stopped
  mariadb:
    image: 'mariadb:10.11'
    environment:
      - MYSQL_ROOT_PASSWORD=SERVICE_PASSWORD_ROOT
      - MYSQL_DATABASE=wordpress
      - MYSQL_USER=wordpress
      - MYSQL_PASSWORD=SERVICE_PASSWORD_DB
    volumes:
      - 'mariadb-data:/var/lib/mysql'
    restart: unless-stopped
  sftp:
    image: 'atmoz/sftp:latest'
    command: '{sftp_user}:{sftp_password}:33:33:html'
    volumes:
      - 'wordpress-data:/home/{sftp_user}/html'
    ports:
      - '{sftp_port}:22'
    restart: unless-stopped
volumes:
  wordpress-data:
  mariadb-data:
"#);
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
}
