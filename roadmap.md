Objetivo: Nakomi Studio — sitio web de agencia creativa. Migrado de WordPress a Rust (Axum) + React SPA.
Rama: glory-rust-nakomi
Roadmap de tareas del proyecto: App/roadmap.md

## Estado: 044A-1 completada (migración SPA)

## Stack implementado

| Capa                 | Herramienta                                    |
| -------------------- | ---------------------------------------------- |
| Framework web        | Axum 0.7                                       |
| OpenAPI              | utoipa 4 + utoipa-swagger-ui 7                 |
| Serialización        | serde                                          |
| Base de datos        | SQLx 0.8 (PostgreSQL)                          |
| Migraciones          | SQLx migrate                                   |
| Validación           | validator 0.18                                 |
| Variables de entorno | dotenvy                                        |
| Logging              | tracing + tracing-subscriber                   |
| Errores              | thiserror 2                                    |
| Auth                 | jsonwebtoken + argon2                          |
| CORS                 | tower-http                                     |
| Linter               | clippy (deny all + warn pedantic)              |
| Frontend             | React 18 + TypeScript + Vite                   |
| State                | React Query + Zustand                          |
| Codegen              | Orval 8 (reemplaza openapi-typescript-codegen) |

## Pendientes

# Nakomi Studio — Roadmap

## Contexto

Proyecto migrado de WordPress a Rust (Axum) + React SPA. El frontend React de App/React/ se integra en frontend/src/. El backend PHP se reemplaza por el template Rust.

---

## Pendientes (por prioridad — lo más difícil primero)

> Plan maestro: `Agente/planes/plan-marketplace-2026-04-04.md` (11 fases) — ✅ completado
> Plan de chat: `Agente/planes/plan-live-chat-2026-04-04.md` (5 fases) — ✅ completado (streaming IA = mejora futura)
> Plan de hosting: `Agente/planes/plan-hosting-coolify-2026-04-04.md` (5 fases) — Fases 3-4 ✅, Fases 1-2-5 bloqueadas por infraestructura externa (VPS2, DNS, Google Drive OAuth)
> Status hosting: `Agente/documentacion/hosting/status-hosting-administrado-2026-04-07.md`
> 084A-24: Contabo API + Stripe checkout + VPS panel admin + resource stats → ✅ completada (3 commits)

## Hosting — ✅ Plan v2 completado (094A-1 a 094A-11)

> Plan v2: `Agente/planes/plan-hosting-v2-2026-04-09.md` — ✅ 10 tareas + UI polish
> Auditoría de seguridad: `Agente/documentacion/hosting/auditoria-seguridad-hosting-2026-04-09.md`
> Completadas: `Agente/completados/tareas-2026-04-09.md`
>
> Resumen: Página detalle individual (6 tabs), self-service + Stripe checkout, dominios DNS,
> SSH/PuTTY, upgrade/downgrade, soporte, stats reales uptime, auditoría seguridad (6 fixes), 21 tests unitarios.

## Chatbot — Plan v2 en ejecución

> Plan maestro: `Agente/planes/plan-chatbot-v2-2026-04-10.md`
> MemPalace evaluado: no aplica (Python/ChromaDB, no justifica agregar al stack Rust)
>
> **Fase I** — Captación de clientes (front-facing): anti-spam, tool use, facturas, memoria, sync, archivos, escalación, branding
> **Fase II** — Clientes registrados: flujo autenticado, IA intermediaria en pedidos

## Pendientes nuevas (por prioridad)

- ~~084A-38: Factura duplicada texto plano — 3 bugs corregidos~~ ✅
- ~~084A-39: Deploy al servidor — Revisar Coolify por deploys anteriores. El usuario dice que ya se hizo un despliegue antes. Investigar.~~ ✅
- ~~084A-40: Comando /reset en chat — borrar todo el contexto de la conversación y usuario para empezar de cero al testear~~ ✅
- ~~084A-41: Ampliar modelos Gemini — 6 modelos en cadena fallback (2.5-flash, 2.5-flash-lite, 2.5-pro, 3-flash-preview, 3.1-pro-preview, 3.1-flash-lite-preview)~~ ✅
- ~~084A-42: Sistema antibots en chat — rate limit por IP + max conexiones WS por IP + cap longitud mensajes~~ ✅
- ~~084A-43: chatAreaMensajes — ajustar altura a la pantalla con min-height:0 en cadena flex~~ ✅
- ~~084A-44: pagosOrdenesLista — rediseñado como tabla profesional con headers, filas clickeables y responsive~~ ✅
- ~~084A-45: hostingDetallePlanOption — minimalista (badges sin color, botón separado, beneficios como lista) + hostingEventos sin padding~~ ✅
- ~~084A-46: Renombrar agente de chat a "Claudia" + reducir delay de respuesta en 3 segundos~~ ✅
- ~~084A-47: Desactivar sistema de relevancia del chatbot (falsos positivos bloquean mensajes legítimos)~~ ✅
- ~~084A-48: Reducir contexto máximo de 64k a 32k tokens + verificar que compresión/resumen funcione al llegar al límite~~ ✅
- ~~084A-49: Mejorar system prompt para redirigir off-topic sin bloquear — reconducción suave en 3 niveles~~ ✅
- ~~084A-50: IA no debe usar markdown — instrucción de texto plano en system prompt~~ ✅
- ~~084A-51: Desactivar show_service/list_services — service cards eliminadas, solo facturas como rich messages~~ ✅
- ~~084A-52: Investigar y corregir error al generar factura con Stripe — parsing defensivo de tool call args + amount_cents robusto~~ ✅

## Notas del usuario 

- ¿Como que no hay? Si antes se hizo un despliegue!!! Revisa bien coolify!!!

- Agrega un comando para reiniciar el chat, asi puedo testear mejor, lo que hare este comando es borrar todo el contexto de la conversacion y usuario para empezar de cero el comando sería " /reset ", 

- gemini-2.5-flash no es el unico modelo, hay mas, hay gemini flash 3.0 y 3.5 creo o 3.1, investiga bien

# Mas tareas

- Creo que hay que verificar que el chat tenga un sistema antibots, para que los bot de internet no consuman los token de la api.

- He dicho muchisimas vez que pagosOrdenesLista se ve muy mal, debería ser una tabla mas profesional. 

- hostingDetallePlanOption se ve mal, tiene que ser minimalista, los badge no tienen que tener color, el boton esta muy pegado, los beneficios deberían ser una lista, y hostingEventos tiene un padding innecesario.

- He dicho esto varias veces, chatAreaMensajes  necesita ajustar al la altura de la pantalla para que no supere la altura y haya un scroll en el chat. 

- El nombre del agente "¡Hola! Soy Nakomi de Nakomi Studio. ¿En qué puedo ayudarte hoy?" Cambialo a Claudia, y presiento que tarda mucho en responder, bajalo 3 segundos

- Vi este problema, a mitad de conversacion fallo, supongo que es el sistema para detectar la relevancia, este sistema falla, creo que lo mejor es desactivarlo

yo: quiero pagar, despues doy detalles

ia: Para generar la factura y enviarte el enlace de pago, necesitaré tu correo electrónico. ¿Me lo puedes compartir, por favor?

yo: andoryyu@gmail.com

Interesante. ¿Hay algo relacionado con nuestros servicios en lo que pueda ayudarte? Ofrecemos diseño web, desarrollo de aplicaciones, branding y agentes IA.

- /reset tiene que borrar los mensajes del front

- Reduce el contexto maximo a 64k a 32k, creo que es mas eficiente asi, verifica que cuando llegue al contexto maximo de una conversacion comprimirla o resumirla funcione.

- Ya que el sistema de relevancia esta desactivado hay que mejorar la ia principal para que no continue con temas que no tiene nada que ver con el proposito (sin basarse unicamente el ultimo mensaje), no hacerlo tran drastico, que intente regresar el usuario al punto y si no lo hace dejar de contestar. (Yo voy a testear esto, espero que /reset me permita volver a hablar si el chatbot decide dejar de hablarme)

- Mira esta conversacion

Hola que tal

¡Hola! Muy bien, gracias. ¿En qué puedo ayudarte hoy?
Quiero contratar el servicio de diseño web, tengo un presupuesto de 100$

🎨
Diseño de Sitios Web
Diseño web profesional adaptado a las necesidades del negocio

Desde $100.00 USD

Diseño de Sitios Web — desde $100.00 USD


¡Perfecto! Aquí tienes la información de nuestro servicio de **Diseño de Sitios Web**:

- **Precio base:** $100 USD
- **Descripción:** Diseño web profesional adaptado a las necesidades de tu negocio.

¿Te gustaría avanzar y generar la factura para iniciar el proyecto? Si es así, ¿podrías compartir tu correo electrónico para enviarte el enlace de pago?
andoryyu@gmail.com

Ha ocurrido un error al generar la factura. Voy a transferir tu solicitud a un miembro de nuestro equipo para que pueda asistirte de inmediato. Gracias por tu paciencia.

El error es que usa ** como si fuera un md
el segund error es que si, me acabo de dar cuenta que usar chatWidgetMsgRich chatWidgetServiceCard para mostrar servicios o cosas es antinatural, solo debe ser para cosas necesarias como pagar facturas.

## Algo que no llegaste a leer: 

el tercer problema es "Ha ocurrido un error al generar la factura. Voy a transferir tu solicitud a un miembro de nuestro equipo para que pueda asistirte de inmediato. Gracias por tu paciencia." Porque fallo en generar la factura? 

- Sigue fallando 

2026-04-09T01:41:39.383770Z  INFO glory_backend::services::ai_chat: AI OK: proveedor=Groq, modelo=openai/gpt-oss-120b, key=gsk_YvZ4..., intento=1
2026-04-09T01:41:39.384628Z DEBUG glory_backend::services::ai_chat: AI tool call: create_invoice({"amount_cents":10000,"client_email":"andoryyu@gmail.com","description":"Diseño web para blog de comida"})
2026-04-09T01:41:41.207263Z ERROR glory_backend::services::ai_tools: Stripe create invoice error: Line item creation failed: {
  "error": {
    "message": "You cannot combine currencies on a single invoice. This invoice has invoice items currency usd that conflicts with the invoice currency mxn.",
    "request_log_url": "https://dashboard.stripe.com/acct_1M9uLoCdHJpmDkrr/workbench/logs?object=req_RXIIFj06CGuImC",
    "type": "invalid_request_error"
  }
}

2026-04-09T01:41:41.209368Z DEBUG glory_backend::services::ai_chat: AI tool call iteration 3, 1 tools executed
2026-04-09T01:41:42.084715Z ERROR glory_backend::services::ai_chat: AI Groq [openai/gpt-oss-120b] key gsk_2uFi... intento 1/3: HTTP 400 Bad Request: {"error":{"message":"Tool choice is none, but model called a tool","type":"invalid_request_error","code":"tool_use_failed","failed_generation":"{\"name\": \"create_invoice\", \"arguments\": {\"amount_cents\":10000,\"client_email\":\"andoryyu@gmail.com\",\"description\":\"Diseño web para blog de comida\"}}"}}

2026-04-09T01:41:42.523320Z  INFO glory_backend::services::ai_chat: AI OK: proveedor=Groq, modelo=openai/gpt-oss-120b, key=gsk_5Ltv..., intento=2

parece ser algo con el tipo de moneda, todo tiene que ser en usd o genestionarse bien

- Por que el diespliegue costo tanto? Definitivamente coolify funciona muy mal, arreglalo