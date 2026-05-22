/* [205A-1] Bootstrap administrativo para el caso Guillermo.
 * Crea solo cliente, hostings legacy y cobros pendientes; no sincroniza fixtures ni reprovisiona infraestructura. */

use std::collections::HashMap;

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::UserRole;
use crate::repositories::UserRepository;
use crate::services::hash_password;
use crate::AppState;

const GUILLERMO_EMAIL: &str = "guillermo@nakomi.com";
const GUILLERMO_NAME: &str = "Guillermo";
const DOMAIN_DUE_AT: &str = "2027-01-14T12:00:00Z";
const DOMAIN_GRACE_ENDS_AT: &str = "2027-04-14T12:00:00Z";

#[derive(Deserialize)]
pub struct GuillermoBootstrapRequest {
    pub temporary_password: String,
}

#[derive(Serialize)]
pub struct GuillermoBootstrapResponse {
    pub user_id: Uuid,
    pub user_created: bool,
    pub hostings_upserted: usize,
    pub billing_items_upserted: usize,
    pub message: String,
}

struct GuillermoHosting {
    domain: &'static str,
    coolify_site_name: &'static str,
    server_uuid: &'static str,
    paid_subscription_id: Option<&'static str>,
}

const GUILLERMO_HOSTINGS: &[GuillermoHosting] = &[
    GuillermoHosting {
        domain: "materialdepadel.es",
        coolify_site_name: "padel",
        server_uuid: "zkcc040cc0scock4kcooowkc",
        paid_subscription_id: Some("sub_legacy_paid_padel"),
    },
    GuillermoHosting {
        domain: "guillechatbots.es",
        coolify_site_name: "guillermo",
        server_uuid: "owck8sww4ogk8gskgwcsk4w0",
        paid_subscription_id: None,
    },
    GuillermoHosting {
        domain: "cap.wandori.us",
        coolify_site_name: "cap",
        server_uuid: "qgskgw8wwc08o444o08wko8o",
        paid_subscription_id: None,
    },
    GuillermoHosting {
        domain: "restaurante.wandori.us",
        coolify_site_name: "glory-rest",
        server_uuid: "b8s0cks444o0sogo8kg8wcgw",
        paid_subscription_id: Some("sub_legacy_paid_glory_rest"),
    },
];

struct GuillermoBillingItem {
    id: &'static str,
    resource_type: &'static str,
    hosting_domain: Option<&'static str>,
    title: &'static str,
    description: &'static str,
    amount_cents: i32,
    billing_period: &'static str,
    /* [205A-4] initially_paid: si true, el upsert inserta/actualiza el item como 'paid'.
     * Sirve para marcar cobros ya saldados sin perder el registro historico.
     * [195A-1] Fechas computadas en upsert_guillermo_billing_items:
     * hosting -> Utc::now() + 90 dias de gracia; domain -> fecha fija GoDaddy. */
    initially_paid: bool,
}

const GUILLERMO_BILLING_ITEMS: &[GuillermoBillingItem] = &[
    GuillermoBillingItem {
        id: "d1000001-0001-4000-8000-000000000001",
        resource_type: "hosting",
        hosting_domain: Some("guillechatbots.es"),
        title: "Hosting guillechatbots.es",
        description: "El cobro empieza al pagar; periodo de revision de 3 meses.",
        amount_cents: 248,
        billing_period: "month",
        /* [205A-4] guillechatbots.es ya esta pagado — se registra como paid para que
         * no aparezca como deuda pendiente en la vista del cliente. */
        initially_paid: true,
    },
    GuillermoBillingItem {
        id: "d1000001-0001-4000-8000-000000000002",
        resource_type: "hosting",
        hosting_domain: Some("cap.wandori.us"),
        title: "Hosting cap.wandori.us",
        description: "El dominio wandori.us no se cobra al cliente.",
        amount_cents: 248,
        billing_period: "month",
        initially_paid: false,
    },
    GuillermoBillingItem {
        id: "d1000001-0001-4000-8000-000000000003",
        resource_type: "domain",
        hosting_domain: None,
        title: "Dominio materialdepadel.es",
        description: "Dominio comprado en GoDaddy. Renovacion Jan 14, 2027.",
        amount_cents: 1500,
        billing_period: "year",
        initially_paid: false,
    },
    GuillermoBillingItem {
        id: "d1000001-0001-4000-8000-000000000004",
        resource_type: "domain",
        hosting_domain: None,
        title: "Dominio guillechatbots.es",
        description: "Dominio comprado en GoDaddy. Renovacion Jan 14, 2027.",
        amount_cents: 1500,
        billing_period: "year",
        initially_paid: false,
    },
    GuillermoBillingItem {
        id: "d1000001-0001-4000-8000-000000000005",
        resource_type: "hosting",
        hosting_domain: Some("restaurante.wandori.us"),
        title: "Hosting restaurante.wandori.us \u{2014} Plan Pro",
        description: "Plan Pro ($4.13/mes). El cobro empieza al pagar.",
        amount_cents: 413,
        billing_period: "month",
        initially_paid: false,
    },
];

async fn bootstrap_guillermo(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<GuillermoBootstrapRequest>,
) -> Result<Json<GuillermoBootstrapResponse>, AppError> {
    auth.require_role(&[UserRole::Admin])?;
    validate_temporary_password(&request.temporary_password)?;

    let (user_id, user_created) =
        upsert_guillermo_user(&state.pool, &request.temporary_password).await?;
    let hosting_ids = upsert_guillermo_hostings(&state.pool, user_id).await?;
    upsert_guillermo_billing_items(&state.pool, user_id, &hosting_ids).await?;

    Ok(Json(GuillermoBootstrapResponse {
        user_id,
        user_created,
        hostings_upserted: hosting_ids.len(),
        billing_items_upserted: GUILLERMO_BILLING_ITEMS.len(),
        message: "Cliente Guillermo listo para revision: 4 hostings activos, 4 cobros (1 pagado) y 1 pendiente nuevo (restaurante Pro)."
            .into(),
    }))
}

fn validate_temporary_password(password: &str) -> Result<(), AppError> {
    if password.trim().len() < 10 {
        return Err(AppError::Validation(
            "La password temporal debe tener al menos 10 caracteres".into(),
        ));
    }
    Ok(())
}

async fn upsert_guillermo_user(
    pool: &PgPool,
    temporary_password: &str,
) -> Result<(Uuid, bool), AppError> {
    if let Some(existing) = UserRepository::find_by_email(pool, GUILLERMO_EMAIL).await? {
        let password_hash = hash_password(temporary_password)?;
        sqlx::query(
            "UPDATE users SET display_name = COALESCE(display_name, $1), role = 'client'::user_role, password_hash = $3 WHERE id = $2",
        )
        .bind(GUILLERMO_NAME)
        .bind(existing.id)
        .bind(password_hash)
        .execute(pool)
        .await?;
        return Ok((existing.id, false));
    }

    let password_hash = hash_password(temporary_password)?;
    let user = UserRepository::create_with_role(
        pool,
        GUILLERMO_EMAIL,
        &password_hash,
        UserRole::Client,
        true,
    )
    .await?;
    sqlx::query("UPDATE users SET display_name = $1, email_verified = false WHERE id = $2")
        .bind(GUILLERMO_NAME)
        .bind(user.id)
        .execute(pool)
        .await?;

    Ok((user.id, true))
}

async fn upsert_guillermo_hostings(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<HashMap<&'static str, Uuid>, AppError> {
    let mut hosting_ids = HashMap::new();
    let verified_at = Utc::now();

    for hosting in GUILLERMO_HOSTINGS {
        let row: (Uuid,) = sqlx::query_as(
            r"INSERT INTO hosting_subscriptions (
                    user_id, client_name, client_email, plan, domain,
                    domain_verification_status, domain_verified_at, coolify_site_name,
                    status, stripe_subscription_id, monthly_price_cents, storage_limit_mb,
                    server_uuid, server_ip
                )
                VALUES ($1, $2, $3, 'normal-basico', $4, 'active', $5, $6, 'active', $7, 248, 5120, $8, '66.94.100.241')
                ON CONFLICT (domain) DO UPDATE SET
                    user_id = EXCLUDED.user_id,
                    client_name = EXCLUDED.client_name,
                    client_email = EXCLUDED.client_email,
                    plan = EXCLUDED.plan,
                    domain_verification_status = 'active',
                    domain_verified_at = COALESCE(hosting_subscriptions.domain_verified_at, EXCLUDED.domain_verified_at),
                    coolify_site_name = EXCLUDED.coolify_site_name,
                    status = 'active',
                    stripe_subscription_id = COALESCE(hosting_subscriptions.stripe_subscription_id, EXCLUDED.stripe_subscription_id),
                    monthly_price_cents = EXCLUDED.monthly_price_cents,
                    storage_limit_mb = EXCLUDED.storage_limit_mb,
                    server_uuid = EXCLUDED.server_uuid,
                    server_ip = EXCLUDED.server_ip,
                    updated_at = NOW()
                RETURNING id",
        )
        .bind(user_id)
        .bind(GUILLERMO_NAME)
        .bind(GUILLERMO_EMAIL)
        .bind(hosting.domain)
        .bind(verified_at)
        .bind(hosting.coolify_site_name)
        .bind(hosting.paid_subscription_id)
        .bind(hosting.server_uuid)
        .fetch_one(pool)
        .await?;
        hosting_ids.insert(hosting.domain, row.0);
    }

    Ok(hosting_ids)
}

async fn upsert_guillermo_billing_items(
    pool: &PgPool,
    user_id: Uuid,
    hosting_ids: &HashMap<&'static str, Uuid>,
) -> Result<(), AppError> {
    for item in GUILLERMO_BILLING_ITEMS {
        let item_id = Uuid::parse_str(item.id)
            .map_err(|err| AppError::Internal(format!("ID billing bootstrap invalido: {err}")))?;
        let resource_id = resolve_resource_id(item, hosting_ids)?;
        let metadata = billing_metadata(item);
        /* [195A-1] Fechas dinamicas: hosting = ahora + 90 dias gracia; domain = fecha fija GoDaddy. */
        let (due_at, grace_period_ends_at) = if item.resource_type == "hosting" {
            let now = Utc::now();
            (now, now + Duration::days(90))
        } else {
            (
                parse_timestamp(DOMAIN_DUE_AT)?,
                parse_timestamp(DOMAIN_GRACE_ENDS_AT)?,
            )
        };

        /* [205A-4] $12 = initial_status ('paid' | 'pending').
         * ON CONFLICT: si el item ya es 'paid' O el bootstrap lo marca paid, queda paid.
         * Esto permite re-ejecutar el bootstrap sin revertir pagos ya registrados
         * y sin crear deuda falsa para cobros ya saldados (e.g. guillechatbots.es). */
        let initial_status = if item.initially_paid {
            "paid"
        } else {
            "pending"
        };
        sqlx::query(
            r"INSERT INTO billing_items (
                    id, user_id, resource_type, resource_id, title, description,
                    amount_cents, currency, billing_period, status, due_at,
                    grace_period_ends_at, metadata
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, 'USD', $8, $12, $9, $10, $11)
                ON CONFLICT (id) DO UPDATE SET
                    user_id = EXCLUDED.user_id,
                    resource_type = EXCLUDED.resource_type,
                    resource_id = EXCLUDED.resource_id,
                    title = EXCLUDED.title,
                    description = EXCLUDED.description,
                    amount_cents = EXCLUDED.amount_cents,
                    currency = EXCLUDED.currency,
                    billing_period = EXCLUDED.billing_period,
                    status = CASE
                        WHEN $12 = 'paid' OR billing_items.status = 'paid' THEN 'paid'
                        ELSE 'pending'
                    END,
                    due_at = EXCLUDED.due_at,
                    grace_period_ends_at = EXCLUDED.grace_period_ends_at,
                    paid_at = CASE
                        WHEN $12 = 'paid' THEN COALESCE(billing_items.paid_at, NOW())
                        WHEN billing_items.status = 'paid' THEN billing_items.paid_at
                        ELSE NULL
                    END,
                    stripe_session_id = CASE WHEN billing_items.status = 'paid' THEN billing_items.stripe_session_id ELSE NULL END,
                    metadata = EXCLUDED.metadata,
                    updated_at = NOW()",
        )
        .bind(item_id)
        .bind(user_id)
        .bind(item.resource_type)
        .bind(resource_id)
        .bind(item.title)
        .bind(item.description)
        .bind(item.amount_cents)
        .bind(item.billing_period)
        .bind(due_at)
        .bind(grace_period_ends_at)
        .bind(metadata)
        .bind(initial_status)
        .execute(pool)
        .await?;
    }

    Ok(())
}

fn resolve_resource_id(
    item: &GuillermoBillingItem,
    hosting_ids: &HashMap<&'static str, Uuid>,
) -> Result<Option<Uuid>, AppError> {
    match item.hosting_domain {
        Some(domain) => hosting_ids.get(domain).copied().map(Some).ok_or_else(|| {
            AppError::Internal(format!("Hosting bootstrap no encontrado para {domain}"))
        }),
        None => Ok(None),
    }
}

fn billing_metadata(item: &GuillermoBillingItem) -> Value {
    match item.hosting_domain {
        Some(domain) => serde_json::json!({
            "coolify_site_name": GUILLERMO_HOSTINGS
                .iter()
                .find(|hosting| hosting.domain == domain)
                .map(|hosting| hosting.coolify_site_name),
            "already_hosted_on": "VPS1"
        }),
        None => serde_json::json!({
            "registrar": "GoDaddy",
            "renewal_date": "2027-01-14",
            "transfer_status": "manual_review"
        }),
    }
}

fn parse_timestamp(value: &str) -> Result<DateTime<Utc>, AppError> {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.with_timezone(&Utc))
        .map_err(|err| AppError::Internal(format!("Fecha bootstrap invalida: {err}")))
}

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/admin/client-bootstrap/guillermo",
        post(bootstrap_guillermo),
    )
}
