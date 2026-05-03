/* [154A-1] Contabo Domains & DNS API.
 * Extensión del servicio Contabo para gestión de dominios, handles y zonas DNS.
 * Usa la misma autenticación OAuth2 de ContaboService (get_token).
 * API docs: https://api.contabo.com/ — sección Domains, Handles, DNS. */

use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use super::contabo::{ContaboService, API_BASE};

/* ── Tipos: Dominios ──────────────────── */

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContaboDomain {
    pub sld: Option<String>,
    pub tld: Option<String>,
    pub status: Option<String>,
    pub handles: Option<DomainHandles>,
    pub nameservers: Option<Vec<Nameserver>>,
    #[serde(rename = "createdDate")]
    pub created_date: Option<String>,
    #[serde(rename = "paidUntil")]
    pub paid_until: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DomainHandles {
    pub owner: Option<String>,
    pub admin: Option<String>,
    pub tech: Option<String>,
    pub zone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Nameserver {
    pub hostname: Option<String>,
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
}

/* ── Tipos: Handles (contactos WHOIS) ── */

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContaboHandle {
    pub handle_id: Option<String>,
    pub handle_type: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub organization: Option<String>,
    pub email: Option<String>,
    pub gender: Option<String>,
    pub address: Option<HandleAddress>,
    pub phone: Option<HandlePhone>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HandleAddress {
    pub street: Option<String>,
    pub street_number: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub zip_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HandlePhone {
    pub prefix: Option<String>,
    pub number: Option<String>,
}

/* ── Tipos: DNS ────────────────────────── */

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DnsZone {
    pub zone_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DnsRecord {
    pub record_id: Option<i64>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub record_type: Option<String>,
    pub ttl: Option<i64>,
    pub prio: Option<i64>,
    pub data: Option<String>,
    pub port: Option<i64>,
    pub weight: Option<i64>,
    pub flag: Option<i64>,
    pub tag: Option<String>,
}

/* ── Tipos: Requests ───────────────────── */

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderDomainRequest {
    pub domain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_code: Option<String>,
    pub handles: DomainHandles,
    pub nameservers: Vec<Nameserver>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateHandleRequest {
    pub handle_type: String,
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    pub email: String,
    pub gender: String,
    pub address: HandleAddress,
    pub phone: HandlePhone,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDnsRecordRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub record_type: String,
    pub ttl: i64,
    pub prio: i64,
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDnsRecordRequest {
    #[serde(rename = "type")]
    pub record_type: String,
    pub ttl: i64,
    pub prio: i64,
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

/* ── Responses internas (wrapping Contabo) */

#[derive(Debug, Deserialize)]
struct ListResponse<T> {
    data: Vec<T>,
}

#[derive(Deserialize)]
struct AuthCodeEntry {
    #[serde(rename = "authCode")]
    auth_code: String,
}

/* ── Helper para request ID ────────────── */

fn req_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/* ── Implementación ────────────────────── */

impl ContaboService {
    /* ============================
    DOMINIOS
    ============================ */

    /// Verificar si un dominio está disponible para registro.
    /// Retorna true si disponible (204), false si no (404).
    pub async fn check_domain_availability(&self, domain: &str) -> Result<bool, String> {
        let token = self.get_token().await?;

        let resp = self
            .client
            .post(format!(
                "{API_BASE}/registries-domains/{domain}/check-availability"
            ))
            .bearer_auth(&token)
            .header("x-request-id", req_id())
            .send()
            .await
            .map_err(|e| format!("Domain availability check failed: {e}"))?;

        match resp.status().as_u16() {
            204 => Ok(true),
            404 => Ok(false),
            status => {
                let body = resp.text().await.unwrap_or_default();
                error!("Domain availability check error {status}: {body}");
                Err(format!("Domain check error: {status}"))
            }
        }
    }

    /// Listar todos los dominios registrados en Contabo.
    pub async fn list_domains(&self) -> Result<Vec<ContaboDomain>, String> {
        let token = self.get_token().await?;

        let resp = self
            .client
            .get(format!("{API_BASE}/domains"))
            .bearer_auth(&token)
            .header("x-request-id", req_id())
            .query(&[("size", "100")])
            .send()
            .await
            .map_err(|e| format!("List domains failed: {e}"))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("List domains error: {body}"));
        }

        let data: ListResponse<ContaboDomain> = resp
            .json()
            .await
            .map_err(|e| format!("Domain parse error: {e}"))?;

        Ok(data.data)
    }

    /// Obtener detalles de un dominio específico.
    pub async fn get_domain(&self, domain: &str) -> Result<ContaboDomain, String> {
        let token = self.get_token().await?;

        let resp = self
            .client
            .get(format!("{API_BASE}/domains/{domain}"))
            .bearer_auth(&token)
            .header("x-request-id", req_id())
            .send()
            .await
            .map_err(|e| format!("Get domain failed: {e}"))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Get domain {domain} error: {body}"));
        }

        let data: ListResponse<ContaboDomain> = resp
            .json()
            .await
            .map_err(|e| format!("Domain parse error: {e}"))?;

        data.data
            .into_iter()
            .next()
            .ok_or_else(|| format!("Domain {domain} not found"))
    }

    /// Registrar o transferir un dominio.
    pub async fn order_domain(&self, req: &OrderDomainRequest) -> Result<ContaboDomain, String> {
        let token = self.get_token().await?;

        let resp = self
            .client
            .post(format!("{API_BASE}/domains"))
            .bearer_auth(&token)
            .header("x-request-id", req_id())
            .json(req)
            .send()
            .await
            .map_err(|e| format!("Order domain failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("Order domain error {status}: {body}");
            return Err(format!("Order domain error: {status} — {body}"));
        }

        let data: ListResponse<ContaboDomain> = resp
            .json()
            .await
            .map_err(|e| format!("Domain parse error: {e}"))?;

        data.data
            .into_iter()
            .next()
            .ok_or_else(|| "Order domain: empty response".into())
    }

    /// Actualizar nameservers y handles de un dominio.
    pub async fn update_domain(
        &self,
        domain: &str,
        nameservers: Option<Vec<Nameserver>>,
        handles: Option<DomainHandles>,
    ) -> Result<ContaboDomain, String> {
        let token = self.get_token().await?;

        let body = serde_json::json!({
            "nameservers": nameservers,
            "handles": handles,
        });

        let resp = self
            .client
            .patch(format!("{API_BASE}/domains/{domain}"))
            .bearer_auth(&token)
            .header("x-request-id", req_id())
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Update domain failed: {e}"))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Update domain {domain} error: {body}"));
        }

        let data: ListResponse<ContaboDomain> = resp
            .json()
            .await
            .map_err(|e| format!("Domain parse error: {e}"))?;

        data.data
            .into_iter()
            .next()
            .ok_or_else(|| format!("Update domain {domain}: empty response"))
    }

    /// Cancelar un dominio.
    pub async fn cancel_domain(&self, domain: &str, reason: Option<&str>) -> Result<(), String> {
        let token = self.get_token().await?;

        let body = serde_json::json!({
            "reason": reason.unwrap_or("Product not needed anymore"),
        });

        let resp = self
            .client
            .post(format!("{API_BASE}/domains/{domain}/cancel"))
            .bearer_auth(&token)
            .header("x-request-id", req_id())
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Cancel domain failed: {e}"))?;

        if !resp.status().is_success() {
            let body_text = resp.text().await.unwrap_or_default();
            return Err(format!("Cancel domain {domain} error: {body_text}"));
        }

        Ok(())
    }

    /// Obtener auth code para transferencia saliente.
    pub async fn get_domain_auth_code(&self, domain: &str) -> Result<String, String> {
        let token = self.get_token().await?;

        let resp = self
            .client
            .post(format!("{API_BASE}/domains/{domain}/generate-auth-code"))
            .bearer_auth(&token)
            .header("x-request-id", req_id())
            .send()
            .await
            .map_err(|e| format!("Get auth code failed: {e}"))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Auth code error for {domain}: {body}"));
        }

        let data: ListResponse<AuthCodeEntry> = resp
            .json()
            .await
            .map_err(|e| format!("Auth code parse error: {e}"))?;

        data.data
            .into_iter()
            .next()
            .map(|e| e.auth_code)
            .ok_or_else(|| format!("No auth code for {domain}"))
    }

    /* ============================
    HANDLES (contactos WHOIS)
    ============================ */

    /// Listar todos los handles (contactos registrados en Contabo).
    pub async fn list_handles(&self) -> Result<Vec<ContaboHandle>, String> {
        let token = self.get_token().await?;

        let resp = self
            .client
            .get(format!("{API_BASE}/domains/handles"))
            .bearer_auth(&token)
            .header("x-request-id", req_id())
            .query(&[("size", "100")])
            .send()
            .await
            .map_err(|e| format!("List handles failed: {e}"))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("List handles error: {body}"));
        }

        let data: ListResponse<ContaboHandle> = resp
            .json()
            .await
            .map_err(|e| format!("Handle parse error: {e}"))?;

        Ok(data.data)
    }

    /// Crear un handle (contacto para dominios).
    pub async fn create_handle(&self, req: &CreateHandleRequest) -> Result<ContaboHandle, String> {
        let token = self.get_token().await?;

        let resp = self
            .client
            .post(format!("{API_BASE}/domains/handles"))
            .bearer_auth(&token)
            .header("x-request-id", req_id())
            .json(req)
            .send()
            .await
            .map_err(|e| format!("Create handle failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Create handle error: {status} — {body}"));
        }

        let data: ListResponse<ContaboHandle> = resp
            .json()
            .await
            .map_err(|e| format!("Handle parse error: {e}"))?;

        data.data
            .into_iter()
            .next()
            .ok_or_else(|| "Create handle: empty response".into())
    }

    /* ============================
    DNS ZONES
    ============================ */

    /// Listar todas las zonas DNS.
    pub async fn list_dns_zones(&self) -> Result<Vec<DnsZone>, String> {
        let token = self.get_token().await?;

        let resp = self
            .client
            .get(format!("{API_BASE}/dns/zones"))
            .bearer_auth(&token)
            .header("x-request-id", req_id())
            .query(&[("size", "100")])
            .send()
            .await
            .map_err(|e| format!("List DNS zones failed: {e}"))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("List DNS zones error: {body}"));
        }

        let data: ListResponse<DnsZone> = resp
            .json()
            .await
            .map_err(|e| format!("DNS zone parse error: {e}"))?;

        Ok(data.data)
    }

    /// Crear una zona DNS.
    pub async fn create_dns_zone(&self, zone_name: &str) -> Result<DnsZone, String> {
        let token = self.get_token().await?;

        let resp = self
            .client
            .post(format!("{API_BASE}/dns/zones"))
            .bearer_auth(&token)
            .header("x-request-id", req_id())
            .json(&serde_json::json!({"zoneName": zone_name}))
            .send()
            .await
            .map_err(|e| format!("Create DNS zone failed: {e}"))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Create DNS zone error: {body}"));
        }

        let data: ListResponse<DnsZone> = resp
            .json()
            .await
            .map_err(|e| format!("DNS zone parse error: {e}"))?;

        data.data
            .into_iter()
            .next()
            .ok_or_else(|| "Create DNS zone: empty response".into())
    }

    /// Eliminar una zona DNS.
    pub async fn delete_dns_zone(&self, zone_name: &str) -> Result<(), String> {
        let token = self.get_token().await?;

        let resp = self
            .client
            .delete(format!("{API_BASE}/dns/zones/{zone_name}"))
            .bearer_auth(&token)
            .header("x-request-id", req_id())
            .send()
            .await
            .map_err(|e| format!("Delete DNS zone failed: {e}"))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Delete DNS zone {zone_name} error: {body}"));
        }

        Ok(())
    }

    /* ============================
    DNS RECORDS
    ============================ */

    /// Listar registros de una zona DNS.
    pub async fn list_dns_records(&self, zone_name: &str) -> Result<Vec<DnsRecord>, String> {
        let token = self.get_token().await?;

        let resp = self
            .client
            .get(format!("{API_BASE}/dns/zones/{zone_name}/records"))
            .bearer_auth(&token)
            .header("x-request-id", req_id())
            .query(&[("size", "100")])
            .send()
            .await
            .map_err(|e| format!("List DNS records failed: {e}"))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("List DNS records for {zone_name} error: {body}"));
        }

        let data: ListResponse<DnsRecord> = resp
            .json()
            .await
            .map_err(|e| format!("DNS record parse error: {e}"))?;

        Ok(data.data)
    }

    /// Crear un registro DNS en una zona.
    pub async fn create_dns_record(
        &self,
        zone_name: &str,
        req: &CreateDnsRecordRequest,
    ) -> Result<DnsRecord, String> {
        let token = self.get_token().await?;

        let resp = self
            .client
            .post(format!("{API_BASE}/dns/zones/{zone_name}/records"))
            .bearer_auth(&token)
            .header("x-request-id", req_id())
            .json(req)
            .send()
            .await
            .map_err(|e| format!("Create DNS record failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Create DNS record error: {status} — {body}"));
        }

        let data: ListResponse<DnsRecord> = resp
            .json()
            .await
            .map_err(|e| format!("DNS record parse error: {e}"))?;

        data.data
            .into_iter()
            .next()
            .ok_or_else(|| "Create DNS record: empty response".into())
    }

    /// Actualizar un registro DNS.
    pub async fn update_dns_record(
        &self,
        zone_name: &str,
        record_id: i64,
        req: &UpdateDnsRecordRequest,
    ) -> Result<DnsRecord, String> {
        let token = self.get_token().await?;

        let resp = self
            .client
            .patch(format!(
                "{API_BASE}/dns/zones/{zone_name}/records/{record_id}"
            ))
            .bearer_auth(&token)
            .header("x-request-id", req_id())
            .json(req)
            .send()
            .await
            .map_err(|e| format!("Update DNS record failed: {e}"))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Update DNS record {record_id} error: {body}"));
        }

        let data: ListResponse<DnsRecord> = resp
            .json()
            .await
            .map_err(|e| format!("DNS record parse error: {e}"))?;

        data.data
            .into_iter()
            .next()
            .ok_or_else(|| format!("Update DNS record {record_id}: empty response"))
    }

    /// Eliminar un registro DNS.
    pub async fn delete_dns_record(&self, zone_name: &str, record_id: i64) -> Result<(), String> {
        let token = self.get_token().await?;

        let resp = self
            .client
            .delete(format!(
                "{API_BASE}/dns/zones/{zone_name}/records/{record_id}"
            ))
            .bearer_auth(&token)
            .header("x-request-id", req_id())
            .send()
            .await
            .map_err(|e| format!("Delete DNS record failed: {e}"))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Delete DNS record {record_id} error: {body}"));
        }

        Ok(())
    }
}
