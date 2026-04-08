/* [064A-29] AI Chat Service: integración con Groq API (OpenAI-compatible).
 * Rotacion de 3 API keys por mensaje (round-robin via AtomicUsize).
 * System prompt dinámico: pre-venta (servicios, precios) vs soporte de orden
 * (contexto de orden, fase actual, historial). Usa reqwest HTTP client. */

use std::fmt::Write;
use std::sync::atomic::{AtomicUsize, Ordering};

use sqlx::PgPool;
use uuid::Uuid;

use crate::repositories::{ChatRepository, OrderRepository};

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

        /* [064A-73] Whitelist de modelos permitidos. Si el env var tiene un modelo
         * no reconocido, fallback al default. Previene inyección de modelo arbitrario. */
        let allowed_models = [
            "llama-3.3-70b-versatile",
            "llama-3.1-8b-instant",
            "llama-3.2-90b-vision-preview",
            "mixtral-8x7b-32768",
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
    fn next_key(&self) -> Option<&str> {
        if self.api_keys.is_empty() {
            return None;
        }
        let idx = KEY_COUNTER.fetch_add(1, Ordering::Relaxed) % self.api_keys.len();
        Some(&self.api_keys[idx])
    }
}

pub struct AiChatService;

impl AiChatService {
    /// Genera respuesta de IA para un mensaje en una sesión.
    /// Usa Groq API (OpenAI-compatible) con rotación de keys y retry automático.
    pub async fn generate_response(
        pool: &PgPool,
        config: &AiChatConfig,
        session_id: Uuid,
        user_message: &str,
    ) -> Result<String, String> {
        if !config.is_configured() {
            tracing::warn!("AI: sin API keys configuradas, usando fallback");
            return Ok(
                "Un miembro del equipo se conectará pronto para ayudarte. \
                 Mientras tanto, ¿en qué puedo orientarte?"
                    .to_string(),
            );
        }

        let system_prompt = build_system_prompt(pool, session_id).await;

        let history = ChatRepository::list_messages(pool, session_id, 20, 0)
            .await
            .unwrap_or_default();

        let mut messages = vec![serde_json::json!({
            "role": "system",
            "content": system_prompt
        })];

        for msg in &history {
            let role = if msg.sender_type == "ai" {
                "assistant"
            } else {
                "user"
            };
            messages.push(serde_json::json!({
                "role": role,
                "content": msg.content
            }));
        }

        messages.push(serde_json::json!({
            "role": "user",
            "content": user_message
        }));

        let body = serde_json::json!({
            "model": config.model,
            "messages": messages,
            "temperature": 0.7,
            "max_tokens": 800,
            "top_p": 0.9
        });

        /* [064A-69] Retry con todas las keys disponibles antes de caer al fallback.
         * Si key N falla con error HTTP, intenta key N+1, N+2, etc. */
        let client = reqwest::Client::new();
        let num_keys = config.api_keys.len();
        let mut last_error = String::new();

        for attempt in 0..num_keys {
            let Some(api_key) = config.next_key() else {
                break;
            };

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
                    tracing::error!("AI key {key_hint}... intento {}/{num_keys}: error red: {e}", attempt + 1);
                    last_error = format!("Error de red: {e}");
                }
                Ok(response) => {
                    if !response.status().is_success() {
                        let status = response.status();
                        let text = response.text().await.unwrap_or_default();
                        tracing::error!("AI key {key_hint}... intento {}/{num_keys}: HTTP {status}: {text}", attempt + 1);
                        last_error = format!("HTTP {status}");
                        continue;
                    }

                    let json: serde_json::Value = match response.json().await {
                        Ok(j) => j,
                        Err(e) => {
                            tracing::error!("AI: error parseando JSON: {e}");
                            last_error = format!("Parse error: {e}");
                            continue;
                        }
                    };

                    let text = json["choices"][0]["message"]["content"]
                        .as_str()
                        .unwrap_or("");

                    if text.is_empty() {
                        tracing::warn!("AI: respuesta vacía de la API, reintentando");
                        last_error = "Respuesta vacía".to_string();
                        continue;
                    }

                    return Ok(text.to_string());
                }
            }
        }

        tracing::error!("AI: todas las keys fallaron ({num_keys} intentos). Último error: {last_error}");
        Ok(
            "Disculpa, estamos teniendo problemas técnicos momentáneos. \
             Un miembro del equipo te atenderá en breve."
                .to_string(),
        )
    }
}

/// Construye system prompt dinámico según contexto de la sesión
async fn build_system_prompt(pool: &PgPool, session_id: Uuid) -> String {
    let mut prompt = String::from(
        /* [T-7] El agente NO debe revelar que es IA. Se presenta como miembro del equipo.
         * Esto aumenta la confianza del visitante y evita sesgos contra chatbots. */
        "Eres un miembro del equipo de Nakomi Studio, una agencia de desarrollo web y diseño. \
         Tu nombre es Nakomi. Responde de forma concisa, amable y profesional en el mismo \
         idioma que el usuario. Nunca menciones que eres una inteligencia artificial, chatbot, \
         bot, modelo de lenguaje o asistente virtual. Si te preguntan directamente si eres IA, \
         responde que eres parte del equipo de atención de Nakomi Studio. \
         Si la pregunta requiere atención especializada (presupuesto específico, problema \
         técnico complejo, reunión), indica que derivarás al especialista del equipo.\n\n"
    );

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

    prompt
}
