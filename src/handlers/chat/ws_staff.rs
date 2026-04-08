/* [P-1 Chatbot v2] WS handler del staff (autenticado con JWT).
 * Endpoint: /ws/chat/staff?token=JWT
 * Recibe sesiones activas al conectar, se suscribe a nuevas sesiones,
 * puede join/toggle_ai/close sesiones individuales. */

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use futures::{SinkExt, StreamExt};
use uuid::Uuid;

use crate::models::{WsClientMessage, WsServerMessage};
use crate::AppState;

use super::StaffWsParams;

/* ============================================================
   WEBSOCKET: STAFF (autenticado con JWT)
   ============================================================ */

async fn ws_staff(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(params): Query<StaffWsParams>,
) -> impl IntoResponse {
    /* Verificar JWT desde query param (WS no soporta headers custom) */
    let Ok(claims) = crate::services::AuthService::verify_token(
        &params.token,
        &state.jwt_secret,
    ) else {
        return (StatusCode::UNAUTHORIZED, "Token inválido").into_response();
    };

    ws.on_upgrade(move |socket| handle_staff_ws(socket, state, claims.sub))
        .into_response()
}

async fn handle_staff_ws(socket: WebSocket, state: AppState, staff_id: Uuid) {
    let (mut ws_sender, mut receiver) = socket.split();

    let hub = state.chat_hub.clone();
    let pool = state.pool.clone();

    /* Canal interno: las suscripciones a sesiones envían aquí, un task central escribe al WS */
    let (tx, mut rx) = tokio::sync::mpsc::channel::<WsServerMessage>(128);

    /* Enviar lista de sesiones activas al conectar */
    if let Ok(sessions) = hub.list_all_active_sessions().await {
        let init_msg = serde_json::json!({
            "type": "init",
            "sessions": sessions
        });
        if let Ok(json) = serde_json::to_string(&init_msg) {
            let _ = ws_sender.send(Message::Text(json)).await;
        }
    }

    /* [064A-68] Suscripción global al canal de staff para nuevas sesiones.
     * Cualquier sesión creada (visitante u orden) se reenvía a este WS. */
    let mut staff_rx = hub.subscribe_staff();
    let tx_staff = tx.clone();
    let staff_sub = tokio::spawn(async move {
        while let Ok(server_msg) = staff_rx.recv().await {
            if tx_staff.send(server_msg).await.is_err() {
                break;
            }
        }
    });

    /* Task: leer del mpsc y enviar al WS */
    let send_task = tokio::spawn(async move {
        while let Some(server_msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&server_msg) {
                if ws_sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    /* Track de suscripciones */
    let mut subscriptions = Vec::new();

    /* Recibir mensajes del staff */
    while let Some(Ok(msg)) = receiver.next().await {
        let Message::Text(text) = msg else {
            continue;
        };
        let Ok(ws_msg) = serde_json::from_str::<WsClientMessage>(&text) else {
            continue;
        };

        match ws_msg {
            WsClientMessage::Join { session_id } => {
                let _ = hub.staff_join_session(session_id, staff_id).await;

                /* Suscribirse al canal de esta sesión → reenviar al mpsc */
                let mut session_rx = hub.subscribe(session_id);
                let tx_clone = tx.clone();
                let handle = tokio::spawn(async move {
                    while let Ok(server_msg) = session_rx.recv().await {
                        if tx_clone.send(server_msg).await.is_err() {
                            break;
                        }
                    }
                });
                subscriptions.push(handle);
            }
            WsClientMessage::Message { content } => {
                tracing::debug!("Staff message (sin session_id en WsClientMessage::Message): {content}");
            }
            WsClientMessage::Typing { content } => {
                tracing::debug!("Staff typing: {content}");
            }
            WsClientMessage::ToggleAi {
                session_id,
                enabled,
            } => {
                let _ = hub.toggle_ai(session_id, enabled).await;
            }
            WsClientMessage::Close => {
                break;
            }
        }
    }

    /* Cleanup */
    for handle in subscriptions {
        handle.abort();
    }
    staff_sub.abort();
    send_task.abort();

    let _ = pool;
}

/* ============================================================
   ROUTES (Staff WS — montadas junto a ws_visitor en root)
   ============================================================ */

pub fn ws_staff_routes() -> Router<AppState> {
    Router::new().route("/ws/chat/staff", get(ws_staff))
}
