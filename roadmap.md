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

## Hosting 

- Esta es la tarea mas complicada supongo y tiene que ver con los hosting

Viendo el usuario de prueba cliente, veo que no puedo gestionar los hosting, tal vz porque el plan de hosting esta bloqueado.

He colocado en el env variables de contabo, necesito que las pruebes, lees la documentacion de contabo a ver si conecta, una vez lo intente y no funcionaba a pesar de que estoy copiando los datos correctos, en caso de que no funcione, tendremos que hacer todo manual usando las vps que ya tenemos, tenemos 2 vps, tienes que ser capaz de conectarte a ambas, la configuracion ya debería estar prehecha con coolify porque ya me conecto facil antes.

Sobre auth-drive tengo un problema, en mi pc salen ventanas de cmd a media noche, al menos parece que las copias de seguridad estan funcionando porque veo en mi google drive los archivos.

He agregado un mcp de stripe con el que creo que te puedes encargar de configurar lo necesario. 

Voy a necesitar testear como se ve el panel de hosting, claramente esta es una tarea titanica pero podemos empezar por algo sencillo, no agregaremos gestor de archivos ni nada, algo basico, que el cliente pueda ver la info de su hosting (supongo que simularemos la estadisticas o algo porque como un vps y los hosting estaran en despliegues de coolify no se como podemos hacer como hacen los hosting tradicioanles que claramente son hosting compartidos para mostrar estadistica de almacenamiento, uso cpu, y estas cosas si que sea el uso real de todos los despliegues y solo sea el despliegue del cliente), claramente me estoy dejando muchisimas cosas por fuera, tienes que intuir que me dejo por fuera para que esto sea un servicio de hosting profesional y completo.

## Notas adicionales sobre el plan de hosting

- Algo que me falto aclarar que para ahorrar costos, comprar vps de contabo solo se puede hacer en caso que tengamos una vps que ya no soporta mas despliegues, tenemos que controlar la ram y uso cpu, esto implica otro plan de optimización para revisar los despliegues actuales. La api de contabo debe documentarse bien, es para los dominios, dns, etc, todo lo que pueda ser necesario para crear un servicio de hosting 100% funcional y completo. Lo mas importante, la acciones de compra de cualquier cosa de contabo requiere confirmación directa del usuario, mejor, no autorizar la IA ni ningun proceso automatico para hacer compras, aún no, hasta que todo este revisado y asegurado de que va a funcionar.

- Esto sería la tarea final de hosting y tambien es una tarea gigante, se necesita revisiones de seguridad profunda, asegurarse que no hayan hackeos, caidas del vps completo debido a malas acciones de los usuarios, etc, revisar todo detadallamente y hacer un plan detallado de preveción y auditoría de seguridad.

## Chatbot

- El chat bot parece funcionar, pero necesitamos controlarlo mejor, en el panel, necesitamos controlar todas las acciones que puede hacer, darles la capacidad de crear facturas, esta es otra tarea grande, se necesita buena planificación y todo debe ser controlable con una interfaz minimalista y moderna en el panel. Probablemente me olvide de muchas cosas, asi que ayudame a intuir que falta.

A continuación, todas estas tareas tienen que poder testearse y comprobarse empiricamente, es decir, tener test o alguna forma de validarse como cumplidas.

Fase I - La atención en el front, clientes que entran a la pagina, resumen todo lo necesario para captar clientes.

Nota: usa este proyecto para la memoria del chatbot, servirá la organizar los contextos eficientemente, en realidad no se si es util o no https://github.com/milla-jovovich/mempalace

1. Hace falta que los clientes puedan enviar imagenes, y archivos, hay que revisar como groq gestiona los archivos para que los agentes puedan leer pdf, documentos, audios, etc, todo tiene que ser manejable. 

2. Hace falta sistema antispam, antiabuso, que se detecte cuando un tema del que habla del usuario no tiene que ver con el tema (usando otro modelo mas pequeño que revise si el mensaje del cliente es digno de procesarse al modelo mas grande), evitar que el cliente mande muchos mensajes en poco tiempo, agregar un tiempo de espera de 3-10 segundos y detenerse mientras el cliente escribe, agregar una inteligencia para detectarse si debe esperar que el cliente escriba o no, pues, este sistema, tiene que emular lo que haría un humano, a veces un cliente manda un mensaje tipo, "tengo un proyecto que es sobre..." y escribe varios mensajes explicando, en esos casos es importante las pausas para que el cliente escriba, no caer tampoco en un vacío donde el modelo no responde proque se quedo esperando que el cliente respondiera.  Esta inteligencia evitara que se genere una respuesta instanea a cada mensaje que el usuario escriba.

3. Capacidades de generar un pedido, solicitar pagos supongo que generando una factura, el contexto tiene que estar optimizado, tiene que leer contenido real de los servicios que esta puestos en el cms, tiene tener acceso a los proyectos en caso de que el cliente pida trabajos de ejemplo, la forma en la que envia toda esta informacion tiene que ser un mensaje especial, no texto plano, en el panel tiene que ser visible cada mensaje especial, cuando genera una factura, cuando comparte un servicio o proyecto, esos mensajes tienen que contener botones de accion, el chat bot tiene que ser capaz de detectar esa acción y procesarla. No se de que me estoy olvidando. 

4. Hay que asegurarse de que el chatbot pueda guardar informacion del usuario en base a su ip, dispositivo, cuenta y generar contexto del usuario para cuando vuelva, tiene que poder crearle una cuenta con su correo, al acceder el usuario con su correo se le pedira una constraseña. Que no se necesite cuenta para realizar pedidos, comprar servicios, para esto lo vital sería capturar el correo del cliente para que después en el panel de login cree una contraseña. Esto requiere seguridad adicional y que el sistema de correo funcione, en caso de que el cliente quiera logearse y no haya puesto una contraseña porque vuelva después debería poder registrarse con el correo que uso para comprar y luego al registrarse tendrá su pedido. 

5. El chatbot tiene que identificar al usuario ya logeado (si es posible de usuarios sin registrarse), los contextos de las conversaciones pasadas tienen que resumirse en la info del usuario para que el chatbot tenga contexto. Para los admin se necesitan contextos distinto pues lo admin y empleados claro que no son clientes, el chatbot tiene que organizar eficientemente todo para que los admin puedan consultar la informacion que necesitan e igual con los empleados. 

6. El mismo chat en todas las ventanas, pestañas y dispositivos para el mismo usuario, nada de una sesion por cada ventana o dispositivo. 

Fase II

Con l




