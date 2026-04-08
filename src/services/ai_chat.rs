/* [064A-29] AI Chat Service: integración con Groq API (OpenAI-compatible).
 * Rotacion de 3 API keys por mensaje (round-robin via AtomicUsize).
 * System prompt dinámico: pre-venta (servicios, precios) vs soporte de orden
 * (contexto de orden, fase actual, historial). Usa reqwest HTTP client. */

use std::fmt::Write;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::repositories::{ChatRepository, HostingRepository, OrderRepository, UserRepository};
use crate::models::{ChatMessage, Order};
use crate::services::ai_tools::{self, RichMessage};

/* [064A-29] Contador global para rotacion round-robin de API keys.
 * Cada llamada a generate_response incrementa y usa mod num_keys. */
static KEY_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Configuración del servicio de IA (Groq con rotación de keys)
#[derive(Clone)]
pub struct AiChatConfig {
    pub api_keys: Vec<String>,
    pub model: String,
    pub api_url: String,
}

impl AiChatConfig {
    /// Carga config desde variables de entorno. Soporta `GROQ_API_1`, `GROQ_API_2`, `GROQ_API_3`.
    /// Fallback a `AI_API_KEY`/`GEMINI_API_KEY`/`OPENAI_API_KEY` si no hay keys Groq.
    #[must_use]
    pub fn from_env() -> Self {
        let mut keys = Vec::new();
        for var in ["GROQ_API_1", "GROQ_API_2", "GROQ_API_3", "GROQ_API"] {
            if let Ok(k) = std::env::var(var) {
                if !k.is_empty() && !keys.contains(&k) {
                    keys.push(k);
                }
            }
        }
        /* Fallback a keys genéricas si no hay Groq */
        if keys.is_empty() {
            for var in ["AI_API_KEY", "GEMINI_API_KEY", "OPENAI_API_KEY"] {
                if let Ok(k) = std::env::var(var) {
                    if !k.is_empty() {
                        keys.push(k);
                        break;
                    }
                }
            }
        }

        let model = std::env::var("AI_MODEL")
            .unwrap_or_else(|_| "llama-3.3-70b-versatile".to_string());

        /* [084A-27] Whitelist de modelos Groq, ordenada por inteligencia descendente.
         * Los modelos más capaces primero — el sistema los usa como cadena de fallback
         * cuando el modelo primario falla por rate limit (429). */
        let allowed_models = [
            "llama-3.3-70b-versatile",
            "llama-3.2-90b-vision-preview",
            "mixtral-8x7b-32768",
            "llama-3.1-8b-instant",
            "gemma2-9b-it",
        ];
        let model = if allowed_models.contains(&model.as_str()) {
            model
        } else {
            tracing::warn!("Modelo AI no permitido: {model}, usando default");
            "llama-3.3-70b-versatile".to_string()
        };

        let api_url = std::env::var("AI_API_URL").unwrap_or_else(|_| {
            "https://api.groq.com/openai/v1/chat/completions".to_string()
        });

        Self {
            api_keys: keys,
            model,
            api_url,
        }
    }

    #[must_use]
    pub fn is_configured(&self) -> bool {
        !self.api_keys.is_empty()
    }

    /* [064A-29] Selecciona la siguiente API key en rotacion round-robin */
    pub(crate) fn next_key(&self) -> Option<&str> {
        if self.api_keys.is_empty() {
            return None;
        }
        let idx = KEY_COUNTER.fetch_add(1, Ordering::Relaxed) % self.api_keys.len();
        Some(&self.api_keys[idx])
    }

    /* [084A-27] Cadena de modelos fallback ordenados por inteligencia.
     * Cuando el modelo primario falla por rate limit (429), se intenta el siguiente.
     * El modelo primario (self.model) va primero, seguido por los demás. */
    fn model_fallback_chain(&self) -> Vec<&str> {
        let all_models = [
            "llama-3.3-70b-versatile",
            "llama-3.2-90b-vision-preview",
            "mixtral-8x7b-32768",
            "llama-3.1-8b-instant",
        ];
        let mut chain: Vec<&str> = vec![self.model.as_str()];
        for m in &all_models {
            if *m != self.model.as_str() {
                chain.push(m);
            }
        }
        chain
    }
}

pub struct AiChatService;

/* [T-6] Respuesta de IA con señal de escalación y mensajes ricos (T-2).
 * `needs_escalation` = true si la IA detecta que necesita intervención humana.
 * `rich_messages` contiene service_cards, invoices, etc. generados por tool calls. */
pub struct AiResponse {
    pub text: String,
    pub needs_escalation: bool,
    pub rich_messages: Vec<RichMessage>,
}

/* [T-9] Contexto de la sesión de chat para generate_response.
 * Agrupa parámetros relacionados para no exceder 7 argumentos. */
pub struct AiSessionContext<'a> {
    pub session_id: Uuid,
    pub visitor_id: Option<&'a str>,
    pub user_id: Option<Uuid>,
}

impl AiChatService {
    /// Genera respuesta de IA con soporte para tool use (function calling).
    /// Loop: envía mensajes → si la IA llama tools → ejecuta → reenvía resultados → repite.
    /// Máximo 3 iteraciones de tool calls para prevenir loops infinitos.
    pub async fn generate_response(
        pool: &PgPool,
        config: &AiChatConfig,
        http_client: &reqwest::Client,
        stripe_key: Option<&str>,
        ctx: AiSessionContext<'_>,
        user_message: &str,
    ) -> Result<AiResponse, String> {
        if !config.is_configured() {
            tracing::warn!("AI: sin API keys configuradas, usando fallback");
            return Ok(AiResponse {
                text: "Un miembro del equipo se conectará pronto para ayudarte. \
                       Mientras tanto, ¿en qué puedo orientarte?"
                    .to_string(),
                needs_escalation: true,
                rich_messages: Vec::new(),
            });
        }

        let system_prompt = build_system_prompt(pool, ctx.session_id, ctx.visitor_id, ctx.user_id).await;
        let history = ChatRepository::list_messages(pool, ctx.session_id, 20, 0)
            .await
            .unwrap_or_default();

        let mut messages = vec![serde_json::json!({"role": "system", "content": system_prompt})];
        for msg in &history {
            let role = if msg.sender_type == "ai" { "assistant" } else { "user" };
            messages.push(serde_json::json!({"role": role, "content": msg.content}));
        }
        messages.push(serde_json::json!({"role": "user", "content": user_message}));

        let tools = ai_tools::tool_definitions();
        let mut rich_messages: Vec<RichMessage> = Vec::new();
        let mut needs_escalation = false;

        /* [T-2] Tool call loop: máximo 3 iteraciones */
        for iteration in 0..3 {
            let resp = call_groq_api(config, &messages, Some(&tools)).await?;

            let choice = &resp["choices"][0];
            let tool_calls = &choice["message"]["tool_calls"];

            if tool_calls.is_array() && !tool_calls.as_array().unwrap().is_empty() {
                /* La IA quiere llamar tools — ejecutarlas */
                messages.push(choice["message"].clone());
                let tool_results = process_tool_calls(
                    pool, http_client, stripe_key, ctx.visitor_id, tool_calls, &mut rich_messages,
                ).await;

                if tool_results.iter().any(|r| r.contains("\"status\":\"escalated\"")) {
                    needs_escalation = true;
                }

                for (tool_call_id, result) in tool_results_with_ids(tool_calls, &tool_results) {
                    messages.push(serde_json::json!({
                        "role": "tool",
                        "tool_call_id": tool_call_id,
                        "content": result
                    }));
                }

                tracing::debug!("AI tool call iteration {}, {} tools executed", iteration + 1, tool_results.len());
                continue;
            }

            /* Respuesta de texto final (sin tool calls) */
            let text = choice["message"]["content"].as_str().unwrap_or("");
            if text.is_empty() {
                return Err("AI: respuesta vacía después de tool calls".to_string());
            }

            let (clean_text, text_escalation) = parse_escalation(text);
            return Ok(AiResponse {
                text: clean_text,
                needs_escalation: needs_escalation || text_escalation,
                rich_messages,
            });
        }

        /* Si agotamos iteraciones de tools, generar sin tools */
        let resp = call_groq_api(config, &messages, None).await?;
        let text = resp["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("Disculpa, hubo un problema procesando tu solicitud.");
        let (clean_text, text_escalation) = parse_escalation(text);
        Ok(AiResponse {
            text: clean_text,
            needs_escalation: needs_escalation || text_escalation,
            rich_messages,
        })
    }

    /* [T-10] Genera respuesta IA como intermediario de una orden.
     * System prompt especial con contexto completo del pedido. */
    pub async fn generate_intermediary_response(
        pool: &PgPool,
        config: &AiChatConfig,
        _http_client: &reqwest::Client,
        session_id: Uuid,
        order: &Order,
        user_id: Uuid,
        _user_message: &str,
    ) -> Result<AiResponse, String> {
        if !config.is_configured() {
            return Ok(AiResponse {
                text: "El equipo responderá pronto.".to_string(),
                needs_escalation: false,
                rich_messages: Vec::new(),
            });
        }

        let system_prompt = build_intermediary_prompt(pool, order, user_id).await;
        let history = ChatRepository::list_messages(pool, session_id, 20, 0)
            .await
            .unwrap_or_default();

        let mut messages = vec![serde_json::json!({"role": "system", "content": system_prompt})];
        for msg in &history {
            let role = match msg.sender_type.as_str() {
                "ai" | "ai_intermediary" => "assistant",
                _ => "user",
            };
            messages.push(serde_json::json!({"role": role, "content": msg.content}));
        }

        let resp = call_groq_api(config, &messages, None).await?;
        let text = resp["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("Disculpa, no pude procesar tu mensaje. Un miembro del equipo te asistirá.");
        let (clean_text, escalation) = parse_escalation(text);
        Ok(AiResponse {
            text: clean_text,
            needs_escalation: escalation,
            rich_messages: Vec::new(),
        })
    }

    /* [T-10c] Genera resumen de la conversación de la orden si hay >5 msgs.
     * Usa modelo ligero (8b), reemplaza ai_summary de la orden. */
    pub async fn maybe_update_order_summary(
        pool: &PgPool,
        config: &AiChatConfig,
        http_client: &reqwest::Client,
        order_id: Uuid,
        msgs: &[ChatMessage],
    ) {
        let Some(api_key) = config.next_key() else { return };

        let mut conversation = String::new();
        for m in msgs.iter().take(50) {
            let _ = writeln!(conversation, "[{}]: {}", m.sender_type, m.content);
        }

        let prompt = format!(
            "Resume esta conversación de soporte de pedido en máximo 200 palabras. \
             Incluye: solicitudes del cliente, cambios pedidos, estado emocional, \
             acciones pendientes.\n\nConversación:\n{conversation}"
        );

        let body = serde_json::json!({
            "model": "llama-3.1-8b-instant",
            "messages": [
                {"role": "system", "content": "Eres un asistente que genera resúmenes concisos de conversaciones de soporte."},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.3,
            "max_tokens": 400
        });

        let resp = http_client
            .post(&config.api_url)
            .header("Authorization", format!("Bearer {api_key}"))
            .json(&body)
            .send()
            .await;

        if let Ok(r) = resp {
            if let Ok(json) = r.json::<Value>().await {
                if let Some(summary) = json["choices"][0]["message"]["content"].as_str() {
                    let _ = OrderRepository::update_ai_summary(pool, order_id, summary).await;
                }
            }
        }
    }
}

/* [084A-27] Llamar a Groq API con retry por keys Y fallback de modelo.
 * Primero intenta todas las keys con el modelo primario.
 * Si todas fallan por rate limit (429), pasa al siguiente modelo en la cadena.
 * Esto maximiza el uso de los modelos más inteligentes antes de degradar. */
async fn call_groq_api(
    config: &AiChatConfig,
    messages: &[Value],
    tools: Option<&Value>,
) -> Result<Value, String> {
    let fallback_chain = config.model_fallback_chain();
    let client = reqwest::Client::new();
    let num_keys = config.api_keys.len();
    let mut last_error = String::new();

    for model in &fallback_chain {
        let mut body = serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": 0.7,
            "max_tokens": 800,
            "top_p": 0.9
        });
        if let Some(t) = tools {
            body["tools"] = t.clone();
        }

        let mut all_rate_limited = true;

        for attempt in 0..num_keys {
            let Some(api_key) = config.next_key() else { break };
            let key_hint = &api_key[..api_key.len().min(8)];

            let resp = client
                .post(&config.api_url)
                .header("Authorization", format!("Bearer {api_key}"))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await;

            match resp {
                Err(e) => {
                    tracing::error!("AI [{model}] key {key_hint}... intento {}/{num_keys}: error red: {e}", attempt + 1);
                    last_error = format!("Error de red: {e}");
                    all_rate_limited = false;
                }
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        let json: Value = response.json().await
                            .map_err(|e| format!("AI parse error: {e}"))?;
                        if model != &fallback_chain[0] {
                            tracing::info!("AI fallback exitoso: usando modelo {model}");
                        }
                        return Ok(json);
                    }
                    let text = response.text().await.unwrap_or_default();
                    tracing::error!("AI [{model}] key {key_hint}... intento {}/{num_keys}: HTTP {status}: {text}", attempt + 1);
                    last_error = format!("HTTP {status}");

                    /* Solo hacer fallback de modelo si TODAS las keys dan 429 */
                    if status.as_u16() != 429 {
                        all_rate_limited = false;
                    }
                }
            }
        }

        /* Si no todas fueron rate limit, no tiene sentido probar otro modelo */
        if !all_rate_limited {
            break;
        }
        tracing::warn!("AI modelo {model}: todas las keys con rate limit (429), probando siguiente modelo");
    }

    Err(format!("AI: todos los modelos y keys fallaron. Último error: {last_error}"))
}

/* [T-2] Ejecutar tool calls y recopilar rich messages.
 * Retorna Vec<String> con los resultados JSON de cada tool.
 * [T-3] visitor_id para tools que actualizan visitor_profiles. */
async fn process_tool_calls(
    pool: &PgPool,
    http_client: &reqwest::Client,
    stripe_key: Option<&str>,
    visitor_id: Option<&str>,
    tool_calls: &Value,
    rich_messages: &mut Vec<RichMessage>,
) -> Vec<String> {
    let Some(calls) = tool_calls.as_array() else {
        return Vec::new();
    };

    let mut results = Vec::with_capacity(calls.len());
    for call in calls {
        let name = call["function"]["name"].as_str().unwrap_or("");
        let args_str = call["function"]["arguments"].as_str().unwrap_or("{}");
        let args: Value = serde_json::from_str(args_str).unwrap_or_default();

        let result = ai_tools::execute_tool(pool, http_client, stripe_key, visitor_id, name, &args).await;
        if let Some(rm) = result.rich_message {
            rich_messages.push(rm);
        }
        results.push(result.tool_result_json);
    }
    results
}

/* Emparejar IDs de tool_calls con sus resultados */
fn tool_results_with_ids<'a>(
    tool_calls: &'a Value,
    results: &'a [String],
) -> Vec<(&'a str, &'a str)> {
    let Some(calls) = tool_calls.as_array() else {
        return Vec::new();
    };
    calls
        .iter()
        .zip(results.iter())
        .map(|(call, result)| {
            let id = call["id"].as_str().unwrap_or("");
            (id, result.as_str())
        })
        .collect()
}

/* [T-6] Detecta y elimina el tag [ESCALATE] de la respuesta de la IA.
 * Retorna (texto limpio, necesita_escalación). */
fn parse_escalation(raw: &str) -> (String, bool) {
    let trimmed = raw.trim();
    if trimmed.starts_with("[ESCALATE]") {
        let clean = trimmed.trim_start_matches("[ESCALATE]").trim().to_string();
        (clean, true)
    } else {
        (trimmed.to_string(), false)
    }
}

/// Construye system prompt dinámico según contexto de la sesión, visitante y usuario autenticado
async fn build_system_prompt(
    pool: &PgPool,
    session_id: Uuid,
    visitor_id: Option<&str>,
    /* [T-9] user_id para clientes autenticados — agrega contexto de pedidos/hosting */
    user_id: Option<Uuid>,
) -> String {
    let mut prompt = String::from(
        /* [T-7] El agente NO debe revelar que es IA. Se presenta como miembro del equipo.
         * [T-2] Incluye instrucciones de tool use para mostrar servicios y crear facturas.
         * [T-3] Herramientas de captura de email e info del cliente. */
        "Eres un miembro del equipo de Nakomi Studio, una agencia de desarrollo web y diseño. \
         Tu nombre es Nakomi. Responde de forma concisa, amable y profesional en el mismo \
         idioma que el usuario. Nunca menciones que eres una inteligencia artificial, chatbot, \
         bot, modelo de lenguaje o asistente virtual. Si te preguntan directamente si eres IA, \
         responde que eres parte del equipo de atención de Nakomi Studio. \
         Si la pregunta requiere atención especializada (presupuesto específico, problema \
         técnico complejo, reunión), indica que derivarás al especialista del equipo.\n\n\
         HERRAMIENTAS DISPONIBLES:\n\
         - show_service: Muestra una tarjeta visual de servicio al cliente. Úsala cuando menciones un servicio específico.\n\
         - list_services: Lista todos los servicios. Úsala cuando el cliente pida ver opciones.\n\
         - create_invoice: Genera factura con link de pago. SOLO cuando el cliente confirme que quiere pagar y tengas su email.\n\
         - request_human_assistance: Escala a un humano. Úsala en los casos de la REGLA DE ESCALACIÓN.\n\
         - capture_email: Guarda el email del cliente. Úsala cuando el cliente comparta su correo de forma natural.\n\
         - save_client_info: Guarda info relevante del cliente (industria, presupuesto, intereses). Úsala cuando \
           el cliente mencione datos útiles sobre su negocio o proyecto.\n\
         Usa las herramientas de forma natural en la conversación. No menciones las herramientas directamente al cliente.\n\n\
         CAPTURA DE EMAIL: No pidas el email de forma forzada. Después de 2-3 intercambios productivos, puedes \
         preguntar de forma natural: '¿Me compartes tu correo para enviarte la información?' Si el cliente lo da, \
         usa capture_email inmediatamente. Si no quiere, no insistas.\n\n\
         CAPTURA DE INFO: Cuando el cliente mencione su industria, presupuesto, tipo de proyecto o necesidades \
         específicas, usa save_client_info para guardar esos datos. Esto ayuda a personalizar futuras conversaciones.\n\n\
         REGLA DE ESCALACIÓN: Si detectas alguna de estas situaciones, usa request_human_assistance O inicia tu respuesta \
         con el tag exacto [ESCALATE] (el tag se eliminará antes de mostrarse al usuario):\n\
         - El cliente pide hablar con un humano\n\
         - El cliente está frustrado o insatisfecho después de varias respuestas\n\
         - El tema es legal, contractual, o sobre disputas de pago\n\
         - No puedes resolver la solicitud con la información disponible\n\
         - El cliente reporta un problema técnico urgente que requiere intervención manual\n\n"
    );

    /* [T-3] Agregar contexto del visitante si tiene perfil previo */
    if let Some(vid) = visitor_id {
        append_visitor_context(&mut prompt, pool, vid).await;
    }

    /* Agregar contexto de servicios */
    if let Ok(services) = OrderRepository::list_services(pool).await {
        prompt.push_str("Servicios disponibles:\n");
        for svc in &services {
            let _ = writeln!(
                prompt,
                "- {}: desde ${:.2} USD",
                svc.title,
                f64::from(svc.base_price_cents) / 100.0
            );
        }
        prompt.push('\n');
    }

    /* Contexto de orden si la sesión está vinculada */
    if let Ok(Some(session)) = ChatRepository::find_session_by_id(pool, session_id).await {
        if let Some(order_id) = session.order_id {
            if let Ok(Some(order)) = OrderRepository::find_order_by_id(pool, order_id).await {
                if let Ok((svc_title, _, plan_name)) =
                    OrderRepository::get_order_display_info(pool, order.service_id, order.plan_id)
                        .await
                {
                    let _ = write!(
                        prompt,
                        "CONTEXTO DE ORDEN ACTIVA:\n\
                         - Orden #{}: {} ({})\n\
                         - Estado: {:?}\n\
                         - Fase actual: {}/{}\n\
                         El usuario tiene una orden activa. Responde preguntas sobre su \
                         progreso y ofrece ayuda específica.\n",
                        order.order_number,
                        svc_title,
                        plan_name,
                        order.status,
                        order.current_phase,
                        0 /* total phases retrieved if needed */
                    );
                }
            }
        }
    }

    /* [T-9] Contexto de cliente registrado (autenticado con JWT).
     * Incluye datos del usuario, pedidos activos y suscripciones de hosting. */
    if let Some(uid) = user_id {
        append_registered_client_context(&mut prompt, pool, uid).await;
    }

    prompt
}

/* [T-9] Helper: agrega contexto del visitante (perfil previo) al system prompt */
async fn append_visitor_context(prompt: &mut String, pool: &PgPool, visitor_id: &str) {
    if let Ok(Some(profile)) = ChatRepository::find_visitor_profile(pool, visitor_id).await {
        prompt.push_str("CONTEXTO DEL VISITANTE (conversaciones anteriores):\n");
        if let Some(name) = &profile.display_name {
            let _ = writeln!(prompt, "- Nombre: {name}");
        }
        if let Some(email) = &profile.email {
            let _ = writeln!(prompt, "- Email: {email} (ya capturado, no volver a pedir)");
        }
        if profile.total_sessions > 1 {
            let _ = writeln!(prompt, "- Visitas anteriores: {}", profile.total_sessions);
        }
        if let Some(summary) = &profile.context_summary {
            if !summary.is_empty() {
                let _ = writeln!(prompt, "- Resumen de conversaciones previas: {summary}");
            }
        }
        if let Some(prefs) = &profile.preferences {
            if let Some(obj) = prefs.as_object() {
                if !obj.is_empty() {
                    let _ = writeln!(prompt, "- Info del cliente: {prefs}");
                }
            }
        }
        prompt.push_str("Usa esta información para personalizar la atención. Si el visitante \
                        vuelve, salúdalo por su nombre si lo conoces.\n\n");
    }
}

/* [T-9] Helper: agrega contexto de cliente registrado (usuario, pedidos, hosting) */
async fn append_registered_client_context(prompt: &mut String, pool: &PgPool, uid: Uuid) {
    let Ok(Some(user)) = UserRepository::find_by_id(pool, uid).await else { return };
    prompt.push_str("CLIENTE REGISTRADO:\n");
    let display = user.display_name.as_deref().unwrap_or(&user.username);
    let _ = writeln!(prompt, "- Nombre: {display} ({email})", email = user.email);
    let _ = writeln!(prompt, "- Rol: {:?}", user.role);
    prompt.push_str("Ya está registrado — no pedir email ni nombre.\n\n");

    /* Pedidos del cliente */
    if let Ok(orders) = OrderRepository::list_orders_for_client(pool, uid).await {
        if !orders.is_empty() {
            prompt.push_str("PEDIDOS DEL CLIENTE:\n");
            for order in orders.iter().take(5) {
                let svc_info = OrderRepository::get_order_display_info(
                    pool, order.service_id, order.plan_id,
                )
                .await
                .ok();
                let svc_title = svc_info
                    .as_ref()
                    .map_or("Servicio", |(t, _, _)| t.as_str());
                let plan_name = svc_info
                    .as_ref()
                    .map_or("", |(_, _, p)| p.as_str());
                let _ = writeln!(
                    prompt,
                    "- Orden #{}: {} ({}) — Estado: {:?}, Fase: {}",
                    order.order_number, svc_title, plan_name,
                    order.status, order.current_phase,
                );
            }
            prompt.push_str("Puedes responder preguntas sobre el estado de sus pedidos.\n\n");
        }
    }

    /* Suscripciones de hosting */
    if let Ok(hostings) = HostingRepository::list_by_user_id(pool, uid).await {
        if !hostings.is_empty() {
            prompt.push_str("HOSTING DEL CLIENTE:\n");
            for h in &hostings {
                let domain = h.domain.as_deref().unwrap_or("sin dominio");
                let _ = writeln!(
                    prompt,
                    "- Plan: {} — Dominio: {domain} — Estado: {}",
                    h.plan, h.status,
                );
            }
            prompt.push_str("Puedes responder preguntas sobre el estado de su hosting.\n\n");
        }
    }
}

/* [T-10] Construye system prompt para IA intermediaria de una orden.
 * Incluye contexto completo del pedido: servicio, plan, fases, notas, empleado asignado. */
async fn build_intermediary_prompt(pool: &PgPool, order: &Order, user_id: Uuid) -> String {
    let mut prompt = String::new();

    /* Info del servicio y plan */
    let (svc_title, _, plan_name) = OrderRepository::get_order_display_info(
        pool, order.service_id, order.plan_id,
    )
    .await
    .unwrap_or_else(|_| ("Servicio".to_string(), String::new(), "Plan".to_string()));

    let employee_name = OrderRepository::get_employee_display_name(
        pool, order.assigned_employee_id,
    )
    .await
    .ok()
    .flatten()
    .unwrap_or_else(|| "No asignado".to_string());

    let _ = write!(
        prompt,
        "Eres un intermediario de atención al cliente de Nakomi Studio para el pedido #{num}.\n\
         Servicio: {svc_title} — Plan: {plan_name}\n\
         Estado: {status:?} — Fase actual: {phase}\n\
         Empleado asignado: {employee_name}\n",
        num = order.order_number,
        status = order.status,
        phase = order.current_phase,
    );

    /* Notas del cliente e internas */
    if let Some(notes) = &order.client_notes {
        if !notes.is_empty() {
            let _ = writeln!(prompt, "Notas del cliente: {notes}");
        }
    }
    if let Some(notes) = &order.internal_notes {
        if !notes.is_empty() {
            let _ = writeln!(prompt, "Notas internas: {notes}");
        }
    }

    /* Historial de fases */
    if let Ok(phases) = OrderRepository::list_order_phases(pool, order.id).await {
        if !phases.is_empty() {
            prompt.push_str("Fases del pedido:\n");
            for p in &phases {
                let _ = writeln!(
                    prompt,
                    "  Fase {}: {} — {:?}",
                    p.phase_number, p.title, p.status,
                );
            }
        }
    }

    /* Info del cliente */
    if let Ok(Some(user)) = UserRepository::find_by_id(pool, user_id).await {
        let display = user.display_name.as_deref().unwrap_or(&user.username);
        let _ = writeln!(prompt, "Cliente: {display} ({email})", email = user.email);
    }

    prompt.push_str(
        "\nIMPORTANTE: Eres un intermediario. Tu rol es:\n\
         1. Responder preguntas del cliente sobre el estado del pedido\n\
         2. Recopilar solicitudes y cambios pedidos por el cliente\n\
         3. Generar información útil para el equipo\n\
         4. Escalar al empleado si requiere acción humana (usa [ESCALATE])\n\
         No tomes decisiones sobre el trabajo — solo comunica y documenta.\n\
         Responde de forma concisa, amable y profesional. Nunca menciones que eres IA.\n",
    );

    prompt
}
