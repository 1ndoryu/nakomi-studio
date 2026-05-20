/* [205A-1] Stripe Checkout para cobros pendientes legacy.
 * Soporta suscripcion por item y prepago anual agrupado sin tocar provisioning. */
use reqwest::Client;
use serde::Deserialize;
use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::{BillingCheckoutMode, BillingItem};
use crate::repositories::BillingRepository;

#[derive(Debug, Deserialize)]
struct CheckoutSession {
    id: String,
    url: Option<String>,
}

type StripeForm = Vec<(String, String)>;

pub struct BillingCheckoutParams<'a> {
    pub http_client: &'a Client,
    pub stripe_key: &'a str,
    pub items: &'a [BillingItem],
    pub mode: BillingCheckoutMode,
    pub customer_email: &'a str,
    pub success_url: &'a str,
    pub cancel_url: &'a str,
}

pub struct BillingStripeService;

fn recurring_interval(period: &str) -> Option<&'static str> {
    match period {
        "month" => Some("month"),
        "year" => Some("year"),
        _ => None,
    }
}

/* [195A-1] 20% descuento por pago anual completo. base = sum mensual x12 (o anual sin cambio). */
const PREPAY_ANNUAL_DISCOUNT_PCT: i32 = 20; /* porcentaje de descuento */

fn prepay_base_annual_cents(item: &BillingItem) -> i32 {
    if item.billing_period == "month" {
        item.amount_cents * 12
    } else {
        item.amount_cents
    }
}

/* [195A-1] Descuento entero para evitar cast f64->i32. */
fn prepay_amount_cents(item: &BillingItem) -> i32 {
    prepay_base_annual_cents(item) * (100 - PREPAY_ANNUAL_DISCOUNT_PCT) / 100
}

fn item_ids_metadata(items: &[BillingItem]) -> String {
    items
        .iter()
        .map(|item| item.id.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn subscription_interval(items: &[BillingItem]) -> Result<&'static str, AppError> {
    let Some(interval) = items
        .first()
        .and_then(|item| recurring_interval(&item.billing_period))
    else {
        return Err(AppError::Validation(
            "Este cobro no soporta suscripcion recurrente".into(),
        ));
    };

    if items.len() > 1
        && items
            .iter()
            .any(|item| recurring_interval(&item.billing_period) != Some(interval))
    {
        return Err(AppError::Validation(
            "Agrupa solo cobros con el mismo periodo para una suscripcion".into(),
        ));
    }

    Ok(interval)
}

fn push_line_description(form: &mut StripeForm, index: usize, item: &BillingItem) {
    if let Some(description) = &item.description {
        form.push((
            format!("line_items[{index}][price_data][product_data][description]"),
            description.clone(),
        ));
    }
}

fn append_subscription_items(form: &mut StripeForm, items: &[BillingItem]) -> Result<(), AppError> {
    let interval = subscription_interval(items)?;
    form.push(("mode".to_string(), "subscription".to_string()));

    for (index, item) in items.iter().enumerate() {
        form.extend([
            (
                format!("line_items[{index}][price_data][currency]"),
                item.currency.to_ascii_lowercase(),
            ),
            (
                format!("line_items[{index}][price_data][unit_amount]"),
                item.amount_cents.to_string(),
            ),
            (
                format!("line_items[{index}][price_data][recurring][interval]"),
                interval.to_string(),
            ),
            (
                format!("line_items[{index}][price_data][product_data][name]"),
                item.title.clone(),
            ),
            (format!("line_items[{index}][quantity]"), "1".to_string()),
        ]);
        push_line_description(form, index, item);
    }

    Ok(())
}

fn append_prepay_items(form: &mut StripeForm, items: &[BillingItem]) {
    form.push(("mode".to_string(), "payment".to_string()));

    for (index, item) in items.iter().enumerate() {
        form.extend([
            (
                format!("line_items[{index}][price_data][currency]"),
                item.currency.to_ascii_lowercase(),
            ),
            (
                format!("line_items[{index}][price_data][unit_amount]"),
                prepay_amount_cents(item).to_string(),
            ),
            (
                format!("line_items[{index}][price_data][product_data][name]"),
                if item.billing_period == "month" {
                    format!("{} · año completo (20% off)", item.title)
                } else {
                    format!("{} (20% off)", item.title)
                },
            ),
            (format!("line_items[{index}][quantity]"), "1".to_string()),
        ]);
        push_line_description(form, index, item);
    }
}

fn build_checkout_form(params: &BillingCheckoutParams<'_>) -> Result<StripeForm, AppError> {
    let mut form = vec![
        (
            "customer_email".to_string(),
            params.customer_email.to_string(),
        ),
        ("success_url".to_string(), params.success_url.to_string()),
        ("cancel_url".to_string(), params.cancel_url.to_string()),
        (
            "metadata[resource_kind]".to_string(),
            "billing_items".to_string(),
        ),
        (
            "metadata[billing_item_ids]".to_string(),
            item_ids_metadata(params.items),
        ),
        (
            "metadata[billing_checkout_mode]".to_string(),
            params.mode.to_string(),
        ),
    ];

    match params.mode {
        BillingCheckoutMode::Subscription => append_subscription_items(&mut form, params.items)?,
        BillingCheckoutMode::PrepayYear => append_prepay_items(&mut form, params.items),
    }

    Ok(form)
}

fn checkout_idempotency_key(params: &BillingCheckoutParams<'_>) -> String {
    format!("billing-{}-{}", params.mode, uuid::Uuid::new_v4())
}

impl BillingStripeService {
    pub async fn create_checkout_session(
        params: &BillingCheckoutParams<'_>,
    ) -> Result<(String, String), AppError> {
        if params.items.is_empty() {
            return Err(AppError::Validation(
                "No hay cobros pendientes para pagar".into(),
            ));
        }

        let form = build_checkout_form(params)?;
        let idempotency_key = checkout_idempotency_key(params);

        let response = params
            .http_client
            .post("https://api.stripe.com/v1/checkout/sessions")
            .basic_auth(params.stripe_key, Option::<&str>::None)
            .header("Idempotency-Key", idempotency_key)
            .form(&form)
            .send()
            .await
            .map_err(|error| AppError::Internal(format!("Stripe billing falló: {error}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!("Stripe billing checkout error: {status} — {body}");
            return Err(AppError::Internal(format!(
                "Stripe checkout de cobros pendientes falló: {status}"
            )));
        }

        let session: CheckoutSession = response.json().await.map_err(|error| {
            AppError::Internal(format!("Stripe parse billing checkout error: {error}"))
        })?;
        let url = session
            .url
            .ok_or_else(|| AppError::Internal("Stripe no retornó URL de checkout".into()))?;
        Ok((session.id, url))
    }

    pub async fn handle_webhook(
        pool: &PgPool,
        event_type: &str,
        data: &serde_json::Value,
    ) -> Result<bool, AppError> {
        if event_type != "checkout.session.completed" {
            return Ok(false);
        }

        let object = &data["object"];
        if object["metadata"]["resource_kind"].as_str() != Some("billing_items") {
            return Ok(false);
        }

        let Some(session_id) = object["id"].as_str().filter(|value| !value.is_empty()) else {
            return Ok(false);
        };

        BillingRepository::mark_paid_by_session(pool, session_id).await?;
        Ok(true)
    }
}
