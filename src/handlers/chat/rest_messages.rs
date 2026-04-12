/* [P-1 Chatbot v2] Endpoints REST de mensajes: obtener y enviar mensajes.
 * Separado de rest.rs para mantener el límite de líneas por archivo.
 * send_message contiene lógica de rate limiting, restricciones de orden,
 * notificaciones, IA automática, escalación y email. */

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{
    ChatMessage, ChatMessageResponse, CreateNotification, SendMessageRequest, NOTIF_NEW_MESSAGE,
};
use crate::repositories::OrderRepository;
use crate::services::AiChatService;
use crate::AppState;

use super::{enrich_messages, MessagesQuery};

/// Obtener mensajes de una sesión
#[utoipa::path(
    get,
    path = "/api/chat/sessions/{session_id}/messages",
    params(
        ("session_id" = Uuid, Path, description = "ID de la sesión"),
        ("limit" = Option<i64>, Query, description = "Límite de mensajes"),
        ("offset" = Option<i64>, Query, description = "Offset para paginación"),
    ),
    responses(
        (status = 200, description = "Mensajes", body = Vec<ChatMessage>),
        (status = 401, description = "No autorizado"),
    ),
    security(("bearer_auth" = [])),
    tag = "chat"
)]
pub async fn get_messages(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(session_id): Path<Uuid>,
    Query(params): Query<MessagesQuery>,
) -> Result<Json<Vec<ChatMessageResponse>>, AppError> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);
    let messages =
        crate::repositories::ChatRepository::list_messages(&state.pool, session_id, limit, offset)
            .await?;

    /* [064A-70] Enriquecer mensajes con avatar + nombre del sender */
    let enriched = enrich_messages(&state.pool, messages).await;

    Ok(Json(enriched))
}

/// Enviar mensaje REST (alternativa a WebSocket)
#[utoipa::path(
    post,
    path = "/api/chat/sessions/{session_id}/messages",
    params(("session_id" = Uuid, Path, description = "ID de la sesión")),
    request_body = SendMessageRequest,
    responses(
        (status = 201, description = "Mensaje enviado", body = ChatMessage),
        (status = 401, description = "No autorizado"),
    ),
    security(("bearer_auth" = [])),
    tag = "chat"
)]
#[allow(clippy::too_many_lines)]
pub async fn send_message(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(session_id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Result<(StatusCode, Json<ChatMessage>), AppError> {
    /* [104A-36] Rate limiting para REST — reutiliza ChatTimingService con user_id como key.
     * Solo aplica a clientes; staff/admin no tienen límite. */
    if auth.effective_role == crate::models::UserRole::Client {
        let (result, _msg) = state.chat_timing.check_rate(&auth.user_id.to_string());
        if matches!(result, crate::services::RateCheckResult::Muted) {
            return Err(AppError::TooManyRequests(
                "Has enviado demasiados mensajes. Espera un momento.".to_string(),
            ));
        }
    }

    let sender_type = match auth.effective_role {
        crate::models::UserRole::Admin => "admin",
        crate::models::UserRole::Employee => "employee",
        crate::models::UserRole::Client => "client",
    };

    /* [154A-15e] Restricción de chat de orden: solo 2 personas (cliente + empleado asignado).
     * Admin solo puede enviar si es el empleado asignado. Si no es sesión de orden, sin restricción. */
    if let Ok(Some(sess)) =
        crate::repositories::ChatRepository::find_session_by_id(&state.pool, session_id).await
    {
        if let Some(order_id) = sess.order_id {
            let participants = OrderRepository::get_order_participants(&state.pool, order_id)
                .await
                .ok()
                .flatten();

            if let Some(p) = &participants {
                let is_client = auth.user_id == p.0;
                let is_assigned = p.1 == Some(auth.user_id);
                if !is_client && !is_assigned {
                    return Err(AppError::Forbidden(
                        "Solo el cliente y el empleado asignado pueden enviar mensajes en este chat."
                            .to_string(),
                    ));
                }

                /* [154A-15e] Auto-desactivar IA cuando el empleado asignado responde */
                if is_assigned {
                    let _ = crate::repositories::OrderRepository::toggle_ai_intermediary(
                        &state.pool, order_id, false,
                    )
                    .await;
                }
            }
        }
    }

    let msg = state
        .chat_hub
        .send_message(session_id, sender_type, Some(&auth.user_id.to_string()), &req.content)
        .await?;

    /* [104A-38] Notificar al otro participante del chat (solo sesiones de orden).
     * Cliente envía → notificar staff asignado. Staff/admin envía → notificar cliente. */
    if let Ok(Some(session)) =
        crate::repositories::ChatRepository::find_session_by_id(&state.pool, session_id).await
    {
        let recipient_id = if sender_type == "client" {
            session.assigned_staff_id
        } else if let Some(oid) = session.order_id {
            OrderRepository::client_id_by_id(&state.pool, oid).await.ok().flatten()
        } else {
            None
        };
        if let Some(rid) = recipient_id {
            let preview: String = req.content.chars().take(80).collect();
            let _ = state.notification_hub.notify(CreateNotification {
                user_id: rid,
                notification_type: NOTIF_NEW_MESSAGE.to_string(),
                title: "Nuevo mensaje en el chat".to_string(),
                body: Some(preview),
                link: Some(format!("/panel/chat?session={session_id}")),
                reference_type: Some("chat_session".to_string()),
                reference_id: Some(session_id),
            }).await;
        }
    }

    /* Si IA habilitada y sender es client, generar respuesta */
    if sender_type == "client" {
        if let Ok(Some(s)) =
            crate::repositories::ChatRepository::find_session_by_id(&state.pool, session_id).await
        {
            /* [T-10] IA intermediaria: si sesión de orden con intermediary habilitado */
            if let Some(order_id) = s.order_id {
                spawn_intermediary_if_enabled(
                    &state, session_id, order_id, auth.user_id, &req.content,
                );
            /* [124A-CHAT1] Solo verificar ai_enabled — assigned_staff_id no bloquea la IA.
             * El toggle de IA es explícito via WS ToggleAi o el botón del panel. */
            } else if s.ai_enabled {
                let ai_resp = AiChatService::generate_response(
                    &state.pool,
                    &state.ai_config,
                    &state.http_client,
                    state.stripe_secret_key.as_deref(),
                    crate::services::AiSessionContext {
                        session_id,
                        visitor_id: s.visitor_id.as_deref(),
                        user_id: None,
                        context: None,
                    },
                    &req.content,
                )
                .await
                .unwrap_or_else(|e| crate::services::AiResponse {
                    text: format!("Error IA: {e}"),
                    needs_escalation: true,
                    rich_messages: Vec::new(),
                });

                /* [T-2] Enviar rich messages antes del texto */
                for rm in &ai_resp.rich_messages {
                    let _ = state.chat_hub
                        .send_rich_message(
                            session_id, "ai", Some("ai"),
                            &rm.content, &rm.message_type, &rm.metadata,
                        )
                        .await;
                }

                let _ = state
                    .chat_hub
                    .send_message(session_id, "ai", Some("ai"), &ai_resp.text)
                    .await;

                /* [T-6] [114A-8] Escalación: notificar admins + enviar email */
                if ai_resp.needs_escalation {
                    let visitor = s.visitor_name.as_deref().unwrap_or("Visitante");

                    if let Ok(admin_ids) =
                        crate::repositories::UserRepository::admin_ids(&state.pool).await
                    {
                        if !admin_ids.is_empty() {
                            let base = crate::models::CreateNotification {
                                user_id: uuid::Uuid::nil(),
                                notification_type: crate::models::NOTIF_ESCALATION_NEEDED
                                    .to_string(),
                                title: format!("Escalación: {visitor} necesita ayuda"),
                                body: Some(
                                    "La IA detectó que se requiere intervención humana en la sesión de chat."
                                        .to_string()
                                ),
                                link: Some(format!("/admin/chat?session={session_id}")),
                                reference_type: Some("chat_session".to_string()),
                                reference_id: Some(session_id),
                            };
                            let _ = state.notification_hub.notify_many(&admin_ids, &base).await;
                        }
                    }

                    /* [114A-8] Email de escalación a admins */
                    if let Some(ref email_cfg) = state.email_config {
                        if let Ok(emails) =
                            crate::repositories::UserRepository::admin_emails(&state.pool).await
                        {
                            if !emails.is_empty() {
                                let site_url = std::env::var("SITE_URL")
                                    .unwrap_or_else(|_| "https://nakomi.studio".to_string());
                                crate::services::EmailService::send_escalation_emails(
                                    email_cfg, &emails, visitor, session_id, &site_url,
                                )
                                .await;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok((StatusCode::CREATED, Json(msg)))
}

/* [T-10] Lanza respuesta IA intermediaria en background si la orden lo tiene habilitado */
fn spawn_intermediary_if_enabled(
    state: &AppState,
    session_id: Uuid,
    order_id: Uuid,
    user_id: Uuid,
    content: &str,
) {
    let pool = state.pool.clone();
    let ai_config = state.ai_config.clone();
    let http_client = state.http_client.clone();
    let hub = state.chat_hub.clone();
    let content = content.to_string();
    tokio::spawn(async move {
        /* Verificar si la orden tiene intermediary habilitado */
        let order = match crate::repositories::OrderRepository::find_order_by_id(&pool, order_id)
            .await
        {
            Ok(Some(o)) if o.ai_intermediary_enabled.unwrap_or(false) => o,
            _ => return,
        };

        /* Generar respuesta con prompt de intermediario */
        let ai_resp = match AiChatService::generate_intermediary_response(
            &pool, &ai_config, &http_client, session_id, &order, user_id, &content,
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Error IA intermediaria orden #{}: {e}", order.order_number);
                return;
            }
        };

        /* Enviar respuesta como ai_intermediary */
        let _ = hub
            .send_message(session_id, "ai_intermediary", Some("ai"), &ai_resp.text)
            .await;

        /* [T-10c] Auto-resumen si >5 mensajes en la sesión */
        if let Ok(msgs) =
            crate::repositories::ChatRepository::list_messages(&pool, session_id, 100, 0).await
        {
            if msgs.len() > 5 {
                AiChatService::maybe_update_order_summary(
                    &pool, &ai_config, &http_client, order_id, &msgs,
                )
                .await;
            }
        }
    });
}
