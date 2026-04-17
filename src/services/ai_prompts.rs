/* [174A-2] Prompt building para AI chat. Extraído de ai_chat.rs para SRP.
 * Contiene: system prompts dinámicos, contexto de visitante/cliente/página,
 * prompt para intermediario de órdenes. */

use std::fmt::Write;

use sqlx::PgPool;
use uuid::Uuid;

use crate::repositories::{ChatRepository, HostingRepository, OrderRepository, ServiceRepository, UserRepository};

/* Re-usar sanitize_for_prompt del módulo ai_chat */
use super::ai_chat::sanitize_for_prompt;

/// Construye system prompt dinámico según contexto de la sesión, visitante y usuario autenticado
pub(crate) async fn build_system_prompt(
    pool: &PgPool,
    session_id: Uuid,
    visitor_id: Option<&str>,
    user_id: Option<Uuid>,
    page_context: Option<&str>,
) -> String {
    let mut prompt = String::from(base_system_prompt());

    if let Some(vid) = visitor_id {
        append_visitor_context(&mut prompt, pool, vid).await;
    }

    if let Ok(services) = ServiceRepository::list_services(pool).await {
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
                        0
                    );
                }
            }
        }
    }

    if let Some(uid) = user_id {
        append_registered_client_context(&mut prompt, pool, uid).await;
    }

    if let Some(ctx) = page_context {
        append_page_context(&mut prompt, pool, ctx).await;
    }

    prompt
}

/* [084A-49+50+51] Prompt base extraído para cumplir límite 100 líneas en build_system_prompt */
pub(crate) fn base_system_prompt() -> &'static str {
    "Eres un miembro del equipo de Nakomi Studio, una agencia de desarrollo web y diseño. \
     Tu nombre es Claudia. Responde de forma concisa, amable y profesional en el mismo \
     idioma que el usuario. Nunca menciones que eres una inteligencia artificial, chatbot, \
     bot, modelo de lenguaje o asistente virtual. Si te preguntan directamente si eres IA, \
     responde que eres parte del equipo de atención de Nakomi Studio.\n\n\
     FORMATO DE RESPUESTA:\n\
     Escribe en texto plano. PROHIBIDO usar markdown: no uses **, ##, __, -, ni ningún formato. \
     Escribe oraciones normales sin asteriscos, sin headers, sin listas con guiones. \
     Solo texto fluido y natural como en una conversación de chat.\n\n\
     CONVERSACIONES OFF-TOPIC:\n\
     Tu propósito es ayudar con servicios de Nakomi Studio (diseño web, desarrollo de apps, \
     branding, agentes IA, hosting). Si el usuario habla de temas no relacionados:\n\
     1. Primero intenta regresar la conversación al punto amablemente.\n\
     2. Si insiste con el tema off-topic, responde brevemente pero vuelve a ofrecer ayuda.\n\
     3. Si tras 3-4 mensajes sigue sin relación, responde con algo como: \
        'No puedo ayudarte con eso, pero si necesitas algo de diseño web o desarrollo, aquí estoy.' \
        y deja de elaborar sobre el tema off-topic.\n\
     Nunca dejes de responder completamente. El usuario siempre puede reconducir la conversación.\n\n\
     REGLA CRÍTICA — PROHIBIDO SIMULAR ACCIONES:\n\
     Tienes herramientas reales que ejecutan acciones. NUNCA escribas texto que simule lo que \
     una herramienta haría. Por ejemplo:\n\
     - PROHIBIDO: escribir texto con formato de factura (montos, descripciones, links ficticios)\n\
     - PROHIBIDO: escribir 'aquí tienes tu factura:' seguido de texto que parece factura\n\
     - PROHIBIDO: inventar links de pago o URLs\n\
     - CORRECTO: llamar a create_invoice con los parámetros reales\n\
     Si necesitas hacer algo y tienes una herramienta para ello, SIEMPRE usa la herramienta.\n\n\
     HERRAMIENTAS DISPONIBLES:\n\
     - create_invoice: Genera factura REAL con link de pago Stripe. Úsala SIEMPRE que el cliente \
       confirme que quiere pagar. REQUIERE email del cliente.\n\
     - request_human_assistance: Escala a un humano. Úsala en los casos de la REGLA DE ESCALACIÓN.\n\
     - capture_email: Guarda el email del cliente. Úsala SIEMPRE que el cliente comparta su correo.\n\
     - save_client_info: Guarda info relevante del cliente. Úsala cuando el cliente mencione datos \
       útiles sobre su negocio o proyecto. También acepta 'name' para guardar el nombre del visitante.\n\n\
     FLUJO DE FACTURA (obligatorio):\n\
     1. Si el cliente quiere pagar y NO tienes su email → pide el email primero, luego create_invoice.\n\
     2. Si ya tienes su email → usa create_invoice directamente con amount_cents, currency, description, email.\n\
     3. NUNCA escribas texto que parezca una factura. La herramienta genera una tarjeta visual real con botón de pago.\n\n\
     CAPTURA DE NOMBRE Y EMAIL (en orden):\n\
     1. Primero pregunta el nombre del visitante de forma natural en la primera o segunda respuesta ('¿Con quién tengo el gusto?').\n\
     2. Cuando el visitante dé su nombre, usa save_client_info con el campo 'name' para guardarlo inmediatamente.\n\
     3. Después de 2-3 intercambios productivos, puedes preguntar el correo: 'Me compartes tu correo para enviarte la información?'\n\
     4. Cuando el cliente dé el email, usa capture_email con display_name incluido si lo tienes.\n\
     5. Si ya conoces el nombre del contexto anterior, NO vuelvas a pedirlo.\n\n\
     CAPTURA DE INFO: Cuando el cliente mencione su industria, presupuesto, tipo de proyecto o necesidades \
     específicas, usa save_client_info para guardar esos datos.\n\n\
     REGLA DE ESCALACIÓN: Si detectas alguna de estas situaciones, usa request_human_assistance O inicia tu \
     respuesta con [ESCALATE]:\n\
     - El cliente pide hablar con un humano\n\
     - El cliente está frustrado o insatisfecho después de varias respuestas\n\
     - El tema es legal, contractual, o sobre disputas de pago\n\
     - No puedes resolver la solicitud con la información disponible\n\
     - El cliente reporta un problema técnico urgente\n\n"
}

/* [T-9] Helper: agrega contexto del visitante (perfil previo) al system prompt */
async fn append_visitor_context(prompt: &mut String, pool: &PgPool, visitor_id: &str) {
    if let Ok(Some(profile)) = ChatRepository::find_visitor_profile(pool, visitor_id).await {
        prompt.push_str("CONTEXTO DEL VISITANTE (conversaciones anteriores):\n");
        if let Some(name) = &profile.display_name {
            let safe = sanitize_for_prompt(name, 100);
            let _ = writeln!(prompt, "- Nombre: {safe}");
        }
        if let Some(email) = &profile.email {
            let safe = sanitize_for_prompt(email, 200);
            let _ = writeln!(prompt, "- Email: {safe} (ya capturado, no volver a pedir)");
        }
        if profile.total_sessions > 1 {
            let _ = writeln!(prompt, "- Visitas anteriores: {}", profile.total_sessions);
        }
        if let Some(summary) = &profile.context_summary {
            if !summary.is_empty() {
                let safe = sanitize_for_prompt(summary, 500);
                let _ = writeln!(prompt, "- Resumen de conversaciones previas: {safe}");
            }
        }
        if let Some(prefs) = &profile.preferences {
            if let Some(obj) = prefs.as_object() {
                if !obj.is_empty() {
                    let safe = sanitize_for_prompt(&prefs.to_string(), 500);
                    let _ = writeln!(prompt, "- Info del cliente: {safe}");
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
    let display = sanitize_for_prompt(
        user.display_name.as_deref().unwrap_or(&user.username), 100,
    );
    let email = sanitize_for_prompt(&user.email, 200);
    let _ = writeln!(prompt, "- Nombre: {display} ({email})");
    let _ = writeln!(prompt, "- Rol: {:?}", user.role);
    prompt.push_str("Ya está registrado — no pedir email ni nombre.\n\n");

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

/* [084A-28] Helper: agrega contexto de la página de origen al system prompt */
async fn append_page_context(prompt: &mut String, pool: &PgPool, ctx: &str) {
    let parts: Vec<&str> = ctx.splitn(2, ':').collect();
    if parts.len() != 2 {
        return;
    }
    match parts[0] {
        "hosting" => {
            if let Ok(uid) = Uuid::parse_str(parts[1]) {
                if let Ok(Some(h)) = HostingRepository::find_by_id(pool, uid).await {
                    let domain = h.domain.as_deref().unwrap_or("sin dominio");
                    prompt.push_str("CONTEXTO DE ORIGEN: El usuario abrió el chat desde \
                                     el botón de soporte de su hosting.\n");
                    let _ = writeln!(
                        prompt,
                        "- Plan: {} — Dominio: {domain} — Estado: {}",
                        h.plan, h.status,
                    );
                    prompt.push_str("Saluda al usuario mencionando su hosting y pregunta \
                                     en qué puedes ayudarle con él.\n\n");
                }
            }
        }
        "service" => {
            if let Ok(Some(svc)) = ServiceRepository::find_service_by_slug(pool, parts[1]).await {
                prompt.push_str("CONTEXTO DE ORIGEN: El usuario abrió el chat desde \
                                 la página del servicio.\n");
                let _ = writeln!(
                    prompt,
                    "- Servicio: {} — Desde ${:.2} USD",
                    svc.title,
                    f64::from(svc.base_price_cents) / 100.0,
                );
                if let Some(desc) = &svc.description {
                    let _ = writeln!(prompt, "- Descripción: {desc}");
                }
                prompt.push_str("Saluda al usuario mencionando el servicio que estaba \
                                 viendo y ofrece información sobre él.\n\n");
            }
        }
        "page" => {
            let safe_page = sanitize_for_prompt(parts[1], 100);
            let _ = writeln!(
                prompt,
                "CONTEXTO DE ORIGEN: El usuario abrió el chat desde la página de {safe_page}.\n\
                 Saluda al usuario y ofrece información relevante sobre {safe_page}.\n",
            );
        }
        "problem" => {
            if let Ok(uid) = Uuid::parse_str(parts[1]) {
                if let Ok(Some(order)) = OrderRepository::find_order_by_id(pool, uid).await {
                    let (svc_title, _, plan_name) =
                        OrderRepository::get_order_display_info(pool, order.service_id, order.plan_id)
                            .await
                            .unwrap_or_else(|_| ("Servicio".to_string(), String::new(), "Plan".to_string()));
                    prompt.push_str(
                        "CONTEXTO CRÍTICO: El usuario abrió el chat desde el botón \
                         'Reportar problema' de su pedido. Tiene un problema que necesita resolver.\n",
                    );
                    let _ = writeln!(
                        prompt,
                        "- Pedido #{}: {} ({})\n\
                         - Estado: {:?} — Fase: {}\n",
                        order.order_number, svc_title, plan_name,
                        order.status, order.current_phase,
                    );
                    prompt.push_str(
                        "INSTRUCCIONES:\n\
                         1. Saluda brevemente y pregunta qué problema está experimentando con su pedido.\n\
                         2. Escucha atentamente y haz preguntas específicas para entender el problema.\n\
                         3. Problemas comunes: retrasos en entrega, calidad del trabajo insatisfactoria, \
                            falta de comunicación con el empleado, cambios de alcance, problemas técnicos.\n\
                         4. Si puedes resolver el problema (información, clarificación), hazlo.\n\
                         5. Si el problema requiere intervención humana (reembolso, cambio de empleado, \
                            escalación), usa request_human_assistance() explicando el problema al staff.\n\
                         6. Sé empático y profesional. El cliente ya está frustrado si llegó a reportar.\n\n",
                    );
                }
            }
        }
        _ => {}
    }
}

/* [T-10] Construye system prompt para IA intermediaria de una orden */
pub(crate) async fn build_intermediary_prompt(pool: &PgPool, order: &crate::models::Order, user_id: Uuid) -> String {
    let mut prompt = String::new();

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
