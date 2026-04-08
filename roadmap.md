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

- En el cms en los botones de 3 puntos no salen opciones de eliminar el contenido, solo archivar, falta opciones directa para eliminar, cambiar status a publicar, etc. → ✅ 084A-10 completada
- elimina la especificacion de color y borde en .editorServicioStatusBtn--activo → ✅ 084A-17 completada
- En la lista de proyecto no se ve lo que hay en el cms se ve otra cosa. → ✅ 084A-11 completada
- En el panel al recergar debería permanecer en la misma tab incluyendo la tab interna, no ir siempre a la primera tab. → ✅ 084A-9 completada
- No veo contenido de prueba para los reembolso, necesito ver como se ven los reembolsos. → ✅ 084A-14 completada (fixture TOML + seed cleanup)

- El boton de 3 puntos de las opciones de hostig no se entiende, claramente es un desastre tantas opciones en vez de ser mas claro "cambiar estado" y luego abrir un modal, en vez de tener los estados sueltos. → ✅ 084A-13 completada
- Nada de lo que esta en hostingCard tiene que estar en (--text-xs); subelo  a sm → ✅ 084A-18 completada
- .hostingCardIcono quita el fondo negro y usa bg accent y pon 140px de ancho. → ✅ 084A-19 completada 
- No hay consistencia visual entre todos los modales → ✅ 084A-8 completada
- la nota en chatInfoSeccion chatInfoSeccionNotas no esta tomando el ancho completo. → ✅ 084A-15 completada
- .hostingEventos no se que es pero quita eso, lo que sea que sea eso no tiene porque verse en un modal. → ✅ 084A-16 completada 
- En modalCompraContenido el precio debería actualizarse segun el tipo de pago que se hará hay que revisar profundamente esto para ver que funcione.  → ✅ 084A-12 completada
- Cuando elijo un plan de hosting abre el chat en vez de abrir modalCompraContenido → ✅ 084A-20 completada
- El historial de pago se ve muy mal, debería verse profesional, como una tabla o algo. → ✅ 084A-21 completada

- Se requiere hacer una auditoría de principios solid a todo el front, todos los archivos para identificar inconsistencias, cosas que deberían ser componentes y no lo son, y cosas que no detecta glory sentinel. → ✅ 084A-22 completada
- Uncaught SyntaxError: The requested module '/src/components/panel/OrdenDetalle.tsx?t=1775654447341' does not provide an export named 'OrdenDetalle' (at SeccionProyectos.tsx:13:9) → ✅ 084A-23 — Error transitorio de Vite HMR, exportación es correcta (named export)
- Cuando llegue un mensaje nuevo estando logeada como admin debería sonar un sonido, y generar una notificación. → ✅ 084A-26 completada 
- Las notificaciones se ven mal, se hace un fondo negro al poner el cursor, en vez de bg-accent. Las notificaciones dberían marcarse como leida al abrirlas, el boton de leerlas es innecesario. → ✅ 084A-27 completada 
- Los planes de los hosting no necesitan modos de pago "modalCompraModos" Lo que si necesita es elegir cuantos meses pagar y generar un descuesto por cada mes pago (maximo 33%) → ✅ 084A-28 completada
- Cuando se esta logeado como admin, en todos los contenidos editables a poner el mouse debería mostrar un boton de editar que abra el modal de editar del cms, en la esquina derecha, deberia ser un boton de 3 puntos con varias opciones, editar, eliminar, archivar, etc. → ✅ 084A-29 completada

- Algo pasa con los servicios y el contenido, primero carga el contenido que habiamos hecho al principio sin el cms y luego aparece el que gestionamos con el cms, aparece rapidamente. → ✅ 084A-30 completada

## Hosting 

- Esta es la tarea mas complicada supongo y tiene que ver con los hosting

Viendo el usuario de prueba cliente, veo que no puedo gestionar los hosting, tal vz porque el plan de hosting esta bloqueado.

He colocado en el env variables de contabo, necesito que las pruebes, lees la documentacion de contabo a ver si conecta, una vez lo intente y no funcionaba a pesar de que estoy copiando los datos correctos, en caso de que no funcione, tendremos que hacer todo manual usando las vps que ya tenemos, tenemos 2 vps, tienes que ser capaz de conectarte a ambas, la configuracion ya debería estar prehecha con coolify porque ya me conecto facil antes.

Sobre auth-drive tengo un problema, en mi pc salen ventanas de cmd a media noche, al menos parece que las copias de seguridad estan funcionando porque veo en mi google drive los archivos.

He agregado un mcp de stripe con el que creo que te puedes encargar de configurar lo necesario. 

Voy a necesitar testear como se ve el panel de hosting, claramente esta es una tarea titanica pero podemos empezar por algo sencillo, no agregaremos gestor de archivos ni nada, algo basico, que el cliente pueda ver la info de su hosting (supongo que simularemos la estadisticas o algo porque como un vps y los hosting estaran en despliegues de coolify no se como podemos hacer como hacen los hosting tradicioanles que claramente son hosting compartidos para mostrar estadistica de almacenamiento, uso cpu, y estas cosas si que sea el uso real de todos los despliegues y solo sea el despliegue del cliente), claramente me estoy dejando muchisimas cosas por fuera, tienes que intuir que me dejo por fuera para que esto sea un servicio de hosting profesional y completo.

## Chatbot

El chat bot parece funcionar, pero necesitamos ocntrolarlo mejor, en el panel, necesitamos controlar 

