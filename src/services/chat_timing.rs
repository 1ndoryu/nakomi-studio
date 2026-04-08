/* [T-1] Chat Timing Service: anti-spam, rate limiting y timing inteligente.
 * Gestiona una máquina de estados por sesión que bufferea mensajes del
 * visitante y decide cuándo generar respuesta IA. Evita respuestas
 * instantáneas a cada mensaje, emulando comportamiento humano.
 *
 * Máquina de estados:
 *   IDLE → recibe mensaje → WAITING (buffer=[msg], timer=4s)
 *   WAITING → timer expira → RESPONDING → genera respuesta → IDLE
 *   WAITING → nuevo mensaje → WAITING (buffer.push, timer reset)
 *   WAITING → typing start → LISTENING (timer=8s desde último typing)
 *   LISTENING → typing stop + no mensaje 3s → RESPONDING
 *   LISTENING → nuevo mensaje → LISTENING (buffer.push, timer reset)
 *   RESPONDING → en progreso → ignora triggers
 *
 * Rate limiting: max 10 msgs/min por visitor_id, cooldown progresivo.
 * Relevancia: modelo pequeño (llama-3.1-8b-instant) filtra spam/off-topic. */

use std::fmt::Write;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use sqlx::PgPool;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::models::{CreateNotification, NOTIF_ESCALATION_NEEDED};
use crate::repositories::UserRepository;
use crate::services::{AiChatConfig, AiChatService, AiResponse, ChatHub, NotificationHub};

/* Dependencias agrupadas para evitar exceso de argumentos en register_session */
pub struct TimingSessionDeps {
    pub pool: PgPool,
    pub ai_config: AiChatConfig,
    pub hub: ChatHub,
    pub notification_hub: NotificationHub,
    pub http_client: reqwest::Client,
    pub stripe_key: Option<String>,
    pub visitor_id: String,
}

/* Constantes de timing configurables */
const WAIT_TIMEOUT: Duration = Duration::from_secs(4);
const LISTEN_TIMEOUT: Duration = Duration::from_secs(8);
const TYPING_COOLDOWN: Duration = Duration::from_secs(3);
const MAX_ACCUMULATION: Duration = Duration::from_secs(30);

/* Rate limiting */
const RATE_LIMIT_PER_MIN: u32 = 10;
const RATE_WINDOW: Duration = Duration::from_secs(60);

/* Relevancia */
const MAX_IRRELEVANT_STREAK: u32 = 3;

/// Eventos que el handler WS envía al timing service
pub enum TimingEvent {
    Message(String),
    TypingStart,
    TypingStop,
    Disconnect,
}

/// Resultado del rate limiter
pub enum RateCheckResult {
    Ok,
    Warning,
    Muted,
    Closed,
}

/* Estado de rate limiting por visitor_id */
struct RateState {
    count: u32,
    window_start: Instant,
    cooldown_level: u8,
    mute_until: Option<Instant>,
}

impl Default for RateState {
    fn default() -> Self {
        Self {
            count: 0,
            window_start: Instant::now(),
            cooldown_level: 0,
            mute_until: None,
        }
    }
}

/// Servicio de timing: gestiona sesiones activas y rate limiting global
#[derive(Clone, Default)]
pub struct ChatTimingService {
    sessions: Arc<DashMap<Uuid, mpsc::Sender<TimingEvent>>>,
    rate_limits: Arc<DashMap<String, RateState>>,
}

impl ChatTimingService {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            rate_limits: Arc::new(DashMap::new()),
        }
    }

    /// Verifica rate limit para un `visitor_id`. Retorna el resultado y
    /// opcionalmente un mensaje de advertencia para enviar al visitante.
    #[must_use]
    pub fn check_rate(&self, visitor_id: &str) -> (RateCheckResult, Option<String>) {
        let mut entry = self
            .rate_limits
            .entry(visitor_id.to_string())
            .or_default();
        let state = entry.value_mut();

        /* Reset ventana si expiró */
        if state.window_start.elapsed() >= RATE_WINDOW {
            state.count = 0;
            state.window_start = Instant::now();
        }

        /* Verificar si está muteado actualmente */
        if let Some(until) = state.mute_until {
            if Instant::now() < until {
                return (
                    RateCheckResult::Muted,
                    Some("Has enviado demasiados mensajes. Espera un momento.".to_string()),
                );
            }
            state.mute_until = None;
        }

        state.count += 1;

        if state.count <= RATE_LIMIT_PER_MIN {
            return (RateCheckResult::Ok, None);
        }

        /* Exceso detectado — cooldown progresivo */
        state.cooldown_level += 1;
        match state.cooldown_level {
            1 => (
                RateCheckResult::Warning,
                Some("Por favor, escribe con calma. Estoy leyendo cada mensaje.".to_string()),
            ),
            2 => {
                state.mute_until = Some(Instant::now() + Duration::from_secs(30));
                (
                    RateCheckResult::Muted,
                    Some(
                        "Has enviado demasiados mensajes. Espera 30 segundos."
                            .to_string(),
                    ),
                )
            }
            _ => (
                RateCheckResult::Closed,
                Some(
                    "La sesión se ha cerrado por exceso de mensajes. Puedes volver más tarde."
                        .to_string(),
                ),
            ),
        }
    }

    /// Registra una sesión en el timing service. Devuelve el sender para
    /// enviar eventos desde el handler WS. Spawna la tarea de timing.
    /// [T-4] Si la sesión ya está registrada (otra pestaña/dispositivo), retorna
    /// el tx existente sin crear nueva tarea de timing.
    #[must_use]
    pub fn register_session(
        &self,
        session_id: Uuid,
        visitor_name: Option<String>,
        deps: TimingSessionDeps,
    ) -> mpsc::Sender<TimingEvent> {
        /* [T-4] Reutilizar timing loop existente si otra conexión ya registró la sesión */
        if let Some(existing) = self.sessions.get(&session_id) {
            return existing.clone();
        }

        let (tx, rx) = mpsc::channel::<TimingEvent>(64);
        self.sessions.insert(session_id, tx.clone());

        tokio::spawn(session_timing_loop(
            session_id,
            visitor_name,
            rx,
            deps.pool,
            deps.ai_config,
            deps.hub,
            deps.notification_hub,
            deps.http_client,
            deps.stripe_key,
            deps.visitor_id,
        ));

        tx
    }

    /// Elimina la sesión del tracking
    pub fn unregister_session(&self, session_id: Uuid) {
        self.sessions.remove(&session_id);
    }

    /// Enviar evento a una sesión existente (fallback si se registró antes)
    pub async fn send_event(
        &self,
        session_id: Uuid,
        event: TimingEvent,
    ) -> Result<(), &'static str> {
        if let Some(tx) = self.sessions.get(&session_id) {
            tx.send(event).await.map_err(|_| "channel closed")
        } else {
            Err("session not registered")
        }
    }
}

/* Máquina de estados del timing por sesión.
 * Recibe eventos, bufferea mensajes, decide cuándo responder.
 * Se ejecuta como tokio::spawn por cada sesión activa. */
#[allow(clippy::too_many_arguments)]
async fn session_timing_loop(
    session_id: Uuid,
    visitor_name: Option<String>,
    mut rx: mpsc::Receiver<TimingEvent>,
    pool: PgPool,
    ai_config: AiChatConfig,
    hub: ChatHub,
    notification_hub: NotificationHub,
    http_client: reqwest::Client,
    stripe_key: Option<String>,
    visitor_id: String,
) {
    let mut buffer: Vec<String> = Vec::new();
    let mut is_typing = false;
    let mut irrelevant_count: u32 = 0;

    loop {
        /* Estado IDLE: esperar primer mensaje */
        let Some(event) = rx.recv().await else {
            break; /* channel cerrado, sesión terminó */
        };

        match event {
            TimingEvent::Disconnect => break,
            TimingEvent::TypingStart => {
                is_typing = true;
                continue;
            }
            TimingEvent::TypingStop => {
                is_typing = false;
                continue;
            }
            TimingEvent::Message(content) => {
                buffer.push(content);
            }
        }

        /* Acumular mensajes hasta timeout */
        let first_msg = Instant::now();
        accumulate_messages(&mut rx, &mut buffer, &mut is_typing, first_msg).await;

        if buffer.is_empty() {
            continue;
        }

        /* Estado RESPONDING */
        let combined = buffer.join("\n");
        buffer.clear();

        irrelevant_count = generate_ai_response(
            session_id,
            visitor_name.as_deref(),
            &visitor_id,
            &combined,
            irrelevant_count,
            &pool,
            &ai_config,
            &hub,
            &notification_hub,
            &http_client,
            stripe_key.as_deref(),
        )
        .await;
    }

    /* [T-3] Al cerrar sesión, generar resumen de contexto para futuras conversaciones.
     * Usa modelo pequeño para resumir el historial y lo guarda en visitor_profiles. */
    tokio::spawn(generate_context_summary(
        pool.clone(),
        ai_config.clone(),
        session_id,
        visitor_id,
    ));
}

/* Acumula mensajes del buffer hasta que expira el timeout.
 * Gestiona transiciones WAITING ↔ LISTENING según typing indicators. */
async fn accumulate_messages(
    rx: &mut mpsc::Receiver<TimingEvent>,
    buffer: &mut Vec<String>,
    is_typing: &mut bool,
    first_msg: Instant,
) {
    loop {
        let deadline = if *is_typing {
            LISTEN_TIMEOUT
        } else {
            WAIT_TIMEOUT
        };

        let max_remaining = MAX_ACCUMULATION
            .checked_sub(first_msg.elapsed())
            .unwrap_or(Duration::ZERO);

        let effective_timeout = deadline.min(max_remaining);

        tokio::select! {
            () = tokio::time::sleep(effective_timeout) => {
                /* Timeout expirado. Si typing activo, esperar cooldown extra */
                if *is_typing {
                    tokio::time::sleep(TYPING_COOLDOWN).await;
                    drain_pending(rx, buffer, is_typing);
                }
                return; /* listo para responder */
            }
            Some(evt) = rx.recv() => {
                match evt {
                    TimingEvent::Message(c) => { buffer.push(c); }
                    TimingEvent::TypingStart => { *is_typing = true; }
                    TimingEvent::TypingStop => { *is_typing = false; }
                    TimingEvent::Disconnect => { buffer.clear(); return; }
                }
                /* Continuar acumulando */
            }
        }
    }
}

/* Drena eventos pendientes sin bloquear (para el cooldown post-typing) */
fn drain_pending(
    rx: &mut mpsc::Receiver<TimingEvent>,
    buffer: &mut Vec<String>,
    is_typing: &mut bool,
) {
    while let Ok(evt) = rx.try_recv() {
        match evt {
            TimingEvent::Message(c) => buffer.push(c),
            TimingEvent::TypingStart => *is_typing = true,
            TimingEvent::TypingStop => *is_typing = false,
            TimingEvent::Disconnect => {
                buffer.clear();
                return;
            }
        }
    }
}

/* Genera respuesta IA con el buffer combinado.
 * Verifica sesión activa, clasifica relevancia, genera respuesta y escala si necesario.
 * [T-2] Envía rich_messages (service_cards, invoices) como mensajes separados.
 * Retorna el nuevo valor de irrelevant_count. */
#[allow(clippy::too_many_arguments)]
async fn generate_ai_response(
    session_id: Uuid,
    visitor_name: Option<&str>,
    visitor_id: &str,
    combined: &str,
    mut irrelevant_count: u32,
    pool: &PgPool,
    ai_config: &AiChatConfig,
    hub: &ChatHub,
    notification_hub: &NotificationHub,
    http_client: &reqwest::Client,
    stripe_key: Option<&str>,
) -> u32 {
    /* Verificar que la sesión sigue con IA activa */
    let session_ok = match crate::repositories::ChatRepository::find_session_by_id(
        pool, session_id,
    )
    .await
    {
        Ok(Some(s)) => s.ai_enabled && s.assigned_staff_id.is_none(),
        _ => false,
    };

    if !session_ok {
        return irrelevant_count;
    }

    /* Clasificador de relevancia: filtrar off-topic con modelo pequeño */
    if let Ok(false) = check_relevance(pool, ai_config, combined).await {
        irrelevant_count += 1;
        let msg = if irrelevant_count >= MAX_IRRELEVANT_STREAK {
            irrelevant_count = 0;
            "Parece que tus consultas no están relacionadas con nuestros \
             servicios. Si necesitas ayuda con diseño web, desarrollo de apps, \
             branding o agentes IA, estoy aquí para ayudarte."
        } else {
            "Interesante. ¿Hay algo relacionado con nuestros servicios \
             en lo que pueda ayudarte? Ofrecemos diseño web, desarrollo \
             de aplicaciones, branding y agentes IA."
        };
        let _ = hub.send_message(session_id, "ai", Some("ai"), msg).await;
        return irrelevant_count;
    }

    /* Relevante: reset streak y generar respuesta principal */
    irrelevant_count = 0;

    let ai_resp = AiChatService::generate_response(
        pool, ai_config, http_client, stripe_key, session_id, Some(visitor_id), combined,
    )
    .await
    .unwrap_or_else(|e| AiResponse {
        text: format!("Error IA: {e}"),
        needs_escalation: true,
        rich_messages: Vec::new(),
    });

    /* [T-2] Enviar rich messages (service_cards, invoices) antes del texto */
    for rm in &ai_resp.rich_messages {
        let _ = hub
            .send_rich_message(
                session_id, "ai", Some("ai"),
                &rm.content, &rm.message_type, &rm.metadata,
            )
            .await;
    }

    let _ = hub
        .send_message(session_id, "ai", Some("ai"), &ai_resp.text)
        .await;

    if ai_resp.needs_escalation {
        send_escalation(pool, notification_hub, session_id, visitor_name).await;
    }

    irrelevant_count
}

/* [T-1] Clasificador de relevancia usando modelo pequeño (llama-3.1-8b-instant).
 * Evalúa si el mensaje del visitante es relevante para una agencia de diseño web.
 * Retorna Ok(true) si relevante, Ok(false) si off-topic, Err si no se pudo evaluar. */
async fn check_relevance(
    _pool: &PgPool,
    config: &AiChatConfig,
    content: &str,
) -> Result<bool, String> {
    if !config.is_configured() {
        return Ok(true); /* sin API keys, asumir relevante */
    }

    let relevance_model = std::env::var("AI_RELEVANCE_MODEL")
        .unwrap_or_else(|_| "llama-3.1-8b-instant".to_string());

    /* Desactivable con AI_RELEVANCE_ENABLED=false */
    if std::env::var("AI_RELEVANCE_ENABLED")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase()
        == "false"
    {
        return Ok(true);
    }

    let Some(api_key) = config.next_key() else {
        return Ok(true);
    };

    let body = serde_json::json!({
        "model": relevance_model,
        "messages": [
            {
                "role": "system",
                "content": "Determina si el siguiente mensaje de un usuario es relevante para \
                    una agencia de diseño web, desarrollo de aplicaciones, branding o agentes IA. \
                    Responde SOLO con 'sí' o 'no'. Considera relevante: consultas sobre precios, \
                    servicios, proyectos, soporte técnico, diseño, hosting, dominios, saludos, \
                    despedidas, y cualquier conversación normal de un potencial cliente. \
                    Considera irrelevante: spam, contenido adulto, temas políticos, \
                    promociones externas, solicitudes de hacking."
            },
            {
                "role": "user",
                "content": content
            }
        ],
        "temperature": 0.1,
        "max_tokens": 5
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&config.api_url)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Relevance check error: {e}"))?;

    if !resp.status().is_success() {
        return Ok(true); /* en caso de error, asumir relevante */
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Relevance parse error: {e}"))?;

    let answer = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("sí")
        .to_lowercase();

    Ok(answer.contains("sí") || answer.contains("si") || answer.contains("yes"))
}

/* Reutilizable: enviar notificación de escalación a todos los admins */
async fn send_escalation(
    pool: &PgPool,
    notification_hub: &NotificationHub,
    session_id: Uuid,
    visitor_name: Option<&str>,
) {
    let name = visitor_name.unwrap_or("Visitante");
    if let Ok(admin_ids) = UserRepository::admin_ids(pool).await {
        if !admin_ids.is_empty() {
            let base = CreateNotification {
                user_id: Uuid::nil(),
                notification_type: NOTIF_ESCALATION_NEEDED.to_string(),
                title: format!("Escalación: {name} necesita ayuda"),
                body: Some(
                    "La IA detectó que se requiere intervención humana en la sesión de chat."
                        .to_string(),
                ),
                link: Some(format!("/admin/chat?session={session_id}")),
                reference_type: Some("chat_session".to_string()),
                reference_id: Some(session_id),
            };
            let _ = notification_hub.notify_many(&admin_ids, &base).await;
            tracing::info!("Escalación enviada a {} admins para sesión {session_id}", admin_ids.len());
        }
    }
}

/* [T-3] Genera resumen de la conversación al cerrar sesión.
 * Usa modelo pequeño (llama-3.1-8b-instant) para resumir el historial
 * del visitante y lo guarda en visitor_profiles.context_summary.
 * Se ejecuta como tokio::spawn para no bloquear el cierre de WS. */
async fn generate_context_summary(
    pool: PgPool,
    config: AiChatConfig,
    session_id: Uuid,
    visitor_id: String,
) {
    if !config.is_configured() {
        return;
    }

    /* Obtener historial de la sesión */
    let messages = match crate::repositories::ChatRepository::list_messages(
        &pool, session_id, 50, 0,
    )
    .await
    {
        Ok(msgs) if !msgs.is_empty() => msgs,
        _ => return,
    };

    /* Construir transcript compacto */
    let mut transcript = String::new();
    for msg in &messages {
        let role = if msg.sender_type == "ai" { "Nakomi" } else { "Cliente" };
        let _ = writeln!(transcript, "{role}: {}", msg.content);
    }

    if transcript.len() < 100 {
        return;
    }

    let transcript_truncated = if transcript.len() > 3000 {
        &transcript[..3000]
    } else {
        &transcript
    };

    let Some(summary) = call_summary_api(&config, transcript_truncated).await else {
        return;
    };

    /* Cargar resumen previo y concatenar (max 2000 chars total) */
    let existing = match crate::repositories::ChatRepository::find_visitor_profile(
        &pool, &visitor_id,
    )
    .await
    {
        Ok(Some(p)) => p.context_summary.unwrap_or_default(),
        _ => String::new(),
    };

    let final_summary = if existing.is_empty() {
        summary
    } else {
        let combined = format!("{existing}\n---\n{summary}");
        if combined.len() > 2000 {
            combined[combined.len() - 2000..].to_string()
        } else {
            combined
        }
    };

    if let Err(e) =
        crate::repositories::ChatRepository::update_context_summary(&pool, &visitor_id, &final_summary)
            .await
    {
        tracing::warn!("Error guardando context summary para {visitor_id}: {e}");
    } else {
        tracing::info!("Context summary actualizado para visitor {visitor_id}");
    }
}

/* [T-3] Llama a la API de Groq con modelo ligero para generar resumen de sesión.
 * Retorna None si la API falla o el resumen está vacío. */
async fn call_summary_api(
    config: &AiChatConfig,
    transcript: &str,
) -> Option<String> {
    let summary_model = std::env::var("AI_RELEVANCE_MODEL")
        .unwrap_or_else(|_| "llama-3.1-8b-instant".to_string());

    let body = serde_json::json!({
        "model": summary_model,
        "messages": [
            {
                "role": "system",
                "content": "Genera un resumen conciso (máximo 500 caracteres) de esta conversación \
                 de chat de soporte. Incluye: qué necesitaba el cliente, qué servicios le interesaron, \
                 si se capturó email, si se generó factura, si quedó algo pendiente, y cualquier \
                 preferencia o dato relevante del cliente. Solo el resumen, sin formato especial."
            },
            {
                "role": "user",
                "content": transcript
            }
        ],
        "max_tokens": 200,
        "temperature": 0.3,
    });

    let key = config.next_key()?;
    let client = reqwest::Client::new();
    let resp = client
        .post(&config.api_url)
        .header("Authorization", format!("Bearer {key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            let s = json["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string();
            if s.is_empty() { None } else { Some(s) }
        }
        Ok(r) => {
            tracing::warn!("Context summary API error: {}", r.status());
            None
        }
        Err(e) => {
            tracing::warn!("Context summary request error: {e}");
            None
        }
    }
}
