/* [T-2] Herramientas de IA para el chatbot (tool use / function calling).
 * Define las tools que la IA puede invocar y ejecuta las llamadas.
 * Groq soporta tool use compatible con OpenAI en llama-3.3-70b-versatile.
 * Cada tool retorna un resultado JSON que se reenvía a la IA + opcionalmente
 * un mensaje rico para el WS del visitante. */

use serde_json::{json, Value};
use sqlx::PgPool;

use crate::repositories::ChatRepository;

/* Resultado de ejecutar una tool: JSON para la IA y opcionalmente un
 * mensaje rico para mostrar en el chat del visitante. */
pub struct ToolExecResult {
    pub tool_result_json: String,
    pub rich_message: Option<RichMessage>,
}

/* Mensaje rico que se envía al WS como message_type + metadata.
 * content es el texto visible; message_type indica el render; metadata tiene datos extra. */
pub struct RichMessage {
    pub content: String,
    pub message_type: String,
    pub metadata: Value,
}

/* [T-2][T-3] Definiciones de tools en formato OpenAI/Groq.
 * Se incluyen en el body de la API junto con los mensajes.
 * Dividido en service_tools + visitor_tools para no exceder líneas. */
pub fn tool_definitions() -> Value {
    let mut tools = service_tool_defs();
    if let Some(arr) = tools.as_array_mut() {
        arr.extend(visitor_tool_defs().as_array().cloned().unwrap_or_default());
        /* [T-9] Tools para clientes registrados */
        arr.extend(registered_client_tool_defs().as_array().cloned().unwrap_or_default());
    }
    tools
}

/* [084A-51] show_service y list_services removidos — las service cards son antinaturales
 * en un chat conversacional. Solo se mantienen tools de factura, escalación y captura. */
fn service_tool_defs() -> Value {
    json!([
        {
            "type": "function",
            "function": {
                "name": "create_invoice",
                "description": "Genera una factura de Stripe con link de pago para el cliente. Solo úsalo cuando el cliente confirme que quiere pagar y tenga claro el servicio/monto.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "amount_cents": { "type": "integer", "description": "Monto en centavos USD (ej: 10000 = $100.00)" },
                        "description": { "type": "string", "description": "Descripción del concepto de la factura" },
                        "client_email": { "type": "string", "description": "Email del cliente para la factura de Stripe" }
                    },
                    "required": ["amount_cents", "description", "client_email"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "request_human_assistance",
                "description": "Solicita intervención humana. Úsalo cuando no puedas resolver la solicitud, el cliente esté frustrado, pida hablar con una persona, o el tema sea legal/contractual.",
                "parameters": {
                    "type": "object",
                    "properties": { "reason": { "type": "string", "description": "Motivo breve de la escalación" } },
                    "required": ["reason"]
                }
            }
        }
    ])
}

/* Tools de captura de email e info del cliente (T-3) */
fn visitor_tool_defs() -> Value {
    json!([
        {
            "type": "function",
            "function": {
                "name": "capture_email",
                "description": "Guarda el email del cliente. Úsalo cuando el cliente comparta su correo electrónico durante la conversación. No lo pidas de forma forzada — espera a que surja naturalmente o cuando sea necesario para una factura.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "email": { "type": "string", "description": "Email del cliente" },
                        "display_name": { "type": "string", "description": "Nombre del cliente (si lo mencionó)" }
                    },
                    "required": ["email"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "save_client_info",
                "description": "Guarda información relevante del cliente para futuras conversaciones (industria, presupuesto, intereses, tipo de proyecto). Úsalo cuando el cliente mencione datos útiles sobre su negocio o necesidades.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "industry": { "type": "string", "description": "Sector o industria del cliente" },
                        "budget_range": { "type": "string", "description": "Rango de presupuesto mencionado" },
                        "interests": { "type": "array", "items": { "type": "string" }, "description": "Servicios o temas de interés" },
                        "project_description": { "type": "string", "description": "Descripción breve del proyecto que necesita" },
                        "notes": { "type": "string", "description": "Cualquier otra info relevante" }
                    }
                }
            }
        }
    ])
}

/* Ejecutar una tool call retornada por la IA.
 * Despacha al handler correcto según el nombre de la función.
 * visitor_id necesario para tools que actualizan visitor_profiles (T-3). */
pub async fn execute_tool(
    pool: &PgPool,
    http_client: &reqwest::Client,
    stripe_key: Option<&str>,
    visitor_id: Option<&str>,
    tool_name: &str,
    arguments: &Value,
) -> ToolExecResult {
    match tool_name {
        "create_invoice" => exec_create_invoice(http_client, stripe_key, arguments).await,
        "request_human_assistance" => exec_request_human(arguments),
        "capture_email" => exec_capture_email(pool, visitor_id, arguments).await,
        "save_client_info" => exec_save_client_info(pool, visitor_id, arguments).await,
        "create_support_ticket" => exec_create_support_ticket(pool, visitor_id, arguments).await,
        _ => ToolExecResult {
            tool_result_json: json!({"error": "Tool desconocida"}).to_string(),
            rich_message: None,
        },
    }
}

/* [084A-51] exec_show_service y exec_list_services eliminados — herramientas desactivadas.
 * Las service cards son antinaturales en el chat conversacional. Si se reactivan en el futuro,
 * recuperar implementación del git history (commit anterior a 084A-51). */

/* create_invoice: crea factura en Stripe con link de pago.
 * Flujo: create customer → create invoice → add line item → finalize → get URL.
 * El resultado es un mensaje de tipo "invoice" con el link de pago. */
async fn exec_create_invoice(
    http_client: &reqwest::Client,
    stripe_key: Option<&str>,
    args: &Value,
) -> ToolExecResult {
    let Some(stripe_key) = stripe_key else {
        return ToolExecResult {
            tool_result_json: json!({"error": "Stripe no configurado"}).to_string(),
            rich_message: None,
        };
    };

    /* [084A-52] Parsing robusto: algunos modelos envían amount_cents como float
     * (ej: 10000.0 en vez de 10000). Intentar i64, luego f64→i64. */
    #[allow(clippy::cast_possible_truncation)]
    let amount_cents = args["amount_cents"]
        .as_i64()
        .or_else(|| args["amount_cents"].as_f64().map(|f| f as i64))
        .unwrap_or(0);
    let description = args["description"].as_str().unwrap_or("Servicio Nakomi Studio");
    let client_email = args["client_email"].as_str().unwrap_or("");

    if amount_cents <= 0 || client_email.is_empty() {
        /* [084A-52] Log detallado para diagnosticar qué parámetro falta.
         * Causa común: Gemini retorna arguments como objeto en vez de string,
         * y el parsing ignoraba ese caso (ya corregido en ai_chat.rs). */
        tracing::error!(
            "create_invoice args inválidos: amount_cents={amount_cents}, \
             email_empty={}, args_raw={args}",
            client_email.is_empty()
        );
        return ToolExecResult {
            tool_result_json: json!({
                "error": "Monto y email son requeridos",
                "detail": format!(
                    "amount_cents={amount_cents}, client_email={}",
                    if client_email.is_empty() { "(vacío)" } else { "(presente)" }
                )
            }).to_string(),
            rich_message: None,
        };
    }

    /* Paso 1: crear/buscar customer por email */
    let customer_id = match find_or_create_stripe_customer(http_client, stripe_key, client_email).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Stripe create customer error: {e}");
            return ToolExecResult {
                tool_result_json: json!({"error": "Error creando cliente en Stripe"}).to_string(),
                rich_message: None,
            };
        }
    };

    /* Paso 2: crear invoice */
    let invoice = match create_stripe_invoice(
        http_client,
        stripe_key,
        &customer_id,
        description,
        amount_cents,
    )
    .await
    {
        Ok(inv) => inv,
        Err(e) => {
            tracing::error!("Stripe create invoice error: {e}");
            return ToolExecResult {
                tool_result_json: json!({"error": "Error creando factura"}).to_string(),
                rich_message: None,
            };
        }
    };

    #[allow(clippy::cast_precision_loss)]
    let price_usd = amount_cents as f64 / 100.0;
    let metadata = json!({
        "stripe_invoice_id": invoice.id,
        "amount_cents": amount_cents,
        "currency": "usd",
        "status": invoice.status,
        "payment_url": invoice.hosted_invoice_url,
        "description": description,
    });

    ToolExecResult {
        /* [084A-38] No incluir payment_url en tool_result_json — la IA lo repite en texto plano.
         * El link de pago solo va en la RichMessage metadata (la card lo muestra con botón). */
        tool_result_json: json!({
            "invoice_id": invoice.id,
            "amount_usd": price_usd,
            "status": invoice.status,
            "message": "Factura creada exitosamente. El cliente verá una tarjeta visual con botón de pago. NO repitas el link de pago en tu respuesta."
        })
        .to_string(),
        rich_message: Some(RichMessage {
            content: format!("Factura por ${price_usd:.2} USD — {description}"),
            message_type: "invoice".to_string(),
            metadata,
        }),
    }
}

/* request_human_assistance: marca escalación. El resultado es para la IA,
 * la lógica de notificación real la maneja chat_timing/escalation. */
fn exec_request_human(args: &Value) -> ToolExecResult {
    let reason = args["reason"].as_str().unwrap_or("Sin motivo especificado");
    ToolExecResult {
        tool_result_json: json!({
            "status": "escalated",
            "reason": reason,
            "message": "Se ha notificado al equipo. Un especialista se conectará pronto."
        })
        .to_string(),
        rich_message: None,
    }
}

/* ============================================================
   STRIPE HELPERS (invoice flow)
   ============================================================ */

#[derive(Debug, serde::Deserialize)]
struct StripeCustomerSearch {
    data: Vec<StripeCustomer>,
}

#[derive(Debug, serde::Deserialize)]
struct StripeCustomer {
    id: String,
}

#[derive(Debug, serde::Deserialize)]
struct StripeInvoice {
    id: String,
    status: Option<String>,
    hosted_invoice_url: Option<String>,
}

/* Busca customer por email en Stripe. Si no existe, lo crea. */
async fn find_or_create_stripe_customer(
    client: &reqwest::Client,
    key: &str,
    email: &str,
) -> Result<String, String> {
    /* Buscar por email */
    let search_resp = client
        .get("https://api.stripe.com/v1/customers/search")
        .header("Authorization", format!("Bearer {key}"))
        .query(&[("query", &format!("email:'{email}'"))])
        .send()
        .await
        .map_err(|e| format!("Stripe search error: {e}"))?;

    if search_resp.status().is_success() {
        let body: StripeCustomerSearch = search_resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {e}"))?;
        if let Some(cust) = body.data.first() {
            return Ok(cust.id.clone());
        }
    }

    /* Crear nuevo customer */
    let create_resp = client
        .post("https://api.stripe.com/v1/customers")
        .header("Authorization", format!("Bearer {key}"))
        .form(&[("email", email)])
        .send()
        .await
        .map_err(|e| format!("Stripe create error: {e}"))?;

    if !create_resp.status().is_success() {
        let text = create_resp.text().await.unwrap_or_default();
        return Err(format!("Stripe create customer failed: {text}"));
    }

    let cust: StripeCustomer = create_resp
        .json()
        .await
        .map_err(|e| format!("Parse error: {e}"))?;
    Ok(cust.id)
}

/* Crea invoice en Stripe: invoice + line item + finalize.
 * Retorna invoice con hosted_invoice_url (link de pago). */
async fn create_stripe_invoice(
    client: &reqwest::Client,
    key: &str,
    customer_id: &str,
    description: &str,
    amount_cents: i64,
) -> Result<StripeInvoice, String> {
    /* Crear invoice draft */
    let inv_resp = client
        .post("https://api.stripe.com/v1/invoices")
        .header("Authorization", format!("Bearer {key}"))
        .form(&[
            ("customer", customer_id),
            ("collection_method", "send_invoice"),
            ("days_until_due", "7"),
            ("auto_advance", "true"),
        ])
        .send()
        .await
        .map_err(|e| format!("Create invoice error: {e}"))?;

    if !inv_resp.status().is_success() {
        let text = inv_resp.text().await.unwrap_or_default();
        return Err(format!("Create invoice failed: {text}"));
    }

    let draft: StripeInvoice = inv_resp
        .json()
        .await
        .map_err(|e| format!("Parse invoice error: {e}"))?;

    /* [084A-38] Agregar line item — VALIDAR respuesta. Sin line item, el invoice
     * se finaliza con $0.00 y Stripe lo marca como "paid" inmediatamente. */
    let item_resp = client
        .post("https://api.stripe.com/v1/invoiceitems")
        .header("Authorization", format!("Bearer {key}"))
        .form(&[
            ("customer", customer_id),
            ("invoice", draft.id.as_str()),
            ("amount", &amount_cents.to_string()),
            ("currency", "usd"),
            ("description", description),
        ])
        .send()
        .await
        .map_err(|e| format!("Create line item error: {e}"))?;

    if !item_resp.status().is_success() {
        let text = item_resp.text().await.unwrap_or_default();
        return Err(format!("Line item creation failed: {text}"));
    }

    /* Finalizar invoice (genera hosted_invoice_url) */
    let final_resp = client
        .post(format!(
            "https://api.stripe.com/v1/invoices/{}/finalize",
            draft.id
        ))
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await
        .map_err(|e| format!("Finalize invoice error: {e}"))?;

    if !final_resp.status().is_success() {
        let text = final_resp.text().await.unwrap_or_default();
        return Err(format!("Finalize invoice failed: {text}"));
    }

    let finalized: StripeInvoice = final_resp
        .json()
        .await
        .map_err(|e| format!("Parse finalized error: {e}"))?;

    Ok(finalized)
}

/* ============================================================
   VISITOR PROFILE TOOLS (T-3 — Memoria y contexto)
   ============================================================ */

/* [T-3] capture_email: guarda email del visitante en visitor_profiles.
 * También actualiza display_name si lo proporcionó. No bloquea la conversación. */
async fn exec_capture_email(
    pool: &PgPool,
    visitor_id: Option<&str>,
    args: &Value,
) -> ToolExecResult {
    let Some(vid) = visitor_id else {
        return ToolExecResult {
            tool_result_json: json!({"error": "visitor_id no disponible"}).to_string(),
            rich_message: None,
        };
    };

    let email = args["email"].as_str().unwrap_or("");
    if email.is_empty() || !email.contains('@') {
        return ToolExecResult {
            tool_result_json: json!({"error": "Email inválido"}).to_string(),
            rich_message: None,
        };
    }

    let display_name = args["display_name"].as_str();

    match ChatRepository::update_visitor_email(pool, vid, email, display_name).await {
        Ok(profile) => {
            tracing::info!("Email capturado para visitor {vid}: {email}");
            ToolExecResult {
                tool_result_json: json!({
                    "status": "ok",
                    "email": profile.email,
                    "display_name": profile.display_name,
                    "message": "Email guardado correctamente."
                })
                .to_string(),
                rich_message: None,
            }
        }
        Err(e) => {
            tracing::error!("Error guardando email visitor {vid}: {e}");
            ToolExecResult {
                tool_result_json: json!({"error": "Error guardando email"}).to_string(),
                rich_message: None,
            }
        }
    }
}

/* [T-3] save_client_info: guarda preferencias/datos del cliente en visitor_profiles.
 * Hace merge con preferencias existentes (JSON ||). */
async fn exec_save_client_info(
    pool: &PgPool,
    visitor_id: Option<&str>,
    args: &Value,
) -> ToolExecResult {
    let Some(vid) = visitor_id else {
        return ToolExecResult {
            tool_result_json: json!({"error": "visitor_id no disponible"}).to_string(),
            rich_message: None,
        };
    };

    /* Construir objeto de preferencias solo con campos que la IA proporcionó */
    let mut prefs = serde_json::Map::new();
    if let Some(v) = args.get("industry") {
        prefs.insert("industry".to_string(), v.clone());
    }
    if let Some(v) = args.get("budget_range") {
        prefs.insert("budget_range".to_string(), v.clone());
    }
    if let Some(v) = args.get("interests") {
        prefs.insert("interests".to_string(), v.clone());
    }
    if let Some(v) = args.get("project_description") {
        prefs.insert("project_description".to_string(), v.clone());
    }
    if let Some(v) = args.get("notes") {
        prefs.insert("notes".to_string(), v.clone());
    }

    if prefs.is_empty() {
        return ToolExecResult {
            tool_result_json: json!({"status": "ok", "message": "Sin datos nuevos"}).to_string(),
            rich_message: None,
        };
    }

    let prefs_value = Value::Object(prefs);
    match ChatRepository::update_visitor_preferences(pool, vid, &prefs_value).await {
        Ok(()) => {
            tracing::info!("Preferencias actualizadas para visitor {vid}");
            ToolExecResult {
                tool_result_json: json!({
                    "status": "ok",
                    "saved_fields": prefs_value,
                    "message": "Información del cliente guardada."
                })
                .to_string(),
                rich_message: None,
            }
        }
        Err(e) => {
            tracing::error!("Error guardando preferencias visitor {vid}: {e}");
            ToolExecResult {
                tool_result_json: json!({"error": "Error guardando información"}).to_string(),
                rich_message: None,
            }
        }
    }
}

/* [T-9] Definición de tools para clientes registrados */
fn registered_client_tool_defs() -> Value {
    json!([
        {
            "type": "function",
            "function": {
                "name": "create_support_ticket",
                "description": "Crea un ticket de soporte para el cliente. Úsalo cuando el cliente reporte un problema con su hosting, pedido, facturación o necesite asistencia técnica. Esto crea una nota interna que el equipo revisará.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "category": {
                            "type": "string",
                            "enum": ["hosting_issue", "order_issue", "billing_issue", "general"],
                            "description": "Categoría del problema"
                        },
                        "description": {
                            "type": "string",
                            "description": "Descripción detallada del problema reportado"
                        },
                        "priority": {
                            "type": "string",
                            "enum": ["low", "medium", "high"],
                            "description": "Prioridad del ticket"
                        }
                    },
                    "required": ["category", "description"]
                }
            }
        }
    ])
}

/* [T-9] Crear ticket de soporte: guarda como nota interna en la sesión de chat. */
async fn exec_create_support_ticket(
    pool: &PgPool,
    visitor_id: Option<&str>,
    args: &Value,
) -> ToolExecResult {
    let category = args["category"].as_str().unwrap_or("general");
    let description = args["description"].as_str().unwrap_or("");
    let priority = args["priority"].as_str().unwrap_or("medium");

    if description.is_empty() {
        return ToolExecResult {
            tool_result_json: json!({"error": "Se necesita una descripción del problema"}).to_string(),
            rich_message: None,
        };
    }

    let ticket_content = format!(
        "[TICKET SOPORTE] Cat: {category} | Prioridad: {priority} | Visitor: {} | Descripción: {description}",
        visitor_id.unwrap_or("anónimo"),
    );

    /* Se almacena como nota en visitor_profile.context_summary para que el equipo lo vea.
     * En el futuro se puede crear una tabla dedicada de tickets. */
    if let Some(vid) = visitor_id {
        let ticket_json = json!({
            "type": "support_ticket",
            "category": category,
            "priority": priority,
            "description": description,
            "created_at": chrono::Utc::now().to_rfc3339(),
        });
        let _ = ChatRepository::update_visitor_preferences(pool, vid, &ticket_json).await;
    }

    tracing::info!("Ticket de soporte creado: {ticket_content}");

    ToolExecResult {
        tool_result_json: json!({
            "status": "ok",
            "message": "Ticket de soporte creado exitosamente. El equipo lo revisará pronto.",
            "category": category,
            "priority": priority,
        })
        .to_string(),
        rich_message: Some(RichMessage {
            content: format!("📋 Ticket de soporte creado — {category} ({priority})"),
            message_type: "support_ticket".to_string(),
            metadata: json!({
                "category": category,
                "priority": priority,
                "description": description,
            }),
        }),
    }
}
