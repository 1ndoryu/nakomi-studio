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
- ~~P-2: Migración BD para archivos, perfiles, mensajes especiales~~ (ya implementada: migración 20260413, modelos, repositorios completos)
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

## Tareas pendientes (usuario)

(sin tareas pendientes)