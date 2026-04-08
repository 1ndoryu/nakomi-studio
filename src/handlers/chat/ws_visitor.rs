/* [P-1 Chatbot v2] WS handler del visitante (anónimo o cliente).
 * Endpoint: /ws/chat/visitor?visitor_id=X&visitor_name=Y
 * Captura IP y User-Agent, reutiliza sesión activa, reenvía historial. */

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;

use crate::models::{WsClientMessage, WsServerMessage};
use crate::services::AiChatService;
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

    /* [064A-29] Enviar historial de mensajes previos al reconectar.
     * El frontend deduplica por ID, asi que no hay riesgo de duplicados. */
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
            };
            if let Ok(json) = serde_json::to_string(&ws_msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    return;
                }
            }
        }
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

    /* Recibir mensajes del visitante */
    let hub = state.chat_hub.clone();
    let ai_config = state.ai_config.clone();
    let pool = state.pool.clone();
    let vid = params.visitor_id.clone();

    while let Some(Ok(msg)) = receiver.next().await {
        let Message::Text(text) = msg else {
            continue;
        };
        let Ok(ws_msg) = serde_json::from_str::<WsClientMessage>(&text) else {
            continue;
        };

        match ws_msg {
            WsClientMessage::Message { content } => {
                let _ = hub
                    .send_message(session_id, "client", Some(&vid), &content)
                    .await;
                maybe_ai_reply(&pool, &ai_config, &hub, session_id, &content).await;
            }
            WsClientMessage::Typing { content } => {
                hub.send_typing(session_id, "client", &content);
            }
            WsClientMessage::Close => {
                let _ = hub.close_session(session_id).await;
                break;
            }
            _ => {} /* join/toggle_ai son solo para staff */
        }
    }

    send_task.abort();
}

/* Si IA habilitada y no hay staff asignado, generar y enviar respuesta AI */
async fn maybe_ai_reply(
    pool: &sqlx::PgPool,
    ai_config: &crate::services::AiChatConfig,
    hub: &crate::services::ChatHub,
    session_id: uuid::Uuid,
    content: &str,
) {
    if let Ok(Some(s)) =
        crate::repositories::ChatRepository::find_session_by_id(pool, session_id).await
    {
        if s.ai_enabled && s.assigned_staff_id.is_none() {
            let ai_response = AiChatService::generate_response(pool, ai_config, session_id, content)
                .await
                .unwrap_or_else(|e| format!("Error IA: {e}"));
            let _ = hub
                .send_message(session_id, "ai", Some("ai"), &ai_response)
                .await;
        }
    }
}

/* ============================================================
   ROUTES (WebSocket — montadas en root, no bajo /api)
   ============================================================ */

pub fn ws_routes() -> Router<AppState> {
    Router::new().route("/ws/chat/visitor", get(ws_visitor))
}
