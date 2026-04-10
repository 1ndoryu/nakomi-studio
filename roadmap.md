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

## Pendientes 

- Falta que el cliente pueda comprar y manejar dominios en nuestra plataforma. ¿Que es lo que falta?
- Al cancelar/suspender hosting notificar al cliente via email/notificación in-app → ✅ 154A-1

- esto que dices de "El manager está haciendo un git pull dentro del contenedor (flujo WordPress), pero studio es template rust — la actualización debe ser un redeploy de Coolify para rebuild del Docker image. Dejo que termine y uso redeploy:" hay que arreglarlo para que no vuelva a suceder

- Los despliegues en rust no debería dejar el sitio inservible mientras se hace build

- Los menus contextuales en listaServiciosInfo se recortan al tamaño del a tarjeta y probablemente en los otras tarjetas del cms tambien pase. → ✅ 154A-4

- Esto pasa en producción, al intentar crear una orden falla.

Unchecked runtime.lastError: Could not establish connection. Receiving end does not exist.Comprende este error
diseno-web:1 Unchecked runtime.lastError: Could not establish connection. Receiving end does not exist.Comprende este error
m.stripe.network/inner.html#url=https%3A%2F%2Fnakomi.studio%2F&title=Nakomi%20Studio&referrer=https%3A%2F%2Fnakomi.studio%2Fpanel&muid=30dba452-34e4-486e-b610-f9e433bb55348246bd&sid=6b64a63a-3acc-4ea1-8fed-769217c9c1be7ae1ef&version=6&preview=false&__shared_params__[version]=dahlia:1 Unchecked runtime.lastError: Could not establish connection. Receiving end does not exist.Comprende este error
content.js:18 Uncaught (in promise) TypeError: Cannot read properties of undefined (reading 'useCache')
    at le (content.js:18:432164)Comprende este error
polyfill.js:496 Uncaught (in promise) Error: Could not establish connection. Receiving end does not exist.
    at wrappedSendMessageCallback (polyfill.js:496:18)Comprende este error
js.stripe.com/v3/m-outer-3437aaddcdf6922d623e172c2d6f9278.html#url=https%3A%2F%2Fnakomi.studio%2F&title=Nakomi%20Studio&referrer=https%3A%2F%2Fnakomi.studio%2Fpanel&muid=30dba452-34e4-486e-b610-f9e433bb55348246bd&sid=6b64a63a-3acc-4ea1-8fed-769217c9c1be7ae1ef&version=6&preview=false&__shared_params__[version]=dahlia:1 Unchecked runtime.lastError: Could not establish connection. Receiving end does not exist.Comprende este error
/api/orders:1  Failed to load resource: the server responded with a status of 404 () 

- Despues de editar un servicio en el cms las caracteristicas no aparecen y el al abrilo de nuevo en el modal esta en su estado inicial y no en su ultima modificación. → ✅ 154A-2

- El chatbot al abrirlo a veces queda conectando... y solo funciona al recargar a veces. → ✅ 154A-3

- Veo que una vez que el usuario se registra al intentar crear una orden, en el panel debería haber una aviso para crear una contraseña, tampoco debería haber impedimiento que por ejemplo si intenta registrarse con el mismo correo y no tiene contraseña, pues, que le permita registrarse con esa contraseña. → ✅ 154A-5

- https://pagespeed.web.dev/analysis/https-nakomi-studio/yh3fbg56c3?form_factor=mobile revisa, el rendimiento es horrible, hay que arreglar todo. → ✅ 154A-6

- EL primer mensaje inicial al abrir el chat debe ser enviado automaticamente, este no necesita IA puede ser una plantilla pero claro la ia lo necesita en su contexto para no volver a darlo, puede ser algo como. "Hola! Estoy aquí para ayudarte, puedes preguntarme acerca de los servicios, resolver problemas, cualquier duda, etc" no se algo asi

- En el cms de los proyectos se olvido totalmente de la galería y de los iconos de enlace, debería poder elegirse con buscardor. y un pequeño menu de los iconos.

## Hosting

- Lo explique antes, lo solicite antes, pero claramente no se me entiendo, necesito un hosting real para probar en el panel de admin, que sea real, sin wordpress ni nada, quiero ver como se vería un hosting real comprado por el cliente, en este caso yo como el admin sería el cliente y el hosting que tendría en mi panel sería mío, necesito la implementación real para ver las limitaciones y que falta por hacer


## Hosting Automation — ✅ Fases 1-3 completadas (104A-42)

> Commit 2114c91 — provisioning real Coolify post-checkout, cancelación sync, IP/VPS reales en panel
> Fase 4 (dominios y DNS) bloqueada → ver sección Bloqueado

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
- ✅ Fase 1: Provisioning real post-checkout (104A-42)
- ✅ Fase 2: Sync cancelación → delete Coolify (104A-42)  
- ✅ Fase 3: Datos reales en panel — IP/VPS desde backend (104A-42)
- Fase 4: Dominios y DNS — bloqueado (ver sección Bloqueado)

## Notas de infraestructura

- **✅ nakomi.studio**: Desplegado y healthy (104A-42, build completo ~12min primer deploy)
- **✅ 104A-46**: Fix coolify-manager deploy --update para rust template → redirige a redeploy Coolify
- **✅ VPS2 Coolify**: Configurado completamente en settings.json (apiToken, serverUuid, projectUuid) — ver 104A-45 en completados
- **Backups VPS2**: Google Drive OAuth no aplica — backups se realizan en VPS2 directamente
- **Dominios**: Proveedor = Contabo DNS. API keys disponibles (validadas en VPS1). Plan: `Agente/planes/plan-dominios-2026-04-07.md`