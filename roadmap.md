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

> Planes completados: marketplace (11 fases), live-chat (5 fases), hosting v1 (Fases 3-4), hosting v2 (10 tareas + UI polish), hosting automation (Fases 1-3), SSH/SFTP seguro (Fases 2-4)
> Planes activos: chatbot v2, SEO, seed system
> Status hosting: `Agente/documentacion/hosting/status-hosting-administrado-2026-04-07.md`

- Hosting/SSH seguro: Pendiente solo Fase 1 (ops: verificación VFS disco en VPS2).
- Seguridad hosting: Casi completo (10/11 áreas). Pendiente: Fase 4.1 DNS ownership, Fase 5 monitoreo — depriorizados.
- Segunda auditoría profunda de seguridad al sistema de hosting (después de completar seguridad hosting).
- Contabo rechazó la autenticación. Revisa CONTABO_API_PASSWORD y las credenciales OAuth2 configuradas.
- Hay que revisar que haya un limite pre deploy en coolify-manager para no llenar la memoria.

> **Fase I** — Captación de clientes (front-facing): anti-spam, tool use, facturas, memoria, sync, archivos, escalación, branding
> **Fase II** — Clientes registrados: flujo autenticado, IA intermediaria en pedidos

## Delegaciones y pedidos (pendiente: diseño completo)

> T1-withdrawal, T2-assignment, T3-wallet-header completados.

Pendiente: sistema robusto de delegación tipo Fiverr. Flujo esperado:
1. Cliente crea pedido → visible solo para admin por 48 horas.
2. Si admin no delega en 48h → empleados ven notificación y pueden tomarlo.
3. Cancelaciones: empleado envía solicitud → cliente acepta (dinero a wallet) o rechaza (proyecto queda libre).
4. Wallet: dinero de clientes y empleados, posibilidad de retiro.

## Tareas pendientes extraídas de planes activos (104A-26)

### Chatbot v2 (`plan-chatbot-v2-2026-04-10.md`)
- ~~P-1: Refactorizar `chat.rs` (660 líneas → modular)~~ (204A-3)
- ~~P-2: Migración BD para archivos, perfiles, mensajes especiales~~ (ya implementada)
- ~~Fase I (8 tareas): anti-spam, generación pedidos, memoria, sync, archivos, escalación, branding~~ (ya implementada: T-1 rate limiting + timing, T-2 tool use + facturas Stripe, T-3 visitor_profiles + context summary, T-4 cross-device BroadcastChannel, T-5 upload archivos + AI vision/whisper/PDF, T-6 escalación + notificaciones + email, T-7 branding "Claudia" + no disclosure)
- ~~Fase II (3 tareas): clientes registrados, IA intermediaria en pedidos~~ (ya implementada: JWT auth en WS, registered_client_context, toggle ai_intermediary_enabled por orden)
- Testing e2e: 8 smoke tests + tests unitarios (`plan-testing-chatbot-e2e-2026-04-10.md`)

### Seed system (`plan-glory-rs-seed-system-2026-04-07.md`)
- Fase 5: Migrar órdenes, chat, reviews, activity log, notifications de Rust a TOML

### SEO (`plan-seo-completo-2026-04-04.md`)
- ~~Fase 2: width/height explícitos en imágenes~~ (204A-4). Pendiente: conversión WebP server-side (el proxy /api/img/ ya convierte on-the-fly)
- ~~Fase 3: Pre-rendering para crawlers~~ (114A-SEO3)

### Hosting Automation (`plan-hosting-automation-2026-04-10.md`)
- Fase 4: Dominios y DNS — parcialmente resuelto (154A-16: DNS check). Falta: registrar API, gestión DNS via Contabo, auto-SSL

## Notas de infraestructura

- **nakomi.studio**: Desplegado y healthy en VPS1 (66.94.100.241), Coolify service `do8k4w8swccwwogoc0os0ck0`
- **VPS2 Coolify**: Configurado en settings.json (apiToken, serverUuid, projectUuid)
- **COOLIFY_PROJECT_UUID**: Actualizado a `p8zxtfmipwch1b14kfqnroh0` (project "hosting-test"). El .env local ya tiene esto. **PENDIENTE: actualizar también en prod** (env vars del servicio Coolify de nakomi.studio).
- **WordPress real provisionado**: `blog-demo.nakomi.dev` → `http://wordpress-vpag09kzdkfax34h4ttxukqq.173.249.50.44.sslip.io/wp-admin/install.php`
- **Nota Traefik VPS2**: El `coolify-proxy` se cae por un bug de Docker Compose con IPv6 Gateway. Si vuelve a caer, usar: `ssh -i coolify_key root@173.249.50.44 "docker run -d --name coolify-proxy --restart unless-stopped --network coolify -p 80:80 -p 443:443 --add-host=host.docker.internal:host-gateway -v /var/run/docker.sock:/var/run/docker.sock:ro -v /data/coolify/proxy/:/traefik -l coolify.managed=true -l coolify.proxy=true traefik:v3.6 [flags]"`
- **Dominios**: Proveedor = Contabo DNS. API keys disponibles. Plan: `Agente/planes/plan-dominios-2026-04-07.md`

## Tareas pendientes (usuario)

- ~~los clientes tambien pueden solicitar cancelar el pedido.~~ (204A-13)
- ~~los retiros de dinero, mejor dicho las ganancias solo deben poder retirarse a los 7 días.~~ (204A-11)
- ~~Comisiones, los proyectos atendido por los empleados deben generar 10% de comisión a Nakomi.~~ (204A-12)
- ~~tal vez esta tarea se borro n ose si ya se soluciono, el punto es que a veces la pagina queda en blanco y no carga completa, o carga a la mitad y hay que recargar para que se vea completa. (ACTUALIZACION; EL ERROR ES OTRO SUCEDE LO QUE SIGUIENTE: el body o html se quedan arriba cubriendo la parte arriba con scroll y luego hay otro scroll que al bajar es el div root vacío, he probrado poner     overflow: unset; en el html y se soluciona al parecer, puede que [204A-5] era innecesario)~~ (204A-14)
- ~~Cuando intento subir imagen a un proyecto sucede POST /api/admin/uploads 400 (Bad Request) con un proyecto no sucedía pero con otro si~~ (204A-15)
- ~~con panelSidebar debería estar abajo en la parte inferior movil, el boton 3 puntos alli debería ser una hamburgueza, a demás, no funciona ese boton, en movil panelUsuario no necesita padding, panelContenido tampoco necesita padding en movil~~ (204A-16)
- ~~retiroAdminContenedor se ve mal, no tiene la estructura de los otros paneles, presiento.~~ (204A-17)

- URGENTE; LAS IMAGENES FALLAN https://nakomi.studio/api/img/content/cdc038b3-61bf-4c8b-b7b7-efedb2cef5d8.jpg?w=480&q=80&fmt=webp {"error":"not_found","message":"Imagen no encontrada"}

- veo un problema con el prerender que hiciste, los contenidos del cms no pueden prendererizarse asi, tiene que ser dinamico, tuviste eso en cuenta con el prerender? los proyectos, los servicios son modificables desde el cms.