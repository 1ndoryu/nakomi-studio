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

### Prerequisitos
- ~~P-1: Refactorizar handlers/chat.rs (660 líneas → módulo con 4 archivos)~~ ✅
- ~~P-2: Migración BD (visitor_profiles, chat_attachments, message_type/metadata, ai_intermediary)~~ ✅

### Fase I (por dificultad descendente)
- ~~T-1: Anti-spam + timing inteligente (rate limit WS, clasificador relevancia, máquina de estados para pausas)~~ ✅
- ~~T-2: Generación pedidos + facturas (tool use Groq, mensajes ricos, Stripe invoices, botones acción)~~ ✅
- ~~T-3: Memoria usuario + contexto (visitor_profiles, captura email, resúmenes, contexto por rol)~~ ✅
- ~~T-4: Sync cross-device/tab (BroadcastChannel, multi-conexión WS, sesión única por identidad)~~ ✅
- ~~T-5: Archivos en chat (upload multipart, Groq Vision imágenes, Whisper STT audio, PDF extraction)~~ ✅
- T-6: Escalación humana (detección IA, notificación admin, flujo handoff) ✅
- ~~T-7: Sin disclosure IA (system prompt, branding agente, UI sin indicadores IA)~~ ✅

### Fase II
- ~~T-9: Clientes registrados (detección JWT, contexto de servicios/pedidos/hosting, reportes)~~ ✅
- ~~T-10: IA intermediaria pedidos (toggle por orden, contexto completo, resúmenes automáticos)~~ ✅



################################### 

Esto lo tienes que volver a revisar todo para ver que se hizo o que falto, esta es la solicitud original

## Chatbot

- El chat bot parece funcionar, pero necesitamos controlarlo mejor, en el panel, necesitamos controlar todas las acciones que puede hacer, darles la capacidad de crear facturas, esta es otra tarea grande, se necesita buena planificación y todo debe ser controlable con una interfaz minimalista y moderna en el panel. Probablemente me olvide de muchas cosas, asi que ayudame a intuir que falta.

A continuación, todas estas tareas tienen que poder testearse y comprobarse empiricamente, es decir, tener test o alguna forma de validarse como cumplidas.

Fase I - La atención en el front, clientes que entran a la pagina, resumen todo lo necesario para captar clientes.

Nota: usa este proyecto para la memoria del chatbot, servirá la organizar los contextos eficientemente, en realidad no se si es util o no https://github.com/milla-jovovich/mempalace

1. Hace falta que los clientes puedan enviar imagenes, y archivos, hay que revisar como groq gestiona los archivos para que los agentes puedan leer pdf, documentos, audios, etc, todo tiene que ser manejable. 

2. Hace falta sistema antispam, antiabuso, que se detecte cuando un tema del que habla del usuario no tiene que ver con el tema (usando otro modelo mas pequeño que revise si el mensaje del cliente es digno de procesarse al modelo mas grande), evitar que el cliente mande muchos mensajes en poco tiempo, agregar un tiempo de espera de 3-10 segundos y detenerse mientras el cliente escribe, agregar una inteligencia para detectarse si debe esperar que el cliente escriba o no, pues, este sistema, tiene que emular lo que haría un humano, a veces un cliente manda un mensaje tipo, "tengo un proyecto que es sobre..." y escribe varios mensajes explicando, en esos casos es importante las pausas para que el cliente escriba, no caer tampoco en un vacío donde el modelo no responde proque se quedo esperando que el cliente respondiera.  Esta inteligencia evitara que se genere una respuesta instanea a cada mensaje que el usuario escriba.

3. Capacidades de generar un pedido, solicitar pagos supongo que generando una factura, el contexto tiene que estar optimizado, tiene que leer contenido real de los servicios que esta puestos en el cms, tiene tener acceso a los proyectos en caso de que el cliente pida trabajos de ejemplo, la forma en la que envia toda esta informacion tiene que ser un mensaje especial, no texto plano, en el panel tiene que ser visible cada mensaje especial, cuando genera una factura, cuando comparte un servicio o proyecto, esos mensajes tienen que contener botones de accion, el chat bot tiene que ser capaz de detectar esa acción y procesarla. No se de que me estoy olvidando. 

4. Hay que asegurarse de que el chatbot pueda guardar informacion del usuario en base a su ip, dispositivo, cuenta y generar contexto del usuario para cuando vuelva, tiene que poder crearle una cuenta con su correo, al acceder el usuario con su correo se le pedira una constraseña. Que no se necesite cuenta para realizar pedidos, comprar servicios, para esto lo vital sería capturar el correo del cliente para que después en el panel de login cree una contraseña. Esto requiere seguridad adicional y que el sistema de correo funcione, en caso de que el cliente quiera logearse y no haya puesto una contraseña porque vuelva después debería poder registrarse con el correo que uso para comprar y luego al registrarse tendrá su pedido. O sea, al principio mostrar input para el nombre, luego el coreo, y asi, si que nada de esto bloquee lo mas importante "conseguir que el usuario compre un serivicio"

5. El chatbot tiene que identificar al usuario ya logeado (si es posible de usuarios sin registrarse), los contextos de las conversaciones pasadas tienen que resumirse en la info del usuario para que el chatbot tenga contexto. Para los admin se necesitan contextos distinto pues lo admin y empleados claro que no son clientes, el chatbot tiene que organizar eficientemente todo para que los admin puedan consultar la informacion que necesitan e igual con los empleados. 

6. El mismo chat en todas las ventanas, pestañas y dispositivos para el mismo usuario, nada de una sesion por cada ventana o dispositivo. 

7. Consideraciones, se que antes habia indicado que los usuarios humanos pueden intervenir en las conversaciones, si, deberían pero hay que hacer como probablemente nunca pase, el chatbot tiene que ser capaz de identificar situcaciones en la que necesita intervención humana, y en ese caso, enviar un mensaje o una notifcacion a lo que sea al admin en tiempo real para que intervenga en la conversación, esto no se en que casos pueda suceder pero supondré que alguna vez va a suceder que se necesite asistencia humana real.

8. Por ultimo, evitar la molestia de decirle al usuario que esta hablando con una IA, es innecesario y puede reducir la confianza. 

Fase II - Atención de clientes ya considados o registrados

1. El flujo de atención de clientes que ya esta registrados, esto implica muchas cosas, porque hay que ser cuidadadosos, suponiendo que la funcionalidad de memoria funciona y distingue por usuarios, entonces la IA debería poder ver si el cliente tiene servicios activos, pedidos, hosting, etc, debería ser capaz de atender reportes, siempre y cuando considerando que habrán cosas que necesiten asistencia humana. 

## Mas dtalles de la fase II (probablemente no evaluadas en el plan)

2. Las conversaciones en los pedidos entre cliente y quien atiende el pedido la IA no responde, esos son chat directos, pero, los empleados y admin en las conversaciones de los pedidos pueden activar o desactivar que la IA atienda el cliente, para estos casos la IA necesita un contexto distinto, necesita tener toda la informacion del pedido completa, para estos casos casi siempre se necesitará atención humana, asi que la IA en este caso debe ser como un intermediario, imagina que esto se activa casi siempre para clientes que fastidian mucho, un empleado o admin puede estar agotado y activar esto para que la IA atienda el cliente, en configuraciones tambien debe tener una opcion ver en que pedidos esto esta activado o no. Esto implica una cosa. 

Implica que la IA siempre genera un resumen dentro de los detalles de los pedidos (se puede actualizar), de las solicitudes del cliente, dentro del chat de los pedidos el cliente si debería distiguir entre los mensajes del asistente y el del empleado. 

3. Ya de soporte, para los servicios, tendran boton de soporte, esto abrira el chat del agente, el agente tiene que entender la intención del usuario, abrio el chat desde el boton de soporte del hosting tal o servicio tal, asi mismo tiene que identificarse de donde abrio el chat en todos los casos, no solo para los chat de soporte, y atender la solicitud del cliente, para esto el cliente necesitas capacidades y poder realizar acciones, esto requiere un plan. 

4. Hasta ahora hay muchas tareas, mucho contexto, el agente puede alucinar con tanta información, la organización del contento tiene que hacerse de manera precisa, muy probablmenente todo lo anterior dicho deben ser mcp resumidos que puedan brindar mas detalles, o tal vez lo mejor sea separar en agentes y crear subagentes, no lo se, estudiar y evaluar las tecnicas mas aficientes a aplicar, limitar el contexto a 64k tokens y que genere un resumen detallado cuando supere el limite en una conversacion. 

5. Algo que olvide mencionar es que todas las conversaciones entre la IA y los clientes tienen que guardarse ordenamente para ser evaluadas despues, para retroalimentanción. 

6. Tuve esta conversacion pero

yo: Estoy testeando tus funcionalidades, puedes crear una factura o crear un pedido? para ver como se ve

IA: 
Factura por $100.00 USD — Servicio de prueba

Aquí tienes una factura de prueba y una lista de nuestros servicios. ¿Necesitas algo más?

No veo ningun boton para pagar ni nada en el mensaje, esperaba un mensaje con un boton o algo y un resumen de la facturara para abrir en un modal.

- Hay que hacer auditoría de seguridad a todo el sistema de chat, y si todo es seguro subir los cambios al servidor vps nakomi.studio

## Detalles

useCarruselInfinito.ts:12 Warning: React has detected a change in the order of Hooks called by CarruselShowcase. This will lead to bugs and errors if not fixed. For more information, read the Rules of Hooks: https://reactjs.org/link/rules-of-hooks

   Previous render            Next render
   ------------------------------------------------------
1. useContext                 useContext
2. useContext                 useContext
3. useContext                 useContext
4. useEffect                  useEffect
5. useState                   useState
6. useCallback                useCallback
7. useSyncExternalStore       useSyncExternalStore
8. useEffect                  useEffect
9. useMemo                    useMemo
10. undefined                 useState
   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

    at CarruselShowcase (

chunk-RPCDYKBN.js?v=57755138:11678 Uncaught Error: Rendered more hooks than during the previous render.
    at useCarruselInfinito (useCarruselInfinito.ts:12:45)
    at CarruselShowcase (CarruselShowcase.tsx:42:65)

# La tarea final

Tienes que comunicarte con el chatbot para probarlo y testearlo, eres un modelo superior claramente tienes que evaluarlo en todos los escenarios, auditarlo, evaluarlo, ir anotando fallos, posible mejoras e ir configurando para maximar el mejor resultado posible. La simulación tiene que ser lo mas parecida a un escenario real y el modelo no se tiene que dar cuenta que esta hablando con otra IA, solo tu. Lo mejor para esto es hacer un plan detallado de una lista de pruebas y cosas que verificar y ver si las pasa todas.

Segunda tarea final: revisar que realmente se este usando la rotacion de api en cada solicitud para evitar rate limits, que se esten usando principalmente los modelos mas inteligentes, y que pase a un segundo modelo mas inteligente de groq y luego a un tercero y asi sucesivamente si falla por rate limits. En allowed_models veo que claramente no esta usando los modelos mas inteligente.


