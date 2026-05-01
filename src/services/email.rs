/* [154A-15c] Servicio de email con SMTP (lettre).
 * Configuración vía env vars: SMTP_HOST o GLORY_SMTP_HOST, SMTP_PORT o GLORY_SMTP_PORT,
 * SMTP_USER o GLORY_SMTP_USER, SMTP_PASS o GLORY_SMTP_PASSWORD, SMTP_FROM.
 * Se aceptan ambos prefijos para compatibilidad con .env local (GLORY_SMTP_*)
 * y posibles configuraciones legacy (SMTP_*).
 * Non-fatal: si SMTP no está configurado, los emails se loguean y se omiten. */

use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

#[derive(Clone)]
pub struct EmailConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub from_name: String,
    pub from_email: String,
}

impl EmailConfig {
    /// Intenta crear config desde env vars. Retorna None si faltan variables.
    /// Acepta SMTP_* y GLORY_SMTP_* como nombres de variables (compat local/prod).
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
        let from_name = std::env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "Nakomi Studio".to_string());
        let port = std::env::var("SMTP_PORT")
            .or_else(|_| std::env::var("GLORY_SMTP_PORT"))
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(587);

        Some(Self { host, port, user, pass, from_name, from_email })
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
        let email = Message::builder()
            .from(from.parse().map_err(|e| format!("From inválido: {e}"))?)
            .to(to_email.parse().map_err(|e| format!("To inválido: {e}"))?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
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

        if let Err(e) = Self::send(config, to_email, &subject, &html).await {
            tracing::error!("Error enviando email de confirmación orden #{order_number}: {e}");
        }
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn format_usd_cents(amount_cents: i32) -> String {
  format!("${:.2} USD", f64::from(amount_cents) / 100.0)
}

/* [114A-8] Envía email de escalación a todos los admins activos.
 * Se dispara cuando la IA detecta que un visitante necesita asistencia humana.
 * Non-fatal: si SMTP no está configurado o falla, solo se loguea. */
impl EmailService {
    pub async fn send_escalation_emails(
        config: &EmailConfig,
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
            if let Err(e) = Self::send(config, email, &subject, &html).await {
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

    /* [124A-INV] Email al cliente notificando que su factura fue pagada y
     * que puede registrarse con el email de pago para acceder al panel. */
    pub async fn send_chat_invoice_paid_client(
        config: &EmailConfig,
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

        if let Err(e) = Self::send(config, client_email, &subject, &html).await {
            tracing::error!("Error enviando email pago factura chat a {client_email}: {e}");
        } else {
            tracing::info!("Email pago chat invoice enviado a {client_email}");
        }
    }

    /* [124A-INV] Email a admins notificando que una factura de chat fue pagada. */
    pub async fn send_chat_invoice_paid_admin(
        config: &EmailConfig,
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
            if let Err(e) = Self::send(config, email, &subject, &html).await {
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
            if let Err(error) = Self::send(config, email, &subject, &html).await {
                tracing::error!("Error enviando email VPS pendiente a {email}: {error}");
            }
        }
    }

    pub async fn send_vps_approved(
        config: &EmailConfig,
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

        if let Err(error) = Self::send(config, client_email, &subject, &html).await {
            tracing::error!("Error enviando email VPS aprobado a {client_email}: {error}");
        }
    }

    pub async fn send_vps_rejected(
        config: &EmailConfig,
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

        if let Err(error) = Self::send(config, client_email, &subject, &html).await {
            tracing::error!("Error enviando email VPS rechazado a {client_email}: {error}");
        }
    }
}
