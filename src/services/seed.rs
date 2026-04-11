/* sentinel-disable-file sqlx-query-sin-macro sqlx-query-as-sin-macro: seed usa runtime queries
 * porque ejecuta SQL dinámico con datos de prueba y tipos genéricos. */
/* [064A-62] Servicio de seed: recrea datos de prueba suplementarios desde el panel admin.
 * [084A-25] Limpieza: usuarios, órdenes, proyectos, equipo, hosting y pagos ahora
 * se gestionan con fixtures TOML en content/ (glory-rs ContentManager).
 * El seed solo crea datos suplementarios que no tienen fixture: notificaciones,
 * reviews, chat, activity log, hosting_events. */

use sqlx::PgPool;
use uuid::Uuid;

/* [084A-25] Emails de test usados para identificar datos de fixture y seed */
const TEST_EMAILS: [&str; 5] = [
    "cliente@test.com",
    "empleado@test.com",
    "test@test.com",
    "testfase2@test.com",
    "employee1@test.com",
];

pub struct SeedService;

impl SeedService {
    /// Elimina todos los datos generados por el seed (usuarios test + sus órdenes/fases/chat)
    pub async fn delete_test_data(pool: &PgPool) -> Result<u64, sqlx::Error> {
        let ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT id FROM users WHERE email = ANY($1)",
        )
        .bind(&TEST_EMAILS[..])
        .fetch_all(pool)
        .await?;

        if ids.is_empty() {
            return Ok(0);
        }

        /* Borrar en orden por FKs: refunds → payments → deliverables → reviews → fases → órdenes → chat → notificaciones → hosting → activity → usuarios */
        sqlx::query(
            "DELETE FROM order_refunds WHERE order_id IN (SELECT id FROM orders WHERE client_id = ANY($1))",
        )
        .bind(&ids)
        .execute(pool)
        .await?;

        sqlx::query(
            "DELETE FROM order_payments WHERE order_id IN (SELECT id FROM orders WHERE client_id = ANY($1))",
        )
        .bind(&ids)
        .execute(pool)
        .await?;

        sqlx::query(
            "DELETE FROM phase_deliverables WHERE phase_id IN (SELECT id FROM order_phases WHERE order_id IN (SELECT id FROM orders WHERE client_id = ANY($1)))",
        )
        .bind(&ids)
        .execute(pool)
        .await?;

        sqlx::query(
            "DELETE FROM order_reviews WHERE order_id IN (SELECT id FROM orders WHERE client_id = ANY($1))",
        )
        .bind(&ids)
        .execute(pool)
        .await?;

        sqlx::query(
            "DELETE FROM order_delegations WHERE order_id IN (SELECT id FROM orders WHERE client_id = ANY($1))",
        )
        .bind(&ids)
        .execute(pool)
        .await?;

        sqlx::query(
            "DELETE FROM order_phases WHERE order_id IN (SELECT id FROM orders WHERE client_id = ANY($1))",
        )
        .bind(&ids)
        .execute(pool)
        .await?;

        /* Limpiar chat_session_id de orders antes de borrar chat_sessions (FK) */
        sqlx::query("UPDATE orders SET chat_session_id = NULL WHERE client_id = ANY($1)")
            .bind(&ids)
            .execute(pool)
            .await?;

        /* Chat depende de orders (order_id FK) y users (user_id FK) — borrar antes de orders */
        sqlx::query("DELETE FROM chat_messages WHERE session_id IN (SELECT id FROM chat_sessions WHERE user_id = ANY($1) OR order_id IN (SELECT id FROM orders WHERE client_id = ANY($1)))")
            .bind(&ids)
            .execute(pool)
            .await?;

        sqlx::query("DELETE FROM chat_sessions WHERE user_id = ANY($1) OR order_id IN (SELECT id FROM orders WHERE client_id = ANY($1))")
            .bind(&ids)
            .execute(pool)
            .await?;

        sqlx::query("DELETE FROM orders WHERE client_id = ANY($1)")
            .bind(&ids)
            .execute(pool)
            .await?;

        /* Hosting: borrar eventos primero, luego suscripciones */
        sqlx::query(
            "DELETE FROM hosting_events WHERE subscription_id IN (SELECT id FROM hosting_subscriptions WHERE user_id = ANY($1))",
        )
        .bind(&ids)
        .execute(pool)
        .await?;

        sqlx::query("DELETE FROM hosting_subscriptions WHERE user_id = ANY($1)")
            .bind(&ids)
            .execute(pool)
            .await?;

        sqlx::query("DELETE FROM notifications WHERE user_id = ANY($1)")
            .bind(&ids)
            .execute(pool)
            .await?;

        sqlx::query("DELETE FROM activity_log WHERE user_id = ANY($1)")
            .bind(&ids)
            .execute(pool)
            .await?;

        let result = sqlx::query("DELETE FROM users WHERE id = ANY($1)")
            .bind(&ids)
            .execute(pool)
            .await?;

        /* [084A-25] Proyectos y team_members ahora son gestionados por fixtures TOML.
         * No se borran aquí — el fixture sync los re-crea al reiniciar. */

        Ok(result.rows_affected())
    }

    /* [084A-25] Recrea datos suplementarios de prueba (no gestionados por fixtures).
     * Usuarios, órdenes, hosting, proyectos y equipo los gestiona content/ vía glory-rs.
     * Aquí solo creamos notificaciones, reviews, chat, activity y hosting_events. */
    pub async fn recreate_test_data(pool: &PgPool) -> Result<String, sqlx::Error> {
        Self::delete_supplemental_data(pool).await?;

        let client_id = Self::find_user(pool, "cliente@test.com").await?;
        let employee_id = Self::find_user(pool, "empleado@test.com").await?;

        let (Some(client_id), Some(employee_id)) = (client_id, employee_id) else {
            return Ok("No se encontraron usuarios de prueba (cliente@test.com / empleado@test.com). \
                Reiniciar el servidor para que los fixtures de content/ se sincronicen primero.".into());
        };

        let notif_count = Self::create_seed_notifications(pool, client_id, employee_id).await?;
        let review_count = Self::create_seed_reviews(pool, client_id, employee_id).await?;
        let chat_count = Self::create_seed_chat(pool, client_id, employee_id).await?;
        let activity_count = Self::create_seed_activity(pool, client_id, employee_id).await?;
        let events_count = Self::create_seed_hosting_events(pool, client_id).await?;

        Ok(format!(
            "Seed completado: {notif_count} notificaciones + {review_count} reviews + \
             {chat_count} mensajes chat + {activity_count} activity log + \
             {events_count} hosting events. \
             Credenciales: cliente@test.com/cliente, empleado@test.com/empleado"
        ))
    }

    /* [084A-25] Busca un usuario por email. Retorna None si no existe. */
    async fn find_user(pool: &PgPool, email: &str) -> Result<Option<Uuid>, sqlx::Error> {
        sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await
    }

    /* [084A-25] Borra solo datos suplementarios (no gestionados por fixtures).
     * No toca users, orders, phases, payments, hosting, projects, team_members. */
    async fn delete_supplemental_data(pool: &PgPool) -> Result<(), sqlx::Error> {
        let ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT id FROM users WHERE email = ANY($1)",
        )
        .bind(&TEST_EMAILS[..])
        .fetch_all(pool)
        .await?;

        if ids.is_empty() { return Ok(()) }

        /* Reviews de test users */
        sqlx::query("DELETE FROM order_reviews WHERE client_id = ANY($1) OR employee_id = ANY($1)")
            .bind(&ids)
            .execute(pool)
            .await?;

        /* Chat: desvincular sesiones de órdenes antes de borrar */
        sqlx::query("UPDATE orders SET chat_session_id = NULL WHERE client_id = ANY($1)")
            .bind(&ids)
            .execute(pool)
            .await?;
        sqlx::query("DELETE FROM chat_messages WHERE session_id IN (SELECT id FROM chat_sessions WHERE user_id = ANY($1))")
            .bind(&ids)
            .execute(pool)
            .await?;
        sqlx::query("DELETE FROM chat_sessions WHERE user_id = ANY($1)")
            .bind(&ids)
            .execute(pool)
            .await?;

        /* Notificaciones y activity log */
        sqlx::query("DELETE FROM notifications WHERE user_id = ANY($1)")
            .bind(&ids)
            .execute(pool)
            .await?;
        sqlx::query("DELETE FROM activity_log WHERE user_id = ANY($1)")
            .bind(&ids)
            .execute(pool)
            .await?;

        /* Hosting events (las suscripciones son fixture-managed, pero los eventos no) */
        sqlx::query("DELETE FROM hosting_events WHERE subscription_id IN (SELECT id FROM hosting_subscriptions WHERE user_id = ANY($1))")
            .bind(&ids)
            .execute(pool)
            .await?;

        Ok(())
    }

    /* [104A-seed] Crea hosting_events realistas que simulan el ciclo de vida completo
     * de un hosting comprado: pago → provisioning → DNS → SSL → activo.
     * Para hostings suspendidos agrega evento de suspensión por falta de pago. */
    #[allow(clippy::too_many_lines)]
    async fn create_seed_hosting_events(pool: &PgPool, client_id: Uuid) -> Result<u32, sqlx::Error> {
        let sub_ids: Vec<(Uuid, String, Option<String>)> = sqlx::query_as(
            "SELECT id, status, domain FROM hosting_subscriptions WHERE user_id = $1",
        )
        .bind(client_id)
        .fetch_all(pool)
        .await?;

        let mut count = 0u32;
        for (sub_id, status, domain) in &sub_ids {
            let domain_str = domain.as_deref().unwrap_or("sin dominio");

            /* Evento 1: suscripción creada (pago procesado) */
            sqlx::query(
                "INSERT INTO hosting_events (subscription_id, event_type, details, created_at)
                 VALUES ($1, 'created', $2::jsonb, NOW() - INTERVAL '30 days')
                 ON CONFLICT DO NOTHING",
            )
            .bind(sub_id)
            .bind(serde_json::json!({
                "source": "stripe_webhook",
                "plan": "pro",
                "domain": domain_str,
                "message": "Suscripción de hosting creada tras confirmación de pago"
            }))
            .execute(pool)
            .await?;
            count += 1;

            /* Evento 2: provisioning iniciado */
            sqlx::query(
                "INSERT INTO hosting_events (subscription_id, event_type, details, created_at)
                 VALUES ($1, 'provisioning_started', $2::jsonb, NOW() - INTERVAL '29 days 23 hours')
                 ON CONFLICT DO NOTHING",
            )
            .bind(sub_id)
            .bind(serde_json::json!({
                "message": "Servidor VPS asignado, instalando WordPress + SSL",
                "server": "vps1.nakomi.studio"
            }))
            .execute(pool)
            .await?;
            count += 1;

            if status == "active" {
                /* Evento 3: provisioning completado */
                sqlx::query(
                    "INSERT INTO hosting_events (subscription_id, event_type, details, created_at)
                     VALUES ($1, 'provisioning_completed', $2::jsonb, NOW() - INTERVAL '29 days 22 hours')
                     ON CONFLICT DO NOTHING",
                )
                .bind(sub_id)
                .bind(serde_json::json!({
                    "message": "Hosting provisionado exitosamente en Coolify",
                    "server_ip": "66.94.100.241",
                    "coolify_site_name": "mitienda-test"
                }))
                .execute(pool)
                .await?;
                count += 1;

                /* Evento 4: DNS configurado */
                sqlx::query(
                    "INSERT INTO hosting_events (subscription_id, event_type, details, created_at)
                     VALUES ($1, 'dns_configured', $2::jsonb, NOW() - INTERVAL '29 days 20 hours')
                     ON CONFLICT DO NOTHING",
                )
                .bind(sub_id)
                .bind(serde_json::json!({
                    "message": format!("Registros DNS de {} apuntan correctamente al servidor", domain_str),
                    "records": ["A @ → 66.94.100.241", "A www → 66.94.100.241"]
                }))
                .execute(pool)
                .await?;
                count += 1;

                /* Evento 5: SSL emitido */
                sqlx::query(
                    "INSERT INTO hosting_events (subscription_id, event_type, details, created_at)
                     VALUES ($1, 'ssl_issued', $2::jsonb, NOW() - INTERVAL '29 days 19 hours')
                     ON CONFLICT DO NOTHING",
                )
                .bind(sub_id)
                .bind(serde_json::json!({
                    "message": format!("Certificado SSL emitido para {}", domain_str),
                    "provider": "Let's Encrypt",
                    "expires": "2026-07-10"
                }))
                .execute(pool)
                .await?;
                count += 1;

                /* Evento 6: hosting activado */
                sqlx::query(
                    "INSERT INTO hosting_events (subscription_id, event_type, details, created_at)
                     VALUES ($1, 'status_changed', $2::jsonb, NOW() - INTERVAL '29 days 18 hours')
                     ON CONFLICT DO NOTHING",
                )
                .bind(sub_id)
                .bind(serde_json::json!({
                    "from": "provisioning",
                    "to": "active",
                    "message": "Hosting activado y funcionando"
                }))
                .execute(pool)
                .await?;
                count += 1;

                /* Evento 7: renovación de pago (simula 1 mes después) */
                sqlx::query(
                    "INSERT INTO hosting_events (subscription_id, event_type, details, created_at)
                     VALUES ($1, 'payment_received', $2::jsonb, NOW() - INTERVAL '1 day')
                     ON CONFLICT DO NOTHING",
                )
                .bind(sub_id)
                .bind(serde_json::json!({
                    "message": "Pago mensual procesado: $10.00",
                    "amount_cents": 1000,
                    "source": "stripe_webhook"
                }))
                .execute(pool)
                .await?;
                count += 1;
            } else if status == "suspended" {
                /* Hosting suspendido: provisioning completado pero luego suspendido */
                sqlx::query(
                    "INSERT INTO hosting_events (subscription_id, event_type, details, created_at)
                     VALUES ($1, 'provisioning_completed', $2::jsonb, NOW() - INTERVAL '60 days')
                     ON CONFLICT DO NOTHING",
                )
                .bind(sub_id)
                .bind(serde_json::json!({
                    "message": "Hosting provisionado exitosamente"
                }))
                .execute(pool)
                .await?;
                count += 1;

                sqlx::query(
                    "INSERT INTO hosting_events (subscription_id, event_type, details, created_at)
                     VALUES ($1, 'payment_failed', $2::jsonb, NOW() - INTERVAL '5 days')
                     ON CONFLICT DO NOTHING",
                )
                .bind(sub_id)
                .bind(serde_json::json!({
                    "message": "Falló el cobro mensual — tarjeta rechazada",
                    "amount_cents": 1500,
                    "retry_count": 3
                }))
                .execute(pool)
                .await?;
                count += 1;

                sqlx::query(
                    "INSERT INTO hosting_events (subscription_id, event_type, details, created_at)
                     VALUES ($1, 'status_changed', $2::jsonb, NOW() - INTERVAL '2 days')
                     ON CONFLICT DO NOTHING",
                )
                .bind(sub_id)
                .bind(serde_json::json!({
                    "from": "active",
                    "to": "suspended",
                    "message": "Hosting suspendido por falta de pago tras 3 intentos fallidos"
                }))
                .execute(pool)
                .await?;
                count += 1;
            } else {
                /* Provisioning en curso — solo el evento de inicio */
                sqlx::query(
                    "INSERT INTO hosting_events (subscription_id, event_type, details, created_at)
                     VALUES ($1, 'status_changed', $2::jsonb, NOW() - INTERVAL '1 hour')
                     ON CONFLICT DO NOTHING",
                )
                .bind(sub_id)
                .bind(serde_json::json!({
                    "from": "pending",
                    "to": "provisioning",
                    "message": "Servidor asignado, provisioning en curso..."
                }))
                .execute(pool)
                .await?;
                count += 1;
            }
        }

        Ok(count)
    }

    /* [074A-2] Notificaciones de prueba para que la campanita muestre algo */
    async fn create_seed_notifications(pool: &PgPool, client_id: Uuid, employee_id: Uuid) -> Result<u32, sqlx::Error> {
        let notifs: &[(&str, Uuid, &str, &str, Option<&str>)] = &[
            ("order_assigned", employee_id, "Nueva orden asignada", "Se te asignó la orden #1 de Diseño Web Básico.", Some("/panel?tab=ordenes")),
            ("payment_received", client_id, "Pago confirmado", "Tu pago de $100.00 fue procesado exitosamente.", Some("/panel?tab=ordenes")),
            ("phase_delivered", client_id, "Entrega lista para revisión", "El freelancer entregó la Fase 1 de tu orden.", Some("/panel?tab=ordenes")),
            ("message_received", client_id, "Nuevo mensaje", "Empleado te envió un mensaje en el chat de la orden.", Some("/panel?tab=mensajes")),
            ("order_completed", client_id, "Orden completada", "Tu orden de Agentes IA ha sido completada. ¡Déjanos una reseña!", Some("/panel?tab=ordenes")),
            ("order_completed", employee_id, "Orden completada", "La orden #3 de Agentes IA fue marcada como completada.", Some("/panel?tab=ordenes")),
        ];

        for (ntype, user_id, title, body, link) in notifs {
            sqlx::query(
                "INSERT INTO notifications (user_id, notification_type, title, body, link, read)
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(user_id).bind(ntype).bind(title).bind(body).bind(link)
            .bind(ntype == &"order_completed") /* completada = ya leída */
            .execute(pool)
            .await?;
        }

        #[allow(clippy::cast_possible_truncation)]
        Ok(notifs.len() as u32)
    }

    /* [074A-2] Review en la orden completada */
    async fn create_seed_reviews(pool: &PgPool, client_id: Uuid, employee_id: Uuid) -> Result<u32, sqlx::Error> {
        let completed_order: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM orders WHERE client_id = $1 AND status = 'completed' LIMIT 1",
        )
        .bind(client_id)
        .fetch_optional(pool)
        .await?;

        let Some(order_id) = completed_order else { return Ok(0) };

        sqlx::query(
            "INSERT INTO order_reviews (order_id, client_id, employee_id, rating, comment, employee_response, employee_responded_at)
             VALUES ($1, $2, $3, 5, 'Excelente trabajo, muy profesional. Entregas puntuales y comunicación clara.', 'Gracias por confiar en nosotros, fue un placer trabajar contigo.', NOW() - INTERVAL '1 day')
             ON CONFLICT (order_id) DO NOTHING",
        )
        .bind(order_id).bind(client_id).bind(employee_id)
        .execute(pool)
        .await?;

        Ok(1)
    }

    /* [074A-2] Chat con mensajes entre cliente y empleado en la orden in_progress */
    async fn create_seed_chat(pool: &PgPool, client_id: Uuid, employee_id: Uuid) -> Result<u32, sqlx::Error> {
        let in_progress_order: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM orders WHERE client_id = $1 AND status = 'in_progress' LIMIT 1",
        )
        .bind(client_id)
        .fetch_optional(pool)
        .await?;

        let Some(order_id) = in_progress_order else { return Ok(0) };

        let session_id: Uuid = sqlx::query_scalar(
            "INSERT INTO chat_sessions (user_id, order_id, status, assigned_staff_id, ai_enabled)
             VALUES ($1, $2, 'active', $3, false)
             RETURNING id",
        )
        .bind(client_id).bind(order_id).bind(employee_id)
        .fetch_one(pool)
        .await?;

        /* Vincular sesión a la orden */
        sqlx::query("UPDATE orders SET chat_session_id = $1 WHERE id = $2")
            .bind(session_id).bind(order_id)
            .execute(pool)
            .await?;

        let messages: &[(&str, &str, &str)] = &[
            ("client", &client_id.to_string(), "Hola, quería consultar sobre el avance de mi sitio web."),
            ("employee", &employee_id.to_string(), "¡Hola! Estoy trabajando en el diseño. Te comparto un boceto en las próximas horas."),
            ("client", &client_id.to_string(), "Perfecto, ¿podrías incluir tonos verdes como en el logo?"),
            ("employee", &employee_id.to_string(), "Sí, ya lo tengo en cuenta. La paleta principal será verde oscuro + blanco."),
            ("client", &client_id.to_string(), "Genial, quedo atento. ¡Gracias!"),
        ];

        let mut count = 0u32;
        for (idx, (sender_type, sender_id, content)) in messages.iter().enumerate() {
            sqlx::query(
                "INSERT INTO chat_messages (session_id, sender_type, sender_id, content, created_at)
                 VALUES ($1, $2, $3, $4, NOW() - ($5 || ' hours')::INTERVAL)",
            )
            .bind(session_id).bind(sender_type).bind(sender_id).bind(content)
            .bind((messages.len() - idx).to_string())
            .execute(pool)
            .await?;
            count += 1;
        }

        Ok(count)
    }

    /* [074A-2] Activity log para dashboard admin */
    async fn create_seed_activity(pool: &PgPool, client_id: Uuid, employee_id: Uuid) -> Result<u32, sqlx::Error> {
        let orders: Vec<Uuid> = sqlx::query_scalar(
            "SELECT id FROM orders WHERE client_id = $1 ORDER BY created_at LIMIT 4",
        )
        .bind(client_id)
        .fetch_all(pool)
        .await?;

        if orders.is_empty() { return Ok(0) }

        let mut count = 0u32;
        for (idx, order_id) in orders.iter().enumerate() {
            sqlx::query(
                "INSERT INTO activity_log (user_id, action, entity_type, entity_id, details, created_at)
                 VALUES ($1, 'order_created', 'order', $2, $3::jsonb, NOW() - ($4 || ' days')::INTERVAL)",
            )
            .bind(client_id).bind(order_id)
            .bind(serde_json::json!({"source": "seed"}))
            .bind((14 - i32::try_from(idx).unwrap_or(0)).to_string())
            .execute(pool)
            .await?;
            count += 1;

            if idx == 0 {
                sqlx::query(
                    "INSERT INTO activity_log (user_id, action, entity_type, entity_id, details, created_at)
                     VALUES ($1, 'order_assigned', 'order', $2, $3::jsonb, NOW() - '12 days'::INTERVAL)",
                )
                .bind(employee_id).bind(order_id)
                .bind(serde_json::json!({"source": "seed", "employee": "empleado@test.com"}))
                .execute(pool)
                .await?;
                count += 1;
            }
        }

        Ok(count)
    }

}
