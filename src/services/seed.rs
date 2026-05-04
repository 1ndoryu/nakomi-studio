/* sentinel-disable-file sqlx-query-sin-macro sqlx-query-as-sin-macro: seed usa runtime queries
 * porque ejecuta SQL dinámico con datos de prueba y tipos genéricos. */
/* [064A-62] Servicio de seed: recrea datos de prueba suplementarios desde el panel admin.
 * [084A-25] Limpieza: usuarios, órdenes, proyectos, equipo, hosting y pagos ahora
 * se gestionan con fixtures TOML en content/ (glory-rs ContentManager).
 * [025B-2] Fixtures nuevos: order_problems y vps_subscriptions.
 * El seed suplementario crea: notificaciones, reviews, chat, activity, hosting_events,
 * wallet con balance de prueba y withdrawal_requests en diferentes estados. */

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

/* [174A-2] Helper reutilizable: inserta un hosting_event con ON CONFLICT DO NOTHING.
 * Extraído de create_seed_hosting_events para reducir repetición de la query SQL. */
async fn insert_hosting_event(
    pool: &PgPool,
    sub_id: &Uuid,
    event_type: &str,
    details: serde_json::Value,
    interval: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO hosting_events (subscription_id, event_type, details, created_at)
         VALUES ($1, $2, $3::jsonb, NOW() - $4::interval)
         ON CONFLICT DO NOTHING",
    )
    .bind(sub_id)
    .bind(event_type)
    .bind(details)
    .bind(interval)
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_hosting_event_batch(
    pool: &PgPool,
    sub_id: &Uuid,
    events: Vec<(&str, serde_json::Value, &str)>,
) -> Result<u32, sqlx::Error> {
    let mut count = 0u32;
    for (event_type, details, interval) in events {
        insert_hosting_event(pool, sub_id, event_type, details, interval).await?;
        count += 1;
    }
    Ok(count)
}

async fn create_hosting_status_events(
    pool: &PgPool,
    sub_id: &Uuid,
    status: &str,
    domain_str: &str,
) -> Result<u32, sqlx::Error> {
    match status {
        "active" => {
            insert_hosting_event_batch(
                pool,
                sub_id,
                vec![
                    (
                        "provisioning_completed",
                        serde_json::json!({"message": "Hosting provisionado exitosamente en Coolify", "server_ip": "66.94.100.241", "coolify_site_name": "mitienda-test"}),
                        "29 days 22 hours",
                    ),
                    (
                        "dns_configured",
                        serde_json::json!({"message": format!("Registros DNS de {} apuntan correctamente al servidor", domain_str), "records": ["A @ → 66.94.100.241", "A www → 66.94.100.241"]}),
                        "29 days 20 hours",
                    ),
                    (
                        "ssl_issued",
                        serde_json::json!({"message": format!("Certificado SSL emitido para {}", domain_str), "provider": "Let's Encrypt", "expires": "2026-07-10"}),
                        "29 days 19 hours",
                    ),
                    (
                        "status_changed",
                        serde_json::json!({"from": "provisioning", "to": "active", "message": "Hosting activado y funcionando"}),
                        "29 days 18 hours",
                    ),
                    (
                        "payment_received",
                        serde_json::json!({"message": "Pago mensual procesado: $10.00", "amount_cents": 1000, "source": "stripe_webhook"}),
                        "1 day",
                    ),
                ],
            )
            .await
        }
        "suspended" => {
            insert_hosting_event_batch(
                pool,
                sub_id,
                vec![
                    (
                        "provisioning_completed",
                        serde_json::json!({"message": "Hosting provisionado exitosamente"}),
                        "60 days",
                    ),
                    (
                        "payment_failed",
                        serde_json::json!({"message": "Falló el cobro mensual — tarjeta rechazada", "amount_cents": 1500, "retry_count": 3}),
                        "5 days",
                    ),
                    (
                        "status_changed",
                        serde_json::json!({"from": "active", "to": "suspended", "message": "Hosting suspendido por falta de pago tras 3 intentos fallidos"}),
                        "2 days",
                    ),
                ],
            )
            .await
        }
        _ => {
            insert_hosting_event(
                pool,
                sub_id,
                "status_changed",
                serde_json::json!({
                    "from": "pending", "to": "provisioning",
                    "message": "Servidor asignado, provisioning en curso..."
                }),
                "1 hour",
            )
            .await?;
            Ok(1)
        }
    }
}

async fn upsert_seed_wallet(
    pool: &PgPool,
    user_id: Uuid,
    balance_cents: i32,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO user_wallets (user_id, balance_cents, currency)
         VALUES ($1, $2, 'USD')
         ON CONFLICT (user_id) DO UPDATE SET balance_cents = $2
         RETURNING id",
    )
    .bind(user_id)
    .bind(balance_cents)
    .fetch_one(pool)
    .await
}

async fn insert_wallet_history(
    pool: &PgPool,
    wallet_id: Uuid,
    user_id: Uuid,
    transactions: &[(i32, &str, &str)],
) -> Result<(), sqlx::Error> {
    let mut balance_after = 0i32;
    for (amount, tx_type, description) in transactions {
        balance_after += amount;
        sqlx::query(
            "INSERT INTO wallet_transactions (wallet_id, user_id, amount_cents, transaction_type, description, balance_after_cents)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(wallet_id)
        .bind(user_id)
        .bind(amount)
        .bind(tx_type)
        .bind(description)
        .bind(balance_after)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn insert_seed_withdrawal_request(
    pool: &PgPool,
    user_id: Uuid,
    amount_cents: i32,
    status: &str,
    payment_method: &str,
    payment_details: &str,
    admin_notes: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO withdrawal_requests (user_id, amount_cents, status, payment_method, payment_details, admin_notes)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(user_id)
    .bind(amount_cents)
    .bind(status)
    .bind(payment_method)
    .bind(payment_details)
    .bind(admin_notes)
    .execute(pool)
    .await?;
    Ok(())
}

impl SeedService {
    /// Elimina todos los datos generados por el seed (usuarios test + sus órdenes/fases/chat)
    pub async fn delete_test_data(pool: &PgPool) -> Result<u64, sqlx::Error> {
        let ids: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE email = ANY($1)")
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
    pub async fn recreate_test_data(
        pool: &PgPool,
        seed_requester_id: Uuid,
    ) -> Result<String, sqlx::Error> {
        Self::delete_supplemental_data(pool).await?;

        /* [045A-1] La wallet del panel consulta SIEMPRE al user_id autenticado.
         * Sembrar solo cliente@test.com / empleado@test.com deja la vista admin en $0.00.
         * Por eso también limpiamos y recreamos movimientos/retiros para quien ejecuta el seed. */
        Self::delete_wallet_seed_for_user(pool, seed_requester_id).await?;

        let client_id = Self::find_user(pool, "cliente@test.com").await?;
        let employee_id = Self::find_user(pool, "empleado@test.com").await?;

        let (Some(client_id), Some(employee_id)) = (client_id, employee_id) else {
            return Ok(
                "No se encontraron usuarios de prueba (cliente@test.com / empleado@test.com). \
                Reiniciar el servidor para que los fixtures de content/ se sincronicen primero."
                    .into(),
            );
        };

        let notif_count = Self::create_seed_notifications(pool, client_id, employee_id).await?;
        let review_count = Self::create_seed_reviews(pool, client_id, employee_id).await?;
        let chat_count = Self::create_seed_chat(pool, client_id, employee_id).await?;
        let activity_count = Self::create_seed_activity(pool, client_id, employee_id).await?;
        let events_count = Self::create_seed_hosting_events(pool, client_id).await?;
        let (wallet_balance, withdrawal_count) =
            Self::create_seed_wallet(pool, client_id, employee_id, seed_requester_id).await?;

        Ok(format!(
            "Seed completado: {notif_count} notificaciones + {review_count} reviews + \
             {chat_count} mensajes chat + {activity_count} activity log + \
             {events_count} hosting events + wallet ${:.2} + {withdrawal_count} retiros. \
             Incluye wallet demo para la sesión actual. Credenciales: cliente@test.com/cliente, \
             empleado@test.com/empleado",
            f64::from(wallet_balance) / 100.0
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
        let ids: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE email = ANY($1)")
            .bind(&TEST_EMAILS[..])
            .fetch_all(pool)
            .await?;

        if ids.is_empty() {
            return Ok(());
        }

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

        /* [025B-2] Withdrawal requests y wallet transactions son suplementales */
        sqlx::query("DELETE FROM withdrawal_requests WHERE user_id = ANY($1)")
            .bind(&ids)
            .execute(pool)
            .await?;
        sqlx::query("DELETE FROM wallet_transactions WHERE user_id = ANY($1)")
            .bind(&ids)
            .execute(pool)
            .await?;
        sqlx::query("DELETE FROM user_wallets WHERE user_id = ANY($1)")
            .bind(&ids)
            .execute(pool)
            .await?;

        Ok(())
    }

    /* [045A-1] Limpieza puntual de wallet para la cuenta que ejecuta el seed.
     * No tocamos sus notificaciones ni otras tablas ajenas al problema del panel wallet. */
    async fn delete_wallet_seed_for_user(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM withdrawal_requests WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await?;
        sqlx::query("DELETE FROM wallet_transactions WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await?;
        sqlx::query("DELETE FROM user_wallets WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /* [104A-seed] Crea hosting_events realistas que simulan el ciclo de vida completo
     * de un hosting comprado: pago → provisioning → DNS → SSL → activo.
     * Para hostings suspendidos agrega evento de suspensión por falta de pago. */
    #[allow(clippy::too_many_lines)]
    async fn create_seed_hosting_events(
        pool: &PgPool,
        client_id: Uuid,
    ) -> Result<u32, sqlx::Error> {
        let sub_ids: Vec<(Uuid, String, Option<String>)> = sqlx::query_as(
            "SELECT id, status, domain FROM hosting_subscriptions WHERE user_id = $1",
        )
        .bind(client_id)
        .fetch_all(pool)
        .await?;

        let mut count = 0u32;
        for (sub_id, status, domain) in &sub_ids {
            let domain_str = domain.as_deref().unwrap_or("sin dominio");

            /* Eventos comunes: creación + provisioning iniciado */
            count += insert_hosting_event_batch(
                pool,
                sub_id,
                vec![
                    (
                        "created",
                        serde_json::json!({
                            "source": "stripe_webhook", "plan": "pro", "domain": domain_str,
                            "message": "Suscripción de hosting creada tras confirmación de pago"
                        }),
                        "30 days",
                    ),
                    (
                        "provisioning_started",
                        serde_json::json!({
                            "message": "Servidor VPS asignado, instalando WordPress + SSL",
                            "server": "vps1.nakomi.studio"
                        }),
                        "29 days 23 hours",
                    ),
                ],
            )
            .await?;
            count += create_hosting_status_events(pool, sub_id, status, domain_str).await?;
        }

        Ok(count)
    }

    /* [074A-2] Notificaciones de prueba para que la campanita muestre algo */
    async fn create_seed_notifications(
        pool: &PgPool,
        client_id: Uuid,
        employee_id: Uuid,
    ) -> Result<u32, sqlx::Error> {
        let notifs: &[(&str, Uuid, &str, &str, Option<&str>)] = &[
            (
                "order_assigned",
                employee_id,
                "Nueva orden asignada",
                "Se te asignó la orden #1 de Diseño Web Básico.",
                Some("/panel?tab=ordenes"),
            ),
            (
                "payment_received",
                client_id,
                "Pago confirmado",
                "Tu pago de $100.00 fue procesado exitosamente.",
                Some("/panel?tab=ordenes"),
            ),
            (
                "phase_delivered",
                client_id,
                "Entrega lista para revisión",
                "El freelancer entregó la Fase 1 de tu orden.",
                Some("/panel?tab=ordenes"),
            ),
            (
                "message_received",
                client_id,
                "Nuevo mensaje",
                "Empleado te envió un mensaje en el chat de la orden.",
                Some("/panel?tab=mensajes"),
            ),
            (
                "order_completed",
                client_id,
                "Orden completada",
                "Tu orden de Agentes IA ha sido completada. ¡Déjanos una reseña!",
                Some("/panel?tab=ordenes"),
            ),
            (
                "order_completed",
                employee_id,
                "Orden completada",
                "La orden #3 de Agentes IA fue marcada como completada.",
                Some("/panel?tab=ordenes"),
            ),
        ];

        for (ntype, user_id, title, body, link) in notifs {
            sqlx::query(
                "INSERT INTO notifications (user_id, notification_type, title, body, link, read)
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(user_id)
            .bind(ntype)
            .bind(title)
            .bind(body)
            .bind(link)
            .bind(ntype == &"order_completed") /* completada = ya leída */
            .execute(pool)
            .await?;
        }

        #[allow(clippy::cast_possible_truncation)]
        Ok(notifs.len() as u32)
    }

    /* [074A-2] Review en la orden completada */
    async fn create_seed_reviews(
        pool: &PgPool,
        client_id: Uuid,
        employee_id: Uuid,
    ) -> Result<u32, sqlx::Error> {
        let completed_order: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM orders WHERE client_id = $1 AND status = 'completed' LIMIT 1",
        )
        .bind(client_id)
        .fetch_optional(pool)
        .await?;

        let Some(order_id) = completed_order else {
            return Ok(0);
        };

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
    async fn create_seed_chat(
        pool: &PgPool,
        client_id: Uuid,
        employee_id: Uuid,
    ) -> Result<u32, sqlx::Error> {
        let in_progress_order: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM orders WHERE client_id = $1 AND status = 'in_progress' LIMIT 1",
        )
        .bind(client_id)
        .fetch_optional(pool)
        .await?;

        let Some(order_id) = in_progress_order else {
            return Ok(0);
        };

        let session_id: Uuid = sqlx::query_scalar(
            "INSERT INTO chat_sessions (user_id, order_id, status, assigned_staff_id, ai_enabled)
             VALUES ($1, $2, 'active', $3, false)
             RETURNING id",
        )
        .bind(client_id)
        .bind(order_id)
        .bind(employee_id)
        .fetch_one(pool)
        .await?;

        /* Vincular sesión a la orden */
        sqlx::query("UPDATE orders SET chat_session_id = $1 WHERE id = $2")
            .bind(session_id)
            .bind(order_id)
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

    /* [025B-2] Crea wallet con balance de prueba + withdrawal_requests en diferentes estados.
     * El cliente tiene saldo de $120 (liberado por órdenes completadas).
     * El empleado tiene saldo de $38.40 (80% comisión de orden completada).
     * Retornamos (balance_cents_cliente, num_withdrawals). */
    #[allow(clippy::too_many_lines)]
    async fn create_seed_wallet(
        pool: &PgPool,
        client_id: Uuid,
        employee_id: Uuid,
        seed_requester_id: Uuid,
    ) -> Result<(i32, u32), sqlx::Error> {
        /* Wallet cliente: $120.00 (pagos devueltos / créditos demo) */
        let client_wallet_id = upsert_seed_wallet(pool, client_id, 12000).await?;

        /* Wallet empleado: $38.40 (comisión 80% de orden completada $48) */
        let employee_wallet_id = upsert_seed_wallet(pool, employee_id, 3840).await?;

        /* Historial de transacciones del cliente */
        let client_txs: &[(i32, &str, &str)] = &[
            (15000, "credit", "Crédito inicial de bienvenida"),
            (-3000, "debit", "Pago parcial — Diseño Web Básico fase 1"),
            (3000, "refund", "Reembolso aprobado — Agentes IA parcial"),
            (-3000, "withdrawal", "Retiro procesado vía PayPal"),
        ];
        insert_wallet_history(pool, client_wallet_id, client_id, client_txs).await?;

        /* Historial del empleado */
        let emp_txs: &[(i32, &str, &str)] = &[
            (
                4800,
                "commission",
                "Comisión 80% — Agentes IA Básico completado",
            ),
            (
                -960,
                "withdrawal",
                "Retiro procesado vía transferencia bancaria",
            ),
        ];
        insert_wallet_history(pool, employee_wallet_id, employee_id, emp_txs).await?;

        /* Withdrawal requests del cliente: pending + approved + rejected */
        let withdrawals: &[(&str, i32, &str, &str, Option<&str>)] = &[
            (
                "pending",
                5000,
                "PayPal",
                "nakomi_cliente@gmail.com",
                Some("Retiro mensual de saldo acumulado"),
            ),
            (
                "approved",
                3000,
                "bank",
                "ES12 3456 7890 0123 4567",
                Some("Retiro por transferencia SEPA"),
            ),
            (
                "rejected",
                8000,
                "crypto",
                "3FZbgi29cpjq2GjdwV8eyHuJJnkLtktZc5",
                Some("Método de pago no soportado actualmente."),
            ),
        ];
        let mut withdrawal_count = 0u32;
        for (status, amount, method, details, admin_notes) in withdrawals {
            insert_seed_withdrawal_request(
                pool,
                client_id,
                *amount,
                status,
                method,
                details,
                *admin_notes,
            )
            .await?;
            withdrawal_count += 1;
        }

        /* Withdrawal request del empleado: pending */
        insert_seed_withdrawal_request(
            pool,
            employee_id,
            3840,
            "pending",
            "bank",
            "DE89 3704 0044 0532 0130 00",
            None,
        )
        .await?;
        withdrawal_count += 1;

        /* [045A-1] También sembramos la cuenta que ejecuta el seed, salvo que ya sea uno
         * de los usuarios demo anteriores, para que el panel wallet muestre datos reales
         * sin obligar a iniciar sesión con `cliente@test.com`. */
        if seed_requester_id != client_id && seed_requester_id != employee_id {
            let requester_wallet_id = upsert_seed_wallet(pool, seed_requester_id, 25000).await?;
            let requester_txs: &[(i32, &str, &str)] = &[
                (30000, "commission", "Comisión plataforma — Mayo 2026"),
                (-5000, "withdrawal", "Retiro procesado vía PayPal"),
            ];
            insert_wallet_history(pool, requester_wallet_id, seed_requester_id, requester_txs)
                .await?;
            insert_seed_withdrawal_request(
                pool,
                seed_requester_id,
                10000,
                "pending",
                "bank",
                "ES89 3704 0044 0532 0130 00",
                Some("Retiro mensual de comisiones"),
            )
            .await?;
            withdrawal_count += 1;
        }

        Ok((12000, withdrawal_count))
    }

    /* [074A-2] Activity log para dashboard admin */
    async fn create_seed_activity(
        pool: &PgPool,
        client_id: Uuid,
        employee_id: Uuid,
    ) -> Result<u32, sqlx::Error> {
        let orders: Vec<Uuid> = sqlx::query_scalar(
            "SELECT id FROM orders WHERE client_id = $1 ORDER BY created_at LIMIT 4",
        )
        .bind(client_id)
        .fetch_all(pool)
        .await?;

        if orders.is_empty() {
            return Ok(0);
        }

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
