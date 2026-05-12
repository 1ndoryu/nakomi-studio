/* [164A-17] Stripe para VPS dedicados.
 * El checkout es recurrente pero la activación no es automática: queda en pending_approval.
 * Si el admin rechaza, se cancela la suscripción y se intenta refund del último PaymentIntent. */

use reqwest::Client;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{
    CreateNotification, NOTIF_VPS_APPROVED, NOTIF_VPS_PENDING_APPROVAL, NOTIF_VPS_REJECTED,
    NOTIF_VPS_SUSPENDED,
};
use crate::repositories::{UserRepository, VpsRepository};
use crate::services::{EmailConfig, EmailService, NotificationHub};

#[derive(Debug, Deserialize)]
struct CheckoutSession {
    id: String,
    url: Option<String>,
}

pub struct VpsCheckoutParams<'a> {
    pub http_client: &'a Client,
    pub stripe_key: &'a str,
    pub subscription_id: Uuid,
    pub tier_name: &'a str,
    pub amount_cents: i32,
    pub customer_email: &'a str,
    pub success_url: &'a str,
    pub cancel_url: &'a str,
}

pub struct VpsStripeService;

fn humanize_tier_name(tier_name: &str) -> &str {
    match tier_name {
        "vps1" => "Nakomi VPS 1",
        "vps2" => "Nakomi VPS 2",
        "vps3" => "Nakomi VPS 3",
        "vps4" => "Nakomi VPS 4",
        _ => "Nakomi VPS",
    }
}

impl VpsStripeService {
    pub async fn create_checkout_session(
        params: &VpsCheckoutParams<'_>,
    ) -> Result<String, AppError> {
        if params.amount_cents <= 0 {
            return Err(AppError::Validation(
                "El checkout de VPS requiere un precio mensual mayor a 0".into(),
            ));
        }

        let tier_name = humanize_tier_name(params.tier_name);
        let form = vec![
            ("mode", "subscription".to_string()),
            ("line_items[0][price_data][currency]", "usd".to_string()),
            (
                "line_items[0][price_data][unit_amount]",
                params.amount_cents.to_string(),
            ),
            (
                "line_items[0][price_data][product_data][name]",
                format!("{tier_name} — Nakomi VPS"),
            ),
            (
                "line_items[0][price_data][product_data][description]",
                "Servidor VPS dedicado con aprobación manual antes del provisioning".to_string(),
            ),
            (
                "line_items[0][price_data][recurring][interval]",
                "month".to_string(),
            ),
            ("line_items[0][quantity]", "1".to_string()),
            ("customer_email", params.customer_email.to_string()),
            ("success_url", params.success_url.to_string()),
            ("cancel_url", params.cancel_url.to_string()),
            ("metadata[resource_kind]", "vps".to_string()),
            (
                "metadata[vps_subscription_id]",
                params.subscription_id.to_string(),
            ),
            (
                "subscription_data[metadata][resource_kind]",
                "vps".to_string(),
            ),
            (
                "subscription_data[metadata][vps_subscription_id]",
                params.subscription_id.to_string(),
            ),
        ];

        let response = params
            .http_client
            .post("https://api.stripe.com/v1/checkout/sessions")
            .basic_auth(params.stripe_key, Option::<&str>::None)
            .header(
                "Idempotency-Key",
                format!("vps-checkout-{}", params.subscription_id),
            )
            .form(&form)
            .send()
            .await
            .map_err(|error| {
                AppError::Internal(format!(
                    "Stripe checkout VPS falló al enviar request: {error}"
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!("Stripe checkout VPS error: {status} — {body}");
            return Err(AppError::Internal(format!(
                "Stripe checkout session VPS falló: {status}"
            )));
        }

        let session: CheckoutSession = response
            .json()
            .await
            .map_err(|error| AppError::Internal(format!("Stripe parse VPS error: {error}")))?;

        tracing::info!(
            "Checkout session VPS creada: {} para suscripción {}",
            session.id,
            params.subscription_id
        );

        session
            .url
            .ok_or_else(|| AppError::Internal("Stripe no retornó URL para VPS".into()))
    }

    pub async fn handle_webhook(
        pool: &PgPool,
        notification_hub: &NotificationHub,
        email_config: Option<&EmailConfig>,
        event_type: &str,
        data: &serde_json::Value,
    ) -> Result<bool, AppError> {
        match event_type {
            "checkout.session.completed" => {
                Self::on_checkout_completed(pool, notification_hub, email_config, data).await
            }
            "invoice.paid" => Self::on_invoice_paid(pool, data).await,
            "customer.subscription.deleted" => Self::on_subscription_deleted(pool, data).await,
            "invoice.payment_failed" => Self::on_payment_failed(pool, notification_hub, data).await,
            _ => Ok(false),
        }
    }

    async fn on_checkout_completed(
        pool: &PgPool,
        notification_hub: &NotificationHub,
        email_config: Option<&EmailConfig>,
        data: &serde_json::Value,
    ) -> Result<bool, AppError> {
        let mode = data["object"]["mode"].as_str().unwrap_or("");
        if mode != "subscription" {
            return Ok(false);
        }

        let sub_id_str = data["object"]["metadata"]["vps_subscription_id"]
            .as_str()
            .or_else(|| {
                data["object"]["subscription_data"]["metadata"]["vps_subscription_id"].as_str()
            });

        let Some(vps_id) = sub_id_str.and_then(|value| Uuid::parse_str(value).ok()) else {
            return Ok(false);
        };

        let stripe_sub_id = data["object"]["subscription"]
            .as_str()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                tracing::warn!(
                    "Webhook VPS checkout.session.completed sin subscription ID para {vps_id}"
                );
                AppError::BadRequest("Missing subscription field in VPS checkout event".into())
            })?;

        let Some(existing) = VpsRepository::find_by_id(pool, vps_id).await? else {
            tracing::warn!("Webhook VPS: suscripción {vps_id} no existe en BD");
            return Ok(false);
        };

        if existing.status != "pending_payment" {
            tracing::warn!(
                "Webhook VPS: {} tiene status '{}', esperaba 'pending_payment'",
                vps_id,
                existing.status
            );
            return Ok(false);
        }

        VpsRepository::set_stripe_subscription_id(pool, vps_id, stripe_sub_id).await?;
        VpsRepository::update_status(pool, vps_id, "pending_approval").await?;
        let _ = VpsRepository::add_event(
            pool,
            vps_id,
            "stripe_checkout_completed",
            Some(serde_json::json!({"stripe_subscription_id": stripe_sub_id})),
        )
        .await;

        if let Ok(admin_ids) = UserRepository::admin_ids(pool).await {
            if !admin_ids.is_empty() {
                let notification = CreateNotification {
                    user_id: Uuid::nil(),
                    notification_type: NOTIF_VPS_PENDING_APPROVAL.to_string(),
                    title: format!(
                        "VPS pendiente de aprobación: {}",
                        humanize_tier_name(&existing.tier_name)
                    ),
                    body: Some(format!(
                        "{} pagó {} y quedó esperando aprobación manual.",
                        existing.client_email,
                        humanize_tier_name(&existing.tier_name)
                    )),
                    link: Some("/panel".to_string()),
                    reference_type: Some("vps_subscription".to_string()),
                    reference_id: Some(existing.id),
                };
                let _ = notification_hub
                    .notify_many(&admin_ids, &notification)
                    .await;
            }
        }

        if let Some(config) = email_config {
            if let Ok(admin_emails) = UserRepository::admin_emails(pool).await {
                EmailService::send_vps_pending_approval(
                    config,
                    &admin_emails,
                    &existing.client_email,
                    humanize_tier_name(&existing.tier_name),
                    existing.monthly_price_cents,
                )
                .await;
            }
        }

        Ok(true)
    }

    async fn on_invoice_paid(pool: &PgPool, data: &serde_json::Value) -> Result<bool, AppError> {
        let stripe_sub_id = match data["object"]["subscription"].as_str() {
            Some(value) if !value.is_empty() => value,
            _ => return Ok(false),
        };

        let Some(vps_subscription) =
            VpsRepository::find_by_stripe_subscription(pool, stripe_sub_id).await?
        else {
            return Ok(false);
        };

        if vps_subscription.status == "pending_payment" {
            VpsRepository::update_status(pool, vps_subscription.id, "pending_approval").await?;
        }

        let _ = VpsRepository::add_event(
            pool,
            vps_subscription.id,
            "invoice_paid",
            Some(serde_json::json!({
                "invoice_id": data["object"]["id"].as_str(),
                "amount_paid": data["object"]["amount_paid"].as_i64(),
            })),
        )
        .await;

        Ok(true)
    }

    async fn on_subscription_deleted(
        pool: &PgPool,
        data: &serde_json::Value,
    ) -> Result<bool, AppError> {
        let stripe_sub_id = match data["object"]["id"].as_str() {
            Some(value) if !value.is_empty() => value,
            _ => return Ok(false),
        };

        let Some(vps_subscription) =
            VpsRepository::find_by_stripe_subscription(pool, stripe_sub_id).await?
        else {
            return Ok(false);
        };

        VpsRepository::update_status(pool, vps_subscription.id, "cancelled").await?;
        let _ = VpsRepository::add_event(
            pool,
            vps_subscription.id,
            "stripe_subscription_cancelled",
            None,
        )
        .await;
        Ok(true)
    }

    async fn on_payment_failed(
        pool: &PgPool,
        notification_hub: &NotificationHub,
        data: &serde_json::Value,
    ) -> Result<bool, AppError> {
        let stripe_sub_id = match data["object"]["subscription"].as_str() {
            Some(value) if !value.is_empty() => value,
            _ => return Ok(false),
        };

        let Some(vps_subscription) =
            VpsRepository::find_by_stripe_subscription(pool, stripe_sub_id).await?
        else {
            return Ok(false);
        };

        VpsRepository::update_status(pool, vps_subscription.id, "suspended").await?;
        let _ = VpsRepository::add_event(
            pool,
            vps_subscription.id,
            "payment_failed",
            Some(serde_json::json!({"invoice_id": data["object"]["id"].as_str()})),
        )
        .await;

        if let Some(user_id) = vps_subscription.user_id {
            let _ = notification_hub
                .notify(CreateNotification {
                    user_id,
                    notification_type: NOTIF_VPS_SUSPENDED.to_string(),
                    title: "Tu VPS fue suspendido por pago fallido".to_string(),
                    body: Some(
                        "No pudimos procesar el cobro mensual. Actualiza tu método de pago para reactivar el servidor."
                            .to_string(),
                    ),
                    link: Some("/panel".to_string()),
                    reference_type: Some("vps_subscription".to_string()),
                    reference_id: Some(vps_subscription.id),
                })
                .await;
        }

        Ok(true)
    }

    pub async fn cancel_and_refund_subscription(
        http_client: &Client,
        stripe_key: &str,
        stripe_subscription_id: &str,
    ) -> Result<(), AppError> {
        let latest_payment_intent =
            Self::fetch_latest_payment_intent_id(http_client, stripe_key, stripe_subscription_id)
                .await?;

        let cancel_response = http_client
            .delete(format!(
                "https://api.stripe.com/v1/subscriptions/{stripe_subscription_id}"
            ))
            .basic_auth(stripe_key, Option::<&str>::None)
            .send()
            .await
            .map_err(|error| {
                AppError::Internal(format!("Stripe cancel subscription failed: {error}"))
            })?;

        if !cancel_response.status().is_success() {
            let status = cancel_response.status();
            let body = cancel_response.text().await.unwrap_or_default();
            tracing::error!("Stripe cancel subscription error: {status} — {body}");
            return Err(AppError::Internal(format!(
                "No se pudo cancelar la suscripción Stripe: {status}"
            )));
        }

        if let Some(payment_intent_id) = latest_payment_intent {
            let refund_response = http_client
                .post("https://api.stripe.com/v1/refunds")
                .basic_auth(stripe_key, Option::<&str>::None)
                .form(&[("payment_intent", payment_intent_id)])
                .send()
                .await
                .map_err(|error| AppError::Internal(format!("Stripe refund failed: {error}")))?;

            if !refund_response.status().is_success() {
                let status = refund_response.status();
                let body = refund_response.text().await.unwrap_or_default();
                tracing::error!("Stripe refund error: {status} — {body}");
                return Err(AppError::Internal(format!(
                    "No se pudo reembolsar la suscripción Stripe: {status}"
                )));
            }
        }

        Ok(())
    }

    async fn fetch_latest_payment_intent_id(
        http_client: &Client,
        stripe_key: &str,
        stripe_subscription_id: &str,
    ) -> Result<Option<String>, AppError> {
        let response = http_client
            .get(format!(
                "https://api.stripe.com/v1/subscriptions/{stripe_subscription_id}?expand[]=latest_invoice.payment_intent"
            ))
            .basic_auth(stripe_key, Option::<&str>::None)
            .send()
            .await
            .map_err(|error| AppError::Internal(format!("Stripe subscription fetch failed: {error}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::warn!("Stripe subscription fetch error: {status} — {body}");
            return Ok(None);
        }

        let payload: serde_json::Value = response.json().await.map_err(|error| {
            AppError::Internal(format!("Stripe subscription parse failed: {error}"))
        })?;

        Ok(payload["latest_invoice"]["payment_intent"]["id"]
            .as_str()
            .map(ToString::to_string))
    }

    pub async fn notify_approved(
        notification_hub: &NotificationHub,
        user_id: Option<Uuid>,
        subscription_id: Uuid,
        tier_name: &str,
        ip: Option<&str>,
    ) {
        if let Some(owner_id) = user_id {
            let body = match ip {
                Some(public_ip) => format!(
                    "Tu {} ya está activo. IP pública: {}.",
                    humanize_tier_name(tier_name),
                    public_ip
                ),
                None => format!("Tu {} ya está activo.", humanize_tier_name(tier_name)),
            };

            let _ = notification_hub
                .notify(CreateNotification {
                    user_id: owner_id,
                    notification_type: NOTIF_VPS_APPROVED.to_string(),
                    title: format!("{} aprobado", humanize_tier_name(tier_name)),
                    body: Some(body),
                    link: Some("/panel".to_string()),
                    reference_type: Some("vps_subscription".to_string()),
                    reference_id: Some(subscription_id),
                })
                .await;
        }
    }

    pub async fn notify_rejected(
        notification_hub: &NotificationHub,
        user_id: Option<Uuid>,
        subscription_id: Uuid,
        tier_name: &str,
        reason: &str,
    ) {
        if let Some(owner_id) = user_id {
            let _ = notification_hub
                .notify(CreateNotification {
                    user_id: owner_id,
                    notification_type: NOTIF_VPS_REJECTED.to_string(),
                    title: format!("{} rechazado", humanize_tier_name(tier_name)),
                    body: Some(format!("La solicitud fue rechazada: {reason}")),
                    link: Some("/panel".to_string()),
                    reference_type: Some("vps_subscription".to_string()),
                    reference_id: Some(subscription_id),
                })
                .await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn humanize_tier_name_maps_known_slugs() {
        assert_eq!(humanize_tier_name("vps1"), "Cloud VPS 1");
        assert_eq!(humanize_tier_name("vps2"), "Cloud VPS 2");
        assert_eq!(humanize_tier_name("vps3"), "Cloud VPS 3");
        assert_eq!(humanize_tier_name("vps4"), "Cloud VPS 4");
        assert_eq!(humanize_tier_name("otro"), "VPS dedicado");
    }
}
