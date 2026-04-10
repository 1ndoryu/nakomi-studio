use reqwest::Client;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{PaymentMethodResponse, SetupIntentResponse};
use crate::repositories::{PaymentMethodRepository, UpsertPaymentMethodParams, UserRepository};

/* [104A-14] Guardado real de tarjetas con SetupIntent.
 * Crea/reutiliza customer en Stripe, persiste metadata local y desacopla la tarjeta
 * al borrar para que el panel de métodos de pago deje de ser un placeholder roto. */
pub struct PaymentMethodService;

impl PaymentMethodService {
    pub async fn create_setup_intent(
        pool: &PgPool,
        http_client: &Client,
        stripe_key: &str,
        user_id: Uuid,
    ) -> Result<SetupIntentResponse, AppError> {
        let customer_id = Self::ensure_stripe_customer(pool, http_client, stripe_key, user_id).await?;
        let response = http_client
            .post("https://api.stripe.com/v1/setup_intents")
            .basic_auth(stripe_key, None::<&str>)
            .form(&[
                ("customer".to_string(), customer_id),
                ("usage".to_string(), "off_session".to_string()),
                ("payment_method_types[]".to_string(), "card".to_string()),
                ("metadata[user_id]".to_string(), user_id.to_string()),
            ])
            .send()
            .await
            .map_err(|error| AppError::Internal(format!("Error comunicando con Stripe: {error}")))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            tracing::error!("Stripe create_setup_intent fallo: {body}");
            return Err(Self::classify_stripe_error(
                &body,
                "No se pudo preparar el guardado de la tarjeta",
            ));
        }

        let setup_intent = response
            .json::<StripeSetupIntentSecret>()
            .await
            .map_err(|error| AppError::Internal(format!("Error parseando SetupIntent: {error}")))?;

        Ok(SetupIntentResponse {
            client_secret: setup_intent.client_secret,
        })
    }

    pub async fn save_payment_method(
        pool: &PgPool,
        http_client: &Client,
        stripe_key: &str,
        user_id: Uuid,
        setup_intent_id: &str,
    ) -> Result<PaymentMethodResponse, AppError> {
        let customer_id = Self::ensure_stripe_customer(pool, http_client, stripe_key, user_id).await?;
        let setup_intent = Self::retrieve_setup_intent(http_client, stripe_key, setup_intent_id).await?;

        if setup_intent.status != "succeeded" {
            return Err(AppError::BadRequest(
                "La tarjeta no quedo confirmada en Stripe".into(),
            ));
        }

        if setup_intent.customer.as_deref() != Some(customer_id.as_str()) {
            return Err(AppError::Forbidden(
                "La tarjeta no pertenece al usuario autenticado".into(),
            ));
        }

        let payment_method = setup_intent.payment_method.ok_or_else(|| {
            AppError::BadRequest("Stripe no devolvio la tarjeta confirmada".into())
        })?;
        let card = payment_method.card.ok_or_else(|| {
            AppError::BadRequest("Stripe devolvio un metodo de pago no soportado".into())
        })?;

        let saved_method = PaymentMethodRepository::upsert_from_stripe(
            pool,
            UpsertPaymentMethodParams {
                user_id,
                stripe_payment_method_id: &payment_method.id,
                card_fingerprint: &card.fingerprint,
                brand: &card.brand,
                last_four: &card.last4,
                exp_month: card.exp_month,
                exp_year: card.exp_year,
                is_default: true,
            },
        )
        .await?;

        Ok(saved_method.into())
    }

    pub async fn list_payment_methods(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<PaymentMethodResponse>, AppError> {
        let methods = PaymentMethodRepository::list_for_user(pool, user_id).await?;
        Ok(methods.into_iter().map(Into::into).collect())
    }

    pub async fn delete_payment_method(
        pool: &PgPool,
        http_client: &Client,
        stripe_key: &str,
        user_id: Uuid,
        payment_method_id: Uuid,
    ) -> Result<(), AppError> {
        let payment_method = PaymentMethodRepository::find_owned(pool, user_id, payment_method_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Tarjeta no encontrada".into()))?;

        Self::detach_payment_method(http_client, stripe_key, &payment_method.stripe_payment_method_id).await?;

        PaymentMethodRepository::delete_owned(pool, user_id, payment_method_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Tarjeta no encontrada".into()))?;

        Ok(())
    }

    async fn ensure_stripe_customer(
        pool: &PgPool,
        http_client: &Client,
        stripe_key: &str,
        user_id: Uuid,
    ) -> Result<String, AppError> {
        if let Some(customer_id) = UserRepository::stripe_customer_id(pool, user_id).await? {
            return Ok(customer_id);
        }

        let user = UserRepository::find_by_id(pool, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Usuario no encontrado".into()))?;
        let customer_id = Self::find_or_create_customer(http_client, stripe_key, &user.email).await?;

        UserRepository::set_stripe_customer_id(pool, user_id, &customer_id).await?;
        Ok(customer_id)
    }

    async fn find_or_create_customer(
        http_client: &Client,
        stripe_key: &str,
        email: &str,
    ) -> Result<String, AppError> {
        let search_response = http_client
            .get("https://api.stripe.com/v1/customers")
            .basic_auth(stripe_key, None::<&str>)
            .query(&[("email", email), ("limit", "1")])
            .send()
            .await
            .map_err(|error| AppError::Internal(format!("Error buscando customer en Stripe: {error}")))?;

        if search_response.status().is_success() {
            let payload = search_response
                .json::<StripeCustomerSearch>()
                .await
                .map_err(|error| AppError::Internal(format!("Error parseando customer Stripe: {error}")))?;

            if let Some(customer) = payload.data.first() {
                return Ok(customer.id.clone());
            }
        }

        let create_response = http_client
            .post("https://api.stripe.com/v1/customers")
            .basic_auth(stripe_key, None::<&str>)
            .form(&[("email", email)])
            .send()
            .await
            .map_err(|error| AppError::Internal(format!("Error creando customer en Stripe: {error}")))?;

        if !create_response.status().is_success() {
            let body = create_response.text().await.unwrap_or_default();
            tracing::error!("Stripe create_customer fallo: {body}");
            return Err(Self::classify_stripe_error(
                &body,
                "No se pudo preparar el cliente de pagos",
            ));
        }

        let customer = create_response
            .json::<StripeCustomer>()
            .await
            .map_err(|error| AppError::Internal(format!("Error parseando customer Stripe: {error}")))?;

        Ok(customer.id)
    }

    async fn retrieve_setup_intent(
        http_client: &Client,
        stripe_key: &str,
        setup_intent_id: &str,
    ) -> Result<StripeSetupIntentExpanded, AppError> {
        let response = http_client
            .get(format!("https://api.stripe.com/v1/setup_intents/{setup_intent_id}"))
            .basic_auth(stripe_key, None::<&str>)
            .query(&[("expand[]", "payment_method")])
            .send()
            .await
            .map_err(|error| AppError::Internal(format!("Error consultando SetupIntent: {error}")))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            tracing::error!("Stripe retrieve_setup_intent fallo: {body}");
            return Err(Self::classify_stripe_error(
                &body,
                "No se pudo validar la tarjeta guardada",
            ));
        }

        response
            .json::<StripeSetupIntentExpanded>()
            .await
            .map_err(|error| AppError::Internal(format!("Error parseando SetupIntent expandido: {error}")))
    }

    async fn detach_payment_method(
        http_client: &Client,
        stripe_key: &str,
        stripe_payment_method_id: &str,
    ) -> Result<(), AppError> {
        let response = http_client
            .post(format!(
                "https://api.stripe.com/v1/payment_methods/{stripe_payment_method_id}/detach"
            ))
            .basic_auth(stripe_key, None::<&str>)
            .send()
            .await
            .map_err(|error| AppError::Internal(format!("Error desacoplando tarjeta en Stripe: {error}")))?;

        if response.status().is_success() {
            return Ok(());
        }

        let body = response.text().await.unwrap_or_default();
        if body.contains("already been detached") || body.contains("not attached to a customer") {
            tracing::warn!("Stripe detach informo tarjeta ya desacoplada: {body}");
            return Ok(());
        }

        tracing::error!("Stripe detach_payment_method fallo: {body}");
        Err(Self::classify_stripe_error(
            &body,
            "No se pudo eliminar la tarjeta guardada",
        ))
    }

    fn classify_stripe_error(body: &str, fallback_message: &str) -> AppError {
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(body);
        let (error_type, message) = match parsed {
            Ok(json) => (
                json["error"]["type"].as_str().unwrap_or("unknown").to_string(),
                json["error"]["message"].as_str().unwrap_or(fallback_message).to_string(),
            ),
            Err(_) => return AppError::BadRequest(fallback_message.into()),
        };

        match error_type.as_str() {
            "authentication_error" => AppError::BadRequest(
                "Error de configuracion de pagos, contacta soporte".into(),
            ),
            "invalid_request_error" | "card_error" => {
                AppError::BadRequest(format!("Error de pago: {message}"))
            }
            "rate_limit_error" => AppError::BadRequest(
                "Demasiadas solicitudes de pago, intenta de nuevo en unos segundos".into(),
            ),
            _ => AppError::BadRequest(fallback_message.into()),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct StripeSetupIntentSecret {
    client_secret: String,
}

#[derive(Debug, serde::Deserialize)]
struct StripeSetupIntentExpanded {
    status: String,
    customer: Option<String>,
    payment_method: Option<StripeExpandedPaymentMethod>,
}

#[derive(Debug, serde::Deserialize)]
struct StripeExpandedPaymentMethod {
    id: String,
    card: Option<StripeCardDetails>,
}

#[derive(Debug, serde::Deserialize)]
struct StripeCardDetails {
    brand: String,
    last4: String,
    exp_month: i32,
    exp_year: i32,
    fingerprint: String,
}

#[derive(Debug, serde::Deserialize)]
struct StripeCustomerSearch {
    data: Vec<StripeCustomer>,
}

#[derive(Debug, serde::Deserialize)]
struct StripeCustomer {
    id: String,
}