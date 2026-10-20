/* [01AA-4-f3q] Escalación a humano con CTA de WhatsApp (extraido de ai_tools_misc.rs).
 * [237A-7g] Marca escalación + genera contact_cta; la URL sale de
 * PUBLIC_SUPPORT_WHATSAPP (número público del soporte). */

use serde_json::{json, Value};

use crate::services::ai_tools::{RichMessage, ToolExecResult};

pub(crate) fn exec_request_human(args: &Value) -> ToolExecResult {
    let reason = args["reason"].as_str().unwrap_or("Sin motivo especificado");

    /* Construir CTA de WhatsApp si el número público está configurado */
    let support_whatsapp = std::env::var("PUBLIC_SUPPORT_WHATSAPP")
        .ok()
        .filter(|s| !s.is_empty());

    let rich_message = support_whatsapp.and_then(|raw_number| {
        /* Normalizar a dígitos para wa.me — requiere al menos 7 dígitos (número real) */
        let digits: String = raw_number.chars().filter(char::is_ascii_digit).collect();
        if digits.len() < 7 {
            tracing::warn!("PUBLIC_SUPPORT_WHATSAPP no tiene dígitos suficientes: {raw_number}");
            return None;
        }
        let prefill = "Hola, quiero conversar más a fondo sobre mi proyecto con Nakomi Studio.";
        let href = format!(
            "https://wa.me/{digits}?text={}",
            urlencoding::encode(prefill)
        );

        Some(RichMessage {
            content: "Este caso necesita atención personal.".to_string(),
            message_type: "contact_cta".to_string(),
            metadata: json!({
                "label": "Escribir por WhatsApp",
                "href": href,
                "fallback": "El equipo fue notificado y responderá por este chat.",
                "reason": reason,
            }),
        })
    });

    ToolExecResult {
        tool_result_json: json!({
            "status": "escalated",
            "reason": reason,
            "message": "Se ha notificado al equipo. Un especialista se conectará pronto."
        })
        .to_string(),
        rich_message,
    }
}

#[cfg(test)]
mod tests {
    /* [19AA-7] El CTA depende de PUBLIC_SUPPORT_WHATSAPP: sin ella no hay botón. Se serializan
     * los dos casos porque la variable de entorno es global al proceso de test. */
    use super::*;
    use std::sync::Mutex;

    static ENV: Mutex<()> = Mutex::new(());

    #[test]
    fn el_numero_configurado_produce_el_cta_de_whatsapp() {
        let _guard = ENV
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        std::env::set_var("PUBLIC_SUPPORT_WHATSAPP", "+18149575416");

        let r = exec_request_human(&json!({ "reason": "El cliente pide una persona." }));

        let rich = r
            .rich_message
            .expect("con número configurado debe haber CTA");
        assert_eq!(rich.message_type, "contact_cta");
        assert_eq!(rich.metadata["label"], "Escribir por WhatsApp");
        let href = rich.metadata["href"].as_str().unwrap();
        assert!(
            href.starts_with("https://wa.me/18149575416?text="),
            "href inesperado: {href}"
        );
        /* El número viaja solo en dígitos: wa.me no admite «+» ni separadores. */
        assert!(!href.contains('+'));
        assert_eq!(rich.metadata["reason"], "El cliente pide una persona.");
        assert!(r.tool_result_json.contains("\"status\":\"escalated\""));

        std::env::remove_var("PUBLIC_SUPPORT_WHATSAPP");
    }

    #[test]
    fn sin_numero_no_hay_cta_pero_la_escalacion_sigue() {
        let _guard = ENV
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        std::env::remove_var("PUBLIC_SUPPORT_WHATSAPP");

        let r = exec_request_human(&json!({ "reason": "sin número" }));

        assert!(r.rich_message.is_none(), "sin número no debe haber CTA");
        assert!(r.tool_result_json.contains("\"status\":\"escalated\""));
    }

    #[test]
    fn un_numero_con_menos_de_siete_digitos_no_produce_cta() {
        let _guard = ENV
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        std::env::set_var("PUBLIC_SUPPORT_WHATSAPP", "12345");

        let r = exec_request_human(&json!({}));

        assert!(
            r.rich_message.is_none(),
            "un número incompleto no es un CTA"
        );
        std::env::remove_var("PUBLIC_SUPPORT_WHATSAPP");
    }
}
