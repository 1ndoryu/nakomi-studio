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
 * Usa llama-3.2-90b-vision-preview que soporta imágenes inline. */
async fn process_image_vision(
    config: &crate::services::AiChatConfig,
    _http_client: &reqwest::Client,
    data: &[u8],
    mime_type: &str,
) -> Option<String> {
    use base64::Engine;
    let api_key = config.next_key()?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(data);

    let body = serde_json::json!({
        "model": "llama-3.2-90b-vision-preview",
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:{mime_type};base64,{b64}")
                        }
                    },
                    {
                        "type": "text",
                        "text": "Describe brevemente esta imagen en español. Si contiene texto, transcríbelo. Si es un mockup o diseño, describe los elementos."
                    }
                ]
            }
        ],
        "max_tokens": 300
    });

    /* [114A-6] Timeout 30s para vision API — previene deadlock por API colgada */
    let resp = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default()
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let json: serde_json::Value = r.json().await.ok()?;
            let text = json["choices"][0]["message"]["content"].as_str()?;
            tracing::info!("Vision: imagen descrita ({} chars)", text.len());
            Some(text.to_string())
        }
        Ok(r) => {
            let status = r.status();
            let body_text = r.text().await.unwrap_or_default();
            tracing::error!("Vision API error HTTP {status}: {body_text}");
            None
        }
        Err(e) => {
            tracing::error!("Vision API error: {e}");
            None
        }
    }
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
                return Some("(PDF sin texto extraíble — puede ser un documento escaneado)".to_string());
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
