/* [084A-24] Servicio Contabo API: autenticación OAuth2 y consulta de instancias VPS.
 * Permite obtener info real de las VPS desde el backend para mostrar stats en el panel.
 * Token se cachea en memoria con expiración. */

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

/// Configuración cargada desde variables de entorno
#[derive(Debug, Clone)]
pub struct ContaboConfig {
    pub client_id: String,
    pub client_secret: String,
    pub api_user: String,
    pub password: String,
}

fn first_env(keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        std::env::var(key)
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

impl ContaboConfig {
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let client_id = first_env(&["CONTABO_CLIENT_ID", "CLIENT_ID_CONTABO"])?;
        let client_secret = first_env(&["CONTABO_CLIENT_SECRET", "CLIENT_SECRET_CONTABO"])?;
        let api_user = first_env(&["CONTABO_API_USER", "API_USER_CONTABO"])?;
        let password = first_env(&["CONTABO_API_PASSWORD", "PASSWORD_CONTABO"])?;

        Some(Self {
            client_id,
            client_secret,
            api_user,
            password,
        })
    }
}

/* Respuestas de la API de Contabo */

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContaboInstance {
    #[serde(rename = "instanceId")]
    pub instance_id: i64,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub name: String,
    pub status: String,
    pub region: String,
    #[serde(rename = "cpuCores")]
    pub cpu_cores: i32,
    #[serde(rename = "ramMb")]
    pub ram_mb: i64,
    #[serde(rename = "diskMb")]
    pub disk_mb: i64,
    #[serde(rename = "ipConfig")]
    pub ip_config: IpConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpConfig {
    pub v4: IpV4Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpV4Config {
    pub ip: String,
}

#[derive(Debug, Deserialize)]
struct InstancesResponse {
    data: Vec<ContaboInstance>,
}

/* Resumen de VPS para exponer al frontend (sin datos sensibles) */
#[derive(Debug, Clone, Serialize)]
pub struct VpsSummary {
    pub instance_id: i64,
    pub name: String,
    pub ip: String,
    pub status: String,
    pub region: String,
    pub cpu_cores: i32,
    pub ram_mb: i64,
    pub disk_mb: i64,
}

pub struct CreateInstanceParams<'a> {
    pub product_id: &'a str,
    pub region: &'a str,
    pub display_name: &'a str,
    pub image_id: Option<&'a str>,
    pub root_password: &'a str,
    pub default_user: &'a str,
    pub user_data: &'a str,
}

impl From<ContaboInstance> for VpsSummary {
    fn from(i: ContaboInstance) -> Self {
        Self {
            instance_id: i.instance_id,
            name: if i.display_name.is_empty() {
                i.name
            } else {
                i.display_name
            },
            ip: i.ip_config.v4.ip,
            status: i.status,
            region: i.region,
            cpu_cores: i.cpu_cores,
            ram_mb: i.ram_mb,
            disk_mb: i.disk_mb,
        }
    }
}

/* Token cacheado con timestamp de expiración */
struct CachedToken {
    token: String,
    expires_at: std::time::Instant,
}

/// Servicio Contabo: obtiene token `OAuth2` y consulta VPS
#[derive(Clone)]
pub struct ContaboService {
    config: ContaboConfig,
    pub(crate) client: reqwest::Client,
    cached_token: Arc<RwLock<Option<CachedToken>>>,
}

const AUTH_URL: &str =
    "https://auth.contabo.com/auth/realms/contabo/protocol/openid-connect/token";
pub(crate) const API_BASE: &str = "https://api.contabo.com/v1";

impl ContaboService {
    #[must_use]
    pub fn new(config: ContaboConfig, client: reqwest::Client) -> Self {
        Self {
            config,
            client,
            cached_token: Arc::new(RwLock::new(None)),
        }
    }

    /// Obtener token `OAuth2` (cacheado, se renueva 60s antes de expirar)
    pub(crate) async fn get_token(&self) -> Result<String, String> {
        /* Leer cache */
        {
            let guard = self.cached_token.read().await;
            if let Some(cached) = guard.as_ref() {
                if cached.expires_at > std::time::Instant::now() + std::time::Duration::from_secs(60)
                {
                    return Ok(cached.token.clone());
                }
            }
        }

        /* Renovar token */
        let params = [
            ("grant_type", "password"),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("username", &self.config.api_user),
            ("password", &self.config.password),
        ];

        let resp = self
            .client
            .post(AUTH_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Contabo auth request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("Contabo auth failed: {status} — {body}");
            return Err(format!("Contabo auth failed: {status} — {body}"));
        }

        let token_resp: TokenResponse = resp
            .json()
            .await
            .map_err(|e| format!("Contabo token parse error: {e}"))?;

        let expires_at = std::time::Instant::now()
            + std::time::Duration::from_secs(token_resp.expires_in);

        let token = token_resp.access_token.clone();

        /* Guardar en cache */
        {
            let mut guard = self.cached_token.write().await;
            *guard = Some(CachedToken {
                token: token_resp.access_token,
                expires_at,
            });
        }

        info!("Contabo token renovado (expira en {}s)", token_resp.expires_in);
        Ok(token)
    }

    /// Listar todas las instancias VPS
    pub async fn list_instances(&self) -> Result<Vec<VpsSummary>, String> {
        let token = self.get_token().await?;
        let request_id = uuid::Uuid::new_v4().to_string();

        let resp = self
            .client
            .get(format!("{API_BASE}/compute/instances"))
            .bearer_auth(&token)
            .header("x-request-id", &request_id)
            .send()
            .await
            .map_err(|e| format!("Contabo list instances failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("Contabo list instances error: {status} — {body}");
            return Err(format!("Contabo API error: {status} — {body}"));
        }

        let data: InstancesResponse = resp
            .json()
            .await
            .map_err(|e| format!("Contabo parse error: {e}"))?;

        Ok(data.data.into_iter().map(VpsSummary::from).collect())
    }

    /// Obtener instancia específica por ID
    pub async fn get_instance(&self, instance_id: i64) -> Result<VpsSummary, String> {
        let token = self.get_token().await?;
        let request_id = uuid::Uuid::new_v4().to_string();

        let resp = self
            .client
            .get(format!("{API_BASE}/compute/instances/{instance_id}"))
            .bearer_auth(&token)
            .header("x-request-id", &request_id)
            .send()
            .await
            .map_err(|e| format!("Contabo get instance failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Contabo instance {instance_id}: {status} — {body}"));
        }

        let data: InstancesResponse = resp
            .json()
            .await
            .map_err(|e| format!("Contabo parse error: {e}"))?;

        data.data
            .into_iter()
            .next()
            .map(VpsSummary::from)
            .ok_or_else(|| format!("Instance {instance_id} not found"))
    }

    pub async fn create_instance(
        &self,
        params: &CreateInstanceParams<'_>,
    ) -> Result<VpsSummary, String> {
        let token = self.get_token().await?;
        let request_id = uuid::Uuid::new_v4().to_string();

        let mut body = serde_json::json!({
            "productId": params.product_id,
            "region": params.region,
            "displayName": params.display_name,
            "rootPassword": params.root_password,
            "defaultUser": params.default_user,
            "userData": params.user_data,
        });

        if let Some(image_id) = params.image_id {
            body["imageId"] = serde_json::json!(image_id);
        }

        let response = self
            .client
            .post(format!("{API_BASE}/compute/instances"))
            .bearer_auth(&token)
            .header("x-request-id", &request_id)
            .json(&body)
            .send()
            .await
            .map_err(|error| format!("Contabo create instance failed: {error}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Contabo create instance error: {status} — {body}");
            return Err(format!("Contabo API create instance error: {status} — {body}"));
        }

        let data: InstancesResponse = response
            .json()
            .await
            .map_err(|error| format!("Contabo create parse error: {error}"))?;

        data.data
            .into_iter()
            .next()
            .map(VpsSummary::from)
            .ok_or_else(|| "Contabo no devolvió la instancia creada".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn clear_contabo_env() {
        for key in [
            "CONTABO_CLIENT_ID",
            "CLIENT_ID_CONTABO",
            "CONTABO_CLIENT_SECRET",
            "CLIENT_SECRET_CONTABO",
            "CONTABO_API_USER",
            "API_USER_CONTABO",
            "CONTABO_API_PASSWORD",
            "PASSWORD_CONTABO",
        ] {
            std::env::remove_var(key);
        }
    }

    #[test]
    fn from_env_accepts_explicit_contabo_names() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        clear_contabo_env();

        std::env::set_var("CONTABO_CLIENT_ID", "client-id");
        std::env::set_var("CONTABO_CLIENT_SECRET", "client-secret");
        std::env::set_var("CONTABO_API_USER", "api-user");
        std::env::set_var("CONTABO_API_PASSWORD", "api-password");

        let config = ContaboConfig::from_env().expect("config should exist");
        assert_eq!(config.client_id, "client-id");
        assert_eq!(config.client_secret, "client-secret");
        assert_eq!(config.api_user, "api-user");
        assert_eq!(config.password, "api-password");

        clear_contabo_env();
    }

    #[test]
    fn from_env_falls_back_to_legacy_names() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        clear_contabo_env();

        std::env::set_var("CLIENT_ID_CONTABO", "legacy-client-id");
        std::env::set_var("CLIENT_SECRET_CONTABO", "legacy-client-secret");
        std::env::set_var("API_USER_CONTABO", "legacy-api-user");
        std::env::set_var("PASSWORD_CONTABO", "legacy-password");

        let config = ContaboConfig::from_env().expect("config should exist");
        assert_eq!(config.client_id, "legacy-client-id");
        assert_eq!(config.client_secret, "legacy-client-secret");
        assert_eq!(config.api_user, "legacy-api-user");
        assert_eq!(config.password, "legacy-password");

        clear_contabo_env();
    }
}
