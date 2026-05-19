/* [195A-1] Stripe Checkout para dominios self-service.
 * El pago crea una orden paid_pending_registration; el registro Contabo queda para completar handles reales. */
use reqwest::Client;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::repositories::DomainOrderRepository;

#[derive(Debug, Deserialize)]
struct CheckoutSession {
    id: String,
    url: Option<String>,
}

pub struct DomainCheckoutParams<'a> {
    pub http_client: &'a Client,
    pub stripe_key: &'a str,
    pub order_id: Uuid,
    pub domain: &'a str,
    pub amount_cents: i32,
    pub customer_email: &'a str,
    pub success_url: &'a str,
    pub cancel_url: &'a str,
}

pub struct DomainStripeService;

impl DomainStripeService {
    pub async fn create_checkout_session(
        params: &DomainCheckoutParams<'_>,
    ) -> Result<(String, String), AppError> {
        if params.amount_cents <= 0 {
            return Err(AppError::Validation(
                "El checkout de dominio requiere un precio mayor a 0".into(),
            ));
        }

        let form = vec![
            ("mode", "payment".to_string()),
            ("line_items[0][price_data][currency]", "usd".to_string()),
            (
                "line_items[0][price_data][unit_amount]",
                params.amount_cents.to_string(),
            ),
            (
                "line_items[0][price_data][product_data][name]",
                format!("Dominio {}", params.domain),
            ),
            (
                "line_items[0][price_data][product_data][description]",
                "Registro anual de dominio gestionado por Nakomi".to_string(),
            ),
            ("line_items[0][quantity]", "1".to_string()),
            ("customer_email", params.customer_email.to_string()),
            ("success_url", params.success_url.to_string()),
            ("cancel_url", params.cancel_url.to_string()),
            ("metadata[resource_kind]", "domain_order".to_string()),
            ("metadata[domain_order_id]", params.order_id.to_string()),
            ("metadata[domain]", params.domain.to_string()),
        ];

        let response = params
            .http_client
            .post("https://api.stripe.com/v1/checkout/sessions")
            .basic_auth(params.stripe_key, Option::<&str>::None)
            .header(
                "Idempotency-Key",
                format!("domain-checkout-{}", params.order_id),
            )
            .form(&form)
            .send()
            .await
            .map_err(|error| AppError::Internal(format!("Stripe dominio falló: {error}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!("Stripe dominio checkout error: {status} — {body}");
            return Err(AppError::Internal(format!(
                "Stripe checkout dominio falló: {status}"
            )));
        }

        let session: CheckoutSession = response
            .json()
            .await
            .map_err(|error| AppError::Internal(format!("Stripe parse dominio error: {error}")))?;
        let url = session
            .url
            .ok_or_else(|| AppError::Internal("Stripe no retornó URL de dominio".into()))?;
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
        if object["metadata"]["resource_kind"].as_str() != Some("domain_order") {
            return Ok(false);
        }

        let Some(session_id) = object["id"].as_str().filter(|value| !value.is_empty()) else {
            return Ok(false);
        };

        DomainOrderRepository::mark_paid_by_session(pool, session_id).await?;
        Ok(true)
    }
}
