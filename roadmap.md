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

- Inconsistencias entre el blog de incio y la pagina de blog, algo no cuadra. 
- Del lado de las notificaciones agregar un boton de chat, mostrara los chat al abrir y dar click redigirá al panel abriendo ese chat. 
- en chatBurbuja  chatBurbujaIA los mensajes especiales del agente no funciona, deberían funcionar alli tambien. 
- Despues que le di a cancelar orden (datos de pruebas), parece que las ordenes se duplicaron.
- Elimina el estado pendiente de pago, solo es un estado para cuando se termine una fase y falte pagar por otra. 
- Cuando le doy a reportar un problema, debería abrir el chatbot y el chatbot debe entender que se abrio el chat desde ahi para que el cliente escriba el problema, y sea atentido por el chatbot, tenemos que preparar el chatbot para los problemas comunes de un pedido.
- En los pedidos los empleados no tienen acciones, claramente falta acciones reportar un problema, para delegar, cancelar pedido, para cada cosa tiene que escribir una razon, el reportar problema no abre el chat para los empleados, tienen que escribir la razon, tampoco veo que en el panel de admin haya algo para ver y atender los problemas, esto falta.
- hostingCardCliente no necesitar estart italic, mueve lo que hay en hostingCardRecursos al footer de la tarjeta y quita el precio
- Necesito probar un hosting real, usar la segunda vps que maneja coolify para crear un hosting, emulando como si un usuario lo hubiera comprado, tiene que ser un hosting real, o sea un despliegue dentro de la vps.
- Al completar el checkout de hosting se debe provisionar el sitio real en Coolify y guardar `coolify_site_name` + datos reales del servidor.
- Cuando falle una renovación o se cancele la suscripción, hay que notificar al cliente y sincronizar suspensión/cancelación real en Coolify, no solo en base de datos.
- HostingDetalle debe dejar de usar IP hardcodeada y mostrar VPS/IP reales desde backend.
- Falta que el cliente pueda comprar y manejar dominios en nuestra plataforma. ¿Que es lo que falta?
- Ejecutar Hosting Automation
- sentinel report md tiene muchos Warning, arreglarlos todos si son reales o corregir el falso positivo.
- Hay muchos planes que no estan en la carpeta de completados, revisar si realmente es que tienen cosas pendientes, y en caso de que tengan pendiente, organizar las tareas pendientes aca, y realizar todos planes, lo que necesita accion externa mia organizalo aca en el roadmap. 