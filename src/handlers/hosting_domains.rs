/* [154A-1] Handlers para gestión de dominios Contabo.
 * Endpoints para verificar disponibilidad, comprar/transferir dominios,
 * gestionar zonas DNS y registros, y handles (contactos WHOIS).
 * Solo admin puede comprar/cancelar/transferir dominios.
 * Lectura de dominios y DNS accesible para admin y clientes autorizados. */

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{
    CreateDomainCheckoutRequest, DomainCheckoutResponse, DomainOrder, DomainPriceQuote, UserRole,
};
use crate::repositories::{CreateDomainOrderParams, DomainOrderRepository, UserRepository};
use crate::services::contabo_domains::{
    ContaboDomain, ContaboHandle, CreateDnsRecordRequest, CreateHandleRequest, DnsRecord, DnsZone,
    DomainHandles, Nameserver, OrderDomainRequest, UpdateDnsRecordRequest,
};
use crate::services::{DomainCheckoutParams, DomainStripeService};
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

fn resolve_public_base_url(headers: &HeaderMap) -> String {
    if let Some(origin) = headers.get("origin").and_then(|value| value.to_str().ok()) {
        let trimmed = origin.trim_end_matches('/');
        if !trimmed.is_empty() && !trimmed.contains("localhost") {
            return trimmed.to_string();
        }
    }

    if let Ok(env_url) = std::env::var("GLORY_PUBLIC_URL") {
        return env_url.trim_end_matches('/').to_string();
    }

    "http://localhost:5173".to_string()
}

fn normalize_domain(raw: &str) -> Result<String, AppError> {
    let value = raw
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_matches('/')
        .to_ascii_lowercase();
    if value.contains('/') || value.contains(' ') || !value.contains('.') {
        return Err(AppError::Validation("Dominio inválido".into()));
    }
    Ok(value)
}

fn domain_tld(domain: &str) -> Result<String, AppError> {
    domain
        .rsplit('.')
        .next()
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| AppError::Validation("Dominio sin TLD válido".into()))
}

fn base_domain_cost_cents(tld: &str) -> Result<i32, AppError> {
    match tld {
        "com" => Ok(1200),
        "net" | "org" => Ok(1400),
        "studio" => Ok(3500),
        "io" => Ok(4500),
        _ => Err(AppError::Validation(format!(
            "El TLD .{tld} aún no está habilitado para checkout automático"
        ))),
    }
}

fn domain_price_quote(domain: String, available: bool) -> Result<DomainPriceQuote, AppError> {
    let tld = domain_tld(&domain)?;
    let base_cost_cents = base_domain_cost_cents(&tld)?;
    let price_cents = ((base_cost_cents * 105) + 99) / 100;
    Ok(DomainPriceQuote {
        domain,
        available,
        tld,
        base_cost_cents,
        price_cents,
    })
}

/* ── Disponibilidad ────────────────────── */

pub async fn check_domain_availability(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(domain): Path<String>,
) -> Result<Json<DomainPriceQuote>, AppError> {
    let normalized = normalize_domain(&domain)?;
    let svc = contabo(&state)?;
    let available = svc
        .check_domain_availability(&normalized)
        .await
        .map_err(AppError::Internal)?;

    Ok(Json(domain_price_quote(normalized, available)?))
}

pub async fn list_domain_orders(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<DomainOrder>>, AppError> {
    let orders = if auth.effective_role == UserRole::Admin {
        DomainOrderRepository::list_all(&state.pool).await?
    } else {
        DomainOrderRepository::list_by_user(&state.pool, auth.user_id).await?
    };
    Ok(Json(orders))
}

pub async fn create_domain_checkout(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(body): Json<CreateDomainCheckoutRequest>,
) -> Result<(StatusCode, Json<DomainCheckoutResponse>), AppError> {
    body.validate()
        .map_err(|error| AppError::Validation(error.to_string()))?;
    let normalized = normalize_domain(&body.domain)?;
    let svc = contabo(&state)?;
    let available = svc
        .check_domain_availability(&normalized)
        .await
        .map_err(AppError::Internal)?;
    if !available {
        return Err(AppError::Validation("El dominio no está disponible".into()));
    }

    let quote = domain_price_quote(normalized.clone(), true)?;
    let user = UserRepository::find_by_id(&state.pool, auth.user_id)
        .await?
        .ok_or(AppError::NotFound("Usuario no encontrado".into()))?;
    let order = DomainOrderRepository::create(
        &state.pool,
        CreateDomainOrderParams {
            user_id: auth.user_id,
            domain: &quote.domain,
            tld: &quote.tld,
            base_cost_cents: quote.base_cost_cents,
            price_cents: quote.price_cents,
        },
    )
    .await?;

    let stripe_key = state
        .stripe_secret_key
        .as_deref()
        .ok_or_else(|| AppError::ServiceUnavailable("Stripe no configurado".into()))?;
    let base_url = resolve_public_base_url(&headers);
    let success_url = format!("{base_url}/panel?seccion=dominios&domain=success");
    let cancel_url = format!("{base_url}/panel?seccion=dominios&domain=cancelled");
    let (session_id, checkout_url) =
        DomainStripeService::create_checkout_session(&DomainCheckoutParams {
            http_client: &state.http_client,
            stripe_key,
            order_id: order.id,
            domain: &order.domain,
            amount_cents: order.price_cents,
            customer_email: &user.email,
            success_url: &success_url,
            cancel_url: &cancel_url,
        })
        .await?;
    let order =
        DomainOrderRepository::set_checkout_session(&state.pool, order.id, &session_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(DomainCheckoutResponse {
            order,
            checkout_url,
        }),
    ))
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
        .route("/hosting/domain-orders", get(list_domain_orders))
        .route("/hosting/domains/checkout", post(create_domain_checkout))
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
