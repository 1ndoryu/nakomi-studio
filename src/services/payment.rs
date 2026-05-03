/* [044A-38 Fase 3] Servicio de pagos: integración con Stripe REST API.
 * PaymentIntent con capture_method manual (escrow).
 * 3 modos: full (20% desc), half_half (10% desc), phased (sin desc).
 * Webhook verifica firma HMAC-SHA256 con constant-time comparison. */

use reqwest::Client;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{
    OrderPayment, OrderStatus, PaymentIntentResponse, PaymentMode, PaymentResponse, PaymentStatus,
    PhaseStatus,
};
use crate::repositories::{CreatePaymentParams, OrderRepository, PaymentRepository};

pub struct PaymentService;

impl PaymentService {
    /// Inicia un pago: crea `PaymentIntent` en Stripe con capture manual, guarda en BD.
    /// [064A-65] `client_id` es None si el caller es admin (no aplica ownership check).
    /// [064A-59] `receipt_email` pre-llena el email en Stripe para no pedirlo en checkout.
    pub async fn initiate_payment(
        pool: &PgPool,
        http_client: &Client,
        stripe_key: &str,
        order_id: Uuid,
        client_id: Option<Uuid>,
        phase_number: Option<i32>,
        receipt_email: Option<&str>,
    ) -> Result<PaymentIntentResponse, AppError> {
        let order = OrderRepository::find_order_by_id(pool, order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Orden no encontrada".into()))?;

        if let Some(cid) = client_id {
            if order.client_id != cid {
                return Err(AppError::Forbidden("No tienes acceso a esta orden".into()));
            }
        }

        let (amount_cents, phase_id, description) =
            Self::resolve_payment_amount(pool, &order, phase_number).await?;

        let stripe_resp = Self::create_stripe_intent(
            http_client,
            stripe_key,
            amount_cents,
            &order.currency,
            order_id,
            phase_number,
            receipt_email,
        )
        .await?;

        let payment = PaymentRepository::create_payment(
            pool,
            CreatePaymentParams {
                order_id,
                phase_id,
                amount_cents,
                currency: &order.currency,
                payment_mode: order.payment_mode,
                stripe_payment_intent_id: &stripe_resp.id,
                description: Some(&description),
            },
        )
        .await
        .map_err(|e| AppError::Internal(format!("Error guardando pago: {e}")))?;

        Ok(PaymentIntentResponse {
            payment_id: payment.id,
            client_secret: stripe_resp.client_secret,
            amount_cents,
            currency: order.currency,
        })
    }

    /// Procesa webhook de Stripe (ya verificada la firma)
    pub async fn handle_webhook(
        pool: &PgPool,
        event_type: &str,
        data: &serde_json::Value,
    ) -> Result<(), AppError> {
        match event_type {
            "payment_intent.succeeded" => {
                let pi_id = data["object"]["id"]
                    .as_str()
                    .ok_or_else(|| AppError::BadRequest("Missing payment_intent id".into()))?;
                let charge_id = data["object"]["latest_charge"].as_str();

                let payment = PaymentRepository::find_by_stripe_intent(pool, pi_id)
                    .await?
                    .ok_or_else(|| {
                        AppError::NotFound(format!("Payment for intent {pi_id} not found"))
                    })?;

                PaymentRepository::update_status_held(pool, payment.id).await?;
                if let Some(cid) = charge_id {
                    PaymentRepository::update_charge_id(pool, payment.id, cid).await?;
                }

                Self::handle_payment_success(pool, &payment).await?;
            }
            "payment_intent.payment_failed" => {
                let pi_id = data["object"]["id"]
                    .as_str()
                    .ok_or_else(|| AppError::BadRequest("Missing payment_intent id".into()))?;

                if let Some(payment) = PaymentRepository::find_by_stripe_intent(pool, pi_id).await?
                {
                    PaymentRepository::update_status(pool, payment.id, PaymentStatus::Failed)
                        .await?;
                }
            }
            _ => {
                tracing::debug!("Evento Stripe no manejado: {event_type}");
            }
        }
        Ok(())
    }

    /// Captura todos los pagos retenidos de una orden (al completarse)
    pub async fn capture_held_payments(
        pool: &PgPool,
        http_client: &Client,
        stripe_key: &str,
        order_id: Uuid,
    ) -> Result<(), AppError> {
        let held = PaymentRepository::find_held_for_order(pool, order_id).await?;
        for payment in held {
            if let Some(ref pi_id) = payment.stripe_payment_intent_id {
                Self::capture_stripe_intent(http_client, stripe_key, pi_id).await?;
                PaymentRepository::update_status_released(pool, payment.id).await?;
            }
        }
        Ok(())
    }

    /// Lista pagos de una orden con números de fase resueltos
    pub async fn list_payments(
        pool: &PgPool,
        order_id: Uuid,
    ) -> Result<Vec<PaymentResponse>, AppError> {
        let payments = PaymentRepository::list_for_order(pool, order_id).await?;
        let phases = OrderRepository::list_order_phases(pool, order_id).await?;

        Ok(payments
            .into_iter()
            .map(|p| {
                let phase_number = p.phase_id.and_then(|pid| {
                    phases
                        .iter()
                        .find(|ph| ph.id == pid)
                        .map(|ph| ph.phase_number)
                });
                PaymentResponse {
                    id: p.id,
                    order_id: p.order_id,
                    phase_number,
                    amount_cents: p.amount_cents,
                    currency: p.currency,
                    status: p.status,
                    payment_mode: p.payment_mode,
                    description: p.description,
                    created_at: p.created_at,
                }
            })
            .collect())
    }

    /// Verifica firma HMAC-SHA256 del webhook de Stripe (constant-time comparison)
    pub fn verify_webhook_signature(
        payload: &[u8],
        signature_header: &str,
        webhook_secret: &str,
    ) -> Result<(), AppError> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let mut timestamp = None;
        let mut expected_sig = None;

        for part in signature_header.split(',') {
            if let Some((key, value)) = part.split_once('=') {
                match key {
                    "t" => timestamp = Some(value),
                    "v1" => expected_sig = Some(value),
                    _ => {}
                }
            }
        }

        let ts = timestamp
            .ok_or_else(|| AppError::BadRequest("Missing timestamp in Stripe signature".into()))?;
        let sig = expected_sig
            .ok_or_else(|| AppError::BadRequest("Missing v1 in Stripe signature".into()))?;

        let signed_payload = format!("{ts}.{}", String::from_utf8_lossy(payload));

        let mut mac = Hmac::<Sha256>::new_from_slice(webhook_secret.as_bytes())
            .map_err(|e| AppError::Internal(format!("HMAC init failed: {e}")))?;
        mac.update(signed_payload.as_bytes());

        let decoded_sig = hex::decode(sig)
            .map_err(|_| AppError::BadRequest("Invalid hex in Stripe signature".into()))?;

        mac.verify_slice(&decoded_sig)
            .map_err(|_| AppError::Forbidden("Invalid webhook signature".into()))?;

        /* [064A-73] Verificar freshness: máximo 2 minutos de tolerancia (antes 5 min).
         * Reducido para estrechar la ventana de replay attacks. */
        if let Ok(ts_num) = ts.parse::<i64>() {
            let now = chrono::Utc::now().timestamp();
            if (now - ts_num).unsigned_abs() > 120 {
                return Err(AppError::BadRequest("Webhook timestamp too old".into()));
            }
        }

        Ok(())
    }

    /* ============================================================
    HELPERS PRIVADOS
    ============================================================ */

    /// Resuelve monto, `phase_id` y descripción según `payment_mode`
    async fn resolve_payment_amount(
        pool: &PgPool,
        order: &crate::models::Order,
        phase_number: Option<i32>,
    ) -> Result<(i32, Option<Uuid>, String), AppError> {
        match order.payment_mode {
            PaymentMode::Full => {
                if order.status != OrderStatus::PaymentHeld {
                    return Err(AppError::BadRequest("La orden ya fue pagada".into()));
                }
                Ok((
                    order.final_price_cents,
                    None,
                    format!("Pago completo - Orden #{}", order.order_number),
                ))
            }
            PaymentMode::HalfHalf => {
                let existing = PaymentRepository::list_for_order(pool, order.id).await?;
                let paid_count = existing
                    .iter()
                    .filter(|p| {
                        p.status == PaymentStatus::Held || p.status == PaymentStatus::Released
                    })
                    .count();

                if paid_count >= 2 {
                    return Err(AppError::BadRequest(
                        "Ambos pagos ya fueron realizados".into(),
                    ));
                }

                let half = order.final_price_cents / 2;
                let (amount, desc) = if paid_count == 0 {
                    (
                        half,
                        format!("Primer pago 50% - Orden #{}", order.order_number),
                    )
                } else {
                    (
                        order.final_price_cents - half,
                        format!("Segundo pago 50% - Orden #{}", order.order_number),
                    )
                };
                Ok((amount, None, desc))
            }
            PaymentMode::Phased => {
                let pn = phase_number.ok_or_else(|| {
                    AppError::BadRequest("Se requiere phase_number para pago por fases".into())
                })?;
                let phase = OrderRepository::find_phase_by_number(pool, order.id, pn)
                    .await?
                    .ok_or_else(|| AppError::NotFound(format!("Fase {pn} no encontrada")))?;

                if phase.status != PhaseStatus::PendingPayment {
                    return Err(AppError::BadRequest(format!(
                        "La fase {pn} no está pendiente de pago (estado: {:?})",
                        phase.status
                    )));
                }

                Ok((
                    phase.price_cents,
                    Some(phase.id),
                    format!("Fase {} - Orden #{}", pn, order.order_number),
                ))
            }
        }
    }

    /// Crea `PaymentIntent` en Stripe con `capture_method` manual (escrow)
    /// [064A-59] `receipt_email` pre-llena el email — Stripe no lo pide de nuevo en checkout
    async fn create_stripe_intent(
        client: &Client,
        api_key: &str,
        amount: i32,
        currency: &str,
        order_id: Uuid,
        phase_number: Option<i32>,
        receipt_email: Option<&str>,
    ) -> Result<StripePaymentIntentMin, AppError> {
        let mut form = vec![
            ("amount".to_string(), amount.to_string()),
            ("currency".to_string(), currency.to_lowercase()),
            ("capture_method".to_string(), "manual".to_string()),
            ("metadata[order_id]".to_string(), order_id.to_string()),
        ];
        if let Some(pn) = phase_number {
            form.push(("metadata[phase_number]".to_string(), pn.to_string()));
        }
        if let Some(email) = receipt_email {
            form.push(("receipt_email".to_string(), email.to_string()));
        }

        let resp = client
            .post("https://api.stripe.com/v1/payment_intents")
            .basic_auth(api_key, None::<&str>)
            .form(&form)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Error comunicando con Stripe: {e}")))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            /* [064A-73] Log explícito de errores Stripe para debugging.
             * [074A-24] Clasificar el error Stripe para dar feedback útil al usuario
             * sin exponer detalles internos (ej: claves API). */
            tracing::error!("Stripe create_payment_intent falló: {body}");
            return Err(Self::classify_stripe_error(&body));
        }

        resp.json::<StripePaymentIntentMin>()
            .await
            .map_err(|e| AppError::Internal(format!("Error parseando respuesta Stripe: {e}")))
    }

    /// Captura un `PaymentIntent` en Stripe (libera fondos retenidos)
    async fn capture_stripe_intent(
        client: &Client,
        api_key: &str,
        payment_intent_id: &str,
    ) -> Result<(), AppError> {
        let url = format!("https://api.stripe.com/v1/payment_intents/{payment_intent_id}/capture");
        let resp = client
            .post(&url)
            .basic_auth(api_key, None::<&str>)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Error capturando pago Stripe: {e}")))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            /* [064A-73] Log explícito de errores Stripe para debugging */
            tracing::error!("Stripe capture falló: {body}");
            return Err(AppError::Internal(format!("Stripe capture error: {body}")));
        }
        Ok(())
    }

    /// Post-pago: avanza la máquina de estados de la orden según `payment_mode`
    async fn handle_payment_success(pool: &PgPool, payment: &OrderPayment) -> Result<(), AppError> {
        let order = OrderRepository::find_order_by_id(pool, payment.order_id)
            .await?
            .ok_or_else(|| AppError::Internal("Orden no encontrada post-pago".into()))?;

        match order.payment_mode {
            PaymentMode::Full => {
                /* Pago completo → todas las fases a Paid, orden a awaiting_assignment */
                OrderRepository::set_awaiting_assignment(pool, order.id).await?;
                let phases = OrderRepository::list_order_phases(pool, order.id).await?;
                for phase in phases {
                    if phase.status == PhaseStatus::PendingPayment
                        || phase.status == PhaseStatus::Locked
                    {
                        OrderRepository::update_phase_status(pool, phase.id, PhaseStatus::Paid)
                            .await?;
                    }
                }
            }
            PaymentMode::HalfHalf => {
                let all_payments = PaymentRepository::list_for_order(pool, order.id).await?;
                let held_count = all_payments
                    .iter()
                    .filter(|p| p.status == PaymentStatus::Held)
                    .count();

                if held_count >= 1 && order.status == OrderStatus::PaymentHeld {
                    /* Primer 50% → awaiting_assignment, desbloquear primera mitad de fases */
                    OrderRepository::set_awaiting_assignment(pool, order.id).await?;
                    let phases = OrderRepository::list_order_phases(pool, order.id).await?;
                    let midpoint = phases.len().div_ceil(2);
                    for (i, phase) in phases.iter().enumerate() {
                        if i < midpoint
                            && (phase.status == PhaseStatus::PendingPayment
                                || phase.status == PhaseStatus::Locked)
                        {
                            OrderRepository::update_phase_status(pool, phase.id, PhaseStatus::Paid)
                                .await?;
                        }
                    }
                }
                /* Segundo 50% → desbloquear fases restantes */
                if held_count >= 2 {
                    let phases = OrderRepository::list_order_phases(pool, order.id).await?;
                    for phase in phases {
                        if phase.status == PhaseStatus::Locked
                            || phase.status == PhaseStatus::PendingPayment
                        {
                            OrderRepository::update_phase_status(pool, phase.id, PhaseStatus::Paid)
                                .await?;
                        }
                    }
                }
            }
            PaymentMode::Phased => {
                /* Pago de fase individual → actualizar esa fase a Paid */
                if let Some(phase_id) = payment.phase_id {
                    OrderRepository::update_phase_status(pool, phase_id, PhaseStatus::Paid).await?;
                }
                /* Si la orden estaba en payment_held y la primera fase se pagó, avanzar */
                if order.status == OrderStatus::PaymentHeld {
                    OrderRepository::set_awaiting_assignment(pool, order.id).await?;
                }
            }
        }
        Ok(())
    }

    /// [044A-38 Fase 7] Ejecutar reembolso en Stripe.
    /// Para pagos con `capture_method=manual` (held): cancela el `PaymentIntent`.
    /// Para pagos ya capturados (released): crea un `Refund`.
    pub async fn refund_payment(
        http_client: &Client,
        stripe_key: &str,
        payment: &OrderPayment,
    ) -> Result<String, AppError> {
        let pi_id = payment
            .stripe_payment_intent_id
            .as_deref()
            .ok_or_else(|| AppError::Internal("Pago sin PaymentIntent de Stripe".into()))?;

        match payment.status {
            PaymentStatus::Held => {
                /* Fondos retenidos → cancelar PaymentIntent libera el dinero */
                let url = format!("https://api.stripe.com/v1/payment_intents/{pi_id}/cancel");
                let resp = http_client
                    .post(&url)
                    .basic_auth(stripe_key, None::<&str>)
                    .send()
                    .await
                    .map_err(|e| {
                        AppError::Internal(format!("Error cancelando PaymentIntent Stripe: {e}"))
                    })?;

                if !resp.status().is_success() {
                    let body = resp.text().await.unwrap_or_default();
                    tracing::error!("Stripe cancel falló: {body}");
                    return Err(AppError::Internal(format!("Stripe cancel error: {body}")));
                }
                /* Para cancel, el refund_id es el PI id mismo */
                Ok(format!("cancel_{pi_id}"))
            }
            PaymentStatus::Released => {
                /* Fondos ya capturados → crear Refund en Stripe */
                let resp = http_client
                    .post("https://api.stripe.com/v1/refunds")
                    .basic_auth(stripe_key, None::<&str>)
                    .form(&[("payment_intent", pi_id)])
                    .send()
                    .await
                    .map_err(|e| AppError::Internal(format!("Error creando refund Stripe: {e}")))?;

                if !resp.status().is_success() {
                    let body = resp.text().await.unwrap_or_default();
                    tracing::error!("Stripe refund falló: {body}");
                    return Err(AppError::Internal(format!("Stripe refund error: {body}")));
                }

                let refund_resp = resp.json::<StripeRefundMin>().await.map_err(|e| {
                    AppError::Internal(format!("Error parseando refund Stripe: {e}"))
                })?;
                Ok(refund_resp.id)
            }
            _ => Err(AppError::BadRequest(
                "El pago no está en un estado reembolsable".into(),
            )),
        }
    }

    /* [074A-24] Clasifica errores de Stripe API para dar feedback útil al usuario
     * sin exponer detalles internos (claves API, IDs internos de Stripe).
     * Stripe devuelve: { "error": { "type": "...", "message": "...", "code": "..." } }
     * [074A-27] Todos retornan BadRequest para que el mensaje llegue al cliente.
     * AppError::Internal oculta el mensaje — seguro pero inútil para el usuario. */
    fn classify_stripe_error(body: &str) -> AppError {
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(body);
        let (error_type, message) = match &parsed {
            Ok(json) => (
                json["error"]["type"].as_str().unwrap_or("unknown"),
                json["error"]["message"]
                    .as_str()
                    .unwrap_or("Error desconocido de Stripe"),
            ),
            Err(_) => return AppError::BadRequest("Error inesperado del servicio de pagos".into()),
        };

        match error_type {
            "authentication_error" => {
                AppError::BadRequest("Error de configuración de pagos — contacta soporte".into())
            }
            "invalid_request_error" | "card_error" => {
                AppError::BadRequest(format!("Error de pago: {message}"))
            }
            "rate_limit_error" => AppError::BadRequest(
                "Demasiadas solicitudes de pago, intenta de nuevo en unos segundos".into(),
            ),
            _ => AppError::BadRequest("Servicio de pagos no disponible temporalmente".into()),
        }
    }
}

/* Tipo mínimo para parsear la respuesta de Stripe PaymentIntent */
#[derive(Debug, serde::Deserialize)]
struct StripePaymentIntentMin {
    id: String,
    client_secret: String,
}

/* [044A-38 Fase 7] Tipo mínimo para parsear respuesta de Stripe Refund */
#[derive(Debug, serde::Deserialize)]
struct StripeRefundMin {
    id: String,
}
