/* [P-1 Chatbot v2] WS handler del visitante (anónimo o cliente).
 * Endpoint: /ws/chat/visitor?visitor_id=X&visitor_name=Y
 * Captura IP y User-Agent, reutiliza sesión activa, reenvía historial.
 * [T-1] Usa ChatTimingService para rate limiting y timing inteligente. */

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;

use crate::models::{WsClientMessage, WsServerMessage};
use crate::repositories::ChatRepository;
use crate::services::{RateCheckResult, TimingEvent};
use crate::AppState;

use super::VisitorWsParams;

/* ============================================================
   WEBSOCKET: VISITOR (anónimo o cliente autenticado)
   ============================================================ */

async fn ws_visitor(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
    Query(params): Query<VisitorWsParams>,
) -> impl IntoResponse {
    /* [064A-72] [074A-41] Capturar IP (proxy headers → fallback a socket) y User-Agent */
    let visitor_ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(String::from)
        })
        .or_else(|| Some(addr.ip().to_string()));
    let visitor_ua = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    ws.on_upgrade(move |socket| {
        handle_visitor_ws(socket, state, params, visitor_ip, visitor_ua)
    })
}

async fn handle_visitor_ws(
    socket: WebSocket,
    state: AppState,
    params: VisitorWsParams,
    visitor_ip: Option<String>,
    visitor_ua: Option<String>,
) {
    let session = match state
        .chat_hub
        .get_or_create_visitor_session(
            &params.visitor_id,
            params.visitor_name.as_deref(),
            visitor_ip.as_deref(),
            visitor_ua.as_deref(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Error creando sesión visitor: {e}");
            return;
        }
    };

    let session_id = session.id;
    let mut rx = state.chat_hub.subscribe(session_id);
    let (mut sender, mut receiver) = socket.split();

    /* [T-3] Upsert visitor_profile: crea o actualiza perfil persistente.
     * Trackea IPs, fingerprints, sesiones. Usado para contexto IA. */
    if let Err(e) = ChatRepository::upsert_visitor_profile(
        &state.pool,
        &params.visitor_id,
        visitor_ip.as_deref(),
        visitor_ua.as_deref(),
    )
    .await
    {
        tracing::warn!("Error upserting visitor_profile: {e}");
    }

    /* [064A-29] Enviar historial de mensajes previos al reconectar */
    if send_history(&state, session_id, &mut sender).await.is_err() {
        return;
    }

    /* Notificar a staff de nueva sesión */
    state.chat_hub.broadcast(
        session_id,
        WsServerMessage::SessionNew {
            session: session.clone(),
        },
    );

    /* Task: enviar mensajes del broadcast al WS del visitante */
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    /* [T-1] Registrar sesión en timing service */
    let timing_tx = state.chat_timing.register_session(
        session_id,
        session.visitor_name.clone(),
        crate::services::TimingSessionDeps {
            pool: state.pool.clone(),
            ai_config: state.ai_config.clone(),
            hub: state.chat_hub.clone(),
            notification_hub: state.notification_hub.clone(),
            http_client: state.http_client.clone(),
            stripe_key: state.stripe_secret_key.clone(),
            visitor_id: params.visitor_id.clone(),
        },
    );

    /* Procesar mensajes del visitante */
    process_visitor_messages(
        &mut receiver,
        &state,
        session_id,
        &params.visitor_id,
        &timing_tx,
    )
    .await;

    /* Cleanup */
    state.chat_timing.unregister_session(session_id);
    send_task.abort();
}

/* Enviar historial de mensajes al visitante al reconectar */
async fn send_history(
    state: &AppState,
    session_id: uuid::Uuid,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> Result<(), ()> {
    if let Ok(history) =
        crate::repositories::ChatRepository::list_messages(&state.pool, session_id, 50, 0).await
    {
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
    }
    Ok(())
}

/* Loop principal de procesamiento de mensajes del visitante.
 * Aplica rate limiting, persiste mensajes y envía eventos al timing service. */
async fn process_visitor_messages(
    receiver: &mut futures::stream::SplitStream<WebSocket>,
    state: &AppState,
    session_id: uuid::Uuid,
    visitor_id: &str,
    timing_tx: &tokio::sync::mpsc::Sender<TimingEvent>,
) {
    while let Some(Ok(msg)) = receiver.next().await {
        let Message::Text(text) = msg else {
            continue;
        };
        let Ok(ws_msg) = serde_json::from_str::<WsClientMessage>(&text) else {
            continue;
        };

        match ws_msg {
            WsClientMessage::Message { content } => {
                /* [T-1] Rate limiting antes de procesar */
                let (rate_result, rate_msg) =
                    state.chat_timing.check_rate(visitor_id);
                match rate_result {
                    RateCheckResult::Muted => {
                        if let Some(w) = rate_msg {
                            let _ = state.chat_hub.send_message(session_id, "ai", Some("ai"), &w).await;
                        }
                        continue;
                    }
                    RateCheckResult::Closed => {
                        if let Some(w) = rate_msg {
                            let _ = state.chat_hub.send_message(session_id, "ai", Some("ai"), &w).await;
                        }
                        let _ = state.chat_hub.close_session(session_id).await;
                        return;
                    }
                    RateCheckResult::Warning => {
                        if let Some(w) = rate_msg {
                            let _ = state.chat_hub.send_message(session_id, "ai", Some("ai"), &w).await;
                        }
                    }
                    RateCheckResult::Ok => {}
                }

                let _ = state.chat_hub
                    .send_message(session_id, "client", Some(visitor_id), &content)
                    .await;
                let _ = timing_tx.send(TimingEvent::Message(content)).await;
            }
            WsClientMessage::Typing { content } => {
                state.chat_hub.send_typing(session_id, "client", &content);
                if content.is_empty() {
                    let _ = timing_tx.send(TimingEvent::TypingStop).await;
                } else {
                    let _ = timing_tx.send(TimingEvent::TypingStart).await;
                }
            }
            WsClientMessage::Close => {
                let _ = timing_tx.send(TimingEvent::Disconnect).await;
                let _ = state.chat_hub.close_session(session_id).await;
                return;
            }
            /* [T-2] Acciones desde botones de mensajes ricos.
             * El frontend envía action_type + payload que se re-inyectan
             * como mensaje de texto para que la IA lo procese en contexto. */
            WsClientMessage::Action { action_type, payload } => {
                let action_text = format!(
                    "[Acción: {action_type}] {}",
                    payload.as_str().unwrap_or(&payload.to_string())
                );
                let _ = state.chat_hub
                    .send_message(session_id, "client", Some(visitor_id), &action_text)
                    .await;
                let _ = timing_tx.send(TimingEvent::Message(action_text)).await;
            }
            _ => {} /* join/toggle_ai son solo para staff */
        }
    }
}

/* ============================================================
   ROUTES (WebSocket — montadas en root, no bajo /api)
   ============================================================ */

pub fn ws_routes() -> Router<AppState> {
    Router::new().route("/ws/chat/visitor", get(ws_visitor))
}
