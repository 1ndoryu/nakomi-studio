/* [084A-24] Servicio de suscripciones Stripe para hosting.
 * Crea Checkout Sessions en modo subscription usando price_data dinámico desde BD.
 * Maneja webhooks de invoice.paid y customer.subscription.* para sincronizar estado.
 * Gotcha: NO usa PaymentIntents directos — usa Stripe Subscriptions nativas.
 * [094A-9] Auditoría de seguridad: validación estricta de campos JSON en webhooks,
 * idempotency key en checkout, verificación de status antes de activar. */

use reqwest::Client;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{CreateNotification, NOTIF_HOSTING_CANCELLED, NOTIF_HOSTING_SUSPENDED};
use crate::repositories::{HostingRepository, NotificationRepository, ServerInfo};
use crate::services::coolify::{CoolifyConfig, CoolifyService};

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
    pub subscription_id: Uuid,
    pub plan: &'a str,
    pub amount_cents: i32,
    pub customer_email: &'a str,
    pub success_url: &'a str,
    pub cancel_url: &'a str,
}

pub struct HostingStripeService;

fn humanize_plan_name(plan: &str) -> &'static str {
    match plan.strip_prefix("normal-").unwrap_or(plan) {
        "basico" => "Basico",
        "pro" => "Pro",
        "ecommerce" => "E-commerce",
        _ => "Personalizado",
    }
}

fn hosting_product_copy(plan: &str) -> (String, String) {
    let plan_name = humanize_plan_name(plan);
    if plan.starts_with("normal-") {
        return (
            format!("Hosting {plan_name}"),
            format!("Plan {plan_name} de hosting administrado para sitios a medida y frontends"),
        );
    }

    (
        format!("Hosting WordPress {plan_name}"),
        format!("Plan {plan_name} de hosting WordPress administrado"),
    )
}

impl HostingStripeService {
    /// Crea una Stripe Checkout Session para suscripción de hosting.
    /// Retorna la URL a la que redirigir al cliente.
    pub async fn create_checkout_session(params: &CheckoutParams<'_>) -> Result<String, AppError> {
        if params.amount_cents <= 0 {
            return Err(AppError::Validation(
                "El checkout de hosting requiere un precio mensual mayor a 0".into(),
            ));
        }

        let (product_name, product_description) = hosting_product_copy(params.plan);

        let form = vec![
            ("mode", "subscription".to_string()),
            ("line_items[0][price_data][currency]", "usd".to_string()),
            (
                "line_items[0][price_data][unit_amount]",
                params.amount_cents.to_string(),
            ),
            (
                "line_items[0][price_data][product_data][name]",
                product_name,
            ),
            (
                "line_items[0][price_data][product_data][description]",
                product_description,
            ),
            (
                "line_items[0][price_data][recurring][interval]",
                "month".to_string(),
            ),
            ("line_items[0][quantity]", "1".to_string()),
            ("customer_email", params.customer_email.to_string()),
            ("success_url", params.success_url.to_string()),
            ("cancel_url", params.cancel_url.to_string()),
            ("metadata[resource_kind]", "hosting".to_string()),
            (
                "metadata[hosting_subscription_id]",
                params.subscription_id.to_string(),
            ),
            (
                "subscription_data[metadata][resource_kind]",
                "hosting".to_string(),
            ),
            (
                "subscription_data[metadata][hosting_subscription_id]",
                params.subscription_id.to_string(),
            ),
        ];

        let resp = params
            .http_client
            .post("https://api.stripe.com/v1/checkout/sessions")
            .basic_auth(params.stripe_key, Option::<&str>::None)
            /* [094A-9] Idempotency key: evita sesiones duplicadas si la request se reintenta */
            .header(
                "Idempotency-Key",
                format!("hosting-checkout-{}", params.subscription_id),
            )
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

        session
            .url
            .ok_or_else(|| AppError::Internal("Stripe no retornó URL de checkout".into()))
    }

    /// Procesa webhooks de Stripe relacionados a hosting subscriptions.
    /// Eventos: checkout.session.completed, invoice.paid, customer.subscription.deleted
    pub async fn handle_webhook(
        pool: &PgPool,
        http_client: &Client,
        coolify_config: Option<&CoolifyConfig>,
        event_type: &str,
        data: &serde_json::Value,
    ) -> Result<bool, AppError> {
        match event_type {
            "checkout.session.completed" => {
                Self::on_checkout_completed(pool, http_client, coolify_config, data).await
            }
            "invoice.paid" => Self::on_invoice_paid(pool, data).await,
            "customer.subscription.deleted" => {
                Self::on_subscription_deleted(pool, http_client, coolify_config, data).await
            }
            "invoice.payment_failed" => Self::on_payment_failed(pool, data).await,
            _ => Ok(false),
        }
    }

    /* Cliente completó el checkout — activar suscripción */
    #[allow(clippy::too_many_lines)]
    /* sentinel-disable-next-line funcion-larga-rs: activa hosting, intenta provisioning y deja auditoría consistente en un único punto transaccional de webhook. */
    async fn on_checkout_completed(
        pool: &PgPool,
        http_client: &Client,
        coolify_config: Option<&CoolifyConfig>,
        data: &serde_json::Value,
    ) -> Result<bool, AppError> {
        let mode = data["object"]["mode"].as_str().unwrap_or("");
        if mode != "subscription" {
            return Ok(false);
        }

        let sub_id_str = data["object"]["metadata"]["hosting_subscription_id"]
            .as_str()
            .or_else(|| {
                data["object"]["subscription_data"]["metadata"]["hosting_subscription_id"].as_str()
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

        /* [104A-42] Provisioning real en Coolify — no-fatal: si falla, el pago ya fue aceptado.
         * El admin puede provisionar manualmente desde el panel de Coolify.
         * service_name usa los 8 primeros chars del UUID para idempotencia.
         * [164A-16] Puerto SFTP generado con verificación de unicidad en BD. */
        if let Some(config) = coolify_config {
            let service_name = CoolifyService::service_name_for(&hosting_id);
            let sftp_port = match HostingRepository::find_available_sftp_port(pool).await {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!("No se pudo generar puerto SFTP para {hosting_id}: {e}");
                    return Ok(true);
                }
            };
            /* [114A-3] Obtener config del plan para límites dinámicos */
            let plan_config = match HostingRepository::get_plan_config(pool, &existing.plan).await {
                Ok(Some(cfg)) => cfg,
                Ok(None) => {
                    tracing::warn!(
                        "Plan config '{}' no encontrado para hosting {hosting_id}",
                        existing.plan
                    );
                    return Ok(true);
                }
                Err(e) => {
                    tracing::warn!("Error obteniendo plan config para {hosting_id}: {e}");
                    return Ok(true);
                }
            };
            match CoolifyService::provision_hosting(
                http_client,
                config,
                &service_name,
                sftp_port,
                &plan_config,
            )
            .await
            {
                Ok(result) => {
                    tracing::info!(
                        "Hosting {} provisionado en Coolify: uuid={}, domain={}, ip={}",
                        hosting_id,
                        result.service_uuid,
                        result.domain,
                        result.server_ip
                    );
                    if let Err(e) = HostingRepository::update_server_info(
                        pool,
                        hosting_id,
                        &ServerInfo {
                            coolify_site_name: &service_name,
                            server_uuid: &result.service_uuid,
                            server_ip: &result.server_ip,
                            sftp_user: &result.sftp_user,
                            sftp_password: &result.sftp_password,
                            sftp_port: result.sftp_port,
                        },
                    )
                    .await
                    {
                        tracing::warn!(
                            "Error guardando server_info para hosting {}: {e}",
                            hosting_id
                        );
                    }
                    if let Err(e) = HostingRepository::add_event(
                        pool,
                        hosting_id,
                        "coolify_provisioned",
                        Some(serde_json::json!({
                            "service_uuid": result.service_uuid,
                            "domain": result.domain,
                            "server_ip": result.server_ip,
                        })),
                    )
                    .await
                    {
                        tracing::warn!(
                            "Error registrando evento coolify_provisioned para {}: {e}",
                            hosting_id
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Coolify provisioning falló para hosting {} (error: {e}). Requiere setup manual.",
                        hosting_id
                    );
                    if let Err(ev_err) = HostingRepository::add_event(
                        pool,
                        hosting_id,
                        "coolify_provision_failed",
                        Some(serde_json::json!({"error": e.to_string()})),
                    )
                    .await
                    {
                        tracing::warn!(
                            "Error registrando evento coolify_provision_failed para {}: {ev_err}",
                            hosting_id
                        );
                    }
                }
            }
        } else {
            tracing::warn!(
                "Coolify no configurado — hosting {} activado sin provisioning automático.",
                hosting_id
            );
        }

        /* [094A-9] Log de errores en evento, no silenciar */
        if let Err(e) = HostingRepository::add_event(
            pool,
            hosting_id,
            "stripe_checkout_completed",
            Some(serde_json::json!({"stripe_subscription_id": stripe_sub_id})),
        )
        .await
        {
            tracing::warn!(
                "Error registrando evento stripe_checkout_completed para {hosting_id}: {e}"
            );
        }

        tracing::info!("Hosting {hosting_id} activado via Stripe checkout (sub: {stripe_sub_id})");
        Ok(true)
    }

    /* Factura pagada — renovación mensual exitosa */
    async fn on_invoice_paid(pool: &PgPool, data: &serde_json::Value) -> Result<bool, AppError> {
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
            pool,
            hosting.id,
            "invoice_paid",
            Some(serde_json::json!({
                "invoice_id": data["object"]["id"].as_str(),
                "amount_paid": data["object"]["amount_paid"].as_i64(),
            })),
        )
        .await
        {
            tracing::warn!(
                "Error registrando evento invoice_paid para {}: {e}",
                hosting.id
            );
        }

        tracing::info!("Hosting {} — invoice paid", hosting.id);
        Ok(true)
    }

    /* Suscripción cancelada en Stripe */
    async fn on_subscription_deleted(
        pool: &PgPool,
        http_client: &Client,
        coolify_config: Option<&CoolifyConfig>,
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

        /* [104A-42] Eliminar servicio Coolify al cancelar — no-fatal */
        if let (Some(config), Some(service_uuid)) = (coolify_config, &hosting.server_uuid) {
            if let Err(e) =
                CoolifyService::delete_service(http_client, config, service_uuid, false).await
            {
                tracing::warn!(
                    "Error eliminando servicio Coolify {} para hosting {}: {e}",
                    service_uuid,
                    hosting.id
                );
            } else {
                tracing::info!(
                    "Servicio Coolify {} eliminado por cancelación de hosting {}",
                    service_uuid,
                    hosting.id
                );
            }
        }

        /* [Notificación cancelación] Notificar al propietario del hosting si tiene cuenta */
        if let Some(user_id) = hosting.user_id {
            let notif = CreateNotification {
                user_id,
                notification_type: NOTIF_HOSTING_CANCELLED.to_string(),
                title: "Tu plan de hosting fue cancelado".to_string(),
                body: Some(
                    "Tu suscripción de hosting ha sido cancelada. Si crees que es un error, contáctanos.".to_string(),
                ),
                link: Some("/panel/hosting".to_string()),
                reference_type: Some("hosting_subscription".to_string()),
                reference_id: Some(hosting.id),
            };
            if let Err(e) = NotificationRepository::create(pool, &notif).await {
                tracing::warn!(
                    "Error creando notificación hosting_cancelled para user {user_id}: {e}"
                );
            }
        }

        /* [094A-9] Log de errores en evento, no silenciar */
        if let Err(e) =
            HostingRepository::add_event(pool, hosting.id, "stripe_subscription_cancelled", None)
                .await
        {
            tracing::warn!(
                "Error registrando evento stripe_subscription_cancelled para {}: {e}",
                hosting.id
            );
        }

        tracing::info!("Hosting {} cancelado via Stripe webhook", hosting.id);
        Ok(true)
    }

    /* Pago fallido — suspender hosting */
    async fn on_payment_failed(pool: &PgPool, data: &serde_json::Value) -> Result<bool, AppError> {
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

        /* [Notificación suspensión] Notificar al propietario si tiene cuenta */
        if let Some(user_id) = hosting.user_id {
            let notif = CreateNotification {
                user_id,
                notification_type: NOTIF_HOSTING_SUSPENDED.to_string(),
                title: "Tu hosting fue suspendido por pago fallido".to_string(),
                body: Some(
                    "No pudimos procesar tu pago. Tu hosting está suspendido. Actualiza tu método de pago para restaurarlo.".to_string(),
                ),
                link: Some("/panel/hosting".to_string()),
                reference_type: Some("hosting_subscription".to_string()),
                reference_id: Some(hosting.id),
            };
            if let Err(e) = NotificationRepository::create(pool, &notif).await {
                tracing::warn!(
                    "Error creando notificación hosting_suspended para user {user_id}: {e}"
                );
            }
        }

        /* [094A-9] Log de errores en evento, no silenciar */
        if let Err(e) = HostingRepository::add_event(
            pool,
            hosting.id,
            "payment_failed",
            Some(serde_json::json!({"invoice_id": data["object"]["id"].as_str()})),
        )
        .await
        {
            tracing::warn!(
                "Error registrando evento payment_failed para {}: {e}",
                hosting.id
            );
        }

        tracing::warn!("Hosting {} suspendido por pago fallido", hosting.id);
        Ok(true)
    }
}

/* ============================================================
TESTS — [094A-10] Validación de lógica Stripe hosting
============================================================ */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn humanize_plan_name_maps_known_slugs() {
        assert_eq!(humanize_plan_name("basico"), "Basico");
        assert_eq!(humanize_plan_name("pro"), "Pro");
        assert_eq!(humanize_plan_name("ecommerce"), "E-commerce");
        assert_eq!(humanize_plan_name("normal-pro"), "Pro");
        assert_eq!(humanize_plan_name("custom"), "Personalizado");
    }

    #[test]
    fn hosting_product_copy_distinguishes_normal_hosting() {
        let (normal_name, normal_desc) = hosting_product_copy("normal-basico");
        assert_eq!(normal_name, "Hosting Basico");
        assert!(normal_desc.contains("sitios a medida"));

        let (wordpress_name, wordpress_desc) = hosting_product_copy("basico");
        assert_eq!(wordpress_name, "Hosting WordPress Basico");
        assert!(wordpress_desc.contains("hosting WordPress"));
    }
}
