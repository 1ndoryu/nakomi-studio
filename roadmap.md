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

- Sigo sin ven hosting de prueba, esta mrd la he dicho ya la 4ta vez. 

- En el dato de prueba el boton de entregar no sirve, lo que supone que debería hacer es que abre un modal para escribr algo y adjuntar algo, lo que sea, no es obligatorio adjuntar.

- Al cambiar al usuario de cliente no veo pedidos, ni historial ni hosting de prueba!! De verdad esta mrd de los datos de prueba, necesitamos crear una solución de raiz, un codigo o algo robuzto en glory en que se encargue de los datos de prueba, que se encargue suministrar la logica de contenido, cms, y estas cosas para facilitar la gestión de contenido, tambien hay que actualizar las reglas en .github\instructions\test.instructions.md para que explique mejor /glory-rs es el nucleo de nuestro framework para agregar cosas utiles que sirven para todos los proyectos, agrega la regla de que /glory-rs debe ser agnostico, y que cuando es complete una tarea en la estructura de bajo de - **Sentinel:**  haya - **GLORY-RS:** Evualuando si la logica o lo que se hizo se puede mover a glory-rs para que pueda ser reutlizada en otro proyecto si es necesario, se tiene que pensar en GLORY-RS siempre como algo que sirve como nuestra caja de herramienta permantente. HEMOS ESTADO TOOD EL TIEMPO LUCHANDO CON LOS DATOS DE PRUEBA; NECESITO ALGO INFALIBLE BIEN PLANIFICADO PARA EVITARNOS ESTE PROBLEMA EN EL FUTURO. 

Nueva inforamción: La solución que me ocurre es que glory-rs sirva como cms cuando se necesite, y que pueda inyectar contenido dinamicamente, tiene que ser del tipo de contenido que escribas en un archivo y que se pueda modificar de alli, asi se puede agregar cualquier tipo de datos de prueba donde se necesita y de cualquier tipo, tiene que ser inteligente, si se borra un contenido en el codigo, se borra en la base de dato tambien, esto servira para datos iniciales en los proyectos que se necesiten que tambien sean reales e inyecten al correr e iniciar un proyecto en cualquier parte. 

- Ejecuta el plan # Plan: Glory-RS Content Fixture System (074A-22 completada: Fases 1-4 listas, faltan pedidos en TOML)

- Vi que hiciste varios toml pero no vi que hicieras toml para los pedidos, borras los pedidos y hazlos de nuevo. Y sigo sin ver hosting de prueba. Ni siquiera cuando toy en el usuario de cliente al cambiar en el swift del panel.