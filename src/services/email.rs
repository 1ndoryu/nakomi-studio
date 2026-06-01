/* [154A-15c] Servicio de email con SMTP (lettre).
 * Configuración vía env vars: SMTP_HOST o GLORY_SMTP_HOST, SMTP_PORT o GLORY_SMTP_PORT,
 * SMTP_USER o GLORY_SMTP_USER, SMTP_PASS o GLORY_SMTP_PASSWORD, SMTP_FROM.
 * Se aceptan ambos prefijos para compatibilidad con .env local (GLORY_SMTP_*)
 * y posibles configuraciones legacy (SMTP_*).
 * Non-fatal: si SMTP no está configurado, los emails se loguean y se omiten. */

use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use sqlx::PgPool;

use crate::repositories::EmailLogRepository;

#[derive(Clone)]
pub struct EmailConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub from_name: String,
    pub from_email: String,
    pub bcc_email: Option<String>,
}

impl EmailConfig {
    /// Intenta crear config desde env vars. Retorna None si faltan variables.
    /// Acepta `SMTP_*` y `GLORY_SMTP_*` como nombres de variables (compat local/prod).
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let host = std::env::var("SMTP_HOST")
            .or_else(|_| std::env::var("GLORY_SMTP_HOST"))
            .ok()?;
        let user = std::env::var("SMTP_USER")
            .or_else(|_| std::env::var("GLORY_SMTP_USER"))
            .ok()?;
        let pass = std::env::var("SMTP_PASS")
            .or_else(|_| std::env::var("GLORY_SMTP_PASSWORD"))
            .ok()?;
        let from_email = std::env::var("SMTP_FROM").unwrap_or_else(|_| user.clone());
        let from_name =
            std::env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "Nakomi Studio".to_string());
        let port = std::env::var("SMTP_PORT")
            .or_else(|_| std::env::var("GLORY_SMTP_PORT"))
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(587);
        let bcc_email = std::env::var("SMTP_BCC").ok();

        Some(Self {
            host,
            port,
            user,
            pass,
            from_name,
            from_email,
            bcc_email,
        })
    }
}

pub struct EmailService;

impl EmailService {
    /// Envía un email HTML. Non-fatal: loguea error si falla.
    pub async fn send(
        config: &EmailConfig,
        to_email: &str,
        subject: &str,
        html_body: &str,
    ) -> Result<(), String> {
        let from = format!("{} <{}>", config.from_name, config.from_email);
        /* [311A-1] BCC configurable via SMTP_BCC para que el admin reciba copia
         * de TODO correo enviado desde la plataforma. Non-fatal: si la dirección
         * es inválida o no está configurada, el email se envía sin BCC. */
        let mut builder = Message::builder()
            .from(from.parse().map_err(|e| format!("From inválido: {e}"))?)
            .to(to_email.parse().map_err(|e| format!("To inválido: {e}"))?)
            .subject(subject)
            .header(ContentType::TEXT_HTML);
        if let Some(ref bcc_email) = config.bcc_email {
            if let Ok(bcc_addr) = bcc_email.parse() {
                builder = builder.bcc(bcc_addr);
            }
        }
        let email = builder
            .body(html_body.to_string())
            .map_err(|e| format!("Error construyendo email: {e}"))?;

        let creds = Credentials::new(config.user.clone(), config.pass.clone());

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)
            .map_err(|e| format!("Error conectando SMTP: {e}"))?
            .port(config.port)
            .credentials(creds)
            .build();

        mailer
            .send(email)
            .await
            .map_err(|e| format!("Error enviando email: {e}"))?;

        Ok(())
    }

    /// Genera y envía email de confirmación de pedido.
    pub async fn send_order_confirmation(
        config: &EmailConfig,
        pool: &PgPool,
        to_email: &str,
        client_name: &str,
        order_number: i32,
        service_title: &str,
        plan_name: &str,
        price_display: &str,
    ) {
        let subject = format!("¡Pedido #{order_number} recibido! — Nakomi Studio");

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="es">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"></head>
<body style="margin:0;padding:0;background:#f8f8f8;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
<div style="max-width:600px;margin:24px auto;background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.06);">
  <div style="background:#1a1a1a;padding:32px 24px;text-align:center;">
    <h1 style="margin:0;color:#c9a84c;font-size:24px;font-weight:600;">Nakomi Studio</h1>
  </div>
  <div style="padding:32px 24px;">
    <h2 style="margin:0 0 8px;color:#1a1a1a;font-size:20px;">¡Hola, {client_name}!</h2>
    <p style="color:#555;font-size:15px;line-height:1.6;margin:0 0 24px;">
      Tu pedido <strong>#{order_number}</strong> ha sido recibido exitosamente.
      Nuestro equipo lo revisará y será atendido dentro de las próximas <strong>48 horas</strong>.
    </p>
    <div style="background:#f8f8f8;border-radius:8px;padding:20px;margin-bottom:24px;">
      <table style="width:100%;border-collapse:collapse;font-size:14px;color:#333;">
        <tr><td style="padding:6px 0;color:#888;">Servicio</td><td style="padding:6px 0;font-weight:500;text-align:right;">{service_title}</td></tr>
        <tr><td style="padding:6px 0;color:#888;">Plan</td><td style="padding:6px 0;font-weight:500;text-align:right;">{plan_name}</td></tr>
        <tr><td style="padding:6px 0;color:#888;">Precio</td><td style="padding:6px 0;font-weight:600;text-align:right;color:#c9a84c;">{price_display}</td></tr>
      </table>
    </div>
    <p style="color:#555;font-size:14px;line-height:1.6;margin:0 0 24px;">
      Puedes seguir el progreso de tu pedido en tiempo real desde tu panel.
    </p>
    <a href="https://nakomi.studio/panel" style="display:inline-block;background:#c9a84c;color:#fff;text-decoration:none;padding:12px 28px;border-radius:8px;font-weight:600;font-size:14px;">
      Ver mi pedido
    </a>
  </div>
  <div style="padding:16px 24px;border-top:1px solid #eee;text-align:center;">
    <p style="margin:0;color:#999;font-size:12px;">© 2026 Nakomi Studio · Este email fue enviado porque realizaste un pedido.</p>
  </div>
</div>
</body></html>"#,
            client_name = html_escape(client_name),
            order_number = order_number,
            service_title = html_escape(service_title),
            plan_name = html_escape(plan_name),
            price_display = html_escape(price_display),
        );

        /* [311A-1] Logging del envío en email_logs para trazabilidad. */
        let result = Self::send(config, to_email, &subject, &html).await;
        let status = if result.is_ok() { "sent" } else { "failed" };
        let error_msg = result.as_ref().err().map(String::as_str);

        if let Err(log_err) = EmailLogRepository::insert(
            pool, to_email, &subject, "order_confirmation",
            Some("order"), None, status, error_msg,
        ).await {
            tracing::warn!("Error registrando email_log: {log_err}");
        }

        if let Err(e) = result {
            tracing::error!("Error enviando email de confirmación orden #{order_number}: {e}");
        }
    }
}

pub(crate) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub(crate) fn recipient_label(display_name: Option<&str>, email: &str) -> String {
    let raw = display_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(email);
    html_escape(raw)
}

pub(crate) fn format_usd_cents(amount_cents: i32) -> String {
    format!("${:.2} USD", f64::from(amount_cents) / 100.0)
}

/* [311A-1] Envía email a todos los admins activos notificando una nueva orden.
 * Se dispara desde create_order() en handlers/orders.rs.
 * Non-fatal: si falla, solo se loguea. */
impl EmailService {
    #[allow(clippy::too_many_arguments)]
    pub async fn send_new_order_admin(
        config: &EmailConfig,
        pool: &PgPool,
        admin_emails: &[String],
        client_email: &str,
        client_name: &str,
        order_number: i32,
        service_title: &str,
        plan_name: &str,
        price_display: &str,
        payment_mode: &str,
        order_id: uuid::Uuid,
        site_url: &str,
    ) {
        let subject = format!("🆕 Nueva orden #{order_number} — {client_name} — Nakomi Studio");
        let escaped_client = html_escape(client_name);
        let escaped_email = html_escape(client_email);
        let escaped_service = html_escape(service_title);
        let escaped_plan = html_escape(plan_name);
        let panel_link = format!("{site_url}/panel?seccion=ordenes&id={order_id}");

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="es">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"></head>
<body style="margin:0;padding:0;background:#f8f8f8;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
<div style="max-width:600px;margin:24px auto;background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.06);">
  <div style="background:#1a1a1a;padding:24px;text-align:center;">
    <h1 style="margin:0;color:#c9a84c;font-size:22px;font-weight:600;">🆕 Nueva Orden</h1>
  </div>
  <div style="padding:32px 24px;">
    <p style="color:#333;font-size:15px;line-height:1.6;margin:0 0 20px;">
      Se ha creado un nuevo pedido en Nakomi Studio.
    </p>
    <table style="width:100%;border-collapse:collapse;font-size:14px;color:#333;">
      <tr><td style="padding:6px 0;color:#888;">Pedido</td><td style="padding:6px 0;font-weight:500;text-align:right;">#{order_number}</td></tr>
      <tr><td style="padding:6px 0;color:#888;">Cliente</td><td style="padding:6px 0;font-weight:500;text-align:right;">{escaped_client} ({escaped_email})</td></tr>
      <tr><td style="padding:6px 0;color:#888;">Servicio</td><td style="padding:6px 0;font-weight:500;text-align:right;">{escaped_service}</td></tr>
      <tr><td style="padding:6px 0;color:#888;">Plan</td><td style="padding:6px 0;font-weight:500;text-align:right;">{escaped_plan}</td></tr>
      <tr><td style="padding:6px 0;color:#888;">Modalidad</td><td style="padding:6px 0;font-weight:500;text-align:right;">{payment_mode}</td></tr>
      <tr><td style="padding:6px 0;color:#888;">Precio</td><td style="padding:6px 0;font-weight:600;text-align:right;color:#c9a84c;">{price_display}</td></tr>
    </table>
    <a href="{panel_link}" style="display:inline-block;margin-top:24px;background:#c9a84c;color:#fff;text-decoration:none;padding:12px 28px;border-radius:8px;font-weight:600;font-size:14px;">
      Revisar pedido
    </a>
  </div>
  <div style="padding:16px 24px;border-top:1px solid #eee;text-align:center;">
    <p style="margin:0;color:#999;font-size:12px;">Nakomi Studio · Notificación automática de nuevo pedido</p>
  </div>
</div>
</body></html>"#,
        );

        for email in admin_emails {
            /* [311A-1] Logging individual por admin para trazabilidad. */
            let result = Self::send(config, email, &subject, &html).await;
            let status = if result.is_ok() { "sent" } else { "failed" };
            let error_msg = result.as_ref().err().map(String::as_str);

            if let Err(log_err) = EmailLogRepository::insert(
                pool, email, &subject, "new_order_admin",
                Some("order"), Some(order_id), status, error_msg,
            ).await {
                tracing::warn!("Error registrando email_log: {log_err}");
            }

            if let Err(e) = result {
                tracing::error!("Error enviando email nueva orden admin a {email}: {e}");
            }
        }
        if !admin_emails.is_empty() {
            tracing::info!("Email nueva orden #{order_number} enviado a {} admins", admin_emails.len());
        }
    }

    /* [114A-8] Envía email de escalación a todos los admins activos.
     * Se dispara cuando la IA detecta que un visitante necesita asistencia humana.
     * Non-fatal: si SMTP no está configurado o falla, solo se loguea. */
    pub async fn send_escalation_emails(
        config: &EmailConfig,
        pool: &PgPool,
        admin_emails: &[String],
        visitor_name: &str,
        session_id: uuid::Uuid,
        site_url: &str,
    ) {
        let subject = format!("⚠ Escalación: {visitor_name} necesita ayuda — Nakomi Studio");
        let panel_link = format!("{site_url}/panel/chat?session={session_id}");
        let escaped_name = html_escape(visitor_name);

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="es">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"></head>
<body style="margin:0;padding:0;background:#f8f8f8;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
<div style="max-width:600px;margin:24px auto;background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.06);">
  <div style="background:#b91c1c;padding:24px;text-align:center;">
    <h1 style="margin:0;color:#fff;font-size:20px;font-weight:600;">⚠ Escalación de Chat</h1>
  </div>
  <div style="padding:32px 24px;">
    <p style="color:#333;font-size:15px;line-height:1.6;margin:0 0 16px;">
      La IA detectó que <strong>{escaped_name}</strong> necesita asistencia humana.
    </p>
    <p style="color:#555;font-size:14px;line-height:1.6;margin:0 0 24px;">
      Por favor, revisa la sesión de chat lo antes posible para atender al visitante.
    </p>
    <a href="{panel_link}" style="display:inline-block;background:#c9a84c;color:#fff;text-decoration:none;padding:12px 28px;border-radius:8px;font-weight:600;font-size:14px;">
      Abrir sesión de chat
    </a>
  </div>
  <div style="padding:16px 24px;border-top:1px solid #eee;text-align:center;">
    <p style="margin:0;color:#999;font-size:12px;">Nakomi Studio · Notificación automática de escalación</p>
  </div>
</div>
</body></html>"#,
        );

        for email in admin_emails {
            /* [311A-1] Logging individual por admin para trazabilidad. */
            let result = Self::send(config, email, &subject, &html).await;
            let status = if result.is_ok() { "sent" } else { "failed" };
            let error_msg = result.as_ref().err().map(String::as_str);

            if let Err(log_err) = EmailLogRepository::insert(
                pool, email, &subject, "escalation",
                Some("chat_session"), Some(session_id), status, error_msg,
            ).await {
                tracing::warn!("Error registrando email_log: {log_err}");
            }

            if let Err(e) = result {
                tracing::error!("Error enviando email escalación a {email}: {e}");
            }
        }
        if !admin_emails.is_empty() {
            tracing::info!(
                "Email de escalación enviado a {} admins para sesión {session_id}",
                admin_emails.len()
            );
        }
    }

    /* [311A-1] Email a todos los admins activos notificando que un pago de orden fue recibido.
     * Se dispara desde stripe_webhook() en handlers/payments.rs.
     * Non-fatal: si falla, solo se loguea. */
    #[allow(clippy::too_many_arguments)]
    pub async fn send_payment_received_admin(
        config: &EmailConfig,
        pool: &PgPool,
        admin_emails: &[String],
        client_email: &str,
        client_name: &str,
        order_number: i32,
        amount_display: &str,
        order_id: uuid::Uuid,
        site_url: &str,
    ) {
        let subject = format!("💰 Pago recibido — Orden #{order_number} — Nakomi Studio");
        let escaped_client = html_escape(client_name);
        let escaped_email = html_escape(client_email);
        let panel_link = format!("{site_url}/panel/orders/{order_id}");

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="es">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"></head>
<body style="margin:0;padding:0;background:#f8f8f8;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
<div style="max-width:600px;margin:24px auto;background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.06);">
  <div style="background:#166534;padding:24px;text-align:center;">
    <h1 style="margin:0;color:#fff;font-size:20px;font-weight:600;">💰 Pago recibido</h1>
  </div>
  <div style="padding:32px 24px;">
    <p style="color:#333;font-size:15px;line-height:1.6;margin:0 0 20px;">
      El cliente <strong>{escaped_client}</strong> ({escaped_email}) realizó un pago.
    </p>
    <table style="width:100%;border-collapse:collapse;font-size:14px;color:#333;">
      <tr><td style="padding:6px 0;color:#888;">Pedido</td><td style="padding:6px 0;font-weight:500;text-align:right;">#{order_number}</td></tr>
      <tr><td style="padding:6px 0;color:#888;">Monto</td><td style="padding:6px 0;font-weight:600;text-align:right;color:#166534;">{amount_display}</td></tr>
    </table>
    <a href="{panel_link}" style="display:inline-block;margin-top:24px;background:#c9a84c;color:#fff;text-decoration:none;padding:12px 28px;border-radius:8px;font-weight:600;font-size:14px;">
      Ver orden
    </a>
  </div>
  <div style="padding:16px 24px;border-top:1px solid #eee;text-align:center;">
    <p style="margin:0;color:#999;font-size:12px;">Nakomi Studio · Notificación automática de pago</p>
  </div>
</div>
</body></html>"#,
        );

        for email in admin_emails {
            /* [311A-1] Logging individual por admin para trazabilidad. */
            let result = Self::send(config, email, &subject, &html).await;
            let status = if result.is_ok() { "sent" } else { "failed" };
            let error_msg = result.as_ref().err().map(String::as_str);

            if let Err(log_err) = EmailLogRepository::insert(
                pool, email, &subject, "payment_received_admin",
                Some("order"), Some(order_id), status, error_msg,
            ).await {
                tracing::warn!("Error registrando email_log: {log_err}");
            }

            if let Err(e) = result {
                tracing::error!("Error enviando email pago recibido admin a {email}: {e}");
            }
        }
        if !admin_emails.is_empty() {
            tracing::info!("Email pago recibido orden #{order_number} enviado a {} admins", admin_emails.len());
        }
    }

    /* [124A-INV] Email al cliente notificando que su factura fue pagada y
     * que puede registrarse con el email de pago para acceder al panel. */
    pub async fn send_chat_invoice_paid_client(
        config: &EmailConfig,
        pool: &PgPool,
        client_email: &str,
        amount_usd: f64,
        site_url: &str,
        register_url: &str,
    ) {
        let subject = "Tu pago fue recibido — Nakomi Studio".to_string();
        let escaped_email = html_escape(client_email);

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="es">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"></head>
<body style="margin:0;padding:0;background:#f8f8f8;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
<div style="max-width:600px;margin:24px auto;background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.06);">
  <div style="background:#c9a84c;padding:24px;text-align:center;">
    <h1 style="margin:0;color:#fff;font-size:20px;font-weight:600;">✓ Pago recibido</h1>
  </div>
  <div style="padding:32px 24px;">
    <p style="color:#333;font-size:15px;line-height:1.6;margin:0 0 16px;">
      ¡Hola! Tu pago de <strong>${amount_usd:.2} USD</strong> fue procesado exitosamente.
    </p>
    <p style="color:#555;font-size:14px;line-height:1.6;margin:0 0 16px;">
      Para hacer seguimiento de tu proyecto y comunicarte con nuestro equipo, crea tu cuenta
      usando el correo con el que realizaste el pago: <strong>{escaped_email}</strong>
    </p>
    <a href="{register_url}" style="display:inline-block;background:#c9a84c;color:#fff;text-decoration:none;padding:12px 28px;border-radius:8px;font-weight:600;font-size:14px;">
      Crear mi cuenta
    </a>
    <p style="color:#999;font-size:12px;margin-top:20px;">
      Si ya tienes cuenta con ese correo, simplemente inicia sesión en
      <a href="{site_url}/panel" style="color:#c9a84c;">{site_url}/panel</a>
    </p>
  </div>
  <div style="padding:16px 24px;border-top:1px solid #eee;text-align:center;">
    <p style="margin:0;color:#999;font-size:12px;">Nakomi Studio · Notificación automática de pago</p>
  </div>
</div>
</body></html>"#,
        );

        /* [311A-1] Logging del envío en email_logs para trazabilidad. */
        let result = Self::send(config, client_email, &subject, &html).await;
        let status = if result.is_ok() { "sent" } else { "failed" };
        let error_msg = result.as_ref().err().map(String::as_str);

        if let Err(log_err) = EmailLogRepository::insert(
            pool, client_email, &subject, "chat_invoice_paid_client",
            Some("chat_invoice"), None, status, error_msg,
        ).await {
            tracing::warn!("Error registrando email_log: {log_err}");
        }

        if let Err(e) = result {
            tracing::error!("Error enviando email pago factura chat a {client_email}: {e}");
        } else {
            tracing::info!("Email pago chat invoice enviado a {client_email}");
        }
    }

    /* [124A-INV] Email a admins notificando que una factura de chat fue pagada. */
    pub async fn send_chat_invoice_paid_admin(
        config: &EmailConfig,
        pool: &PgPool,
        admin_emails: &[String],
        client_email: &str,
        amount_usd: f64,
        session_id: uuid::Uuid,
        site_url: &str,
    ) {
        let subject = format!("Factura pagada: {client_email} — Nakomi Studio");
        let panel_link = format!("{site_url}/panel/chat?session={session_id}");
        let escaped_email = html_escape(client_email);

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="es">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"></head>
<body style="margin:0;padding:0;background:#f8f8f8;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
<div style="max-width:600px;margin:24px auto;background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.06);">
  <div style="background:#166534;padding:24px;text-align:center;">
    <h1 style="margin:0;color:#fff;font-size:20px;font-weight:600;">💰 Pago recibido via chat</h1>
  </div>
  <div style="padding:32px 24px;">
    <p style="color:#333;font-size:15px;line-height:1.6;margin:0 0 16px;">
      El cliente <strong>{escaped_email}</strong> pagó <strong>${amount_usd:.2} USD</strong> via factura de chat.
    </p>
    <a href="{panel_link}" style="display:inline-block;background:#c9a84c;color:#fff;text-decoration:none;padding:12px 28px;border-radius:8px;font-weight:600;font-size:14px;">
      Ver sesión de chat
    </a>
  </div>
  <div style="padding:16px 24px;border-top:1px solid #eee;text-align:center;">
    <p style="margin:0;color:#999;font-size:12px;">Nakomi Studio · Notificación automática de pago</p>
  </div>
</div>
</body></html>"#,
        );

        for email in admin_emails {
            /* [311A-1] Logging individual por admin para trazabilidad. */
            let result = Self::send(config, email, &subject, &html).await;
            let status = if result.is_ok() { "sent" } else { "failed" };
            let error_msg = result.as_ref().err().map(String::as_str);

            if let Err(log_err) = EmailLogRepository::insert(
                pool, email, &subject, "chat_invoice_paid_admin",
                Some("chat_session"), Some(session_id), status, error_msg,
            ).await {
                tracing::warn!("Error registrando email_log: {log_err}");
            }

            if let Err(e) = result {
                tracing::error!("Error enviando email pago chat admin a {email}: {e}");
            }
        }
        if !admin_emails.is_empty() {
            tracing::info!(
                "Email pago chat invoice enviado a {} admins para sesión {session_id}",
                admin_emails.len()
            );
        }
    }

    pub async fn send_vps_pending_approval(
        config: &EmailConfig,
        pool: &PgPool,
        admin_emails: &[String],
        client_email: &str,
        tier_name: &str,
        monthly_price_cents: i32,
    ) {
        let subject = format!("VPS pendiente de aprobación: {tier_name} — Nakomi Studio");
        let escaped_email = html_escape(client_email);
        let escaped_tier = html_escape(tier_name);
        let amount_display = format_usd_cents(monthly_price_cents);

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="es">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"></head>
<body style="margin:0;padding:0;background:#f8f8f8;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
<div style="max-width:600px;margin:24px auto;background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.06);">
  <div style="background:#92400e;padding:24px;text-align:center;">
    <h1 style="margin:0;color:#fff;font-size:20px;font-weight:600;">VPS pendiente de aprobación</h1>
  </div>
  <div style="padding:32px 24px;">
    <p style="color:#333;font-size:15px;line-height:1.6;margin:0 0 16px;">
      El cliente <strong>{escaped_email}</strong> pagó un <strong>{escaped_tier}</strong>.
    </p>
    <p style="color:#555;font-size:14px;line-height:1.6;margin:0 0 16px;">
      Importe mensual: <strong>{amount_display}</strong>. La suscripción quedó en espera de aprobación manual.
    </p>
    <p style="color:#555;font-size:14px;line-height:1.6;margin:0;">
      Revisa el panel de hosting para aprobar o rechazar la provisión.
    </p>
  </div>
</div>
</body></html>"#,
        );

        for email in admin_emails {
            /* [311A-1] Logging individual por admin para trazabilidad. */
            let result = Self::send(config, email, &subject, &html).await;
            let status = if result.is_ok() { "sent" } else { "failed" };
            let error_msg = result.as_ref().err().map(String::as_str);

            if let Err(log_err) = EmailLogRepository::insert(
                pool, email, &subject, "vps_pending_approval",
                Some("vps"), None, status, error_msg,
            ).await {
                tracing::warn!("Error registrando email_log: {log_err}");
            }

            if let Err(error) = result {
                tracing::error!("Error enviando email VPS pendiente a {email}: {error}");
            }
        }
    }

    pub async fn send_vps_approved(
        config: &EmailConfig,
        pool: &PgPool,
        client_email: &str,
        tier_name: &str,
        public_ip: Option<&str>,
        username: &str,
        password: &str,
    ) {
        let subject = format!("Tu {tier_name} ya está activo — Nakomi Studio");
        let escaped_tier = html_escape(tier_name);
        let escaped_username = html_escape(username);
        let escaped_password = html_escape(password);
        let ip_block = public_ip.map_or_else(
          String::new,
            |ip| format!(
                "<tr><td style=\"padding:6px 0;color:#888;\">IP pública</td><td style=\"padding:6px 0;font-weight:500;text-align:right;\">{}</td></tr>",
                html_escape(ip)
            ),
        );

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="es">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"></head>
<body style="margin:0;padding:0;background:#f8f8f8;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
<div style="max-width:600px;margin:24px auto;background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.06);">
  <div style="background:#166534;padding:24px;text-align:center;">
    <h1 style="margin:0;color:#fff;font-size:20px;font-weight:600;">VPS activo</h1>
  </div>
  <div style="padding:32px 24px;">
    <p style="color:#333;font-size:15px;line-height:1.6;margin:0 0 20px;">
      Tu <strong>{escaped_tier}</strong> ya fue provisionado y está listo para usar.
    </p>
    <table style="width:100%;border-collapse:collapse;font-size:14px;color:#333;background:#f8f8f8;border-radius:8px;padding:20px;">
      {ip_block}
      <tr><td style="padding:6px 0;color:#888;">Usuario</td><td style="padding:6px 0;font-weight:500;text-align:right;">{escaped_username}</td></tr>
      <tr><td style="padding:6px 0;color:#888;">Contraseña inicial</td><td style="padding:6px 0;font-weight:500;text-align:right;">{escaped_password}</td></tr>
    </table>
    <p style="color:#555;font-size:13px;line-height:1.6;margin:20px 0 0;">
      Cambia la contraseña en tu primera conexión y guarda estas credenciales en un gestor seguro.
    </p>
  </div>
</div>
</body></html>"#,
        );

        /* [311A-1] Logging del envío en email_logs para trazabilidad. */
        let result = Self::send(config, client_email, &subject, &html).await;
        let status = if result.is_ok() { "sent" } else { "failed" };
        let error_msg = result.as_ref().err().map(String::as_str);

        if let Err(log_err) = EmailLogRepository::insert(
            pool, client_email, &subject, "vps_approved",
            Some("vps"), None, status, error_msg,
        ).await {
            tracing::warn!("Error registrando email_log: {log_err}");
        }

        if let Err(error) = result {
            tracing::error!("Error enviando email VPS aprobado a {client_email}: {error}");
        }
    }

    pub async fn send_vps_rejected(
        config: &EmailConfig,
        pool: &PgPool,
        client_email: &str,
        tier_name: &str,
        reason: &str,
    ) {
        let subject = format!("Tu solicitud de {tier_name} fue rechazada — Nakomi Studio");
        let escaped_tier = html_escape(tier_name);
        let escaped_reason = html_escape(reason);

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="es">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"></head>
<body style="margin:0;padding:0;background:#f8f8f8;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
<div style="max-width:600px;margin:24px auto;background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.06);">
  <div style="background:#991b1b;padding:24px;text-align:center;">
    <h1 style="margin:0;color:#fff;font-size:20px;font-weight:600;">Solicitud rechazada</h1>
  </div>
  <div style="padding:32px 24px;">
    <p style="color:#333;font-size:15px;line-height:1.6;margin:0 0 16px;">
      Revisamos tu solicitud de <strong>{escaped_tier}</strong> y no pudimos aprobarla en este momento.
    </p>
    <p style="color:#555;font-size:14px;line-height:1.6;margin:0;">
      Motivo: <strong>{escaped_reason}</strong>
    </p>
  </div>
</div>
</body></html>"#,
        );

        /* [311A-1] Logging del envío en email_logs para trazabilidad. */
        let result = Self::send(config, client_email, &subject, &html).await;
        let status = if result.is_ok() { "sent" } else { "failed" };
        let error_msg = result.as_ref().err().map(String::as_str);

        if let Err(log_err) = EmailLogRepository::insert(
            pool, client_email, &subject, "vps_rejected",
            Some("vps"), None, status, error_msg,
        ).await {
            tracing::warn!("Error registrando email_log: {log_err}");
        }

        if let Err(error) = result {
            tracing::error!("Error enviando email VPS rechazado a {client_email}: {error}");
        }
    }

    /* [205A-2] Notificación informativa al correo nuevo tras cambiar el email desde perfil.
     * No verifica ownership; solo confirma que el cambio ya se aplicó y deja rastro en la bandeja. */
    pub async fn send_profile_email_changed_new_address(
        config: &EmailConfig,
        pool: &PgPool,
        new_email: &str,
        display_name: Option<&str>,
        old_email: &str,
    ) {
        let subject = "Tu correo de acceso fue actualizado — Nakomi Studio";
        let recipient = recipient_label(display_name, new_email);
        let escaped_old = html_escape(old_email);
        let escaped_new = html_escape(new_email);

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="es">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"></head>
<body style="margin:0;padding:0;background:#f8f8f8;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
<div style="max-width:600px;margin:24px auto;background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.06);">
  <div style="background:#1a1a1a;padding:24px;text-align:center;">
    <h1 style="margin:0;color:#fff;font-size:20px;font-weight:600;">Correo actualizado</h1>
  </div>
  <div style="padding:32px 24px;">
    <p style="color:#333;font-size:15px;line-height:1.6;margin:0 0 16px;">Hola, <strong>{recipient}</strong>.</p>
    <p style="color:#555;font-size:14px;line-height:1.6;margin:0 0 16px;">
      Tu cuenta cambió el correo de acceso de <strong>{escaped_old}</strong> a <strong>{escaped_new}</strong>.
    </p>
    <p style="color:#555;font-size:14px;line-height:1.6;margin:0;">
      Desde ahora puedes iniciar sesión con este correo. Este cambio no requirió verificación por email.
    </p>
  </div>
</div>
</body></html>"#,
        );

        /* [311A-1] Logging del envío en email_logs para trazabilidad. */
        let result = Self::send(config, new_email, subject, &html).await;
        let status = if result.is_ok() { "sent" } else { "failed" };
        let error_msg = result.as_ref().err().map(String::as_str);

        if let Err(log_err) = EmailLogRepository::insert(
            pool, new_email, subject, "profile_email_changed_new",
            Some("user"), None, status, error_msg,
        ).await {
            tracing::warn!("Error registrando email_log: {log_err}");
        }

        match result {
            Ok(()) => {
                tracing::info!("Email de confirmación de cambio de correo enviado a {new_email}");
            }
            Err(error) => {
                tracing::error!("Error enviando email de cambio de correo a {new_email}: {error}");
            }
        }
    }

    /* [205A-2] Notificación defensiva al correo anterior tras un cambio de email.
     * Sirve para alertar al usuario si no reconoce la modificación. */
    pub async fn send_profile_email_changed_old_address(
        config: &EmailConfig,
        pool: &PgPool,
        old_email: &str,
        display_name: Option<&str>,
        new_email: &str,
    ) {
        let subject = "Tu correo de acceso fue reemplazado — Nakomi Studio";
        let recipient = recipient_label(display_name, old_email);
        let escaped_old = html_escape(old_email);
        let escaped_new = html_escape(new_email);

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="es">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"></head>
<body style="margin:0;padding:0;background:#f8f8f8;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
<div style="max-width:600px;margin:24px auto;background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.06);">
  <div style="background:#991b1b;padding:24px;text-align:center;">
    <h1 style="margin:0;color:#fff;font-size:20px;font-weight:600;">Cambio de correo detectado</h1>
  </div>
  <div style="padding:32px 24px;">
    <p style="color:#333;font-size:15px;line-height:1.6;margin:0 0 16px;">Hola, <strong>{recipient}</strong>.</p>
    <p style="color:#555;font-size:14px;line-height:1.6;margin:0 0 16px;">
      Tu cuenta dejó de usar <strong>{escaped_old}</strong> y ahora usa <strong>{escaped_new}</strong> para iniciar sesión.
    </p>
    <p style="color:#555;font-size:14px;line-height:1.6;margin:0;">
      Si no reconoces este cambio, responde a este correo o contacta a soporte de inmediato.
    </p>
  </div>
</div>
</body></html>"#,
        );

        /* [311A-1] Logging del envío en email_logs para trazabilidad. */
        let result = Self::send(config, old_email, subject, &html).await;
        let status = if result.is_ok() { "sent" } else { "failed" };
        let error_msg = result.as_ref().err().map(String::as_str);

        if let Err(log_err) = EmailLogRepository::insert(
            pool, old_email, subject, "profile_email_changed_old",
            Some("user"), None, status, error_msg,
        ).await {
            tracing::warn!("Error registrando email_log: {log_err}");
        }

        match result {
            Ok(()) => tracing::info!("Email de alerta por cambio de correo enviado a {old_email}"),
            Err(error) => {
                tracing::error!(
                    "Error enviando alerta por cambio de correo a {old_email}: {error}"
                );
            }
        }
    }

    /* [205A-2] Notificación informativa tras cambio de contraseña desde perfil.
     * Permite comprobar entrega SMTP y avisar al usuario si el cambio fue inesperado. */
    pub async fn send_profile_password_changed(
        config: &EmailConfig,
        pool: &PgPool,
        to_email: &str,
        display_name: Option<&str>,
    ) {
        let subject = "Tu contraseña fue actualizada — Nakomi Studio";
        let recipient = recipient_label(display_name, to_email);

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="es">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"></head>
<body style="margin:0;padding:0;background:#f8f8f8;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
<div style="max-width:600px;margin:24px auto;background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.06);">
  <div style="background:#1a1a1a;padding:24px;text-align:center;">
    <h1 style="margin:0;color:#fff;font-size:20px;font-weight:600;">Contraseña actualizada</h1>
  </div>
  <div style="padding:32px 24px;">
    <p style="color:#333;font-size:15px;line-height:1.6;margin:0 0 16px;">Hola, <strong>{recipient}</strong>.</p>
    <p style="color:#555;font-size:14px;line-height:1.6;margin:0 0 16px;">
      La contraseña de tu cuenta fue cambiada correctamente desde la configuración de perfil.
    </p>
    <p style="color:#555;font-size:14px;line-height:1.6;margin:0;">
      Si no fuiste tú, cambia tu contraseña de nuevo de inmediato y contacta al equipo de soporte.
    </p>
  </div>
</div>
</body></html>"#,
        );

        /* [311A-1] Logging del envío en email_logs para trazabilidad. */
        let result = Self::send(config, to_email, subject, &html).await;
        let status = if result.is_ok() { "sent" } else { "failed" };
        let error_msg = result.as_ref().err().map(String::as_str);

        if let Err(log_err) = EmailLogRepository::insert(
            pool, to_email, subject, "profile_password_changed",
            Some("user"), None, status, error_msg,
        ).await {
            tracing::warn!("Error registrando email_log: {log_err}");
        }

        match result {
            Ok(()) => tracing::info!("Email de cambio de contraseña enviado a {to_email}"),
            Err(error) => tracing::error!(
                "Error enviando email de cambio de contraseña a {to_email}: {error}"
            ),
        }
    }

    /* [311A-1] Email al cliente notificando que su orden fue completada.
     * Se dispara cuando el admin completa una orden. Non-fatal. */
    #[allow(clippy::too_many_arguments)]
    pub async fn send_order_completed_client(
        config: &EmailConfig,
        pool: &PgPool,
        to_email: &str,
        client_name: &str,
        order_number: i32,
        site_url: &str,
        order_id: uuid::Uuid,
    ) {
        let subject = format!("✅ Orden #{order_number} completada — Nakomi Studio");
        let escaped_name = html_escape(client_name);
        let panel_link = format!("{site_url}/panel?seccion=ordenes&id={order_id}");

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="es">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"></head>
<body style="margin:0;padding:0;background:#f8f8f8;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
<div style="max-width:600px;margin:24px auto;background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.06);">
  <div style="background:#166534;padding:24px;text-align:center;">
    <h1 style="margin:0;color:#fff;font-size:20px;font-weight:600;">✅ Orden completada</h1>
  </div>
  <div style="padding:32px 24px;">
    <h2 style="margin:0 0 8px;color:#1a1a1a;font-size:18px;">¡Hola, {escaped_name}!</h2>
    <p style="color:#555;font-size:15px;line-height:1.6;margin:0 0 20px;">
      Tu orden <strong>#{order_number}</strong> ha sido completada por nuestro equipo.
      Ya puedes revisar los entregables desde tu panel.
    </p>
    <a href="{panel_link}" style="display:inline-block;background:#c9a84c;color:#fff;text-decoration:none;padding:12px 28px;border-radius:8px;font-weight:600;font-size:14px;">
      Ver mi orden
    </a>
  </div>
  <div style="padding:16px 24px;border-top:1px solid #eee;text-align:center;">
    <p style="margin:0;color:#999;font-size:12px;">Nakomi Studio · Notificación automática</p>
  </div>
</div>
</body></html>"#,
        );

        let result = Self::send(config, to_email, &subject, &html).await;
        let status = if result.is_ok() { "sent" } else { "failed" };
        let error_msg = result.as_ref().err().map(String::as_str);

        if let Err(log_err) = EmailLogRepository::insert(
            pool, to_email, &subject, "order_completed_client",
            Some("order"), Some(order_id), status, error_msg,
        ).await {
            tracing::warn!("Error registrando email_log: {log_err}");
        }

        if let Err(e) = result {
            tracing::error!("Error enviando email orden completada a {to_email}: {e}");
        }
    }

    /* [311A-1] Email al cliente notificando que su orden fue cancelada.
     * Se dispara cuando el admin cancela una orden. Non-fatal. */
    #[allow(clippy::too_many_arguments)]
    pub async fn send_order_cancelled_client(
        config: &EmailConfig,
        pool: &PgPool,
        to_email: &str,
        client_name: &str,
        order_number: i32,
        reason: &str,
        order_id: uuid::Uuid,
    ) {
        let subject = format!("❌ Orden #{order_number} cancelada — Nakomi Studio");
        let escaped_name = html_escape(client_name);
        let escaped_reason = html_escape(reason);

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="es">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"></head>
<body style="margin:0;padding:0;background:#f8f8f8;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
<div style="max-width:600px;margin:24px auto;background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.06);">
  <div style="background:#991b1b;padding:24px;text-align:center;">
    <h1 style="margin:0;color:#fff;font-size:20px;font-weight:600;">❌ Orden cancelada</h1>
  </div>
  <div style="padding:32px 24px;">
    <h2 style="margin:0 0 8px;color:#1a1a1a;font-size:18px;">Hola, {escaped_name}</h2>
    <p style="color:#555;font-size:15px;line-height:1.6;margin:0 0 16px;">
      Tu orden <strong>#{order_number}</strong> fue cancelada.
    </p>
    <p style="color:#555;font-size:14px;line-height:1.6;margin:0;">
      Motivo: <strong>{escaped_reason}</strong>
    </p>
  </div>
  <div style="padding:16px 24px;border-top:1px solid #eee;text-align:center;">
    <p style="margin:0;color:#999;font-size:12px;">Nakomi Studio · Notificación automática</p>
  </div>
</div>
</body></html>"#,
        );

        let result = Self::send(config, to_email, &subject, &html).await;
        let status = if result.is_ok() { "sent" } else { "failed" };
        let error_msg = result.as_ref().err().map(String::as_str);

        if let Err(log_err) = EmailLogRepository::insert(
            pool, to_email, &subject, "order_cancelled_client",
            Some("order"), Some(order_id), status, error_msg,
        ).await {
            tracing::warn!("Error registrando email_log: {log_err}");
        }

        if let Err(e) = result {
            tracing::error!("Error enviando email orden cancelada a {to_email}: {e}");
        }
    }

    /* [311A-1] Email al cliente notificando que una fase fue entregada.
     * Se dispara cuando el admin sube archivos de una fase. Non-fatal. */
    #[allow(clippy::too_many_arguments)]
    pub async fn send_phase_delivered_client(
        config: &EmailConfig,
        pool: &PgPool,
        to_email: &str,
        client_name: &str,
        order_number: i32,
        phase_title: &str,
        site_url: &str,
        order_id: uuid::Uuid,
    ) {
        let subject = format!("📦 Fase entregada — Orden #{order_number} — Nakomi Studio");
        let escaped_name = html_escape(client_name);
        let escaped_phase = html_escape(phase_title);
        let panel_link = format!("{site_url}/panel?seccion=ordenes&id={order_id}");

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="es">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"></head>
<body style="margin:0;padding:0;background:#f8f8f8;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
<div style="max-width:600px;margin:24px auto;background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.06);">
  <div style="background:#1a1a1a;padding:24px;text-align:center;">
    <h1 style="margin:0;color:#c9a84c;font-size:20px;font-weight:600;">📦 Fase entregada</h1>
  </div>
  <div style="padding:32px 24px;">
    <h2 style="margin:0 0 8px;color:#1a1a1a;font-size:18px;">¡Hola, {escaped_name}!</h2>
    <p style="color:#555;font-size:15px;line-height:1.6;margin:0 0 20px;">
      La fase <strong>{escaped_phase}</strong> de tu orden <strong>#{order_number}</strong> ha sido entregada.
      Revisa los archivos y confirma si todo está correcto desde tu panel.
    </p>
    <a href="{panel_link}" style="display:inline-block;background:#c9a84c;color:#fff;text-decoration:none;padding:12px 28px;border-radius:8px;font-weight:600;font-size:14px;">
      Revisar entrega
    </a>
  </div>
  <div style="padding:16px 24px;border-top:1px solid #eee;text-align:center;">
    <p style="margin:0;color:#999;font-size:12px;">Nakomi Studio · Notificación automática</p>
  </div>
</div>
</body></html>"#,
        );

        let result = Self::send(config, to_email, &subject, &html).await;
        let status = if result.is_ok() { "sent" } else { "failed" };
        let error_msg = result.as_ref().err().map(String::as_str);

        if let Err(log_err) = EmailLogRepository::insert(
            pool, to_email, &subject, "phase_delivered_client",
            Some("order"), Some(order_id), status, error_msg,
        ).await {
            tracing::warn!("Error registrando email_log: {log_err}");
        }

        if let Err(e) = result {
            tracing::error!("Error enviando email fase entregada a {to_email}: {e}");
        }
    }

    /* [311A-1] Email al cliente notificando que se reportó un problema en su orden.
     * Se dispara cuando el cliente o admin reporta un problema. Non-fatal. */
    #[allow(clippy::too_many_arguments)]
    pub async fn send_problem_reported_client(
        config: &EmailConfig,
        pool: &PgPool,
        to_email: &str,
        client_name: &str,
        order_number: i32,
        problem_title: &str,
        problem_description: &str,
        site_url: &str,
        order_id: uuid::Uuid,
    ) {
        let subject = format!("⚠️ Problema reportado — Orden #{order_number} — Nakomi Studio");
        let escaped_name = html_escape(client_name);
        let escaped_title = html_escape(problem_title);
        let escaped_desc = html_escape(problem_description);
        let panel_link = format!("{site_url}/panel?seccion=ordenes&id={order_id}");

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="es">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"></head>
<body style="margin:0;padding:0;background:#f8f8f8;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
<div style="max-width:600px;margin:24px auto;background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.06);">
  <div style="background:#92400e;padding:24px;text-align:center;">
    <h1 style="margin:0;color:#fff;font-size:20px;font-weight:600;">⚠️ Problema reportado</h1>
  </div>
  <div style="padding:32px 24px;">
    <h2 style="margin:0 0 8px;color:#1a1a1a;font-size:18px;">Hola, {escaped_name}</h2>
    <p style="color:#555;font-size:15px;line-height:1.6;margin:0 0 16px;">
      Se reportó un problema en tu orden <strong>#{order_number}</strong>:
    </p>
    <div style="background:#f8f8f8;border-radius:8px;padding:16px;margin-bottom:20px;">
      <p style="margin:0 0 8px;font-weight:600;color:#333;font-size:14px;">{escaped_title}</p>
      <p style="margin:0;color:#555;font-size:14px;line-height:1.5;">{escaped_desc}</p>
    </div>
    <p style="color:#555;font-size:14px;line-height:1.6;margin:0 0 20px;">
      Nuestro equipo revisará el problema y te dará seguimiento a la brevedad.
    </p>
    <a href="{panel_link}" style="display:inline-block;background:#c9a84c;color:#fff;text-decoration:none;padding:12px 28px;border-radius:8px;font-weight:600;font-size:14px;">
      Ver orden
    </a>
  </div>
  <div style="padding:16px 24px;border-top:1px solid #eee;text-align:center;">
    <p style="margin:0;color:#999;font-size:12px;">Nakomi Studio · Notificación automática</p>
  </div>
</div>
</body></html>"#,
        );

        let result = Self::send(config, to_email, &subject, &html).await;
        let status = if result.is_ok() { "sent" } else { "failed" };
        let error_msg = result.as_ref().err().map(String::as_str);

        if let Err(log_err) = EmailLogRepository::insert(
            pool, to_email, &subject, "problem_reported_client",
            Some("order"), Some(order_id), status, error_msg,
        ).await {
            tracing::warn!("Error registrando email_log: {log_err}");
        }

        if let Err(e) = result {
            tracing::error!("Error enviando email problema reportado a {to_email}: {e}");
        }
    }
}
