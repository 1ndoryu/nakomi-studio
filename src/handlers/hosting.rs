/* sentinel-disable-file limite-lineas: controlador legacy de hosting.
 * [164A-17] Ya estaba consolidado como superficie principal del dominio y esta tarea solo
 * reemplaza hardcodes y expone catálogo público sin abrir una refactorización masiva. */
/* [054A-2] Handlers de hosting: CRUD suscripciones + eventos.
 * Admin puede gestionar todo. Clientes pueden actualizar dominio/plan de sus propias
 * suscripciones y solicitar cancelación.
 * [084A-4] Cliente ahora puede editar su suscripción y solicitar cancelación.
 * [084A-24] Endpoints Contabo VPS + Stripe hosting checkout.
 * [164A-19] Añade despliegues reales de Coolify para VPS2 en panel admin. */

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use rand::Rng;
use std::collections::HashMap;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use uuid::Uuid;
use validator::Validate;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{
    AssignHostingRequest, CoolifyDeploymentResponse, CreateHostingRequest, HostingEvent,
    HostingPlanConfig, HostingStatsResponse, HostingSubscriptionResponse, PublicHostingPlan,
    SelfSubscribeRequest, SelfSubscribeResponse, UpdateHostingRequest, UpdateHostingStatusRequest,
    UpdatePlanConfigRequest, UserRole,
};
use crate::repositories::{CreateHostingParams, HostingRepository, ServerInfo, UserRepository};
use crate::services::coolify::CoolifyServiceSummary;
use crate::services::{
    is_checkout_bypass_email, CoolifyConfig, CoolifyService, HostingStripeService,
};
use crate::AppState;

/// Deriva la URL base pública desde Origin header, env var, o fallback dev.
fn resolve_public_base_url(headers: &HeaderMap) -> String {
    // 1. Origin header (enviado por el browser en requests cross-origin y same-origin POST)
    if let Some(origin) = headers.get("origin").and_then(|v| v.to_str().ok()) {
        let trimmed = origin.trim_end_matches('/');
        if !trimmed.is_empty() && !trimmed.contains("localhost") {
            return trimmed.to_string();
        }
    }
    // 2. Referer header como fallback (contiene la URL completa — extraemos scheme://host)
    if let Some(referer) = headers.get("referer").and_then(|v| v.to_str().ok()) {
        if let Some(idx) = referer.find("://") {
            // scheme://host... → encontrar el primer '/' después de "://"
            let after_scheme = &referer[idx + 3..];
            let host_end = after_scheme.find('/').unwrap_or(after_scheme.len());
            let base = &referer[..idx + 3 + host_end];
            if !base.contains("localhost") {
                return base.to_string();
            }
        }
    }
    // 3. Variable de entorno explícita
    if let Ok(env_url) = std::env::var("GLORY_PUBLIC_URL") {
        return env_url.trim_end_matches('/').to_string();
    }
    // 4. Solo en dev
    "http://localhost:5173".to_string()
}

struct HostingPlanMarketing {
    label: &'static str,
    description: &'static str,
    features: &'static [&'static str],
    recommended: bool,
}

fn normal_hosting_base_plan(plan: &str) -> (&str, bool) {
    plan.strip_prefix("normal-")
        .map_or((plan, false), |base| (base, true))
}

fn hosting_plan_marketing(plan: &str) -> HostingPlanMarketing {
    let (base_plan, is_normal) = normal_hosting_base_plan(plan);
    if is_normal {
        /* [155A-20] El catálogo público evita "hosting normal" y describe el caso de uso.
         * Mantiene los slugs `normal-*` por compatibilidad, pero el copy visible habla de
         * hosting administrado para sitios a medida, landings y frontends. */
        return match base_plan {
            "pro" => HostingPlanMarketing {
                label: "Hosting Profesional",
                description: "Hosting administrado para sitios con más tráfico, frontends personalizados y despliegues con mayor exigencia operativa.",
                features: &[
                    "Nginx administrado",
                    "20 GB almacenamiento SSD",
                    "SSL gratuito",
                    "Backups diarios",
                    "3 dominios incluidos",
                    "SFTP seguro",
                    "Recursos aislados",
                ],
                recommended: true,
            },
            "ecommerce" => HostingPlanMarketing {
                label: "Hosting E-commerce",
                description: "Hosting administrado de mayor capacidad para catálogos amplios, assets pesados y operaciones con más demanda.",
                features: &[
                    "Nginx administrado",
                    "50 GB almacenamiento SSD",
                    "SSL gratuito",
                    "Backups diarios + snapshots",
                    "5 dominios incluidos",
                    "SFTP seguro",
                    "Recursos ampliados",
                ],
                recommended: false,
            },
            _ => HostingPlanMarketing {
                label: "Hosting Básico",
                description: "Hosting administrado con Nginx, SSL y SFTP para landings, sitios corporativos y proyectos sin WordPress.",
                features: &[
                    "Nginx administrado",
                    "5 GB almacenamiento SSD",
                    "SSL gratuito",
                    "Backups semanales",
                    "1 dominio incluido",
                    "SFTP seguro",
                ],
                recommended: false,
            },
        };
    }

    match base_plan {
        "pro" => HostingPlanMarketing {
            label: "WordPress Profesional",
            description: "WordPress para negocios que necesitan más recursos, backups diarios y staging listo.",
            features: &[
                "WordPress pre-instalado",
                "20 GB almacenamiento SSD",
                "SSL gratuito",
                "Backups diarios",
                "3 dominios incluidos",
                "WP-CLI vía SSH",
                "Staging environment",
            ],
            recommended: true,
        },
        "ecommerce" => HostingPlanMarketing {
            label: "WordPress E-commerce",
            description: "WooCommerce optimizado para tiendas con más tráfico, caché agresiva y margen operativo dedicado.",
            features: &[
                "WordPress + WooCommerce",
                "50 GB almacenamiento SSD",
                "SSL gratuito",
                "Backups diarios + snapshots",
                "5 dominios incluidos",
                "WP-CLI vía SSH",
                "Caché avanzada WordPress",
            ],
            recommended: false,
        },
        _ => HostingPlanMarketing {
            label: "WordPress Básico",
            description: "WordPress administrado para sitios livianos, landings y contenido institucional con costo controlado.",
            features: &[
                "WordPress pre-instalado",
                "5 GB almacenamiento SSD",
                "SSL gratuito",
                "Backups semanales",
                "1 dominio incluido",
                "WP-CLI vía SSH",
            ],
            recommended: false,
        },
    }
}

fn public_plan_from_config(config: HostingPlanConfig) -> PublicHostingPlan {
    let marketing = hosting_plan_marketing(&config.plan_name);
    PublicHostingPlan {
        plan_name: config.plan_name,
        label: marketing.label.to_string(),
        description: marketing.description.to_string(),
        monthly_price_cents: config.monthly_price_cents,
        wp_cpu_millicores: config.wp_cpu_millicores,
        wp_memory_mb: config.wp_memory_mb,
        db_cpu_millicores: config.db_cpu_millicores,
        db_memory_mb: config.db_memory_mb,
        ssh_cpu_millicores: config.ssh_cpu_millicores,
        ssh_memory_mb: config.ssh_memory_mb,
        storage_limit_mb: config.storage_limit_mb,
        bandwidth_limit_gb: config.bandwidth_limit_gb,
        features: marketing
            .features
            .iter()
            .map(|feature| (*feature).to_string())
            .collect(),
        recommended: marketing.recommended,
    }
}

/* [104A-17] Contabo es un upstream opcional del panel admin.
 * Si falla por credenciales, parseo o indisponibilidad, no debe degradar como 500 opaco. */
fn map_contabo_error(message: &str) -> AppError {
    let lower = message.to_ascii_lowercase();
    tracing::warn!("Contabo request failed: {message}");

    if lower.contains("invalid_grant")
        || lower.contains("auth failed: 400")
        || lower.contains("auth failed: 401")
        || lower.contains("unauthorized")
    {
        return AppError::ServiceUnavailable(
            "Contabo rechazó la autenticación. Revisa CONTABO_API_PASSWORD y las credenciales OAuth2 configuradas.".into(),
        );
    }

    if lower.contains("parse error") {
        return AppError::ServiceUnavailable(
            "Contabo respondió con un formato inesperado. Revisa la integración antes de usar el panel VPS.".into(),
        );
    }

    if lower.contains("api error: 5")
        || lower.contains("timed out")
        || lower.contains("dns")
        || lower.contains("temporarily unavailable")
    {
        return AppError::ServiceUnavailable(
            "Contabo no está disponible temporalmente. Intenta de nuevo en unos minutos.".into(),
        );
    }

    AppError::ServiceUnavailable(
        "No se pudo consultar Contabo. Revisa la configuración y el estado del proveedor.".into(),
    )
}

/* ============================================================
HANDLERS
============================================================ */

/// Listar suscripciones de hosting (admin: todas, cliente: las suyas)
#[utoipa::path(
    get,
    path = "/api/hosting/subscriptions",
    responses(
        (status = 200, description = "Lista de suscripciones", body = Vec<HostingSubscriptionResponse>),
        (status = 401, description = "No autorizado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn list_subscriptions(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<HostingSubscriptionResponse>>, AppError> {
    let subs = HostingRepository::list_all(&state.pool).await?;
    let filtered: Vec<HostingSubscriptionResponse> =
        if auth.effective_role == UserRole::Admin || auth.effective_role == UserRole::Employee {
            subs.into_iter().map(Into::into).collect()
        } else {
            let mut repaired = Vec::new();
            for sub in subs.into_iter().filter(|s| s.user_id == Some(auth.user_id)) {
                repaired.push(
                    maybe_backfill_test_hosting_access(
                        &state,
                        sub,
                        "test_checkout_backfill_list",
                    )
                    .await?
                    .into(),
                );
            }
            repaired
        };
    Ok(Json(filtered))
}

/// Crear suscripción de hosting (semi-automático: crea registro, admin provisiona después)
#[utoipa::path(
    post,
    path = "/api/hosting/subscriptions",
    request_body = CreateHostingRequest,
    responses(
        (status = 201, description = "Suscripción creada", body = HostingSubscriptionResponse),
        (status = 400, description = "Datos inválidos"),
        (status = 401, description = "No autorizado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn create_subscription(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateHostingRequest>,
) -> Result<(StatusCode, Json<HostingSubscriptionResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    /* [114A-3] Precios y límites desde BD (admin-configurable) */
    let plan_config = HostingRepository::get_plan_config(&state.pool, &req.plan)
        .await?
        .ok_or_else(|| {
            AppError::Validation(format!(
                "Plan inválido: {}. Opciones disponibles en /hosting/plan-configs",
                req.plan
            ))
        })?;
    let price = plan_config.monthly_price_cents;
    let storage = plan_config.storage_limit_mb;

    let sub = HostingRepository::create(
        &state.pool,
        CreateHostingParams {
            user_id: Some(auth.user_id),
            client_name: &req.client_name,
            client_email: &req.client_email,
            plan: &req.plan,
            domain: req.domain.as_deref(),
            /* [304A-3] Admin puede vincular a un despliegue Coolify existente */
            coolify_site_name: req.coolify_site_name.as_deref(),
            monthly_price_cents: price,
            storage_limit_mb: storage,
        },
    )
    .await?;

    /* Registrar evento de creación */
    /* [094A-9] Log de errores en evento, no silenciar */
    if let Err(e) = HostingRepository::add_event(
        &state.pool,
        sub.id,
        "created",
        Some(serde_json::json!({"plan": req.plan, "by": auth.user_id.to_string()})),
    )
    .await
    {
        tracing::warn!("Error registrando evento created para {}: {e}", sub.id);
    }

    Ok((StatusCode::CREATED, Json(sub.into())))
}

/* [094A-3] Self-service: cliente contrata hosting + paga en un solo paso.
 * Toma plan y dominio opcional, obtiene nombre/email del perfil del usuario autenticado,
 * crea la suscripción y la Stripe Checkout Session, retorna URL de pago. */
#[utoipa::path(
    post,
    path = "/api/hosting/subscribe",
    request_body = SelfSubscribeRequest,
    responses(
        (status = 201, description = "Suscripción creada + URL de checkout", body = SelfSubscribeResponse),
        (status = 400, description = "Plan inválido"),
        (status = 401, description = "No autorizado"),
        (status = 503, description = "Stripe no configurado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn subscribe_self(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(req): Json<SelfSubscribeRequest>,
) -> Result<(StatusCode, Json<SelfSubscribeResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    /* [114A-3] Precios y límites desde BD (admin-configurable) */
    let plan_config = HostingRepository::get_plan_config(&state.pool, &req.plan)
        .await?
        .ok_or_else(|| {
            AppError::Validation(format!(
                "Plan inválido: {}. Opciones disponibles en /hosting/plan-configs",
                req.plan
            ))
        })?;
    let price = plan_config.monthly_price_cents;
    let storage = plan_config.storage_limit_mb;

    /* Obtener nombre y email del perfil del usuario autenticado */
    let user = UserRepository::find_by_id(&state.pool, auth.user_id)
        .await?
        .ok_or(AppError::NotFound("Usuario no encontrado".into()))?;

    let client_name = user.display_name.unwrap_or_else(|| user.email.clone());
    let client_email = user.email;

    /* Crear la suscripción en estado pending */
    let sub = HostingRepository::create(
        &state.pool,
        CreateHostingParams {
            user_id: Some(auth.user_id),
            client_name: &client_name,
            client_email: &client_email,
            plan: &req.plan,
            domain: req.domain.as_deref(),
            coolify_site_name: None,
            monthly_price_cents: price,
            storage_limit_mb: storage,
        },
    )
    .await?;

    /* [094A-9] Log de errores en evento, no silenciar */
    if let Err(e) = HostingRepository::add_event(
        &state.pool,
        sub.id,
        "created",
        Some(serde_json::json!({
            "plan": req.plan,
            "by": auth.user_id.to_string(),
            "source": "self-service"
        })),
    )
    .await
    {
        tracing::warn!(
            "Error registrando evento created (self-service) para {}: {e}",
            sub.id
        );
    }

    let base_url = resolve_public_base_url(&headers);
    if let Some(response) = complete_test_hosting_checkout(
        &state.pool,
        &state.http_client,
        state.coolify_config.as_ref(),
        &base_url,
        sub.clone(),
        &client_email,
        "self-service",
    )
    .await?
    {
        return Ok((StatusCode::CREATED, Json(response)));
    }

    /* Crear Stripe Checkout Session inmediatamente */
    let stripe_key = state
        .stripe_secret_key
        .as_deref()
        .ok_or_else(|| AppError::ServiceUnavailable("Stripe no configurado".into()))?;

    let success_url =
        format!("{base_url}/panel?hosting=success&session_id={{CHECKOUT_SESSION_ID}}");
    let cancel_url = format!("{base_url}/panel?hosting=cancelled");

    let checkout_url = crate::services::HostingStripeService::create_checkout_session(
        &crate::services::CheckoutParams {
            http_client: &state.http_client,
            stripe_key,
            subscription_id: sub.id,
            plan: &sub.plan,
            amount_cents: sub.monthly_price_cents,
            customer_email: &client_email,
            success_url: &success_url,
            cancel_url: &cancel_url,
        },
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(SelfSubscribeResponse {
            subscription: sub.into(),
            checkout_url,
        }),
    ))
}

async fn complete_test_hosting_checkout(
    pool: &sqlx::PgPool,
    http_client: &reqwest::Client,
    coolify_config: Option<&CoolifyConfig>,
    base_url: &str,
    sub: crate::models::HostingSubscription,
    client_email: &str,
    source: &str,
) -> Result<Option<SelfSubscribeResponse>, AppError> {
    if !is_checkout_bypass_email(client_email) {
        return Ok(None);
    }

    HostingRepository::update_status(pool, sub.id, "active").await?;
    if let Err(e) = HostingRepository::add_event(
        pool,
        sub.id,
        "test_checkout_bypassed",
        Some(serde_json::json!({"email": client_email, "source": source})),
    )
    .await
    {
        tracing::warn!("Error registrando bypass test hosting para {}: {e}", sub.id);
    }

    HostingStripeService::try_auto_provision_subscription(
        pool,
        http_client,
        coolify_config,
        &sub,
        "test_checkout_bypassed",
    )
    .await;

    let subscription = HostingRepository::find_by_id(pool, sub.id)
        .await?
        .unwrap_or(sub);
    let checkout_url = format!(
        "{base_url}/panel?hosting=test-bypass&subscription_id={}",
        subscription.id
    );

    Ok(Some(SelfSubscribeResponse {
        subscription: subscription.into(),
        checkout_url,
    }))
}

/* [165A-1] Los hostings de prueba creados antes del fix quedaron `active` pero sin
 * `server_uuid` ni credenciales SFTP. Cuando el cliente test vuelve al panel,
 * intentamos provisionarlos una sola vez usando el mismo flujo automático. */
async fn maybe_backfill_test_hosting_access(
    state: &AppState,
    sub: crate::models::HostingSubscription,
    source: &str,
) -> Result<crate::models::HostingSubscription, AppError> {
    if sub.status != "active" || sub.server_uuid.is_some() || !is_checkout_bypass_email(&sub.client_email) {
        return Ok(sub);
    }

    HostingStripeService::try_auto_provision_subscription(
        &state.pool,
        &state.http_client,
        state.coolify_config.as_ref(),
        &sub,
        source,
    )
    .await;

    Ok(HostingRepository::find_by_id(&state.pool, sub.id)
        .await?
        .unwrap_or(sub))
}

/// Obtener suscripción por ID
#[utoipa::path(
    get,
    path = "/api/hosting/subscriptions/{id}",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    responses(
        (status = 200, description = "Suscripción", body = HostingSubscriptionResponse),
        (status = 404, description = "No encontrada"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn get_subscription(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<HostingSubscriptionResponse>, AppError> {
    let sub = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    /* Clientes solo ven sus propias suscripciones */
    if auth.effective_role == UserRole::Client && sub.user_id != Some(auth.user_id) {
        return Err(AppError::Forbidden("Sin permisos".into()));
    }

    let sub = maybe_backfill_test_hosting_access(&state, sub, "test_checkout_backfill_detail").await?;

    Ok(Json(sub.into()))
}

/// Actualizar status de suscripción (solo admin)
#[utoipa::path(
    patch,
    path = "/api/hosting/subscriptions/{id}/status",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    request_body = UpdateHostingStatusRequest,
    responses(
        (status = 204, description = "Status actualizado"),
        (status = 400, description = "Status inválido"),
        (status = 403, description = "Sin permisos"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn update_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateHostingStatusRequest>,
) -> Result<StatusCode, AppError> {
    auth.require_role(&[UserRole::Admin])?;
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let valid_statuses = [
        "pending",
        "provisioning",
        "active",
        "suspended",
        "cancelled",
    ];
    if !valid_statuses.contains(&req.status.as_str()) {
        return Err(AppError::Validation(format!(
            "Status inválido: {}. Opciones: {}",
            req.status,
            valid_statuses.join(", ")
        )));
    }

    /* Verificar que existe */
    HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    HostingRepository::update_status(&state.pool, id, &req.status).await?;

    /* [094A-9] Log de errores en evento, no silenciar */
    if let Err(e) = HostingRepository::add_event(
        &state.pool,
        id,
        "status_change",
        Some(serde_json::json!({
            "new_status": req.status,
            "reason": req.reason,
            "by": auth.user_id.to_string()
        })),
    )
    .await
    {
        tracing::warn!("Error registrando evento status_change para {id}: {e}");
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Listar eventos de una suscripción
#[utoipa::path(
    get,
    path = "/api/hosting/subscriptions/{id}/events",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    responses(
        (status = 200, description = "Eventos", body = Vec<HostingEvent>),
        (status = 404, description = "Suscripción no encontrada"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn list_events(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<HostingEvent>>, AppError> {
    /* Verificar acceso */
    let sub = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    if auth.effective_role == UserRole::Client && sub.user_id != Some(auth.user_id) {
        return Err(AppError::Forbidden("Sin permisos".into()));
    }

    let events = HostingRepository::list_events(&state.pool, id, 100).await?;
    Ok(Json(events))
}

/* ============================================================
ROUTES
============================================================ */

/* [074A-65] Actualizar suscripción de hosting (admin: cualquiera, cliente: solo las suyas)
 * [084A-4] Relajado de admin-only a role-aware: cliente valida ownership. */
#[utoipa::path(
    put,
    path = "/api/hosting/subscriptions/{id}",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    request_body = UpdateHostingRequest,
    responses(
        (status = 200, description = "Suscripción actualizada", body = HostingSubscriptionResponse),
        (status = 400, description = "Datos inválidos"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "No encontrada"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn update_subscription(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateHostingRequest>,
) -> Result<Json<HostingSubscriptionResponse>, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let sub = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    /* [084A-4] Clientes solo pueden editar sus propias suscripciones */
    if auth.effective_role == UserRole::Client && sub.user_id != Some(auth.user_id) {
        return Err(AppError::Forbidden(
            "Sin permisos para editar esta suscripción".into(),
        ));
    }

    let plan_config = HostingRepository::get_plan_config(&state.pool, &req.plan)
        .await?
        .ok_or_else(|| {
            AppError::Validation(format!(
                "Plan inválido: {}. Opciones disponibles en /hosting/public-plans",
                req.plan
            ))
        })?;

    let updated = HostingRepository::update(
        &state.pool,
        id,
        &req.plan,
        req.domain.as_deref(),
        plan_config.monthly_price_cents,
        plan_config.storage_limit_mb,
    )
    .await?;

    /* [094A-9] Log de errores en evento, no silenciar */
    if let Err(e) = HostingRepository::add_event(
        &state.pool,
        id,
        "updated",
        Some(serde_json::json!({
            "plan": req.plan,
            "domain": req.domain,
            "by": auth.user_id.to_string()
        })),
    )
    .await
    {
        tracing::warn!("Error registrando evento updated para {id}: {e}");
    }

    /* [154A-4] Si el dominio cambió y hay servicio Coolify, actualizar FQDN.
     * No bloquea el update si falla — se registra como warning. */
    if let Some(ref new_domain) = req.domain {
        let domain_changed = sub.domain.as_deref() != Some(new_domain.as_str());
        if domain_changed && !new_domain.is_empty() {
            if let (Some(ref server_uuid), Some(ref config)) =
                (&sub.server_uuid, &state.coolify_config)
            {
                if let Err(e) = CoolifyService::update_service_domain(
                    &state.http_client,
                    config,
                    server_uuid,
                    new_domain,
                )
                .await
                {
                    tracing::warn!("Coolify FQDN update fallo para {id}: {e}");
                }
            }
        }
    }

    Ok(Json(updated.into()))
}

/* [074A-65] Eliminar suscripción de hosting (solo admin) */
#[utoipa::path(
    delete,
    path = "/api/hosting/subscriptions/{id}",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    responses(
        (status = 204, description = "Eliminada"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "No encontrada"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn delete_subscription(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let sub = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    /* [154A-16] Si tiene servicio Coolify provisionado, eliminarlo antes de borrar BD */
    if let (Some(ref server_uuid), Some(ref coolify_config)) =
        (&sub.server_uuid, &state.coolify_config)
    {
        if let Err(e) =
            CoolifyService::delete_service(&state.http_client, coolify_config, server_uuid, true)
                .await
        {
            tracing::warn!(
                "Error eliminando servicio Coolify {} para suscripción {id}: {e}",
                server_uuid
            );
            /* No bloquear la eliminación de BD por un fallo en Coolify */
        }
    }

    /* CASCADE borra eventos asociados automáticamente */
    HostingRepository::delete(&state.pool, id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/* [084A-4] Solicitar cancelación de hosting (cliente o admin).
 * El cliente puede cancelar sus propias suscripciones activas.
 * No se usa DELETE directamente para clientes — esto cambia el status a cancelled
 * y registra un evento, preservando el registro para auditoría. */
#[utoipa::path(
    post,
    path = "/api/hosting/subscriptions/{id}/cancel",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    responses(
        (status = 204, description = "Cancelación procesada"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "No encontrada"),
        (status = 409, description = "Ya cancelada"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn request_cancel(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let sub = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    /* Clientes solo cancelan las suyas */
    if auth.effective_role == UserRole::Client && sub.user_id != Some(auth.user_id) {
        return Err(AppError::Forbidden(
            "Sin permisos para cancelar esta suscripción".into(),
        ));
    }

    if sub.status == "cancelled" {
        return Err(AppError::Conflict(
            "La suscripción ya está cancelada".into(),
        ));
    }

    HostingRepository::update_status(&state.pool, id, "cancelled").await?;

    /* [094A-9] Log de errores en evento, no silenciar */
    if let Err(e) = HostingRepository::add_event(
        &state.pool,
        id,
        "status_change",
        Some(serde_json::json!({
            "new_status": "cancelled",
            "reason": "Cancelación solicitada por el usuario",
            "by": auth.user_id.to_string()
        })),
    )
    .await
    {
        tracing::warn!("Error registrando evento cancellation para {id}: {e}");
    }

    Ok(StatusCode::NO_CONTENT)
}

/* ============================================================
STRIPE CHECKOUT — Suscripción de hosting
[084A-24] Crea Stripe Checkout Session para pagar hosting mensual
============================================================ */

/// Crear Checkout Session de Stripe para suscripción de hosting
#[utoipa::path(
    post,
    path = "/api/hosting/subscriptions/{id}/checkout",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    responses(
        (status = 200, description = "URL de checkout"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "No encontrada"),
        (status = 503, description = "Stripe no configurado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn create_checkout(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let sub = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    /* Solo el dueño o admin puede generar checkout */
    if auth.effective_role == UserRole::Client && sub.user_id != Some(auth.user_id) {
        return Err(AppError::Forbidden("Sin permisos".into()));
    }

    /* Ya tiene suscripción Stripe activa */
    if sub.stripe_subscription_id.is_some() && sub.status == "active" {
        return Err(AppError::Conflict(
            "La suscripción ya tiene un pago activo en Stripe".into(),
        ));
    }

    /* URLs de retorno (frontend SPA) */
    let base_url = resolve_public_base_url(&headers);
    if is_checkout_bypass_email(&sub.client_email) {
        HostingRepository::update_status(&state.pool, sub.id, "active").await?;
        if let Err(e) = HostingRepository::add_event(
            &state.pool,
            sub.id,
            "test_checkout_bypassed",
            Some(serde_json::json!({"email": sub.client_email, "source": "existing-subscription-checkout"})),
        )
        .await
        {
            tracing::warn!("Error registrando bypass test hosting existente para {}: {e}", sub.id);
        }
        return Ok(Json(serde_json::json!({
            "checkout_url": format!("{base_url}/panel?hosting=test-bypass&subscription_id={}", sub.id)
        })));
    }

    let stripe_key = state
        .stripe_secret_key
        .as_deref()
        .ok_or_else(|| AppError::ServiceUnavailable("Stripe no configurado".into()))?;

    let success_url =
        format!("{base_url}/panel?hosting=success&session_id={{CHECKOUT_SESSION_ID}}");
    let cancel_url = format!("{base_url}/panel?hosting=cancelled");

    let url = crate::services::HostingStripeService::create_checkout_session(
        &crate::services::CheckoutParams {
            http_client: &state.http_client,
            stripe_key,
            subscription_id: id,
            plan: &sub.plan,
            amount_cents: sub.monthly_price_cents,
            customer_email: &sub.client_email,
            success_url: &success_url,
            cancel_url: &cancel_url,
        },
    )
    .await?;

    Ok(Json(serde_json::json!({ "checkout_url": url })))
}

/* ============================================================
HOSTING STATS — Estadísticas reales de una suscripción
[094A-8] Calcula uptime desde eventos, muestra límites reales del plan.
[114A-15+] Docker stats reales via SSH para CPU/RAM.
============================================================ */

/* [114A-15+] Obtiene estadísticas de contenedores Docker via SSH.
 * Usa cache de 30s para evitar SSH en cada request.
 * Retorna (cpu_percent, ram_used, ram_limit, containers) — todo None si SSH no disponible. */
async fn fetch_container_resources(
    state: &AppState,
    sub: &crate::models::HostingSubscription,
) -> (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<Vec<crate::services::docker_stats::ContainerStats>>,
) {
    let coolify_name = match &sub.coolify_site_name {
        Some(name) if !name.is_empty() => name,
        _ => return (None, None, None, None),
    };
    let server_ip = match &sub.server_ip {
        Some(ip) if !ip.is_empty() => ip,
        _ => return (None, None, None, None),
    };
    let Some(ssh_key) = state
        .coolify_config
        .as_ref()
        .and_then(|c| c.ssh_key_path.as_ref())
    else {
        return (None, None, None, None);
    };

    /* Cache check */
    let cache_key = format!("{server_ip}:{coolify_name}");
    if let Some(cached) = state.docker_stats_cache.get(&cache_key).await {
        return (
            Some(cached.total_cpu_percent),
            Some(cached.total_ram_used_mb),
            Some(cached.total_ram_limit_mb),
            Some(cached.containers),
        );
    }

    /* Fetch via SSH */
    match crate::services::docker_stats::fetch_docker_stats(server_ip, ssh_key, coolify_name).await
    {
        Ok(resource_stats) => {
            let result = (
                Some(resource_stats.total_cpu_percent),
                Some(resource_stats.total_ram_used_mb),
                Some(resource_stats.total_ram_limit_mb),
                Some(resource_stats.containers.clone()),
            );
            state
                .docker_stats_cache
                .set(cache_key, resource_stats)
                .await;
            result
        }
        Err(e) => {
            tracing::warn!("Docker stats fallo para {coolify_name}@{server_ip}: {e}");
            (None, None, None, None)
        }
    }
}

/* [154A-3] Obtiene el uso de disco real del contenedor WordPress via SSH.
 * Sigue el mismo patrón que fetch_container_resources: requiere SSH key + server_ip. */
async fn fetch_storage_used(
    state: &AppState,
    sub: &crate::models::HostingSubscription,
) -> Option<i64> {
    let coolify_name = sub.coolify_site_name.as_deref().filter(|n| !n.is_empty())?;
    let server_ip = sub.server_ip.as_deref().filter(|ip| !ip.is_empty())?;
    let ssh_key = state.coolify_config.as_ref()?.ssh_key_path.as_ref()?;

    match crate::services::docker_stats::fetch_storage_usage(server_ip, ssh_key, coolify_name).await
    {
        Ok(mb) => Some(mb),
        Err(e) => {
            tracing::warn!("Storage check fallo para {coolify_name}@{server_ip}: {e}");
            None
        }
    }
}

/* [094A-8] Calcula el porcentaje de uptime analizando transiciones de status en eventos.
 * Recorre los eventos cronológicamente, contando el tiempo total en estado "active".
 * Si la suscripción actualmente está activa, el período abierto se extiende hasta ahora. */
#[allow(clippy::cast_precision_loss)] /* Duración en segundos cabe en f64 sin pérdida práctica */
fn calculate_uptime(
    created_at: chrono::DateTime<chrono::Utc>,
    current_status: &str,
    events: &[crate::models::HostingEvent],
) -> (f64, Option<chrono::DateTime<chrono::Utc>>) {
    let now = chrono::Utc::now();
    let total_duration = (now - created_at).num_seconds().max(1) as f64;

    /* Recorrer eventos cronológicamente para rastrear períodos activos */
    let mut active_seconds: f64 = 0.0;
    let mut last_active_start: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut first_active: Option<chrono::DateTime<chrono::Utc>> = None;

    /* Eventos vienen DESC del repo, revertir para orden cronológico */
    let mut sorted_events: Vec<&crate::models::HostingEvent> = events.iter().collect();
    sorted_events.sort_by_key(|e| e.created_at);

    for event in &sorted_events {
        if event.event_type != "status_change" {
            continue;
        }
        let new_status = event
            .details
            .as_ref()
            .and_then(|d| d.get("new_status"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match new_status {
            "active" => {
                last_active_start = Some(event.created_at);
                if first_active.is_none() {
                    first_active = Some(event.created_at);
                }
            }
            _ => {
                /* Transición fuera de active: cerrar el período activo abierto */
                if let Some(start) = last_active_start.take() {
                    active_seconds += (event.created_at - start).num_seconds().max(0) as f64;
                }
            }
        }
    }

    /* Si actualmente está activo, cerrar el período abierto hasta ahora */
    if current_status == "active" {
        if let Some(start) = last_active_start {
            active_seconds += (now - start).num_seconds().max(0) as f64;
        } else if first_active.is_none() {
            /* Nunca hubo un evento status_change a "active" explícito pero el status es active
             * (puede pasar con suscripciones migradas). Asumir active desde creación. */
            first_active = Some(created_at);
            active_seconds = total_duration;
        }
    }

    let uptime = if total_duration > 0.0 {
        (active_seconds / total_duration * 100.0).min(100.0)
    } else {
        0.0
    };

    (uptime, first_active)
}

/// Estadísticas de uso de una suscripción de hosting
#[utoipa::path(
    get,
    path = "/api/hosting/subscriptions/{id}/stats",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    responses(
        (status = 200, description = "Estadísticas de la suscripción", body = HostingStatsResponse),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "Suscripción no encontrada"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn get_hosting_stats(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<HostingStatsResponse>, AppError> {
    let sub = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    /* Clientes solo ven stats de sus propias suscripciones */
    if auth.effective_role == UserRole::Client && sub.user_id != Some(auth.user_id) {
        return Err(AppError::Forbidden("Sin permisos".into()));
    }

    let events = HostingRepository::list_events(&state.pool, id, 1000).await?;
    let (uptime_percent, active_since) = calculate_uptime(sub.created_at, &sub.status, &events);

    let total_events = i64::try_from(events.len()).unwrap_or(i64::MAX);
    let last_event_at = events.first().map(|e| e.created_at);
    let monitoring_available = sub.coolify_site_name.is_some();

    let bandwidth = HostingRepository::get_plan_config(&state.pool, &sub.plan)
        .await?
        .map(|config| config.bandwidth_limit_gb)
        .ok_or_else(|| {
            AppError::Internal(format!(
                "Plan config '{}' no encontrado para stats",
                sub.plan
            ))
        })?;

    /* [114A-15+] Docker stats reales via SSH si está configurado */
    let (cpu_percent, ram_used_mb, ram_limit_mb, containers) =
        fetch_container_resources(&state, &sub).await;

    /* [154A-3] Storage real via SSH — du -sm en el contenedor WordPress */
    let storage_used_mb = fetch_storage_used(&state, &sub).await;

    Ok(Json(HostingStatsResponse {
        storage_limit_mb: sub.storage_limit_mb,
        storage_used_mb,
        bandwidth_limit_gb: bandwidth,
        bandwidth_used_gb: None,
        uptime_percent,
        active_since,
        total_events,
        last_event_at,
        monitoring_available,
        cpu_percent,
        ram_used_mb,
        ram_limit_mb,
        containers,
    }))
}

/* ============================================================
DESPLIEGUES REALES VPS — Coolify (solo admin)
[164A-19] Corrige la tarea mal cerrada: Contabo lista servidores, no despliegues.
[VPS1-support] Ahora consulta ambos Coolify (VPS principal + VPS2) y retorna
resultados unificados con server_label para distinguir origen.
============================================================ */

/// Listar despliegues reales de todas las VPS configuradas en Coolify (admin only)
#[utoipa::path(
    get,
    path = "/api/hosting/deployments",
    responses(
        (status = 200, description = "Lista de despliegues reales en Coolify (todas las VPS)", body = Vec<CoolifyDeploymentResponse>),
        (status = 403, description = "Sin permisos"),
        (status = 503, description = "Coolify no configurado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn list_vps2_deployments(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<CoolifyDeploymentResponse>>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    /* Necesitamos al menos un config disponible */
    if state.coolify_config.is_none() && state.coolify_config_vps1.is_none() {
        return Err(AppError::ServiceUnavailable(
            "Coolify no está configurado para listar despliegues".into(),
        ));
    }

    let subscriptions = HostingRepository::list_all(&state.pool).await?;

    let subscriptions_by_uuid: HashMap<&str, _> = subscriptions
        .iter()
        .filter_map(|subscription| {
            subscription
                .server_uuid
                .as_deref()
                .map(|server_uuid| (server_uuid, subscription))
        })
        .collect();
    let subscriptions_by_name: HashMap<&str, _> = subscriptions
        .iter()
        .filter_map(|subscription| {
            subscription
                .coolify_site_name
                .as_deref()
                .map(|site_name| (site_name, subscription))
        })
        .collect();

    /* Helper: mapea servicios Coolify a CoolifyDeploymentResponse con una etiqueta de servidor */
    let map_services = |services: Vec<_>, label: &str| -> Vec<CoolifyDeploymentResponse> {
        services
            .into_iter()
            .map(|service: CoolifyServiceSummary| {
                let linked_subscription = subscriptions_by_uuid
                    .get(service.uuid.as_str())
                    .copied()
                    .or_else(|| subscriptions_by_name.get(service.name.as_str()).copied());

                CoolifyDeploymentResponse {
                    uuid: service.uuid,
                    name: service.name,
                    status: service.status,
                    fqdn: service.fqdn,
                    server_uuid: service.server_uuid,
                    server_name: service.server_name,
                    project_uuid: service.project_uuid,
                    environment_name: service.environment_name,
                    linked_subscription_id: linked_subscription.map(|s| s.id),
                    linked_subscription_domain: linked_subscription.and_then(|s| s.domain.clone()),
                    linked_subscription_status: linked_subscription.map(|s| s.status.clone()),
                    linked_subscription_plan: linked_subscription.map(|s| s.plan.clone()),
                    server_label: label.to_string(),
                }
            })
            .collect()
    };

    let mut deployments: Vec<CoolifyDeploymentResponse> = Vec::new();

    /* VPS principal (COOLIFY_VPS1_*) */
    if let Some(cfg) = state.coolify_config_vps1.as_ref() {
        match CoolifyService::list_services(&state.http_client, cfg).await {
            Ok(services) => deployments.extend(map_services(services, "VPS Principal")),
            Err(e) => tracing::warn!("[deployments] Error listando VPS1: {e}"),
        }
    }

    /* VPS2 (COOLIFY_*) */
    if let Some(cfg) = state.coolify_config.as_ref() {
        match CoolifyService::list_services(&state.http_client, cfg).await {
            Ok(services) => deployments.extend(map_services(services, "VPS2")),
            Err(e) => tracing::warn!("[deployments] Error listando VPS2: {e}"),
        }
    }

    deployments.sort_by(|left, right| {
        right
            .linked_subscription_id
            .is_some()
            .cmp(&left.linked_subscription_id.is_some())
            .then(left.name.cmp(&right.name))
            .then(left.uuid.cmp(&right.uuid))
    });

    Ok(Json(deployments))
}

/* ============================================================
VPS STATS — Proxy a Contabo API (solo admin)
[084A-24] Permite al panel admin ver estado real de las VPS
============================================================ */

/// Listar instancias VPS (admin only — proxy Contabo API)
#[utoipa::path(
    get,
    path = "/api/hosting/vps",
    responses(
        (status = 200, description = "Lista de VPS"),
        (status = 403, description = "Sin permisos"),
        (status = 503, description = "Contabo no configurado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn list_vps(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let service = state
        .contabo_service
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Contabo API no configurada".into()))?;

    let instances = service
        .list_instances()
        .await
        .map_err(|error| map_contabo_error(&error))?;

    Ok(Json(serde_json::json!({ "data": instances })))
}

/// Obtener instancia VPS por ID (admin only)
#[utoipa::path(
    get,
    path = "/api/hosting/vps/{instance_id}",
    params(("instance_id" = i64, Path, description = "ID de instancia Contabo")),
    responses(
        (status = 200, description = "Detalles de la VPS"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "Instancia no encontrada"),
        (status = 503, description = "Contabo no configurado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn get_vps(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(instance_id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let service = state
        .contabo_service
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Contabo API no configurada".into()))?;

    let instance = service.get_instance(instance_id).await.map_err(|e| {
        if e.to_ascii_lowercase().contains("not found") {
            AppError::NotFound(format!("VPS {instance_id} no encontrada"))
        } else {
            map_contabo_error(&e)
        }
    })?;

    Ok(Json(serde_json::json!({ "data": instance })))
}

/* ============================================================
PROVISIONING — Crear servicio real en Coolify (solo admin)
[154A-11] Endpoint que conecta la suscripción pendiente con Coolify VPS2
============================================================ */

/// Provisionar un hosting: crea servicio Nginx en Coolify y actualiza la suscripción.
/// Solo admin. La suscripción debe estar en estado "pending" o "provisioning".
#[utoipa::path(
    post,
    path = "/api/hosting/subscriptions/{id}/provision",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    responses(
        (status = 200, description = "Hosting provisionado", body = HostingSubscriptionResponse),
        (status = 400, description = "Estado inválido para provisioning"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "Suscripción no encontrada"),
        (status = 503, description = "Coolify no configurado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn provision_subscription(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<HostingSubscriptionResponse>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let sub = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Suscripción {id} no encontrada")))?;

    /* Solo se provisiona desde pending o provisioning (reintento) */
    if sub.status != "pending" && sub.status != "provisioning" {
        return Err(AppError::Validation(format!(
            "Solo se puede provisionar hostings en estado 'pending' o 'provisioning', actual: '{}'",
            sub.status
        )));
    }

    let config = state.coolify_config.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailable("Coolify no configurado. Variables COOLIFY_* ausentes.".into())
    })?;

    /* Marcar como provisioning antes de llamar a Coolify */
    HostingRepository::update_status(&state.pool, id, "provisioning").await?;

    let service_name = CoolifyService::service_name_for(&id);

    /* [164A-16] Generar puerto SFTP único con verificación en BD */
    let sftp_port = HostingRepository::find_available_sftp_port(&state.pool).await?;

    /* [114A-3] Obtener config del plan para límites dinámicos */
    let plan_config = HostingRepository::get_plan_config(&state.pool, &sub.plan)
        .await?
        .ok_or_else(|| {
            AppError::Internal(format!("Plan config '{}' no encontrado en BD", sub.plan))
        })?;

    let result = match CoolifyService::provision_hosting(
        &state.http_client,
        config,
        &service_name,
        sftp_port,
        &plan_config,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            /* Revertir a pending si Coolify falla —  admin puede reintentar */
            tracing::error!("[Provision] Falló para {id}: {e}");
            HostingRepository::update_status(&state.pool, id, "pending")
                .await
                .ok();
            return Err(e);
        }
    };

    /* Guardar datos reales del servidor */
    HostingRepository::update_server_info(
        &state.pool,
        id,
        &ServerInfo {
            coolify_site_name: &service_name,
            server_uuid: &result.service_uuid,
            server_ip: &result.server_ip,
            sftp_user: &result.sftp_user,
            sftp_password: &result.sftp_password,
            sftp_port: result.sftp_port,
        },
    )
    .await?;

    /* Activar la suscripción */
    HostingRepository::update_status(&state.pool, id, "active").await?;

    /* Evento de auditoría */
    if let Err(e) = HostingRepository::add_event(
        &state.pool,
        id,
        "provisioned",
        Some(serde_json::json!({
            "coolify_uuid": result.service_uuid,
            "domain": result.domain,
            "server_ip": result.server_ip,
            "service_name": service_name,
            "by": auth.user_id.to_string(),
        })),
    )
    .await
    {
        tracing::warn!("Error registrando evento provisioned para {id}: {e}");
    }

    let updated = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::Internal("Suscripción perdida tras provisioning".into()))?;

    Ok(Json(updated.into()))
}

/* [154A-16] Verificación DNS: resuelve el dominio y compara con server_ip.
 * Retorna qué IPs apuntan al dominio y si coinciden con el servidor. */
#[utoipa::path(
    get,
    path = "/api/hosting/subscriptions/{id}/dns-check",
    params(("id" = Uuid, Path, description = "ID suscripción")),
    responses(
        (status = 200, description = "Resultado verificación DNS", body = serde_json::Value),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
async fn dns_check(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let sub = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    /* Solo el dueño o admin pueden verificar DNS */
    if auth.effective_role != UserRole::Admin && sub.user_id != Some(auth.user_id) {
        return Err(AppError::Forbidden("Sin permisos".into()));
    }

    let domain = match &sub.domain {
        Some(d) if !d.is_empty() => d.clone(),
        _ => {
            return Ok(Json(serde_json::json!({
                "configured": false,
                "message": "No hay dominio configurado"
            })));
        }
    };

    let server_ip = sub.server_ip.clone().unwrap_or_default();

    /* Resolver DNS usando tokio (no bloquea el runtime) */
    let lookup_target = format!("{domain}:80");
    let resolved_ips: Vec<String> = match tokio::net::lookup_host(&lookup_target).await {
        Ok(addrs) => addrs.map(|a| a.ip().to_string()).collect(),
        Err(e) => {
            return Ok(Json(serde_json::json!({
                "configured": true,
                "domain": domain,
                "resolved": false,
                "error": format!("No se pudo resolver el dominio: {e}"),
                "expected_ip": server_ip,
            })));
        }
    };

    /* Deduplicar IPs */
    let mut unique_ips: Vec<String> = resolved_ips.clone();
    unique_ips.sort();
    unique_ips.dedup();

    let points_to_server = !server_ip.is_empty() && unique_ips.contains(&server_ip);

    Ok(Json(serde_json::json!({
        "configured": true,
        "domain": domain,
        "resolved": true,
        "resolved_ips": unique_ips,
        "expected_ip": server_ip,
        "points_to_server": points_to_server,
        "ssl_provider": "Let's Encrypt (automático vía Coolify)",
    })))
}

/* [114A-1] Rotación de credenciales SFTP: genera nueva contraseña, actualiza BD y
 * compose en Coolify, reinicia servicio SSH para que tome efecto.
 * Solo admin. El hosting debe estar provisionado (server_uuid presente). */
#[utoipa::path(
    post,
    path = "/api/hosting/subscriptions/{id}/rotate-credentials",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    responses(
        (status = 200, description = "Credenciales rotadas"),
        (status = 400, description = "Hosting no provisionado"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "No encontrada"),
        (status = 503, description = "Coolify no configurado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn rotate_credentials(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let sub = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    let server_uuid = sub.server_uuid.as_ref().ok_or_else(|| {
        AppError::Validation("Hosting no provisionado — no se pueden rotar credenciales".into())
    })?;
    let sftp_user = sub.sftp_user.as_ref().ok_or_else(|| {
        AppError::Internal("SFTP user ausente en suscripción provisionada".into())
    })?;
    let sftp_port = sub.sftp_port.ok_or_else(|| {
        AppError::Internal("SFTP port ausente en suscripción provisionada".into())
    })?;

    let config = state
        .coolify_config
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Coolify no configurado".into()))?;

    /* Generar contraseña nueva (20 chars alfanumeric) */
    let new_password: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();

    /* Actualizar en BD primero (si Coolify falla, la BD queda con la nueva y se reintenta) */
    HostingRepository::update_sftp_password(&state.pool, id, &new_password).await?;

    /* [114A-3] Obtener config del plan para límites dinámicos en el compose regenerado */
    let plan_config = HostingRepository::get_plan_config(&state.pool, &sub.plan)
        .await?
        .ok_or_else(|| {
            AppError::Internal(format!("Plan config '{}' no encontrado en BD", sub.plan))
        })?;

    /* Regenerar compose con nueva contraseña y aplicar en Coolify */
    CoolifyService::update_compose_and_restart(
        &state.http_client,
        config,
        server_uuid,
        sftp_user,
        &new_password,
        sftp_port,
        &plan_config,
    )
    .await?;

    if let Err(e) = HostingRepository::add_event(
        &state.pool,
        id,
        "credentials_rotated",
        Some(serde_json::json!({"by": auth.user_id.to_string()})),
    )
    .await
    {
        tracing::warn!("Error registrando evento credentials_rotated para {id}: {e}");
    }

    Ok(Json(serde_json::json!({
        "sftp_user": sftp_user,
        "sftp_password": new_password,
        "sftp_port": sftp_port,
    })))
}

/* ============================================================
PLAN CONFIGS — Configuración de recursos por plan (admin)
[114A-3] Centraliza precios y límites; admin puede ajustarlos sin redeploy.
============================================================ */

/// Listar todas las configuraciones de planes de hosting (admin)
#[utoipa::path(
    get,
    path = "/api/hosting/plan-configs",
    responses(
        (status = 200, description = "Configuraciones de planes", body = Vec<HostingPlanConfig>),
        (status = 403, description = "Sin permisos"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn list_plan_configs(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<HostingPlanConfig>>, AppError> {
    auth.require_role(&[UserRole::Admin])?;
    let configs = HostingRepository::list_plan_configs(&state.pool).await?;
    Ok(Json(configs))
}

#[utoipa::path(
    get,
    path = "/api/hosting/public-plans",
    responses(
        (status = 200, description = "Catálogo público de hosting", body = Vec<PublicHostingPlan>),
    ),
    tag = "hosting"
)]
pub async fn list_public_plans(
    State(state): State<AppState>,
) -> Result<Json<Vec<PublicHostingPlan>>, AppError> {
    let configs = HostingRepository::list_plan_configs(&state.pool).await?;
    Ok(Json(
        configs.into_iter().map(public_plan_from_config).collect(),
    ))
}

/// Actualizar configuración de un plan (admin). Campos opcionales: solo se actualizan los enviados.
#[utoipa::path(
    put,
    path = "/api/hosting/plan-configs/{plan}",
    params(("plan" = String, Path, description = "Nombre del plan (basico, pro, ecommerce)")),
    request_body = UpdatePlanConfigRequest,
    responses(
        (status = 200, description = "Config actualizada", body = HostingPlanConfig),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "Plan no encontrado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn update_plan_config(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(plan): Path<String>,
    Json(req): Json<UpdatePlanConfigRequest>,
) -> Result<Json<HostingPlanConfig>, AppError> {
    auth.require_role(&[UserRole::Admin])?;
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let updated = HostingRepository::update_plan_config(&state.pool, &plan, &req).await?;
    Ok(Json(updated))
}

/* [114A-4] Refresh: regenera compose con plan config actual y redeploya en Coolify.
 * Usado para migrar hostings existentes cuando se cambian limits/features del plan.
 * Solo funciona en hostings provisionados (con server_uuid y sftp_user). */
#[utoipa::path(
    post,
    path = "/api/hosting/subscriptions/{id}/refresh",
    params(("id" = Uuid, Path, description = "ID suscripción")),
    responses(
        (status = 200, description = "Hosting redeployado con config actual"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "Suscripción no encontrada"),
        (status = 503, description = "Coolify no configurado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn refresh_hosting(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let sub = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    let server_uuid = sub.server_uuid.as_ref().ok_or_else(|| {
        AppError::Validation("Hosting no provisionado — no se puede refrescar".into())
    })?;
    let sftp_user = sub.sftp_user.as_ref().ok_or_else(|| {
        AppError::Internal("SFTP user ausente en suscripción provisionada".into())
    })?;
    let sftp_password = sub.sftp_password.as_ref().ok_or_else(|| {
        AppError::Internal("SFTP password ausente en suscripción provisionada".into())
    })?;
    let sftp_port = sub.sftp_port.ok_or_else(|| {
        AppError::Internal("SFTP port ausente en suscripción provisionada".into())
    })?;

    let coolify = state
        .coolify_config
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Coolify no configurado".into()))?;

    let plan_config = HostingRepository::get_plan_config(&state.pool, &sub.plan)
        .await?
        .ok_or_else(|| {
            AppError::Internal(format!("Plan config '{}' no encontrado en BD", sub.plan))
        })?;

    CoolifyService::update_compose_and_restart(
        &state.http_client,
        coolify,
        server_uuid,
        sftp_user,
        sftp_password,
        sftp_port,
        &plan_config,
    )
    .await?;

    if let Err(e) = HostingRepository::add_event(
        &state.pool,
        id,
        "refreshed",
        Some(serde_json::json!({"by": auth.user_id.to_string(), "plan": sub.plan})),
    )
    .await
    {
        tracing::warn!("Error registrando evento refreshed para {id}: {e}");
    }

    Ok(Json(serde_json::json!({
        "message": "Hosting redeployado con configuración actual",
        "plan": sub.plan,
    })))
}

/* ============================================================
[154A-9] CONTROL DE SERVICIO — Restart / Stop / Start
Solo admin o dueño. La suscripción debe estar provisionada.
============================================================ */

async fn resolve_provisioned_sub(
    state: &AppState,
    auth: &AuthUser,
    id: Uuid,
) -> Result<(crate::models::HostingSubscription, String, CoolifyConfig), AppError> {
    let sub = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;
    if auth.effective_role != UserRole::Admin && sub.user_id != Some(auth.user_id) {
        return Err(AppError::Forbidden("Sin permisos".into()));
    }
    let server_uuid = sub
        .server_uuid
        .clone()
        .ok_or(AppError::Validation("Hosting no provisionado".into()))?;
    let config = state
        .coolify_config
        .clone()
        .ok_or(AppError::ServiceUnavailable(
            "Coolify no configurado".into(),
        ))?;
    Ok((sub, server_uuid, config))
}

/// Reiniciar el servicio `WordPress`
#[utoipa::path(
    post,
    path = "/api/hosting/subscriptions/{id}/restart",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    responses(
        (status = 200, description = "Servicio reiniciado"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "No encontrada"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn restart_hosting(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (_sub, server_uuid, config) = resolve_provisioned_sub(&state, &auth, id).await?;
    CoolifyService::restart_service(&state.http_client, &config, &server_uuid).await?;
    if let Err(e) = HostingRepository::add_event(
        &state.pool,
        id,
        "restarted",
        Some(serde_json::json!({"by": auth.user_id.to_string()})),
    )
    .await
    {
        tracing::warn!("Error registrando evento restarted para {id}: {e}");
    }
    Ok(Json(serde_json::json!({"message": "WordPress reiniciado"})))
}

/// Detener el servicio `WordPress`
#[utoipa::path(
    post,
    path = "/api/hosting/subscriptions/{id}/stop",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    responses(
        (status = 200, description = "Servicio detenido"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "No encontrada"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn stop_hosting(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (_sub, server_uuid, config) = resolve_provisioned_sub(&state, &auth, id).await?;
    CoolifyService::stop_service(&state.http_client, &config, &server_uuid).await?;
    if let Err(e) = HostingRepository::add_event(
        &state.pool,
        id,
        "stopped",
        Some(serde_json::json!({"by": auth.user_id.to_string()})),
    )
    .await
    {
        tracing::warn!("Error registrando evento stopped para {id}: {e}");
    }
    Ok(Json(serde_json::json!({"message": "WordPress detenido"})))
}

/// Arrancar el servicio `WordPress`
#[utoipa::path(
    post,
    path = "/api/hosting/subscriptions/{id}/start",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    responses(
        (status = 200, description = "Servicio iniciado"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "No encontrada"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn start_hosting(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (_sub, server_uuid, config) = resolve_provisioned_sub(&state, &auth, id).await?;
    CoolifyService::start_service(&state.http_client, &config, &server_uuid).await?;
    if let Err(e) = HostingRepository::add_event(
        &state.pool,
        id,
        "started",
        Some(serde_json::json!({"by": auth.user_id.to_string()})),
    )
    .await
    {
        tracing::warn!("Error registrando evento started para {id}: {e}");
    }
    Ok(Json(serde_json::json!({"message": "WordPress iniciado"})))
}

/* ============================================================
[154A-14] ADMIN TEST SUBSCRIBE — Bypass Stripe para testing
Crea suscripción + la activa inmediatamente sin pago.
Solo admin.
============================================================ */

/// Admin test: crear hosting sin Stripe (para testing)
#[utoipa::path(
    post,
    path = "/api/hosting/admin-test-subscribe",
    request_body = SelfSubscribeRequest,
    responses(
        (status = 201, description = "Suscripción creada y activada (test)"),
        (status = 403, description = "Solo admin"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn admin_test_subscribe(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<SelfSubscribeRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    auth.require_role(&[UserRole::Admin])?;
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let plan_config = HostingRepository::get_plan_config(&state.pool, &req.plan)
        .await?
        .ok_or_else(|| AppError::Validation(format!("Plan inválido: {}", req.plan)))?;

    let user = UserRepository::find_by_id(&state.pool, auth.user_id)
        .await?
        .ok_or(AppError::NotFound("Usuario no encontrado".into()))?;

    let sub = HostingRepository::create(
        &state.pool,
        CreateHostingParams {
            user_id: Some(auth.user_id),
            client_name: &user.display_name.unwrap_or_else(|| user.email.clone()),
            client_email: &user.email,
            plan: &req.plan,
            domain: req.domain.as_deref(),
            coolify_site_name: None,
            monthly_price_cents: plan_config.monthly_price_cents,
            storage_limit_mb: plan_config.storage_limit_mb,
        },
    )
    .await?;

    /* Activar directamente sin Stripe */
    HostingRepository::update_status(&state.pool, sub.id, "active").await?;

    if let Err(e) = HostingRepository::add_event(
        &state.pool,
        sub.id,
        "created",
        Some(serde_json::json!({
            "plan": req.plan,
            "by": auth.user_id.to_string(),
            "source": "admin-test",
            "note": "Suscripción de prueba sin Stripe"
        })),
    )
    .await
    {
        tracing::warn!(
            "Error registrando evento admin-test-subscribe para {}: {e}",
            sub.id
        );
    }

    let updated = HostingRepository::find_by_id(&state.pool, sub.id)
        .await?
        .ok_or_else(|| AppError::Internal("Suscripción perdida post-create".into()))?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "subscription": HostingSubscriptionResponse::from(updated),
            "message": "Suscripción de prueba creada. Usa /provision para provisionarla.",
        })),
    ))
}

/* [304A-3] Asigna una suscripción de hosting a un usuario por email.
 * Admin only. Permite vincular hostings creados manualmente a cuentas de clientes existentes.
 * Gotcha: el usuario debe existir en BD — no crea cuentas nuevas. */

/// Asignar hosting a cliente por email (admin only)
#[utoipa::path(
    patch,
    path = "/api/hosting/subscriptions/{id}/assign",
    params(("id" = Uuid, Path, description = "ID de la suscripción")),
    request_body = AssignHostingRequest,
    responses(
        (status = 200, description = "Suscripción asignada al usuario", body = HostingSubscriptionResponse),
        (status = 403, description = "Solo admin"),
        (status = 404, description = "Suscripción o usuario no encontrado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub async fn assign_hosting(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<AssignHostingRequest>,
) -> Result<Json<HostingSubscriptionResponse>, AppError> {
    auth.require_role(&[UserRole::Admin])?;
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    /* Verificar que la suscripción existe */
    let _existing = HostingRepository::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Suscripción no encontrada".into()))?;

    /* Buscar usuario por email */
    let user = UserRepository::find_by_email(&state.pool, &req.user_email)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Usuario con email '{}' no encontrado. Debe tener cuenta registrada.",
                req.user_email
            ))
        })?;

    /* Asignar user_id */
    let updated = HostingRepository::assign_user(&state.pool, id, Some(user.id)).await?;

    if let Err(e) = HostingRepository::add_event(
        &state.pool,
        id,
        "assigned",
        Some(serde_json::json!({
            "assigned_to": user.id.to_string(),
            "user_email": req.user_email,
            "by": auth.user_id.to_string(),
        })),
    )
    .await
    {
        tracing::warn!("Error registrando evento assigned para {id}: {e}");
    }

    Ok(Json(updated.into()))
}

pub fn hosting_routes() -> Router<AppState> {
    /* [174A-17] Rate limits específicos para endpoints de pago de hosting.
     * subscribe: máx 3 por hora por IP (evita abuso de checkouts).
     * checkout: máx 5 por hora por IP. */
    let subscribe_gov = GovernorConfigBuilder::default()
        .per_second(1200)
        .burst_size(3)
        .finish()
        .expect("subscribe rate limit config");
    let checkout_gov = GovernorConfigBuilder::default()
        .per_second(720)
        .burst_size(5)
        .finish()
        .expect("checkout rate limit config");

    Router::new()
        .route(
            "/hosting/subscriptions",
            get(list_subscriptions).post(create_subscription),
        )
        .route(
            "/hosting/subscriptions/:id",
            get(get_subscription)
                .put(update_subscription)
                .delete(delete_subscription),
        )
        .route(
            "/hosting/subscriptions/:id/status",
            axum::routing::patch(update_status),
        )
        /* [304A-3] Admin asigna hosting a cliente registrado por email */
        .route(
            "/hosting/subscriptions/:id/assign",
            axum::routing::patch(assign_hosting),
        )
        .route("/hosting/subscriptions/:id/events", get(list_events))
        /* [094A-8] Stats reales de una suscripción */
        .route("/hosting/subscriptions/:id/stats", get(get_hosting_stats))
        .route(
            "/hosting/subscriptions/:id/cancel",
            axum::routing::post(request_cancel),
        )
        /* [084A-24] Stripe checkout para hosting — rate limited */
        .route(
            "/hosting/subscriptions/:id/checkout",
            axum::routing::post(create_checkout).layer(GovernorLayer {
                config: std::sync::Arc::new(checkout_gov),
            }),
        )
        /* [094A-3] Self-service: cliente contrata + paga — rate limited */
        .route(
            "/hosting/subscribe",
            axum::routing::post(subscribe_self).layer(GovernorLayer {
                config: std::sync::Arc::new(subscribe_gov),
            }),
        )
        /* [164A-19] Despliegues reales de Coolify en VPS2 */
        .route("/hosting/deployments", get(list_vps2_deployments))
        /* [084A-24] VPS stats: proxy a Contabo API */
        .route("/hosting/vps", get(list_vps))
        .route("/hosting/vps/:instance_id", get(get_vps))
        /* [154A-11] Provisioning real: crea servicio Nginx en Coolify VPS2 */
        .route(
            "/hosting/subscriptions/:id/provision",
            axum::routing::post(provision_subscription),
        )
        /* [114A-1] Rotación de credenciales SFTP */
        .route(
            "/hosting/subscriptions/:id/rotate-credentials",
            axum::routing::post(rotate_credentials),
        )
        /* [154A-16] Verificación DNS de un dominio */
        .route("/hosting/subscriptions/:id/dns-check", get(dns_check))
        /* [114A-3] Plan configs: admin gestiona precios y límites de recursos */
        .route("/hosting/plan-configs", get(list_plan_configs))
        .route("/hosting/public-plans", get(list_public_plans))
        .route(
            "/hosting/plan-configs/:plan",
            axum::routing::put(update_plan_config),
        )
        /* [114A-4] Refresh: regenera compose con plan config actual */
        .route(
            "/hosting/subscriptions/:id/refresh",
            axum::routing::post(refresh_hosting),
        )
        /* [154A-9] Control de servicio: restart / stop / start */
        .route(
            "/hosting/subscriptions/:id/restart",
            axum::routing::post(restart_hosting),
        )
        .route(
            "/hosting/subscriptions/:id/stop",
            axum::routing::post(stop_hosting),
        )
        .route(
            "/hosting/subscriptions/:id/start",
            axum::routing::post(start_hosting),
        )
        /* [154A-14] Admin test subscribe: crea hosting sin Stripe */
        .route(
            "/hosting/admin-test-subscribe",
            axum::routing::post(admin_test_subscribe),
        )
}

/* ============================================================
TESTS — [094A-10] Lógica de negocio del hosting
============================================================ */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_contabo_error_invalid_grant_is_service_unavailable() {
        let error = map_contabo_error(
            "Contabo auth failed: 400 Bad Request — {\"error_description\":\"invalid_grant\"}",
        );

        match error {
            AppError::ServiceUnavailable(message) => {
                assert!(message.contains("CONTABO_API_PASSWORD"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn map_contabo_error_parse_issue_is_service_unavailable() {
        let error = map_contabo_error("Contabo parse error: missing field data");

        match error {
            AppError::ServiceUnavailable(message) => {
                assert!(message.contains("formato inesperado"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn public_plan_from_config_uses_marketing_metadata() {
        let config = HostingPlanConfig {
            id: Uuid::new_v4(),
            plan_name: "pro".to_string(),
            monthly_price_cents: 413,
            wp_cpu_millicores: 1000,
            wp_memory_mb: 512,
            db_cpu_millicores: 500,
            db_memory_mb: 512,
            ssh_cpu_millicores: 500,
            ssh_memory_mb: 256,
            storage_limit_mb: 20_480,
            bandwidth_limit_gb: 200,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let public_plan = public_plan_from_config(config);
        assert_eq!(public_plan.label, "WordPress Profesional");
        assert!(public_plan.recommended);
        assert!(public_plan
            .features
            .iter()
            .any(|feature| feature.contains("Staging")));
    }

    #[test]
    fn public_plan_from_config_distinguishes_normal_hosting() {
        let config = HostingPlanConfig {
            id: Uuid::new_v4(),
            plan_name: "normal-pro".to_string(),
            monthly_price_cents: 537,
            wp_cpu_millicores: 1000,
            wp_memory_mb: 512,
            db_cpu_millicores: 500,
            db_memory_mb: 512,
            ssh_cpu_millicores: 500,
            ssh_memory_mb: 256,
            storage_limit_mb: 20_480,
            bandwidth_limit_gb: 200,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let public_plan = public_plan_from_config(config);
        assert_eq!(public_plan.label, "Hosting Profesional");
        assert!(public_plan.recommended);
        assert!(public_plan
            .features
            .iter()
            .any(|feature| feature.contains("Nginx")));
    }

    /* --- calculate_uptime --- */

    #[test]
    fn calculate_uptime_no_events_active_status() {
        /* Suscripción activa sin eventos status_change → 100% uptime desde creación */
        let created = chrono::Utc::now() - chrono::Duration::hours(24);
        let events: Vec<crate::models::HostingEvent> = vec![];
        let (uptime, first_active) = calculate_uptime(created, "active", &events);
        assert!((uptime - 100.0).abs() < 0.1);
        assert_eq!(first_active, Some(created));
    }

    #[test]
    fn calculate_uptime_no_events_pending_status() {
        /* Suscripción pending sin eventos → 0% uptime */
        let created = chrono::Utc::now() - chrono::Duration::hours(24);
        let events: Vec<crate::models::HostingEvent> = vec![];
        let (uptime, first_active) = calculate_uptime(created, "pending", &events);
        assert!((uptime - 0.0).abs() < 0.1);
        assert!(first_active.is_none());
    }

    fn make_status_event(
        created_at: chrono::DateTime<chrono::Utc>,
        new_status: &str,
    ) -> crate::models::HostingEvent {
        crate::models::HostingEvent {
            id: uuid::Uuid::new_v4(),
            subscription_id: uuid::Uuid::new_v4(),
            event_type: "status_change".to_string(),
            details: Some(serde_json::json!({"new_status": new_status})),
            created_at,
        }
    }

    #[test]
    fn calculate_uptime_active_then_suspended() {
        /* Activada a las 0h, suspendida a las 12h, total 24h → ~50% */
        let created = chrono::Utc::now() - chrono::Duration::hours(24);
        let activated_at = created + chrono::Duration::hours(0);
        let suspended_at = created + chrono::Duration::hours(12);

        let events = vec![
            make_status_event(activated_at, "active"),
            make_status_event(suspended_at, "suspended"),
        ];

        let (uptime, first_active) = calculate_uptime(created, "suspended", &events);
        assert!((uptime - 50.0).abs() < 1.0);
        assert_eq!(first_active, Some(activated_at));
    }

    #[test]
    fn calculate_uptime_active_suspended_active_again() {
        /* Activada 0h, suspendida 6h, reactivada 12h, ahora activa — ~75% uptime */
        let created = chrono::Utc::now() - chrono::Duration::hours(24);
        let events = vec![
            make_status_event(created, "active"),
            make_status_event(created + chrono::Duration::hours(6), "suspended"),
            make_status_event(created + chrono::Duration::hours(12), "active"),
        ];

        let (uptime, first_active) = calculate_uptime(created, "active", &events);
        /* 6h active + 12h active (12h-24h) = 18h / 24h = 75% */
        assert!((uptime - 75.0).abs() < 2.0);
        assert_eq!(first_active, Some(created));
    }

    #[test]
    fn calculate_uptime_ignores_non_status_events() {
        let created = chrono::Utc::now() - chrono::Duration::hours(24);
        let events = vec![crate::models::HostingEvent {
            id: uuid::Uuid::new_v4(),
            subscription_id: uuid::Uuid::new_v4(),
            event_type: "created".to_string(),
            details: Some(serde_json::json!({"plan": "basico"})),
            created_at: created,
        }];

        let (uptime, first_active) = calculate_uptime(created, "active", &events);
        /* No hay status_change events, pero status es active → 100% */
        assert!((uptime - 100.0).abs() < 0.1);
        assert_eq!(first_active, Some(created));
    }
}
