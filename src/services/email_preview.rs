/* [311A-INV] Módulo de previsualización de plantillas email.
 * Genera HTML de muestra para cada plantilla usando datos ficticios,
 * sin enviar correos reales ni requerir conexión SMTP/BD.
 *
 * Cada función replica el format!() de la plantilla correspondiente en email.rs
 * pero con datos de ejemplo. Si una plantilla cambia, actualizar ambas versiones. */

use crate::services::email::EmailConfig;
use crate::services::email::{format_usd_cents, html_escape, recipient_label};

/// Metadatos de una plantilla de email
#[derive(Debug, Clone, serde::Serialize)]
pub struct TemplateMeta {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub recipients: &'static str,
}

/// Lista completa de plantillas disponibles
pub fn list_templates() -> Vec<TemplateMeta> {
    vec![
        TemplateMeta {
            id: "order_confirmation",
            label: "Confirmación al cliente",
            description: "Se envía al cliente tras crear un pedido",
            category: "orders",
            recipients: "cliente",
        },
        TemplateMeta {
            id: "new_order_admin",
            label: "Nueva orden (admin)",
            description: "Notifica a los admins cuando se crea un pedido",
            category: "orders",
            recipients: "admin",
        },
        TemplateMeta {
            id: "payment_received_admin",
            label: "Pago recibido (admin)",
            description: "Notifica a los admins cuando un pago se recibe",
            category: "payments",
            recipients: "admin",
        },
        TemplateMeta {
            id: "order_completed_client",
            label: "Orden completada (cliente)",
            description: "Se envía al cliente cuando su orden se completa",
            category: "orders",
            recipients: "cliente",
        },
        TemplateMeta {
            id: "order_cancelled_client",
            label: "Orden cancelada (cliente)",
            description: "Se envía al cliente cuando su orden se cancela",
            category: "orders",
            recipients: "cliente",
        },
        TemplateMeta {
            id: "phase_delivered_client",
            label: "Fase entregada (cliente)",
            description: "Notifica al cliente que una fase fue entregada",
            category: "orders",
            recipients: "cliente",
        },
        TemplateMeta {
            id: "problem_reported_client",
            label: "Problema reportado (cliente)",
            description: "Se envía al cliente cuando se reporta un problema",
            category: "orders",
            recipients: "cliente",
        },
        TemplateMeta {
            id: "escalation",
            label: "Escalación de chat",
            description: "Notifica a admins cuando la IA escala un chat",
            category: "chat",
            recipients: "admin",
        },
        TemplateMeta {
            id: "chat_invoice_paid_client",
            label: "Factura chat pagada (cliente)",
            description: "Confirma al cliente el pago de una factura de chat",
            category: "chat",
            recipients: "cliente",
        },
        TemplateMeta {
            id: "chat_invoice_paid_admin",
            label: "Factura chat pagada (admin)",
            description: "Notifica a admins del pago de una factura de chat",
            category: "chat",
            recipients: "admin",
        },
        TemplateMeta {
            id: "vps_pending_approval",
            label: "VPS pendiente (admin)",
            description: "Notifica a admins de una suscripción VPS pendiente",
            category: "vps",
            recipients: "admin",
        },
        TemplateMeta {
            id: "vps_approved",
            label: "VPS aprobado (cliente)",
            description: "Se envía al cliente cuando su VPS es aprobado",
            category: "vps",
            recipients: "cliente",
        },
        TemplateMeta {
            id: "vps_rejected",
            label: "VPS rechazado (cliente)",
            description: "Se envía al cliente cuando su VPS es rechazado",
            category: "vps",
            recipients: "cliente",
        },
        TemplateMeta {
            id: "profile_email_changed_new",
            label: "Email cambiado (nuevo)",
            description: "Confirma al nuevo correo el cambio de email",
            category: "profile",
            recipients: "cliente",
        },
        TemplateMeta {
            id: "profile_email_changed_old",
            label: "Email cambiado (anterior)",
            description: "Alerta al correo anterior sobre el cambio",
            category: "profile",
            recipients: "cliente",
        },
        TemplateMeta {
            id: "profile_password_changed",
            label: "Contraseña cambiada",
            description: "Notifica al usuario que su contraseña fue cambiada",
            category: "profile",
            recipients: "cliente",
        },
    ]
}

/// Renderiza una plantilla con datos de muestra y devuelve el HTML completo.
/// `config` se usa solo para el footer/from (no se envía realmente).
pub fn render_preview(config: &EmailConfig, template: &str) -> Result<String, String> {
    match template {
        "order_confirmation" => Ok(render_order_confirmation(config)),
        "new_order_admin" => Ok(render_new_order_admin(config)),
        "payment_received_admin" => Ok(render_payment_received_admin(config)),
        "order_completed_client" => Ok(render_order_completed_client(config)),
        "order_cancelled_client" => Ok(render_order_cancelled_client(config)),
        "phase_delivered_client" => Ok(render_phase_delivered_client(config)),
        "problem_reported_client" => Ok(render_problem_reported_client(config)),
        "escalation" => Ok(render_escalation(config)),
        "chat_invoice_paid_client" => Ok(render_chat_invoice_paid_client(config)),
        "chat_invoice_paid_admin" => Ok(render_chat_invoice_paid_admin(config)),
        "vps_pending_approval" => Ok(render_vps_pending_approval(config)),
        "vps_approved" => Ok(render_vps_approved(config)),
        "vps_rejected" => Ok(render_vps_rejected(config)),
        "profile_email_changed_new" => Ok(render_profile_email_changed_new(config)),
        "profile_email_changed_old" => Ok(render_profile_email_changed_old(config)),
        "profile_password_changed" => Ok(render_profile_password_changed(config)),
        _ => Err(format!("Plantilla desconocida: {template}")),
    }
}

// ─── Sample data constants ───────────────────────────────────────────
const SAMPLE_CLIENT_NAME: &str = "Juan Pérez";
const SAMPLE_CLIENT_EMAIL: &str = "cliente@ejemplo.com";
const SAMPLE_ORDER_NUMBER: i32 = 12345;
const SAMPLE_SERVICE_TITLE: &str = "Diseño Web Profesional";
const SAMPLE_PLAN_NAME: &str = "Pro";
const SAMPLE_PRICE_DISPLAY: &str = "$499.00 USD";
const SAMPLE_SITE_URL: &str = "https://nakomi.studio";
const SAMPLE_PAYMENT_MODE: &str = "transferencia";
const SAMPLE_AMOUNT_USD: f64 = 499.00;
const SAMPLE_TIER_NAME: &str = "VPS Avanzado";
const SAMPLE_MONTHLY_PRICE_CENTS: i32 = 29900;
const SAMPLE_PUBLIC_IP: &str = "203.0.113.42";
const SAMPLE_USERNAME: &str = "root";
const SAMPLE_PASSWORD: &str = "s3gur4-P@ss!";
const SAMPLE_REASON: &str = "Fondos insuficientes en el método de pago";
const SAMPLE_OLD_EMAIL: &str = "anterior@ejemplo.com";
const SAMPLE_NEW_EMAIL: &str = "nuevo@ejemplo.com";
const SAMPLE_PHASE_TITLE: &str = "Maquetación responsive";
const SAMPLE_PROBLEM_TITLE: &str = "Error en sección de contacto";
const SAMPLE_PROBLEM_DESC: &str = "El formulario de contacto no envía los datos correctamente. Ya verificamos la configuración SMTP y el problema persiste.";
const SAMPLE_VISITOR_NAME: &str = "María García";

fn sample_uuid() -> String {
    "123e4567-e89b-12d3-a456-426614174000".to_string()
}

fn sample_panel_link() -> String {
    format!("{}/panel?seccion=ordenes&id={}", SAMPLE_SITE_URL, sample_uuid())
}

fn sample_chat_link() -> String {
    format!("{}/panel/chat?session={}", SAMPLE_SITE_URL, sample_uuid())
}

// ─── Individual template renderers ───────────────────────────────────

fn render_order_confirmation(_config: &EmailConfig) -> String {
    let client_name = html_escape(SAMPLE_CLIENT_NAME);
    let service_title = html_escape(SAMPLE_SERVICE_TITLE);
    let plan_name = html_escape(SAMPLE_PLAN_NAME);
    let price_display = html_escape(SAMPLE_PRICE_DISPLAY);

    format!(
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
        client_name = client_name,
        order_number = SAMPLE_ORDER_NUMBER,
        service_title = service_title,
        plan_name = plan_name,
        price_display = price_display,
    )
}

fn render_new_order_admin(_config: &EmailConfig) -> String {
    let escaped_client = html_escape(SAMPLE_CLIENT_NAME);
    let escaped_email = html_escape(SAMPLE_CLIENT_EMAIL);
    let escaped_service = html_escape(SAMPLE_SERVICE_TITLE);
    let escaped_plan = html_escape(SAMPLE_PLAN_NAME);
    let panel_link = sample_panel_link();

    format!(
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
        order_number = SAMPLE_ORDER_NUMBER,
        escaped_client = escaped_client,
        escaped_email = escaped_email,
        escaped_service = escaped_service,
        escaped_plan = escaped_plan,
        payment_mode = html_escape(SAMPLE_PAYMENT_MODE),
        price_display = html_escape(SAMPLE_PRICE_DISPLAY),
        panel_link = panel_link,
    )
}

fn render_payment_received_admin(_config: &EmailConfig) -> String {
    let escaped_client = html_escape(SAMPLE_CLIENT_NAME);
    let escaped_email = html_escape(SAMPLE_CLIENT_EMAIL);
    let panel_link = sample_panel_link();

    format!(
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
        escaped_client = escaped_client,
        escaped_email = escaped_email,
        order_number = SAMPLE_ORDER_NUMBER,
        amount_display = html_escape(SAMPLE_PRICE_DISPLAY),
        panel_link = panel_link,
    )
}

fn render_order_completed_client(_config: &EmailConfig) -> String {
    let escaped_name = html_escape(SAMPLE_CLIENT_NAME);
    let panel_link = sample_panel_link();

    format!(
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
        escaped_name = escaped_name,
        order_number = SAMPLE_ORDER_NUMBER,
        panel_link = panel_link,
    )
}

fn render_order_cancelled_client(_config: &EmailConfig) -> String {
    let escaped_name = html_escape(SAMPLE_CLIENT_NAME);
    let escaped_reason = html_escape(SAMPLE_REASON);

    format!(
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
        escaped_name = escaped_name,
        order_number = SAMPLE_ORDER_NUMBER,
        escaped_reason = escaped_reason,
    )
}

fn render_phase_delivered_client(_config: &EmailConfig) -> String {
    let escaped_name = html_escape(SAMPLE_CLIENT_NAME);
    let escaped_phase = html_escape(SAMPLE_PHASE_TITLE);
    let panel_link = sample_panel_link();

    format!(
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
        escaped_name = escaped_name,
        order_number = SAMPLE_ORDER_NUMBER,
        escaped_phase = escaped_phase,
        panel_link = panel_link,
    )
}

fn render_problem_reported_client(_config: &EmailConfig) -> String {
    let escaped_name = html_escape(SAMPLE_CLIENT_NAME);
    let escaped_title = html_escape(SAMPLE_PROBLEM_TITLE);
    let escaped_desc = html_escape(SAMPLE_PROBLEM_DESC);
    let panel_link = sample_panel_link();

    format!(
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
        escaped_name = escaped_name,
        order_number = SAMPLE_ORDER_NUMBER,
        escaped_title = escaped_title,
        escaped_desc = escaped_desc,
        panel_link = panel_link,
    )
}

fn render_escalation(_config: &EmailConfig) -> String {
    let escaped_name = html_escape(SAMPLE_VISITOR_NAME);
    let panel_link = sample_chat_link();

    format!(
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
        escaped_name = escaped_name,
        panel_link = panel_link,
    )
}

fn render_chat_invoice_paid_client(_config: &EmailConfig) -> String {
    let escaped_email = html_escape(SAMPLE_CLIENT_EMAIL);
    let register_url = format!("{}/register", SAMPLE_SITE_URL);

    format!(
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
      ¡Hola! Tu pago de <strong>{amount_usd} USD</strong> fue procesado exitosamente.
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
        amount_usd = format!("${:.2}", SAMPLE_AMOUNT_USD),
        escaped_email = escaped_email,
        register_url = register_url,
        site_url = SAMPLE_SITE_URL,
    )
}

fn render_chat_invoice_paid_admin(_config: &EmailConfig) -> String {
    let escaped_email = html_escape(SAMPLE_CLIENT_EMAIL);
    let panel_link = sample_chat_link();

    format!(
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
      El cliente <strong>{escaped_email}</strong> pagó <strong>{amount_usd} USD</strong> via factura de chat.
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
        escaped_email = escaped_email,
        amount_usd = format!("${:.2}", SAMPLE_AMOUNT_USD),
        panel_link = panel_link,
    )
}

fn render_vps_pending_approval(_config: &EmailConfig) -> String {
    let escaped_email = html_escape(SAMPLE_CLIENT_EMAIL);
    let escaped_tier = html_escape(SAMPLE_TIER_NAME);
    let amount_display = format_usd_cents(SAMPLE_MONTHLY_PRICE_CENTS);

    format!(
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
        escaped_email = escaped_email,
        escaped_tier = escaped_tier,
        amount_display = amount_display,
    )
}

fn render_vps_approved(_config: &EmailConfig) -> String {
    let escaped_tier = html_escape(SAMPLE_TIER_NAME);
    let escaped_username = html_escape(SAMPLE_USERNAME);
    let escaped_password = html_escape(SAMPLE_PASSWORD);
    let ip_block = format!(
        r#"<tr><td style="padding:6px 0;color:#888;">IP pública</td><td style="padding:6px 0;font-weight:500;text-align:right;">{}</td></tr>"#,
        html_escape(SAMPLE_PUBLIC_IP),
    );

    format!(
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
        escaped_tier = escaped_tier,
        ip_block = ip_block,
        escaped_username = escaped_username,
        escaped_password = escaped_password,
    )
}

fn render_vps_rejected(_config: &EmailConfig) -> String {
    let escaped_tier = html_escape(SAMPLE_TIER_NAME);
    let escaped_reason = html_escape(SAMPLE_REASON);

    format!(
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
        escaped_tier = escaped_tier,
        escaped_reason = escaped_reason,
    )
}

fn render_profile_email_changed_new(_config: &EmailConfig) -> String {
    let escaped_old = html_escape(SAMPLE_OLD_EMAIL);
    let escaped_new = html_escape(SAMPLE_NEW_EMAIL);

    format!(
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
        recipient = recipient_label(Some(SAMPLE_CLIENT_NAME), SAMPLE_NEW_EMAIL),
        escaped_old = escaped_old,
        escaped_new = escaped_new,
    )
}

fn render_profile_email_changed_old(_config: &EmailConfig) -> String {
    let escaped_old = html_escape(SAMPLE_OLD_EMAIL);
    let escaped_new = html_escape(SAMPLE_NEW_EMAIL);

    format!(
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
        recipient = recipient_label(Some(SAMPLE_CLIENT_NAME), SAMPLE_OLD_EMAIL),
        escaped_old = escaped_old,
        escaped_new = escaped_new,
    )
}

fn render_profile_password_changed(_config: &EmailConfig) -> String {
    format!(
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
        recipient = recipient_label(Some(SAMPLE_CLIENT_NAME), SAMPLE_CLIENT_EMAIL),
    )
}
