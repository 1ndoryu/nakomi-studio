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


- ~~La pagina de disponibles para el usuario empleado se ve mal, hay que reahacerla desde cero, no la entiendo. Igual la pagina delegaciones.~~ ✅ (074A-58: rediseño completo — cards horizontales light con icon panel)

- El contenido de los servicios del CMS no matchea el front. (Nota: 074A-21 conectó servicios públicos a API. Si persiste en blog/proyectos especificar cuál)

- ~~No veo donde se modifican los planes de los servicios.~~ ✅ (074A-66: tab Planes en EditorServicio + PUT endpoint batch save)

- ~~Sigo sin ver hosting de prueba~~ ✅ (074A-23: fix API paths + fixtures re-inserted)

- ~~En el dato de prueba el boton de entregar no sirve~~ ✅ (074A-54: deliver modal con notas + archivos opcionales)

- ~~Al cambiar al usuario de cliente no veo pedidos, ni hosting de prueba~~ ✅ (074A-23: orders/hosting TOML fixtures + hosting API fix)

- ~~Ejecuta el plan Glory-RS Content Fixture System~~ ✅ (074A-22 + 074A-23: sistema completo con @lookup)

- ~~SIGO SIN VER DATOS DE PRUEBA EN EL USUARIO DE CLIENTE~~ ✅ (074A-54: client_name en orders + payment fixtures con 3 pagos de prueba)

- Falta hacer paginas individuales para cada usuario accesibles publicamente, similar a fiver. Iran sus reseñas dadas y recibidas.

- ~~entregablesModal esta agregando un padding innecesario.~~ ✅ (074A-55: padding doble removido)

- ~~SIGO SIN VER NINGUN FUCKING DATO DE PRueBA EN EL USUARIO CLIEnte~~ ✅ (074A-55: query keys dinámicas — cache se invalida al cambiar rol)

- ~~Tambien necesito delegaciones y ordenes para tomar de prueba para el usuario empleado.~~ ✅ (074A-56: 2o empleado, orden awaiting_assignment, 3 delegaciones)

- ~~La tabla de hosting se ve mal, no es asi como debería de verse. Si debería de ser algo similar a proyectosLista.~~ ✅ (074A-57: rediseño cards flex)

- ~~Sobre "(074A-54: client_name en orders + payment fixtures con 3 pagos de prueba)" MENTIRA SIGO SIN VER NADA.~~ ✅ (074A-59: find_first_by_role priorizaba usuario sin datos — ahora prioriza @test.com + fix galería crash + client_name en cards + usePagos role key)

- ~~chunk-RPCDYKBN.js?v=57755138:14032 The above error occurred in the <SeccionGaleriaServicio> component:~~

    ~~at SeccionGaleriaServicio (http://localhost:5173/src/components/servicios/SeccionGaleri~~ ✅ (074A-59: Rules of Hooks violation — useCarruselInfinito se llamaba después de return null condicional)

- ~~el historial de pago se ve mal, creo que tiene que ser algo mas profesional, y a dar click abrir un modal con los detalles.~~ ✅ (074A-61: rediseño filas compactas + modal de detalles)

- ~~Info del visitante sale en el chat de los clientes, eso solo debe estar disponible para el admin.~~ ✅ (074A-60: info visitante restringida a admin)

- ~~Quita los colores de hostingStatus hostingStatus--suspended, no quiero colores.~~ ✅ (074A-62: colores eliminados, estado neutral)

- ~~Los hosting deberían estar en tab como los proyectos.~~ ✅ (074A-63: tabs Activos/Inactivos + sub-componentes extraidos)

- ~~hostingCardTitulo debería estar el nombre del hosting, en vez del nombre del plan.~~ ✅ (074A-63: titulo ahora muestra dominio o client_name, plan va como subtitulo)

- ~~hostingTabs no se ve como proyectosTabs, es una inconsistencia visual.~~ ✅ (074A-64: variante="texto" + type="button" + hover rule)

- ~~No veo que se pueda gestionar los hosting de prueba.~~ ✅ (074A-65: CRUD completo — edit modal + delete en menú contextual, backend PUT/DELETE admin-only)

- ~~No me sale para gestionar los planes de los servicios en el cms.~~ ✅ (074A-66: tab Planes en EditorServicio)

- No veo que el cliente tenga capacidad de gestionar su hosting, hay que revisar lo que falta e implementar.

- Veo muchos planes que no estan en la carpeta de completados, claramente hay revisarlos todos para ver cuales mover y si hay cosas pendientes completarlas todo.

- ~~El menu contextual de 3 punto de los hosting no se ve.~~ ✅ (084A-2: overflow:hidden→visible en hostingCard)

- ~~La parte de planes se ve mal, los colores de las letras no se ven~~ ✅ (084A-1: colores corregidos de dark theme a light theme)

- En los planes las caracteristicas es ven asi [object Object]