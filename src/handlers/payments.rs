/* sentinel-disable-file sqlx-query-sin-macro: payment handler usa runtime query para
 * UPDATE de estado tras webhook Stripe (tipo dinámico por contexto). */
/* [044A-38 Fase 3] Handlers de pagos: Stripe checkout, webhook, historial.
 * Webhook no requiere auth — se verifica con firma HMAC-SHA256. */

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{
    CreateNotification, InitiatePaymentRequest, PaymentIntentResponse, PaymentResponse, UserRole,
    NOTIF_CHAT_INVOICE_PAID, NOTIF_PAYMENT_RECEIVED,
};
use crate::repositories::{OrderRepository, PaymentRepository, UserRepository};
use crate::services::{AuditService, EmailService, HostingStripeService, PaymentService};
use crate::AppState;

/// Iniciar pago de una orden (crea `PaymentIntent` en Stripe)
#[utoipa::path(
    post,
    path = "/api/orders/{order_id}/pay",
    params(("order_id" = Uuid, Path, description = "ID de la orden")),
    request_body = InitiatePaymentRequest,
    responses(
        (status = 200, description = "PaymentIntent creado", body = PaymentIntentResponse),
        (status = 400, description = "Datos inválidos", body = crate::errors::ErrorResponse),
        (status = 401, description = "No autorizado", body = crate::errors::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "payments"
)]
pub async fn initiate_payment(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(order_id): Path<Uuid>,
    Json(req): Json<InitiatePaymentRequest>,
) -> Result<Json<PaymentIntentResponse>, AppError> {
    auth.require_role(&[UserRole::Client, UserRole::Admin])?;

    let stripe_key = state
        .stripe_secret_key
        .as_ref()
        .ok_or_else(|| AppError::Internal("Stripe no está configurado".into()))?;

    /* [064A-65] Admin puede pagar cualquier orden (testing/soporte).
     * Para clientes, el servicio verifica ownership por client_id. */
    let caller_id = if auth.role == UserRole::Admin {
        None
    } else {
        Some(auth.user_id)
    };

    /* [064A-59] Obtener email del usuario para pre-llenar en Stripe (receipt_email).
     * Así no se le pide email de nuevo en el checkout.
     * [074A-24] Log si falla la consulta en vez de silenciar con .ok(). */
    let user_email = match UserRepository::find_by_id(&state.pool, auth.user_id).await {
        Ok(Some(u)) => Some(u.email),
        Ok(None) => None,
        Err(e) => {
            tracing::warn!("No se pudo obtener email del usuario para receipt_email: {e}");
            None
        }
    };

    let result = PaymentService::initiate_payment(
        &state.pool,
        &state.http_client,
        stripe_key,
        order_id,
        caller_id,
        req.phase_number,
        user_email.as_deref(),
    )
    .await?;

    Ok(Json(result))
}

/// Webhook de Stripe — sin autenticación, verificado por firma HMAC
#[utoipa::path(
    post,
    path = "/api/webhooks/stripe",
    responses(
        (status = 200, description = "Webhook procesado"),
        (status = 400, description = "Firma inválida", body = crate::errors::ErrorResponse),
    ),
    tag = "payments"
)]
pub async fn stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    let webhook_secret = state
        .stripe_webhook_secret
        .as_ref()
        .ok_or_else(|| AppError::Internal("Stripe webhook secret no configurado".into()))?;

    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::BadRequest("Missing Stripe-Signature header".into()))?;

    PaymentService::verify_webhook_signature(&body, signature, webhook_secret)?;

    let event: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| AppError::BadRequest(format!("JSON inválido: {e}")))?;

    let event_type = event["type"]
        .as_str()
        .ok_or_else(|| AppError::BadRequest("Missing event type".into()))?;

    /* [064A-73] Deduplicación: si el event_id ya fue procesado, retornar 200 sin
     * reprocesar. Stripe reenvía webhooks si no obtiene 200, sin esto un pago
     * podría acreditarse doble. */
    let event_id = event["id"]
        .as_str()
        .ok_or_else(|| AppError::BadRequest("Missing event id".into()))?;

    let already_processed: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM stripe_processed_events WHERE event_id = $1)"
    )
    .bind(event_id)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(false);

    if already_processed {
        tracing::info!("Webhook duplicado ignorado: {event_id}");
        return Ok(StatusCode::OK);
    }

    PaymentService::handle_webhook(&state.pool, event_type, &event["data"]).await?;

    /* [104A-38] Notificar al cliente cuando su pago se procesa exitosamente.
     * Buscamos el payment por stripe_intent → order → client_id. */
    if event_type == "payment_intent.succeeded" {
        if let Some(pi_id) = event["data"]["object"]["id"].as_str() {
            if let Ok(Some(payment)) =
                PaymentRepository::find_by_stripe_intent(&state.pool, pi_id).await
            {
                if let Ok(Some(order)) =
                    OrderRepository::find_order_by_id(&state.pool, payment.order_id).await
                {
                    let amount_display = format!("${:.2}", f64::from(payment.amount_cents) / 100.0);
                    let _ = state.notification_hub.notify(CreateNotification {
                        user_id: order.client_id,
                        notification_type: NOTIF_PAYMENT_RECEIVED.to_string(),
                        title: format!("Pago recibido — Orden #{}", order.order_number),
                        body: Some(format!("Tu pago de {amount_display} fue procesado exitosamente")),
                        link: Some(format!("/panel/orders/{}", order.id)),
                        reference_type: Some("order".to_string()),
                        reference_id: Some(order.id),
                    }).await;
                }
            }
        }
    }

    /* [124A-INV] Detectar cuando una factura generada por IA en el chat fue pagada.
     * La factura tiene metadata: source=chat_invoice, session_id, client_email.
     * Al confirmar el pago: notificar admins via hub + email, y enviar email al
     * cliente invitándolo a registrarse con el email de pago para acceder al panel. */
    if event_type == "invoice.paid" {
        let meta = &event["data"]["object"]["metadata"];
        if meta["source"].as_str() == Some("chat_invoice") {
            let client_email = meta["client_email"]
                .as_str()
                .or_else(|| event["data"]["object"]["customer_email"].as_str())
                .unwrap_or_default()
                .to_string();
            let session_id_str = meta["session_id"].as_str().unwrap_or_default();
            let session_id = session_id_str.parse::<Uuid>().unwrap_or(Uuid::nil());
            let amount_cents = event["data"]["object"]["amount_paid"].as_i64().unwrap_or(0);
            #[allow(clippy::cast_precision_loss)]
            let amount_usd = amount_cents as f64 / 100.0;
            let site_url = std::env::var("SITE_URL")
                .unwrap_or_else(|_| "https://nakomi.studio".to_string());

            /* Notificar admins via notification hub */
            if let Ok(admin_ids) = UserRepository::admin_ids(&state.pool).await {
                if !admin_ids.is_empty() {
                    let _ = state
                        .notification_hub
                        .notify_many(
                            &admin_ids,
                            &CreateNotification {
                                user_id: Uuid::nil(),
                                notification_type: NOTIF_CHAT_INVOICE_PAID.to_string(),
                                title: format!(
                                    "Factura pagada: ${amount_usd:.2} — {client_email}"
                                ),
                                body: Some(
                                    "Una factura del chatbot fue pagada. El cliente puede registrarse."
                                        .to_string(),
                                ),
                                link: Some(format!("/panel/chat?session={session_id}")),
                                reference_type: None,
                                reference_id: None,
                            },
                        )
                        .await;
                }
            }

            /* Email a admins y al cliente (no fatal si falla) */
            if let Some(cfg) = &state.email_config {
                if let Ok(admin_emails) = UserRepository::admin_emails(&state.pool).await {
                    EmailService::send_chat_invoice_paid_admin(
                        cfg,
                        &admin_emails,
                        &client_email,
                        amount_usd,
                        session_id,
                        &site_url,
                    )
                    .await;
                }
                if !client_email.is_empty() {
                    /* Encoding mínimo para email en query param (@→%40) */
                    let encoded_email = client_email.replace('@', "%40").replace('+', "%2B");
                    let register_url = format!("{site_url}/registro?email={encoded_email}");
                    EmailService::send_chat_invoice_paid_client(
                        cfg,
                        &client_email,
                        amount_usd,
                        &site_url,
                        &register_url,
                    )
                    .await;
                }
            }

            tracing::info!(
                "Factura chat pagada: {client_email} ${amount_usd:.2} sesión {session_id}"
            );
        }
    }

    /* [084A-24] Hosting subscriptions: procesar eventos de checkout.session.completed,
     * invoice.paid, invoice.payment_failed, customer.subscription.deleted.
     * [104A-42] Se pasa http_client y coolify_config para provisioning automático.
     * HostingStripeService retorna bool indicando si procesó el evento. */
    HostingStripeService::handle_webhook(
        &state.pool,
        &state.http_client,
        state.coolify_config.as_ref(),
        event_type,
        &event["data"],
    ).await?;

    /* [064A-73] Audit: webhook procesado exitosamente */
    AuditService::log(
        &state.pool,
        "stripe_webhook",
        None,
        None,
        serde_json::json!({"event_id": event_id, "event_type": event_type}),
    )
    .await;

    /* Registrar evento como procesado */
    sqlx::query(
        "INSERT INTO stripe_processed_events (event_id, event_type) VALUES ($1, $2) ON CONFLICT DO NOTHING"
    )
    .bind(event_id)
    .bind(event_type)
    .execute(&state.pool)
    .await
    .map_err(AppError::Database)?;

    Ok(StatusCode::OK)
}

/// Historial de pagos de una orden
#[utoipa::path(
    get,
    path = "/api/orders/{order_id}/payments",
    params(("order_id" = Uuid, Path, description = "ID de la orden")),
    responses(
        (status = 200, description = "Historial de pagos", body = Vec<PaymentResponse>),
        (status = 401, description = "No autorizado", body = crate::errors::ErrorResponse),
        (status = 404, description = "Orden no encontrada", body = crate::errors::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "payments"
)]
pub async fn list_payments(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(order_id): Path<Uuid>,
) -> Result<Json<Vec<PaymentResponse>>, AppError> {
    /* Verificar acceso: dueño, asignado o admin */
    let order = OrderRepository::find_order_by_id(&state.pool, order_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Orden no encontrada".into()))?;

    /* [074A-50] Admin real siempre tiene acceso, effective_role solo afecta UI */
    if auth.role != UserRole::Admin {
        match auth.effective_role {
            UserRole::Admin => {}
            UserRole::Client => {
                if order.client_id != auth.user_id {
                    return Err(AppError::Forbidden("No tienes acceso a esta orden".into()));
                }
            }
            UserRole::Employee => {
                if order.assigned_employee_id != Some(auth.user_id) {
                    return Err(AppError::Forbidden("No tienes acceso a esta orden".into()));
                }
            }
        }
    }

    let payments = PaymentService::list_payments(&state.pool, order_id).await?;
    Ok(Json(payments))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/orders/:order_id/pay", post(initiate_payment))
        .route("/orders/:order_id/payments", get(list_payments))
        .route("/webhooks/stripe", post(stripe_webhook))
}
