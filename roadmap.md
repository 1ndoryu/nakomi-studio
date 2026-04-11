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

# Nakomi Studio — Roadmap

## Contexto

Proyecto migrado de WordPress a Rust (Axum) + React SPA. El frontend React de App/React/ se integra en frontend/src/. El backend PHP se reemplaza por el template Rust.

---

## Pendientes (por prioridad — lo más difícil primero)

> Planes completados: marketplace (11 fases), live-chat (5 fases), hosting v1 (Fases 3-4), hosting v2 (10 tareas + UI polish), hosting automation (Fases 1-3)
> Planes activos: chatbot v2, SEO, seed system, SSH/SFTP seguro
> Status hosting: `Agente/documentacion/hosting/status-hosting-administrado-2026-04-07.md`

- Dominios: verificación DNS implementada (154A-16). Falta: compra de dominios (requiere registrar API), gestión registros DNS via Contabo API, auto-SSL.
- Hosting/Contabo: el error "Contabo rechazó autenticación" ocurre en la tab "Servidores" (solo admin) porque las variables CONTABO_* no están configuradas en el servidor (.env). Poner las credenciales reales de Contabo en las env vars del servicio Coolify resuelve la tab de servidores. Las suscripciones de hosting son independientes de Contabo (vienen de BD) — si no aparece ninguna, es porque ningún cliente ha comprado hosting aún (los datos seed de prueba se crean con `/api/admin/seed`).
- Hosting/Recursos y SSH seguro: Plan creado en `Agente/planes/plan-ssh-sftp-seguro-2026-04-16.md`. Fase 2 completada (openssh-server + resource limits). Pendiente: Fases 1/3/4. Decisiones pendientes: shell vs SFTP-only, quota disco, límites por plan.
- en chatListaSesiones chatListaOculta, aparece chat general, esto es un problema porque por cada usuario se va a crear un chat general, esto tiene que actualizarse cuando la ia consiga el nombre del usuario pero inicialmente tiene que diferenciarse con algun numero o algo porque si todos los nombres son iguales es comlpicado, y en la lista tiene que verse la foto de perfil de con quien se habla.
- Ejecuta el plan # Plan: SSH/SFTP Seguro por Despliegue de Hosting.
- Hay que hacer otro plan super profundo para la seguridad respecto # Plan: SSH/SFTP Seguro por Despliegue de Hosting, y al servicio de hosting en general. **→ Plan creado: `Agente/planes/plan-seguridad-hosting-2026-04-16.md` (5 fases, 20+ items)**
- Necesitamos una auditoría completa al chatbot, necesito ver como se comporta con los usuarios deslogeados, y como lo hace con los logeado, necesito ver como se comporta en cada escenario, para los empleados, para el admin y para los clientes, pues tiene que adaptarse a esos usuarios, tambien tiene que adaptarse cuando se abre el chat desde un intento de reporte por ejemplo. **→ Auditoría completada: `Agente/documentacion/chat/auditoria-chatbot-2026-04-16.md`. 3 bugs críticos, 5 altos, 3 bajos identificados.**
- Hay que adaptar el servicio de hosting para que sea un servicio especializado en wordpress, hosting wordpress y que se entienda eso.
- Revisa que el "disponiblesVacio" este para cuando no haya historial de pago, y revisa donde mas puede faltar.
- Glory Sentinel no tiene mecanismo para hacer respetar principios solid en el codigo de Rust, tenemos que planificar algo, no se como, limite de lineas, y otras cosas que puedan servir. 
- (en planificación) Tampoco hay un mecanismo para el orden las carpetas, a veces veo que una sola carpeta tiene mas de 10 archivos lo cual complica a veces encontrar los archivos correcto, no se si limitar a 10 archivos por carpeta este bien porque habrán casos es lo que es legitimo, asi que habría que agregar algo para poner excepciones, esto con la intención de organizar mejor los archivos y no dejarlos tirados todo dentro de una sola carpeta, 
- Revisar que cuando el chatbot necesite asistencia humana, llegue una notificación y un correo a la cuenta de admin.
- El panel se ve mu mal en la versión movil, tenemos que hacer que el sidebar en movil y tablet, sean botones inferiores y un boton de hamburgueza por claro, no caben todos, dejalos en un nav inferior movil solo con el icono.
- Ejecuta # Plan: Seguridad Integral del Servicio de Hosting, no te preocupes por los hosting actuales, no hay (no tocar nada de la vps1, estamos usando vps2 de prueba), el plan basico no debe contener copias de seguridad ni el medio. las copias de seguridad deben 3 maxima diara y 2 maximas semanal, es decir, se mantienen solo las de los 3 ultimos dias, y la de las 2 ultimas semanas. 
- despues de terminar # Plan: Seguridad Integral del Servicio de Hosting, hacer una segunda auditoría profunda de seguridad a todo el sistmea de hosting. 
- Elimina menuMovilSeparador, es innecesario.

> **Fase I** — Captación de clientes (front-facing): anti-spam, tool use, facturas, memoria, sync, archivos, escalación, branding
> **Fase II** — Clientes registrados: flujo autenticado, IA intermediaria en pedidos

## Delegaciones y pedidos

- En los pedidos veo el boton de cancelar pedido en el usuario de empleado, pero, claro. ¿Que pasa despues? Claramente hace falta implementar como en fiverr algo mas robuzto, en el header del panel tiene que haber un monto en usd de dinero, igualmente para los empleados, si un pedido se cancela, el cliente tiene su dinero de vuelta pero lo tiene que gastar en la plataforma, y luego darle la posibilidad de retirarlo si quiere, asi se gestionara el dinero que ganan los empleados y clientes, esto requiere un plan, requiere planificar como se va retirar el dinero, requiere que todos los proyectos cancelados por el empleado primero reciban una solicitud que el cliente puede aceptar o rechazar en el historial con la explicación del empleado, si el cliente acepta recibe el dinero, si el cliente rachaza, el proyecto queda libre para ser atendido por otro usuario. Esto implica arreglar otra cosa que no he planteado, todos los proyectos nuevos tienen que estar visibles solamente para el admin. Esto implica la siguiente inconsistencia

1. No hay forma de delegar un proyecto, o sea, como se delega? como el admin ve los proyectos sin delegar? claramente en el panel debe haber un tab para gestionar esto por el admin, con los empleados veo que hay un tab de disponible, hay que recordar que los proyetos quedan disponible si el admin no lo ha delegado durante 48 horas (pero eso es lo que falta, un sistema para delegar los proyectos nuevos y que el admin pueda elegir si tomarlo). En delegaciones y disponible hace faltan tab como lo tiene asignados. 

2. Aclaracciones, el flujo que es espero es algo como, el cliente hace un pedido, el pedido queda pendiente de ser tomando alguien, pero por 48 horas queda solo visible por el admin. Hace falta varias cosas en este proceso.

2.2 En 48 horas si no fue revisado por el admin, delegado o tomado por el entonces todos los empleados reciben una notificación de un nuevo pedido, y en el panel pueden decidir si tomarlo o no, y en se momento ellos seran los responsables. 

## Tareas pendientes extraídas de planes activos (104A-26)

### Chatbot v2 (`plan-chatbot-v2-2026-04-10.md`)
- P-1: Refactorizar `chat.rs` (660 líneas → modular)
- P-2: Migración BD para archivos, perfiles, mensajes especiales
- Fase I (8 tareas): anti-spam, generación pedidos, memoria, sync, archivos, escalación, branding
- Fase II (3 tareas): clientes registrados, IA intermediaria en pedidos
- Testing e2e: 8 smoke tests + tests unitarios (`plan-testing-chatbot-e2e-2026-04-10.md`)

### Seed system (`plan-glory-rs-seed-system-2026-04-07.md`)
- Fase 5: Migrar órdenes, chat, reviews, activity log, notifications de Rust a TOML

### SEO (`plan-seo-completo-2026-04-04.md`)
- Fase 2 pendiente: conversión WebP + srcset + width/height explícitos en imágenes
- Fase 3: Pre-rendering para crawlers

### Hosting Automation (`plan-hosting-automation-2026-04-10.md`)
- Fase 4: Dominios y DNS — parcialmente resuelto (154A-16: DNS check). Falta: registrar API, gestión DNS via Contabo, auto-SSL

## Notas de infraestructura

- **nakomi.studio**: Desplegado y healthy en VPS1 (66.94.100.241), Coolify service `do8k4w8swccwwogoc0os0ck0`
- **VPS2 Coolify**: Configurado en settings.json (apiToken, serverUuid, projectUuid)
- **COOLIFY_PROJECT_UUID**: Actualizado a `p8zxtfmipwch1b14kfqnroh0` (project "hosting-test"). El .env local ya tiene esto. **PENDIENTE: actualizar también en prod** (env vars del servicio Coolify de nakomi.studio).
- **WordPress real provisionado**: `blog-demo.nakomi.dev` → `http://wordpress-vpag09kzdkfax34h4ttxukqq.173.249.50.44.sslip.io/wp-admin/install.php`
- **Nota Traefik VPS2**: El `coolify-proxy` se cae por un bug de Docker Compose con IPv6 Gateway. Si vuelve a caer, usar: `ssh -i coolify_key root@173.249.50.44 "docker run -d --name coolify-proxy --restart unless-stopped --network coolify -p 80:80 -p 443:443 --add-host=host.docker.internal:host-gateway -v /var/run/docker.sock:/var/run/docker.sock:ro -v /data/coolify/proxy/:/traefik -l coolify.managed=true -l coolify.proxy=true traefik:v3.6 [flags]"`
- **Dominios**: Proveedor = Contabo DNS. API keys disponibles. Plan: `Agente/planes/plan-dominios-2026-04-07.md`