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

use crate::models::WsServerMessage;
use crate::repositories::ChatRepository;
use crate::services::TimingEvent;
use crate::AppState;

use super::VisitorWsParams;
use super::ws_visitor_helpers::{process_visitor_messages, send_history};

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

/* [T-9] Verificar JWT opcional y extraer user_id si es válido.
 * No bloquea la conexión si falla — simplemente trata al usuario como anónimo. */
fn try_extract_user_id(token: Option<&str>, jwt_secret: &str) -> Option<uuid::Uuid> {
    token.and_then(|t| {
        crate::services::AuthService::verify_token(t, jwt_secret)
            .ok()
            .map(|claims| claims.sub)
    })
}

#[allow(clippy::too_many_lines)]
async fn handle_visitor_ws(
    socket: WebSocket,
    state: AppState,
    params: VisitorWsParams,
    visitor_ip: Option<String>,
    visitor_ua: Option<String>,
) {
    /* [084A-42] Anti-bot: verificar límite de conexiones WS por IP */
    let ip_for_tracking = visitor_ip.clone().unwrap_or_default();
    if !ip_for_tracking.is_empty() && !state.chat_timing.track_ip_connect(&ip_for_tracking) {
        tracing::warn!("Anti-bot: rechazada conexión WS de IP {ip_for_tracking}");
        return;
    }

    /* [T-9] Detectar usuario autenticado si envió token JWT */
    let user_id = try_extract_user_id(params.token.as_deref(), &state.jwt_secret);
    if let Some(uid) = user_id {
        tracing::info!("Chat visitor autenticado como user_id={uid}");
    }

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
    let Ok(had_messages) = send_history(&state, session_id, &mut sender).await else {
        return;
    };

    /* [124A-VISIT1] Auto-greeting eliminado: creaba una sesión visible en el panel
     * por cada visitante que abre el chat, incluso sin escribir. La IA responde
     * en cuanto el visitante envía su primer mensaje. */
    let _ = had_messages; /* variable usada para suprimir unused warning */

    /* [104A-40] Registrar timestamp de conexión y notificar al staff que el visitante está online.
     * Sirve como confirmación de lectura: si el visitante está online, vio los mensajes. */
    let visitor_online_at: chrono::DateTime<chrono::Utc> = crate::repositories::ChatRepository::update_visitor_last_connected(
        &state.pool,
        session_id,
    )
    .await
    .unwrap_or_else(|e| {
        tracing::warn!("Error actualizando visitor_last_connected_at: {e}");
        chrono::Utc::now()
    });
    state.chat_hub.notify_visitor_online(session_id, visitor_online_at);

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
            user_id,
            context: params.context.clone(),
            email_config: state.email_config.clone(),
        },
    );

    /* Procesar mensajes del visitante. Retorna true si el cierre fue explícito
     * (usuario cerró chat o rate limit forzó cierre). */
    let explicit_close = process_visitor_messages(
        &mut receiver,
        &state,
        session_id,
        &params.visitor_id,
        visitor_ip.as_deref(),
        &timing_tx,
    )
    .await;

    /* [T-4] Cleanup multi-conexión: solo cerrar sesión si soy la última conexión
     * o si el usuario cerró explícitamente. */
    let remaining = state.chat_hub.unsubscribe(session_id);
    if explicit_close || remaining == 0 {
        /* [104A-40] Última conexión desconectada: notificar offline al staff */
        state.chat_hub.notify_visitor_offline(session_id, Some(visitor_online_at));
        let _ = timing_tx.send(TimingEvent::Disconnect).await;
        state.chat_timing.unregister_session(session_id);
        let _ = state.chat_hub.close_session(session_id).await;
    }
    /* [084A-42] Decrementar conexión IP al desconectar */
    if !ip_for_tracking.is_empty() {
        state.chat_timing.track_ip_disconnect(&ip_for_tracking);
    }
    send_task.abort();
}

/* ============================================================
   ROUTES (WebSocket — montadas en root, no bajo /api)
   ============================================================ */

pub fn ws_routes() -> Router<AppState> {
    Router::new().route("/ws/chat/visitor", get(ws_visitor))
}
