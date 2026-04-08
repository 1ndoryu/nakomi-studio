/* [084A-24] Servicio de suscripciones Stripe para hosting.
 * Crea Checkout Sessions en modo subscription usando los Price IDs de Stripe.
 * Maneja webhooks de invoice.paid y customer.subscription.* para sincronizar estado.
 * Gotcha: NO usa PaymentIntents directos — usa Stripe Subscriptions nativas.
 * [094A-9] Auditoría de seguridad: validación estricta de campos JSON en webhooks,
 * idempotency key en checkout, verificación de status antes de activar. */

use reqwest::Client;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::repositories::HostingRepository;

/* Mapeo plan → Stripe Price ID (cargados desde env) */
#[derive(Debug, Clone)]
pub struct HostingStripeConfig {
    pub price_basico: String,
    pub price_pro: String,
    pub price_ecommerce: String,
}

impl HostingStripeConfig {
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let basico = std::env::var("GLORY_STRIPE_HOSTING_PRICE_BASICO").ok()?;
        let pro = std::env::var("GLORY_STRIPE_HOSTING_PRICE_PRO").ok()?;
        let ecommerce = std::env::var("GLORY_STRIPE_HOSTING_PRICE_ECOMMERCE").ok()?;

        if basico.is_empty() || pro.is_empty() || ecommerce.is_empty() {
            return None;
        }

        /* [094A-9] Validar formato de Price IDs al iniciar */
        if !basico.starts_with("price_") || !pro.starts_with("price_") || !ecommerce.starts_with("price_") {
            tracing::error!("Stripe hosting Price IDs con formato inválido — deben empezar con 'price_'");
            return None;
        }

        Some(Self {
            price_basico: basico,
            price_pro: pro,
            price_ecommerce: ecommerce,
        })
    }

    /// Devuelve el Stripe Price ID para un plan dado
    #[must_use]
    pub fn price_for_plan(&self, plan: &str) -> Option<&str> {
        match plan {
            "basico" => Some(&self.price_basico),
            "pro" => Some(&self.price_pro),
            "ecommerce" => Some(&self.price_ecommerce),
            _ => None,
        }
    }
}

/* Respuesta mínima de Stripe Checkout Session */
#[derive(Debug, Deserialize)]
struct CheckoutSession {
    id: String,
    url: Option<String>,
}

/// Parámetros para crear una Checkout Session de hosting
pub struct CheckoutParams<'a> {
    pub http_client: &'a Client,
    pub stripe_key: &'a str,
    pub config: &'a HostingStripeConfig,
    pub subscription_id: Uuid,
    pub plan: &'a str,
    pub customer_email: &'a str,
    pub success_url: &'a str,
    pub cancel_url: &'a str,
}

pub struct HostingStripeService;

impl HostingStripeService {
    /// Crea una Stripe Checkout Session para suscripción de hosting.
    /// Retorna la URL a la que redirigir al cliente.
    pub async fn create_checkout_session(
        params: &CheckoutParams<'_>,
    ) -> Result<String, AppError> {
        let price_id = params.config.price_for_plan(params.plan).ok_or_else(|| {
            AppError::Validation(format!(
                "Plan '{}' no tiene precio Stripe configurado",
                params.plan
            ))
        })?;

        let form = vec![
            ("mode", "subscription".to_string()),
            ("line_items[0][price]", price_id.to_string()),
            ("line_items[0][quantity]", "1".to_string()),
            ("customer_email", params.customer_email.to_string()),
            ("success_url", params.success_url.to_string()),
            ("cancel_url", params.cancel_url.to_string()),
            (
                "metadata[hosting_subscription_id]",
                params.subscription_id.to_string(),
            ),
            (
                "subscription_data[metadata][hosting_subscription_id]",
                params.subscription_id.to_string(),
            ),
        ];

        let resp = params.http_client
            .post("https://api.stripe.com/v1/checkout/sessions")
            .basic_auth(params.stripe_key, Option::<&str>::None)
            /* [094A-9] Idempotency key: evita sesiones duplicadas si la request se reintenta */
            .header("Idempotency-Key", format!("hosting-checkout-{}", params.subscription_id))
            .form(&form)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Stripe checkout request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::error!("Stripe checkout error: {status} — {body}");
            return Err(AppError::Internal(format!(
                "Stripe checkout session failed: {status}"
            )));
        }

        let session: CheckoutSession = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Stripe parse error: {e}")))?;

        tracing::info!(
            "Checkout session creada: {} para hosting {}",
            session.id,
            params.subscription_id
        );

        session.url.ok_or_else(|| {
            AppError::Internal("Stripe no retornó URL de checkout".into())
        })
    }

    /// Procesa webhooks de Stripe relacionados a hosting subscriptions.
    /// Eventos: checkout.session.completed, invoice.paid, customer.subscription.deleted
    pub async fn handle_webhook(
        pool: &PgPool,
        event_type: &str,
        data: &serde_json::Value,
    ) -> Result<bool, AppError> {
        match event_type {
            "checkout.session.completed" => Self::on_checkout_completed(pool, data).await,
            "invoice.paid" => Self::on_invoice_paid(pool, data).await,
            "customer.subscription.deleted" => Self::on_subscription_deleted(pool, data).await,
            "invoice.payment_failed" => Self::on_payment_failed(pool, data).await,
            _ => Ok(false),
        }
    }

    /* Cliente completó el checkout — activar suscripción */
    async fn on_checkout_completed(
        pool: &PgPool,
        data: &serde_json::Value,
    ) -> Result<bool, AppError> {
        let mode = data["object"]["mode"].as_str().unwrap_or("");
        if mode != "subscription" {
            return Ok(false);
        }

        let sub_id_str = data["object"]["metadata"]["hosting_subscription_id"]
            .as_str()
            .or_else(|| {
                data["object"]["subscription_data"]["metadata"]
                    ["hosting_subscription_id"]
                    .as_str()
            });

        let Some(hosting_id) = sub_id_str.and_then(|s| Uuid::parse_str(s).ok()) else {
            return Ok(false);
        };

        /* [094A-9] Validar que subscription ID no esté vacío — campo crítico para vincular Stripe */
        let stripe_sub_id = data["object"]["subscription"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                tracing::warn!("Webhook checkout.session.completed sin subscription ID para hosting {hosting_id}");
                AppError::BadRequest("Missing subscription field in checkout event".into())
            })?;

        /* [094A-9] Verificar que la suscripción existe y está en estado pendiente antes de activar */
        let existing = HostingRepository::find_by_id(pool, hosting_id).await?;
        let Some(existing) = existing else {
            tracing::warn!("Webhook: hosting subscription {hosting_id} no existe en BD");
            return Ok(false);
        };
        if existing.status != "pending" && existing.status != "provisioning" {
            tracing::warn!(
                "Webhook: hosting {hosting_id} tiene status '{}', esperaba 'pending' o 'provisioning'",
                existing.status
            );
            return Ok(false);
        }

        HostingRepository::set_stripe_subscription_id(pool, hosting_id, stripe_sub_id).await?;
        HostingRepository::update_status(pool, hosting_id, "active").await?;

        /* [094A-9] Log de errores en evento, no silenciar */
        if let Err(e) = HostingRepository::add_event(
            pool, hosting_id, "stripe_checkout_completed",
            Some(serde_json::json!({"stripe_subscription_id": stripe_sub_id})),
        ).await {
            tracing::warn!("Error registrando evento stripe_checkout_completed para {hosting_id}: {e}");
        }

        tracing::info!("Hosting {hosting_id} activado via Stripe checkout (sub: {stripe_sub_id})");
        Ok(true)
    }

    /* Factura pagada — renovación mensual exitosa */
    async fn on_invoice_paid(
        pool: &PgPool,
        data: &serde_json::Value,
    ) -> Result<bool, AppError> {
        /* [094A-9] Validar campo subscription explícitamente */
        let stripe_sub_id = match data["object"]["subscription"].as_str() {
            Some(id) if !id.is_empty() => id,
            _ => return Ok(false),
        };

        let Some(hosting) =
            HostingRepository::find_by_stripe_subscription(pool, stripe_sub_id).await?
        else {
            return Ok(false);
        };

        if hosting.status != "active" {
            HostingRepository::update_status(pool, hosting.id, "active").await?;
        }

        /* [094A-9] Log de errores en evento, no silenciar */
        if let Err(e) = HostingRepository::add_event(
            pool, hosting.id, "invoice_paid",
            Some(serde_json::json!({
                "invoice_id": data["object"]["id"].as_str(),
                "amount_paid": data["object"]["amount_paid"].as_i64(),
            })),
        ).await {
            tracing::warn!("Error registrando evento invoice_paid para {}: {e}", hosting.id);
        }

        tracing::info!("Hosting {} — invoice paid", hosting.id);
        Ok(true)
    }

    /* Suscripción cancelada en Stripe */
    async fn on_subscription_deleted(
        pool: &PgPool,
        data: &serde_json::Value,
    ) -> Result<bool, AppError> {
        /* [094A-9] Validar campo id explícitamente */
        let stripe_sub_id = match data["object"]["id"].as_str() {
            Some(id) if !id.is_empty() => id,
            _ => return Ok(false),
        };
        let Some(hosting) =
            HostingRepository::find_by_stripe_subscription(pool, stripe_sub_id).await?
        else {
            return Ok(false);
        };

        HostingRepository::update_status(pool, hosting.id, "cancelled").await?;
        /* [094A-9] Log de errores en evento, no silenciar */
        if let Err(e) = HostingRepository::add_event(
            pool, hosting.id, "stripe_subscription_cancelled", None,
        ).await {
            tracing::warn!("Error registrando evento stripe_subscription_cancelled para {}: {e}", hosting.id);
        }

        tracing::info!("Hosting {} cancelado via Stripe webhook", hosting.id);
        Ok(true)
    }

    /* Pago fallido — suspender hosting */
    async fn on_payment_failed(
        pool: &PgPool,
        data: &serde_json::Value,
    ) -> Result<bool, AppError> {
        /* [094A-9] Validar campo subscription explícitamente */
        let stripe_sub_id = match data["object"]["subscription"].as_str() {
            Some(id) if !id.is_empty() => id,
            _ => return Ok(false),
        };
        let Some(hosting) =
            HostingRepository::find_by_stripe_subscription(pool, stripe_sub_id).await?
        else {
            return Ok(false);
        };

        HostingRepository::update_status(pool, hosting.id, "suspended").await?;
        /* [094A-9] Log de errores en evento, no silenciar */
        if let Err(e) = HostingRepository::add_event(
            pool, hosting.id, "payment_failed",
            Some(serde_json::json!({"invoice_id": data["object"]["id"].as_str()})),
        ).await {
            tracing::warn!("Error registrando evento payment_failed para {}: {e}", hosting.id);
        }

        tracing::warn!("Hosting {} suspendido por pago fallido", hosting.id);
        Ok(true)
    }
}

/* ============================================================
   TESTS — [094A-10] Validación de config y lógica Stripe
   ============================================================ */

#[cfg(test)]
mod tests {
    use super::*;

    /* --- HostingStripeConfig::from_env --- */

    #[test]
    fn config_from_env_valid_prices() {
        /* Simular variables de entorno con Price IDs válidos */
        std::env::set_var("GLORY_STRIPE_HOSTING_PRICE_BASICO", "price_test_basico");
        std::env::set_var("GLORY_STRIPE_HOSTING_PRICE_PRO", "price_test_pro");
        std::env::set_var("GLORY_STRIPE_HOSTING_PRICE_ECOMMERCE", "price_test_ecommerce");

        let config = HostingStripeConfig::from_env();
        assert!(config.is_some());

        let config = config.unwrap();
        assert_eq!(config.price_basico, "price_test_basico");
        assert_eq!(config.price_pro, "price_test_pro");
        assert_eq!(config.price_ecommerce, "price_test_ecommerce");

        /* Cleanup */
        std::env::remove_var("GLORY_STRIPE_HOSTING_PRICE_BASICO");
        std::env::remove_var("GLORY_STRIPE_HOSTING_PRICE_PRO");
        std::env::remove_var("GLORY_STRIPE_HOSTING_PRICE_ECOMMERCE");
    }

    #[test]
    fn config_from_env_invalid_format_rejected() {
        /* Price IDs que no empiezan con "price_" deben ser rechazados */
        std::env::set_var("GLORY_STRIPE_HOSTING_PRICE_BASICO", "invalid_basico");
        std::env::set_var("GLORY_STRIPE_HOSTING_PRICE_PRO", "price_test_pro");
        std::env::set_var("GLORY_STRIPE_HOSTING_PRICE_ECOMMERCE", "price_test_ecommerce");

        let config = HostingStripeConfig::from_env();
        assert!(config.is_none());

        std::env::remove_var("GLORY_STRIPE_HOSTING_PRICE_BASICO");
        std::env::remove_var("GLORY_STRIPE_HOSTING_PRICE_PRO");
        std::env::remove_var("GLORY_STRIPE_HOSTING_PRICE_ECOMMERCE");
    }

    #[test]
    fn config_from_env_missing_var_returns_none() {
        /* Sin variables de entorno debería retornar None */
        std::env::remove_var("GLORY_STRIPE_HOSTING_PRICE_BASICO");
        std::env::remove_var("GLORY_STRIPE_HOSTING_PRICE_PRO");
        std::env::remove_var("GLORY_STRIPE_HOSTING_PRICE_ECOMMERCE");

        let config = HostingStripeConfig::from_env();
        assert!(config.is_none());
    }

    /* --- price_for_plan --- */

    #[test]
    fn price_for_plan_returns_correct_id() {
        let config = HostingStripeConfig {
            price_basico: "price_aaa".to_string(),
            price_pro: "price_bbb".to_string(),
            price_ecommerce: "price_ccc".to_string(),
        };

        assert_eq!(config.price_for_plan("basico"), Some("price_aaa"));
        assert_eq!(config.price_for_plan("pro"), Some("price_bbb"));
        assert_eq!(config.price_for_plan("ecommerce"), Some("price_ccc"));
        assert_eq!(config.price_for_plan("unknown"), None);
        assert_eq!(config.price_for_plan(""), None);
    }
}
