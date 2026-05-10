/* [T-5] Procesamiento IA de archivos subidos al chat.
 * Imágenes → Groq Vision (Llama 4 Scout multimodal)
 * Audio → Groq Whisper STT
 * PDF → extracción de texto con pdf-extract
 * Max 3000 palabras para PDF, 300 tokens para Vision. */

/* Procesar archivo con IA según tipo MIME. Dispatcher principal. */
pub async fn process_file_with_ai(
    _pool: &sqlx::PgPool,
    config: &crate::services::AiChatConfig,
    http_client: &reqwest::Client,
    mime_type: &str,
    data: &[u8],
    _file_path: &str,
) -> Option<String> {
    if mime_type.starts_with("image/") {
        process_image_vision(config, http_client, data, mime_type).await
    } else if mime_type.starts_with("audio/") {
        process_audio_whisper(config, http_client, data, mime_type).await
    } else if mime_type == "application/pdf" {
        process_pdf_extract(data)
    } else {
        None
    }
}

/* Groq Vision: envía imagen como base64 al modelo multimodal.
 * [105A-4] Usa Llama 4 Scout/Maverick; el antiguo modelo vision-preview quedó
 * fuera de servicio y hacía que el chatbot respondiera que no podía ver imágenes. */
async fn process_image_vision(
    config: &crate::services::AiChatConfig,
    http_client: &reqwest::Client,
    data: &[u8],
    mime_type: &str,
) -> Option<String> {
    use base64::Engine;
    if config.total_keys() == 0 {
        tracing::warn!("Vision: Groq no configurado; no hay API key para describir imagen");
        return None;
    }

    let b64 = base64::engine::general_purpose::STANDARD.encode(data);
    let image_url = format!("data:{mime_type};base64,{b64}");
    let mut last_error = String::new();

    for model in vision_model_chain() {
        for attempt in 0..config.total_keys() {
            let Some(api_key) = config.next_key() else {
                break;
            };
            match call_groq_vision(http_client, api_key, &model, &image_url).await {
                Ok(text) => return Some(text),
                Err(error) => {
                    tracing::warn!(
                        "Vision Groq [{model}] intento {} falló: {error}",
                        attempt + 1
                    );
                    last_error = error;
                }
            }
        }
    }

    tracing::error!("Vision: no se pudo describir imagen con Groq. Último error: {last_error}");
    None
}

fn vision_model_chain() -> Vec<String> {
    const DEFAULTS: [&str; 2] = [
        "meta-llama/llama-4-scout-17b-16e-instruct",
        "meta-llama/llama-4-maverick-17b-128e-instruct",
    ];
    let configured = std::env::var("GROQ_VISION_MODEL").ok();
    let mut models = Vec::with_capacity(DEFAULTS.len() + 1);
    if let Some(model) = configured.as_deref() {
        if DEFAULTS.contains(&model) {
            models.push(model.to_string());
        } else {
            tracing::warn!("GROQ_VISION_MODEL no permitido para vision: {model}");
        }
    }
    for model in DEFAULTS {
        if !models.iter().any(|existing| existing == model) {
            models.push(model.to_string());
        }
    }
    models
}

async fn call_groq_vision(
    http_client: &reqwest::Client,
    api_key: &str,
    model: &str,
    image_url: &str,
) -> Result<String, String> {
    let key_hint = &api_key[..api_key.len().min(8)];

    let body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": "Describe esta imagen en español para que Claudia de Nakomi Studio pueda responder al cliente. Incluye texto visible, elementos de diseño, estilo, colores, problemas o intención probable. No digas que no puedes verla. Máximo 180 palabras."
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": image_url
                        }
                    }
                ]
            }
        ],
        "temperature": 0.2,
        "max_tokens": 450
    });

    let resp = http_client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let json: serde_json::Value = r
                .json()
                .await
                .map_err(|e| format!("parse vision response: {e}"))?;
            let text = json["choices"][0]["message"]["content"]
                .as_str()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "Vision response sin contenido".to_string())?;
            tracing::info!(
                "Vision: imagen descrita con {model} key {key_hint}... ({} chars)",
                text.len()
            );
            Ok(text.to_string())
        }
        Ok(r) => {
            let status = r.status();
            let body_text = r.text().await.unwrap_or_default();
            Err(format!(
                "HTTP {status}: {}",
                truncate_error_body(&body_text, 500)
            ))
        }
        Err(e) => Err(format!("red: {e}")),
    }
}

fn truncate_error_body(body: &str, max_chars: usize) -> String {
    let mut text: String = body.chars().take(max_chars).collect();
    if body.chars().count() > max_chars {
        text.push_str("...");
    }
    text
}

/* Groq Whisper STT: transcribe audio a texto.
 * Usa whisper-large-v3-turbo. Max 25MB. */
async fn process_audio_whisper(
    config: &crate::services::AiChatConfig,
    _http_client: &reqwest::Client,
    data: &[u8],
    mime_type: &str,
) -> Option<String> {
    let api_key = config.next_key()?;

    /* Determinar extensión para el campo filename del multipart */
    let ext = match mime_type {
        "audio/ogg" => "ogg",
        "audio/wav" => "wav",
        "audio/webm" => "webm",
        "audio/mp4" => "m4a",
        "audio/flac" => "flac",
        _ => "mp3",
    };

    let file_part = reqwest::multipart::Part::bytes(data.to_vec())
        .file_name(format!("audio.{ext}"))
        .mime_str(mime_type)
        .ok()?;

    let form = reqwest::multipart::Form::new()
        .text("model", "whisper-large-v3-turbo")
        .text("language", "es")
        .part("file", file_part);

    /* [114A-6] Timeout 30s para whisper API — previene deadlock por API colgada */
    let resp = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default()
        .post("https://api.groq.com/openai/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {api_key}"))
        .multipart(form)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let json: serde_json::Value = r.json().await.ok()?;
            let text = json["text"].as_str()?;
            tracing::info!("Whisper: audio transcrito ({} chars)", text.len());
            Some(text.to_string())
        }
        Ok(r) => {
            let status = r.status();
            let body_text = r.text().await.unwrap_or_default();
            tracing::error!("Whisper API error HTTP {status}: {body_text}");
            None
        }
        Err(e) => {
            tracing::error!("Whisper API error: {e}");
            None
        }
    }
}

/* Extraer texto de PDF con pdf-extract.
 * Limitado a las primeras 3000 palabras para no saturar el context window de la IA. */
pub fn process_pdf_extract(data: &[u8]) -> Option<String> {
    match pdf_extract::extract_text_from_mem(data) {
        Ok(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                tracing::info!("PDF: sin texto extraíble (probablemente escaneado)");
                return Some(
                    "(PDF sin texto extraíble — puede ser un documento escaneado)".to_string(),
                );
            }
            /* Limitar a 3000 palabras */
            let words: Vec<&str> = trimmed.split_whitespace().collect();
            let limited = if words.len() > 3000 {
                let truncated: String = words[..3000].join(" ");
                format!("{truncated}\n[... documento truncado a 3000 palabras]")
            } else {
                trimmed.to_string()
            };
            tracing::info!("PDF: {} palabras extraídas", words.len().min(3000));
            Some(limited)
        }
        Err(e) => {
            tracing::error!("PDF extract error: {e}");
            None
        }
    }
}
