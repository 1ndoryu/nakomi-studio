/* [P-1 Chatbot v2] Funciones auxiliares del WS visitor.
 * Extraídas de ws_visitor.rs para mantener el límite de líneas por archivo.
 * send_history: reenvío de historial al reconectar.
 * handle_reset: comando /reset para limpiar sesión.
 * process_visitor_messages: loop principal de procesamiento de mensajes. */

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};

use crate::models::{WsClientMessage, WsServerMessage};
use crate::repositories::ChatRepository;
use crate::services::{RateCheckResult, TimingEvent};
use crate::AppState;

/* Enviar historial de mensajes al visitante al reconectar.
 * Retorna true si la sesión tenía mensajes previos, false si es nueva. */
pub async fn send_history(
    state: &AppState,
    session_id: uuid::Uuid,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> Result<bool, ()> {
    if let Ok(history) =
        crate::repositories::ChatRepository::list_messages(&state.pool, session_id, 50, 0).await
    {
        let had_messages = !history.is_empty();
        for msg in history {
            let ws_msg = WsServerMessage::Message {
                id: msg.id,
                session_id: msg.session_id,
                sender: msg.sender_type,
                sender_id: msg.sender_id,
                content: msg.content,
                created_at: msg.created_at,
                message_type: msg.message_type,
                metadata: msg.metadata,
            };
            if let Ok(json) = serde_json::to_string(&ws_msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    return Err(());
                }
            }
        }
        Ok(had_messages)
    } else {
        Ok(false)
    }
}

/* [084A-40] Ejecutar /reset: borrar mensajes, perfil, cerrar sesión y notificar al cliente.
 * El visitante obtiene un estado completamente limpio (como si fuera la primera visita). */
pub async fn handle_reset(state: &AppState, session_id: uuid::Uuid, visitor_id: &str) {
    let pool = &state.pool;

    if let Err(e) = ChatRepository::delete_session_messages(pool, session_id).await {
        tracing::error!("Reset: error borrando mensajes session={session_id}: {e}");
    }
    if let Err(e) = ChatRepository::delete_visitor_profile(pool, visitor_id).await {
        tracing::error!("Reset: error borrando perfil visitor={visitor_id}: {e}");
    }
    if let Err(e) = ChatRepository::close_session(pool, session_id).await {
        tracing::error!("Reset: error cerrando session={session_id}: {e}");
    }

    state.chat_hub.broadcast(session_id, WsServerMessage::Reset);
    tracing::info!("Reset ejecutado: session={session_id}, visitor={visitor_id}");
}

enum VisitorTextFlow {
    Continue,
    Close,
}

fn truncate_visitor_message(content: String) -> String {
    let max_length = crate::services::ChatTimingService::max_message_length();
    if content.len() > max_length {
        content[..max_length].to_string()
    } else {
        content
    }
}

async fn apply_rate_limit_feedback(
    state: &AppState,
    session_id: uuid::Uuid,
    rate_result: RateCheckResult,
    rate_msg: Option<String>,
) -> Option<bool> {
    if let Some(message) = rate_msg {
        let _ = state
            .chat_hub
            .send_message(session_id, "ai", Some("ai"), &message)
            .await;
    }

    match rate_result {
        RateCheckResult::Muted => Some(false),
        RateCheckResult::Closed => Some(true),
        RateCheckResult::Warning | RateCheckResult::Ok => None,
    }
}

async fn handle_visitor_text_message(
    state: &AppState,
    session_id: uuid::Uuid,
    visitor_id: &str,
    client_ip: Option<&str>,
    timing_tx: &tokio::sync::mpsc::Sender<TimingEvent>,
    content: String,
) -> VisitorTextFlow {
    if content.trim().eq_ignore_ascii_case("/reset") {
        handle_reset(state, session_id, visitor_id).await;
        return VisitorTextFlow::Close;
    }

    let content = truncate_visitor_message(content);

    if let Some(ip) = client_ip {
        let (ip_result, ip_msg) = state.chat_timing.check_ip_rate(ip);
        if let Some(should_close) =
            apply_rate_limit_feedback(state, session_id, ip_result, ip_msg).await
        {
            return if should_close {
                VisitorTextFlow::Close
            } else {
                VisitorTextFlow::Continue
            };
        }
    }

    let (rate_result, rate_msg) = state.chat_timing.check_rate(visitor_id);
    if let Some(should_close) =
        apply_rate_limit_feedback(state, session_id, rate_result, rate_msg).await
    {
        return if should_close {
            VisitorTextFlow::Close
        } else {
            VisitorTextFlow::Continue
        };
    }

    let _ = state
        .chat_hub
        .send_message(session_id, "client", Some(visitor_id), &content)
        .await;
    let _ = timing_tx.send(TimingEvent::Message(content)).await;

    VisitorTextFlow::Continue
}

/* [T-4] Loop principal de procesamiento de mensajes del visitante.
 * Aplica rate limiting (por visitor_id + IP), persiste mensajes y envía eventos al timing service.
 * Retorna true si el cierre fue explícito (usuario o rate limit), false si el stream terminó. */
#[allow(clippy::too_many_lines)]
pub async fn process_visitor_messages(
    receiver: &mut futures::stream::SplitStream<WebSocket>,
    state: &AppState,
    session_id: uuid::Uuid,
    visitor_id: &str,
    client_ip: Option<&str>,
    timing_tx: &tokio::sync::mpsc::Sender<TimingEvent>,
) -> bool {
    while let Some(Ok(msg)) = receiver.next().await {
        let Message::Text(text) = msg else {
            continue;
        };
        let Ok(ws_msg) = serde_json::from_str::<WsClientMessage>(&text) else {
            continue;
        };

        match ws_msg {
            WsClientMessage::Message { content } => {
                if matches!(
                    handle_visitor_text_message(
                        state, session_id, visitor_id, client_ip, timing_tx, content,
                    )
                    .await,
                    VisitorTextFlow::Close
                ) {
                    return true;
                }
            }
            WsClientMessage::Typing { content, .. } => {
                state.chat_hub.send_typing(session_id, "client", &content);
                if content.is_empty() {
                    let _ = timing_tx.send(TimingEvent::TypingStop).await;
                } else {
                    let _ = timing_tx.send(TimingEvent::TypingStart).await;
                }
            }
            WsClientMessage::Close => {
                /* [T-4] Cierre explícito del usuario — cierra para todas las conexiones */
                let _ = timing_tx.send(TimingEvent::Disconnect).await;
                return true;
            }
            /* [T-2] Acciones desde botones de mensajes ricos.
             * El frontend envía action_type + payload que se re-inyectan
             * como mensaje de texto para que la IA lo procese en contexto. */
            WsClientMessage::Action {
                action_type,
                payload,
            } => {
                let action_text = format!(
                    "[Acción: {action_type}] {}",
                    payload.as_str().unwrap_or(&payload.to_string())
                );
                let _ = state
                    .chat_hub
                    .send_message(session_id, "client", Some(visitor_id), &action_text)
                    .await;
                let _ = timing_tx.send(TimingEvent::Message(action_text)).await;
            }
            _ => {} /* join/toggle_ai son solo para staff */
        }
    }
    /* [T-4] Stream terminó sin cierre explícito (pestaña cerrada, red caída) */
    false
}
