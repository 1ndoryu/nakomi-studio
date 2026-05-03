/* [154A-1] Handlers para gestión de dominios Contabo.
 * Endpoints para verificar disponibilidad, comprar/transferir dominios,
 * gestionar zonas DNS y registros, y handles (contactos WHOIS).
 * Solo admin puede comprar/cancelar/transferir dominios.
 * Lectura de dominios y DNS accesible para admin y clientes autorizados. */

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::UserRole;
use crate::services::contabo_domains::{
    ContaboDomain, ContaboHandle, CreateDnsRecordRequest, CreateHandleRequest, DnsRecord, DnsZone,
    DomainHandles, Nameserver, OrderDomainRequest, UpdateDnsRecordRequest,
};
use crate::AppState;

/* ── Helper ────────────────────────────── */

fn contabo(state: &AppState) -> Result<&crate::services::ContaboService, AppError> {
    state
        .contabo_service
        .as_ref()
        .ok_or_else(|| AppError::Internal("Contabo service not configured".into()))
}

fn require_admin(auth: &AuthUser) -> Result<(), AppError> {
    if auth.effective_role != UserRole::Admin {
        return Err(AppError::Forbidden("Admin only".into()));
    }
    Ok(())
}

/* ── Disponibilidad ────────────────────── */

#[derive(Serialize, ToSchema)]
pub struct DomainAvailability {
    pub domain: String,
    pub available: bool,
}

pub async fn check_domain_availability(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(domain): Path<String>,
) -> Result<Json<DomainAvailability>, AppError> {
    let svc = contabo(&state)?;
    let available = svc
        .check_domain_availability(&domain)
        .await
        .map_err(AppError::Internal)?;

    Ok(Json(DomainAvailability { domain, available }))
}

/* ── Listar dominios ───────────────────── */

pub async fn list_domains(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<ContaboDomain>>, AppError> {
    require_admin(&auth)?;
    let svc = contabo(&state)?;
    let domains = svc.list_domains().await.map_err(AppError::Internal)?;
    Ok(Json(domains))
}

/* ── Detalle de un dominio ─────────────── */

pub async fn get_domain(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(domain): Path<String>,
) -> Result<Json<ContaboDomain>, AppError> {
    require_admin(&auth)?;
    let svc = contabo(&state)?;
    let d = svc.get_domain(&domain).await.map_err(AppError::Internal)?;
    Ok(Json(d))
}

/* ── Comprar / transferir dominio ──────── */

#[derive(Debug, Deserialize, ToSchema)]
pub struct OrderDomainBody {
    pub domain: String,
    pub auth_code: Option<String>,
    pub handles: DomainHandles,
    pub nameservers: Vec<Nameserver>,
}

pub async fn order_domain(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<OrderDomainBody>,
) -> Result<(StatusCode, Json<ContaboDomain>), AppError> {
    require_admin(&auth)?;
    let svc = contabo(&state)?;

    let req = OrderDomainRequest {
        domain: body.domain,
        auth_code: body.auth_code,
        handles: body.handles,
        nameservers: body.nameservers,
        resource_type: None,
        resource_id: None,
    };

    let domain = svc.order_domain(&req).await.map_err(AppError::Internal)?;
    Ok((StatusCode::CREATED, Json(domain)))
}

/* ── Actualizar nameservers ────────────── */

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDomainBody {
    pub nameservers: Option<Vec<Nameserver>>,
    pub handles: Option<DomainHandles>,
}

pub async fn update_domain(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(domain): Path<String>,
    Json(body): Json<UpdateDomainBody>,
) -> Result<Json<ContaboDomain>, AppError> {
    require_admin(&auth)?;
    let svc = contabo(&state)?;
    let d = svc
        .update_domain(&domain, body.nameservers, body.handles)
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(d))
}

/* ── Cancelar dominio ──────────────────── */

#[derive(Debug, Deserialize, ToSchema)]
pub struct CancelDomainBody {
    pub reason: Option<String>,
}

pub async fn cancel_domain(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(domain): Path<String>,
    Json(body): Json<CancelDomainBody>,
) -> Result<StatusCode, AppError> {
    require_admin(&auth)?;
    let svc = contabo(&state)?;
    svc.cancel_domain(&domain, body.reason.as_deref())
        .await
        .map_err(AppError::Internal)?;
    Ok(StatusCode::NO_CONTENT)
}

/* ── Auth code (transfer out) ──────────── */

#[derive(Serialize, ToSchema)]
pub struct AuthCodeResponse {
    pub domain: String,
    pub auth_code: String,
}

pub async fn get_auth_code(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(domain): Path<String>,
) -> Result<Json<AuthCodeResponse>, AppError> {
    require_admin(&auth)?;
    let svc = contabo(&state)?;
    let code = svc
        .get_domain_auth_code(&domain)
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(AuthCodeResponse {
        domain,
        auth_code: code,
    }))
}

/* ── Handles (contactos WHOIS) ─────────── */

pub async fn list_handles(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<ContaboHandle>>, AppError> {
    require_admin(&auth)?;
    let svc = contabo(&state)?;
    let handles = svc.list_handles().await.map_err(AppError::Internal)?;
    Ok(Json(handles))
}

pub async fn create_handle(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateHandleRequest>,
) -> Result<(StatusCode, Json<ContaboHandle>), AppError> {
    require_admin(&auth)?;
    let svc = contabo(&state)?;
    let handle = svc.create_handle(&body).await.map_err(AppError::Internal)?;
    Ok((StatusCode::CREATED, Json(handle)))
}

/* ── DNS Zones ─────────────────────────── */

pub async fn list_dns_zones(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<DnsZone>>, AppError> {
    require_admin(&auth)?;
    let svc = contabo(&state)?;
    let zones = svc.list_dns_zones().await.map_err(AppError::Internal)?;
    Ok(Json(zones))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDnsZoneBody {
    pub zone_name: String,
}

pub async fn create_dns_zone(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateDnsZoneBody>,
) -> Result<(StatusCode, Json<DnsZone>), AppError> {
    require_admin(&auth)?;
    let svc = contabo(&state)?;
    let zone = svc
        .create_dns_zone(&body.zone_name)
        .await
        .map_err(AppError::Internal)?;
    Ok((StatusCode::CREATED, Json(zone)))
}

pub async fn delete_dns_zone(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(zone): Path<String>,
) -> Result<StatusCode, AppError> {
    require_admin(&auth)?;
    let svc = contabo(&state)?;
    svc.delete_dns_zone(&zone)
        .await
        .map_err(AppError::Internal)?;
    Ok(StatusCode::NO_CONTENT)
}

/* ── DNS Records ───────────────────────── */

pub async fn list_dns_records(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(zone): Path<String>,
) -> Result<Json<Vec<DnsRecord>>, AppError> {
    require_admin(&auth)?;
    let svc = contabo(&state)?;
    let records = svc
        .list_dns_records(&zone)
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(records))
}

pub async fn create_dns_record(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(zone): Path<String>,
    Json(body): Json<CreateDnsRecordRequest>,
) -> Result<(StatusCode, Json<DnsRecord>), AppError> {
    require_admin(&auth)?;
    let svc = contabo(&state)?;
    let record = svc
        .create_dns_record(&zone, &body)
        .await
        .map_err(AppError::Internal)?;
    Ok((StatusCode::CREATED, Json(record)))
}

pub async fn update_dns_record(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((zone, record_id)): Path<(String, i64)>,
    Json(body): Json<UpdateDnsRecordRequest>,
) -> Result<Json<DnsRecord>, AppError> {
    require_admin(&auth)?;
    let svc = contabo(&state)?;
    let record = svc
        .update_dns_record(&zone, record_id, &body)
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(record))
}

pub async fn delete_dns_record(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((zone, record_id)): Path<(String, i64)>,
) -> Result<StatusCode, AppError> {
    require_admin(&auth)?;
    let svc = contabo(&state)?;
    svc.delete_dns_record(&zone, record_id)
        .await
        .map_err(AppError::Internal)?;
    Ok(StatusCode::NO_CONTENT)
}

/* ── Client DNS (por suscripción) ──────── */
/* El cliente solo puede gestionar registros DNS del dominio ligado a su suscripción.
 * La zona DNS se deduce del dominio de la suscripción. */

use crate::repositories::HostingRepository;
use uuid::Uuid;

async fn client_zone(state: &AppState, auth: &AuthUser, sub_id: Uuid) -> Result<String, AppError> {
    let sub = HostingRepository::find_by_id(&state.pool, sub_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Suscripción no encontrada".into()))?;

    let is_owner = sub.user_id == Some(auth.user_id);
    let is_admin = auth.effective_role == UserRole::Admin;
    if !is_owner && !is_admin {
        return Err(AppError::Forbidden(
            "No tienes acceso a esta suscripción".into(),
        ));
    }

    sub.domain
        .filter(|d| !d.is_empty())
        .ok_or_else(|| AppError::BadRequest("La suscripción no tiene dominio configurado".into()))
}

pub async fn client_list_dns_records(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(sub_id): Path<Uuid>,
) -> Result<Json<Vec<DnsRecord>>, AppError> {
    let zone = client_zone(&state, &auth, sub_id).await?;
    let svc = contabo(&state)?;
    let records = svc
        .list_dns_records(&zone)
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(records))
}

pub async fn client_create_dns_record(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(sub_id): Path<Uuid>,
    Json(body): Json<CreateDnsRecordRequest>,
) -> Result<(StatusCode, Json<DnsRecord>), AppError> {
    let zone = client_zone(&state, &auth, sub_id).await?;
    let svc = contabo(&state)?;
    let record = svc
        .create_dns_record(&zone, &body)
        .await
        .map_err(AppError::Internal)?;
    Ok((StatusCode::CREATED, Json(record)))
}

pub async fn client_update_dns_record(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((sub_id, record_id)): Path<(Uuid, i64)>,
    Json(body): Json<UpdateDnsRecordRequest>,
) -> Result<Json<DnsRecord>, AppError> {
    let zone = client_zone(&state, &auth, sub_id).await?;
    let svc = contabo(&state)?;
    let record = svc
        .update_dns_record(&zone, record_id, &body)
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(record))
}

pub async fn client_delete_dns_record(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((sub_id, record_id)): Path<(Uuid, i64)>,
) -> Result<StatusCode, AppError> {
    let zone = client_zone(&state, &auth, sub_id).await?;
    let svc = contabo(&state)?;
    svc.delete_dns_record(&zone, record_id)
        .await
        .map_err(AppError::Internal)?;
    Ok(StatusCode::NO_CONTENT)
}

/* ── Rutas ─────────────────────────────── */

pub fn domain_routes() -> Router<AppState> {
    Router::new()
        /* Dominios */
        .route(
            "/hosting/domains/check/:domain",
            get(check_domain_availability),
        )
        .route("/hosting/domains", get(list_domains).post(order_domain))
        .route(
            "/hosting/domains/:domain",
            get(get_domain).patch(update_domain),
        )
        .route(
            "/hosting/domains/:domain/cancel",
            axum::routing::post(cancel_domain),
        )
        .route(
            "/hosting/domains/:domain/auth-code",
            axum::routing::post(get_auth_code),
        )
        /* Handles */
        .route("/hosting/handles", get(list_handles).post(create_handle))
        /* DNS Zones (admin) */
        .route(
            "/hosting/dns/zones",
            get(list_dns_zones).post(create_dns_zone),
        )
        .route(
            "/hosting/dns/zones/:zone",
            axum::routing::delete(delete_dns_zone),
        )
        /* DNS Records (admin) */
        .route(
            "/hosting/dns/zones/:zone/records",
            get(list_dns_records).post(create_dns_record),
        )
        .route(
            "/hosting/dns/zones/:zone/records/:record_id",
            axum::routing::patch(update_dns_record).delete(delete_dns_record),
        )
        /* DNS Records (client — por suscripción) */
        .route(
            "/hosting/subscriptions/:sub_id/dns",
            get(client_list_dns_records).post(client_create_dns_record),
        )
        .route(
            "/hosting/subscriptions/:sub_id/dns/:record_id",
            axum::routing::patch(client_update_dns_record).delete(client_delete_dns_record),
        )
}
