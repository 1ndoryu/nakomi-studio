/* [259A-4c] Avisos a administradores (extraído de email.rs).
 * Todas son non-fatal: ante fallo SMTP solo se loguea. El envío real
 * (`EmailService::send`) y los helpers viven en el hub `email`. */

use sqlx::PgPool;

use crate::repositories::{EmailLogRepository, NuevoEmailLog};

use super::email::{EmailConfig, EmailService};

/* [09AA-1] Contexto común a todos los avisos de admin. Todos los campos son
 * referencias, así que `Copy` permite pasarlo por valor a cada envío. */
#[derive(Clone, Copy)]
pub struct AdminMailCtx<'a> {
    pub config: &'a EmailConfig,
    pub pool: &'a PgPool,
    pub admin_emails: &'a [String],
    pub site_url: &'a str,
}

/* [09AA-1] Datos del aviso de nueva orden. Sin email del cliente, plan ni modo
 * de pago: ningún aviso los usaba (solo alimentaban parámetros `_` sin uso). */
pub struct NewOrderAdminMail<'a> {
    pub client_name: &'a str,
    pub order_number: i32,
    pub service_title: &'a str,
    pub price_display: &'a str,
    pub order_id: uuid::Uuid,
}

/* [09AA-1] Datos del aviso de pago recibido. */
pub struct PaymentReceivedAdminMail<'a> {
    pub client_name: &'a str,
    pub order_number: i32,
    pub amount_display: &'a str,
    pub order_id: uuid::Uuid,
}

/* [09AA-1] Datos del aviso de orden completada. */
pub struct OrderCompletedAdminMail<'a> {
    pub client_name: &'a str,
    pub order_number: i32,
    pub order_id: uuid::Uuid,
}

/* [09AA-1] Datos del aviso de orden cancelada. */
pub struct OrderCancelledAdminMail<'a> {
    pub client_name: &'a str,
    pub order_number: i32,
    pub reason: &'a str,
    pub order_id: uuid::Uuid,
}

/* [09AA-1] Datos del aviso de problema reportado. El título no se usaba en el
 * asunto ni en el HTML, así que solo viaja la descripción. */
pub struct ProblemReportedAdminMail<'a> {
    pub client_name: &'a str,
    pub order_number: i32,
    pub problem_description: &'a str,
    pub order_id: uuid::Uuid,
}

/* [09AA-1] Datos del aviso de reembolso solicitado. */
pub struct RefundRequestedAdminMail<'a> {
    pub client_name: &'a str,
    pub order_number: i32,
    pub amount_display: &'a str,
    pub reason: &'a str,
    pub refund_id: uuid::Uuid,
}

impl EmailService {
    /* [311A-1] Envía email a todos los admins activos notificando una nueva orden.
     * Se dispara desde el webhook de Stripe (handlers/payments/webhook.rs) tras crear la orden.
     * Non-fatal: si falla, solo se loguea. */
    pub async fn send_new_order_admin(ctx: AdminMailCtx<'_>, mail: NewOrderAdminMail<'_>) {
        let AdminMailCtx {
            config,
            pool,
            admin_emails,
            site_url,
        } = ctx;
        let NewOrderAdminMail {
            client_name,
            order_number,
            service_title,
            price_display,
            order_id,
        } = mail;
        let subject = format!("🆕 Nueva orden #{order_number} — {client_name} — Nakomi Studio");
        let panel_link = format!("{site_url}/panel?seccion=ordenes&id={order_id}");

        /* [07AA-7] F3h redujo el template a 5 params (sin email/plan/modo):
         * se adapta la llamada; plan y modo siguen en subject/contexto del panel. */
        let html = super::email_templates::render_new_order_admin(
            client_name,
            order_number,
            service_title,
            price_display,
            &panel_link,
        );

        for email in admin_emails {
            /* [311A-1] Logging individual por admin para trazabilidad. */
            let result = Self::send(config, email, &subject, &html).await;
            let status = if result.is_ok() { "sent" } else { "failed" };
            let error_msg = result.as_ref().err().map(String::as_str);

            if let Err(log_err) = EmailLogRepository::insert(
                pool,
                NuevoEmailLog {
                    to_email: email,
                    subject: &subject,
                    template: "new_order_admin",
                    reference_type: Some("order"),
                    reference_id: Some(order_id),
                    status,
                    error_msg,
                },
            )
            .await
            {
                tracing::warn!("Error registrando email_log: {log_err}");
            }

            if let Err(e) = result {
                tracing::error!("Error enviando email nueva orden admin a {email}: {e}");
            }
        }
        if !admin_emails.is_empty() {
            tracing::info!(
                "Email nueva orden #{order_number} enviado a {} admins",
                admin_emails.len()
            );
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

        let html = super::email_templates::render_escalation(visitor_name, &panel_link);

        for email in admin_emails {
            /* [311A-1] Logging individual por admin para trazabilidad. */
            let result = Self::send(config, email, &subject, &html).await;
            let status = if result.is_ok() { "sent" } else { "failed" };
            let error_msg = result.as_ref().err().map(String::as_str);

            if let Err(log_err) = EmailLogRepository::insert(
                pool,
                NuevoEmailLog {
                    to_email: email,
                    subject: &subject,
                    template: "escalation",
                    reference_type: Some("chat_session"),
                    reference_id: Some(session_id),
                    status,
                    error_msg,
                },
            )
            .await
            {
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
    pub async fn send_payment_received_admin(
        ctx: AdminMailCtx<'_>,
        mail: PaymentReceivedAdminMail<'_>,
    ) {
        let AdminMailCtx {
            config,
            pool,
            admin_emails,
            site_url,
        } = ctx;
        let PaymentReceivedAdminMail {
            client_name,
            order_number,
            amount_display,
            order_id,
        } = mail;
        let subject = format!("💰 Pago recibido — Orden #{order_number} — Nakomi Studio");
        let panel_link = format!("{site_url}/panel/orders/{order_id}");

        let html = super::email_templates::render_payment_received_admin(
            client_name,
            order_number,
            amount_display,
            &panel_link,
        );

        for email in admin_emails {
            /* [311A-1] Logging individual por admin para trazabilidad. */
            let result = Self::send(config, email, &subject, &html).await;
            let status = if result.is_ok() { "sent" } else { "failed" };
            let error_msg = result.as_ref().err().map(String::as_str);

            if let Err(log_err) = EmailLogRepository::insert(
                pool,
                NuevoEmailLog {
                    to_email: email,
                    subject: &subject,
                    template: "payment_received_admin",
                    reference_type: Some("order"),
                    reference_id: Some(order_id),
                    status,
                    error_msg,
                },
            )
            .await
            {
                tracing::warn!("Error registrando email_log: {log_err}");
            }

            if let Err(e) = result {
                tracing::error!("Error enviando email pago recibido admin a {email}: {e}");
            }
        }
        if !admin_emails.is_empty() {
            tracing::info!(
                "Email pago recibido orden #{order_number} enviado a {} admins",
                admin_emails.len()
            );
        }
    }

    /* [011A-1] Email a admins notificando que una orden fue completada.
     * Se dispara cuando el cliente aprueba la última fase. Non-fatal. */
    pub async fn send_order_completed_admin(
        ctx: AdminMailCtx<'_>,
        mail: OrderCompletedAdminMail<'_>,
    ) {
        let AdminMailCtx {
            config,
            pool,
            admin_emails,
            site_url,
        } = ctx;
        let OrderCompletedAdminMail {
            client_name,
            order_number,
            order_id,
        } = mail;
        let subject =
            format!("✅ Orden #{order_number} completada — {client_name} — Nakomi Studio");
        let panel_link = format!("{site_url}/panel?seccion=ordenes&id={order_id}");

        let html = super::email_templates::render_order_completed_admin(
            client_name,
            order_number,
            "",
            &panel_link,
        );

        for email in admin_emails {
            let result = Self::send(config, email, &subject, &html).await;
            let status = if result.is_ok() { "sent" } else { "failed" };
            let error_msg = result.as_ref().err().map(String::as_str);
            if let Err(log_err) = EmailLogRepository::insert(
                pool,
                NuevoEmailLog {
                    to_email: email,
                    subject: &subject,
                    template: "order_completed_admin",
                    reference_type: Some("order"),
                    reference_id: Some(order_id),
                    status,
                    error_msg,
                },
            )
            .await
            {
                tracing::warn!("Error registrando email_log: {log_err}");
            }
            if let Err(e) = result {
                tracing::error!("Error enviando email orden completada admin a {email}: {e}");
            }
        }
        if !admin_emails.is_empty() {
            tracing::info!(
                "Email orden completada #{order_number} enviado a {} admins",
                admin_emails.len()
            );
        }
    }

    /* [011A-1] Email a admins notificando que una orden fue cancelada.
     * Se dispara cuando el admin cancela una orden. Non-fatal. */
    pub async fn send_order_cancelled_admin(
        ctx: AdminMailCtx<'_>,
        mail: OrderCancelledAdminMail<'_>,
    ) {
        let AdminMailCtx {
            config,
            pool,
            admin_emails,
            site_url,
        } = ctx;
        let OrderCancelledAdminMail {
            client_name,
            order_number,
            reason,
            order_id,
        } = mail;
        let subject = format!("❌ Orden #{order_number} cancelada — {client_name} — Nakomi Studio");
        let panel_link = format!("{site_url}/panel?seccion=ordenes&id={order_id}");

        let html = super::email_templates::render_order_cancelled_admin(
            client_name,
            order_number,
            reason,
            &panel_link,
        );

        for email in admin_emails {
            let result = Self::send(config, email, &subject, &html).await;
            let status = if result.is_ok() { "sent" } else { "failed" };
            let error_msg = result.as_ref().err().map(String::as_str);
            if let Err(log_err) = EmailLogRepository::insert(
                pool,
                NuevoEmailLog {
                    to_email: email,
                    subject: &subject,
                    template: "order_cancelled_admin",
                    reference_type: Some("order"),
                    reference_id: Some(order_id),
                    status,
                    error_msg,
                },
            )
            .await
            {
                tracing::warn!("Error registrando email_log: {log_err}");
            }
            if let Err(e) = result {
                tracing::error!("Error enviando email orden cancelada admin a {email}: {e}");
            }
        }
        if !admin_emails.is_empty() {
            tracing::info!(
                "Email orden cancelada #{order_number} enviado a {} admins",
                admin_emails.len()
            );
        }
    }

    /* [011A-1] Email a admins notificando que se reportó un problema.
     * Se dispara cuando un cliente o admin reporta un problema en una orden. Non-fatal. */
    pub async fn send_problem_reported_admin(
        ctx: AdminMailCtx<'_>,
        mail: ProblemReportedAdminMail<'_>,
    ) {
        let AdminMailCtx {
            config,
            pool,
            admin_emails,
            site_url,
        } = ctx;
        let ProblemReportedAdminMail {
            client_name,
            order_number,
            problem_description,
            order_id,
        } = mail;
        let subject = format!("⚠️ Problema reportado — Orden #{order_number} — Nakomi Studio");
        let panel_link = format!("{site_url}/panel?seccion=ordenes&id={order_id}");

        let html = super::email_templates::render_problem_reported_admin(
            client_name,
            order_number,
            problem_description,
            &panel_link,
        );

        for email in admin_emails {
            let result = Self::send(config, email, &subject, &html).await;
            let status = if result.is_ok() { "sent" } else { "failed" };
            let error_msg = result.as_ref().err().map(String::as_str);
            if let Err(log_err) = EmailLogRepository::insert(
                pool,
                NuevoEmailLog {
                    to_email: email,
                    subject: &subject,
                    template: "problem_reported_admin",
                    reference_type: Some("order"),
                    reference_id: Some(order_id),
                    status,
                    error_msg,
                },
            )
            .await
            {
                tracing::warn!("Error registrando email_log: {log_err}");
            }
            if let Err(e) = result {
                tracing::error!("Error enviando email problema reportado admin a {email}: {e}");
            }
        }
        if !admin_emails.is_empty() {
            tracing::info!(
                "Email problema reportado orden #{order_number} enviado a {} admins",
                admin_emails.len()
            );
        }
    }

    /* [011A-1] Email a admins notificando solicitud de reembolso.
     * Se dispara cuando un cliente solicita un reembolso. Non-fatal. */
    pub async fn send_refund_requested_admin(
        ctx: AdminMailCtx<'_>,
        mail: RefundRequestedAdminMail<'_>,
    ) {
        let AdminMailCtx {
            config,
            pool,
            admin_emails,
            site_url,
        } = ctx;
        let RefundRequestedAdminMail {
            client_name,
            order_number,
            amount_display,
            reason,
            refund_id,
        } = mail;
        let subject = format!("🔄 Reembolso solicitado — Orden #{order_number} — Nakomi Studio");
        let panel_link = format!("{site_url}/panel?seccion=reembolsos&id={refund_id}");

        let html = super::email_templates::render_refund_requested_admin(
            client_name,
            order_number,
            amount_display,
            reason,
            &panel_link,
        );

        for email in admin_emails {
            let result = Self::send(config, email, &subject, &html).await;
            let status = if result.is_ok() { "sent" } else { "failed" };
            let error_msg = result.as_ref().err().map(String::as_str);
            if let Err(log_err) = EmailLogRepository::insert(
                pool,
                NuevoEmailLog {
                    to_email: email,
                    subject: &subject,
                    template: "refund_requested_admin",
                    reference_type: Some("refund"),
                    reference_id: Some(refund_id),
                    status,
                    error_msg,
                },
            )
            .await
            {
                tracing::warn!("Error registrando email_log: {log_err}");
            }
            if let Err(e) = result {
                tracing::error!("Error enviando email reembolso solicitado admin a {email}: {e}");
            }
        }
        if !admin_emails.is_empty() {
            tracing::info!(
                "Email reembolso solicitado orden #{order_number} enviado a {} admins",
                admin_emails.len()
            );
        }
    }

    /* [011A-1] Email a admins notificando que un nuevo usuario se registró.
     * Se dispara después de un registro exitoso. Non-fatal. */
    #[allow(clippy::too_many_arguments)]
    pub async fn send_new_user_registered_admin(
        config: &EmailConfig,
        pool: &PgPool,
        admin_emails: &[String],
        user_email: &str,
        user_name: &str,
        user_id: uuid::Uuid,
        site_url: &str,
    ) {
        let subject = format!("🆕 Nuevo usuario registrado — {user_email} — Nakomi Studio");
        let panel_link = format!("{site_url}/panel");

        let html = super::email_templates::render_new_user_registered_admin(
            user_name,
            user_email,
            &panel_link,
        );

        for email in admin_emails {
            let result = Self::send(config, email, &subject, &html).await;
            let status = if result.is_ok() { "sent" } else { "failed" };
            let error_msg = result.as_ref().err().map(String::as_str);
            if let Err(log_err) = EmailLogRepository::insert(
                pool,
                NuevoEmailLog {
                    to_email: email,
                    subject: &subject,
                    template: "new_user_registered_admin",
                    reference_type: Some("user"),
                    reference_id: Some(user_id),
                    status,
                    error_msg,
                },
            )
            .await
            {
                tracing::warn!("Error registrando email_log: {log_err}");
            }
            if let Err(e) = result {
                tracing::error!("Error enviando email nuevo usuario admin a {email}: {e}");
            }
        }
        if !admin_emails.is_empty() {
            tracing::info!(
                "Email nuevo usuario registrado ({user_email}) enviado a {} admins",
                admin_emails.len()
            );
        }
    }
}
