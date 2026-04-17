/* [174A-2] Proveedores de API AI (Groq, Gemini). Extraído de ai_chat.rs para SRP.
 * Contiene: llamadas HTTP a APIs, retry multi-proveedor, construcción de body. */

use serde_json::Value;

use super::ai_chat::AiChatConfig;

/* [084A-27+084A-37] Llamar a API AI con retry multi-proveedor.
 * Flujo: Groq (modelos × keys) → Gemini (modelos × 1 key). */
pub(crate) async fn call_groq_api(
    config: &AiChatConfig,
    messages: &[Value],
    tools: Option<&Value>,
) -> Result<Value, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();
    let mut last_error = String::new();

    if !config.api_keys.is_empty() {
        match try_groq_provider(config, &client, messages, tools).await {
            Ok(json) => return Ok(json),
            Err(e) => last_error = e,
        }
    }

    if config.gemini_key.is_some() {
        match try_gemini_provider(config, &client, messages, tools).await {
            Ok(json) => return Ok(json),
            Err(e) => last_error = e,
        }
    }

    Err(format!("AI: todos los proveedores fallaron. Último error: {last_error}"))
}

/* Intenta Groq con rotación de modelos (fallback_chain) × keys (round-robin). */
async fn try_groq_provider(
    config: &AiChatConfig,
    client: &reqwest::Client,
    messages: &[Value],
    tools: Option<&Value>,
) -> Result<Value, String> {
    let fallback_chain = config.model_fallback_chain();
    let num_keys = config.api_keys.len();
    let mut last_error = String::new();

    for model in &fallback_chain {
        let body = build_chat_body(model, messages, tools);
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
                    tracing::error!("AI Groq [{model}] key {key_hint}... intento {}/{num_keys}: error red: {e}", attempt + 1);
                    last_error = format!("Groq error de red: {e}");
                    all_rate_limited = false;
                }
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        let json: Value = response.json().await
                            .map_err(|e| format!("AI parse error: {e}"))?;
                        if model == &fallback_chain[0] {
                            tracing::info!("AI OK: proveedor=Groq, modelo={model}, key={key_hint}..., intento={}", attempt + 1);
                        } else {
                            tracing::info!("AI fallback Groq: modelo={model}, key={key_hint}..., intento={}", attempt + 1);
                        }
                        return Ok(json);
                    }
                    let text = response.text().await.unwrap_or_default();
                    tracing::error!("AI Groq [{model}] key {key_hint}... intento {}/{num_keys}: HTTP {status}: {text}", attempt + 1);
                    last_error = format!("Groq HTTP {status}");

                    if status.as_u16() != 429 {
                        all_rate_limited = false;
                    }
                }
            }
        }

        if !all_rate_limited {
            break;
        }
        tracing::warn!("AI Groq modelo {model}: todas las keys con rate limit (429), probando siguiente modelo");
    }

    Err(last_error)
}

/* [084A-37] Intenta Google Gemini como proveedor secundario OpenAI-compatible. */
async fn try_gemini_provider(
    config: &AiChatConfig,
    client: &reqwest::Client,
    messages: &[Value],
    tools: Option<&Value>,
) -> Result<Value, String> {
    let gemini_key = config.gemini_key.as_deref().ok_or("Gemini no configurado")?;
    let gemini_models = [
        "gemini-2.5-flash",
        "gemini-2.5-flash-lite",
        "gemini-2.5-pro",
        "gemini-3-flash-preview",
        "gemini-3.1-pro-preview",
        "gemini-3.1-flash-lite-preview",
    ];
    let key_hint = &gemini_key[..gemini_key.len().min(8)];
    let mut last_error = String::new();

    for model in &gemini_models {
        let body = build_chat_body(model, messages, tools);

        let resp = client
            .post(&config.gemini_url)
            .header("Authorization", format!("Bearer {gemini_key}"))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await;

        match resp {
            Err(e) => {
                tracing::error!("AI Gemini [{model}] key {key_hint}...: error red: {e}");
                last_error = format!("Gemini error de red: {e}");
            }
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    let json: Value = response.json().await
                        .map_err(|e| format!("AI parse error: {e}"))?;
                    tracing::info!("AI fallback Gemini: modelo={model}, key={key_hint}...");
                    return Ok(json);
                }
                let text = response.text().await.unwrap_or_default();
                tracing::error!("AI Gemini [{model}] key {key_hint}...: HTTP {status}: {text}");
                last_error = format!("Gemini HTTP {status}");

                if status.as_u16() != 429 {
                    break;
                }
            }
        }
    }

    Err(last_error)
}

/* Construye el body JSON para una llamada OpenAI-compatible (Groq o Gemini). */
fn build_chat_body(model: &str, messages: &[Value], tools: Option<&Value>) -> Value {
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
    body
}
