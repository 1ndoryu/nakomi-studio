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

### CMS Admin (plan: plan-cms-admin-2026-04-07.md)

### SEO y mejoras (plan: plan-seo-completo-2026-04-04.md)

- 074A-14: SEO Fase 2 — Performance (lazy loading, image optimization, Core Web Vitals)

### Bugs / UX reportados por usuario


- La pagina de disponibles para el usuario empleado se ve mal, hay que reahacerla desde cero, no la entiendo. Igual la pagina delegaciones. (Nota: colores CSS corregidos en 074A-15, falta rediseño estructural)

- El contenido de los servicios del CMS no matchea el front. (Nota: 074A-21 conectó servicios públicos a API. Si persiste en blog/proyectos especificar cuál)

- No veo donde se modifican los planes de los servicios.

- ~~Sigo sin ver hosting de prueba~~ ✅ (074A-23: fix API paths + fixtures re-inserted)

- ~~En el dato de prueba el boton de entregar no sirve~~ ✅ (074A-54: deliver modal con notas + archivos opcionales)

- ~~Al cambiar al usuario de cliente no veo pedidos, ni hosting de prueba~~ ✅ (074A-23: orders/hosting TOML fixtures + hosting API fix)

- ~~Ejecuta el plan Glory-RS Content Fixture System~~ ✅ (074A-22 + 074A-23: sistema completo con @lookup)

- ~~SIGO SIN VER DATOS DE PRUEBA EN EL USUARIO DE CLIENTE~~ ✅ (074A-54: client_name en orders + payment fixtures con 3 pagos de prueba)

- Falta hacer paginas individuales para cada usuario accesibles publicamente, similar a fiver. Iran sus reseñas dadas y recibidas.

- entregablesModal esta agregando un padding innecesario.

- Tambien necesito delegaciones y ordenes para tomar de prueba para el usuario empleado.

- La tabla de hosting se ve mal, no es asi como debería de verse. Si debería de ser algo similar a proyectosLista.

- SIGO SIN VER NINGUN FUCKING DATO DE PRueBA EN EL USUARIO CLIEnte AL QUE ACCEDO DESDE sidebarSwitchRole YA ES LA SECTA O NOVENA VEZ QUE LO DIGO