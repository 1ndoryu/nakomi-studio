Objetivo: Nakomi Studio — sitio web de agencia creativa. Migrado de WordPress a Rust (Axum) + React SPA.
Rama: glory-rust-nakomi

## Stack

| Capa          | Herramienta                    |
| ------------- | ------------------------------ |
| Framework web | Axum 0.7                       |
| OpenAPI       | utoipa 4 + utoipa-swagger-ui 7 |
| Base de datos | SQLx 0.8 (PostgreSQL)          |
| Validación    | validator 0.18                 |
| Auth          | jsonwebtoken + argon2          |
| Frontend      | React 18 + TypeScript + Vite   |
| State         | React Query + Zustand          |
| Codegen       | Orval 8                        |
| Deploy        | coolify-manager-rs             |

# Nakomi Studio — Roadmap

## Notas de infraestructura

- **nakomi.studio**: VPS1 (66.94.100.241), Coolify service `do8k4w8swccwwogoc0os0ck0`
- **VPS2 Coolify**: Configurado en settings.json
- **Deploy**: Siempre via coolify-manager-rs, nunca desde Coolify UI (ver doc de persistencia volúmenes)
- **Volúmenes**: Documentado en `Agente/documentacion/hosting/coolify-volumenes-persistencia-2026-04-12.md`
- **Admin contactos**: correo `andoryyu@gmail.com`, whatsapp `+1 (608) 466-8134`

## Contexto

Proyecto migrado de WordPress a Rust (Axum) + React SPA. El frontend React se integra en frontend/src/. El backend Rust sirve API + SPA.

---

## Tareas de producto — Correo para Hosting (bloqueado en decisión de proveedor)

Ver análisis completo en `Agente/documentacion/hosting/producto-correo-proveedores-2026-05-26.md`.

**Decisión pendiente (bloqueante):** Elegir proveedor — MXroute ($59/año, más barato, sin API) vs Migadu ($9/mes, API REST). Esto define la arquitectura de provisioning.

- **265A-11 — Fase 1: Aliases/reenvíos gratis con Cloudflare Email Routing.**
  - Configurar MX/SPF/DKIM/DMARC del dominio del cliente apuntando a Cloudflare.
  - Solo reenvío a Gmail/Outlook del cliente (sin IMAP/SMTP).
  - Incluir 3 alias en plan Pro, 5 alias en Avanzado.
  - Sin costo operativo para Nakomi.
  - Backend: `POST /api/hosting/{id}/aliases`, `DELETE /api/hosting/{id}/aliases/{alias}`.
  - Frontend: TabCorreo con lista de aliases y estado DNS.
  - ~8-10h estimado.

- **265A-12 — Fase 2: Buzones IMAP (MXroute o Migadu).**
  - Contratar proveedor y configurar cuenta reseller.
  - Implementar provisioning: crear/suspender/eliminar mailbox vía API (Migadu) o automatización panel (MXroute).
  - Modelos BD: `mail_domains`, `mailboxes`, `mail_events`.
  - Backend: CRUD de buzones, reset password, DNS automático.
  - Frontend: TabCorreo completo con indicadores de estado.
  - Billing: Stripe add-on a $1.50/buzón/mes.
  - ~20-26h estimado.

- **265A-13 — Incluir 1 buzón IMAP gratis en plan Avanzado.**
  - Modificar `hosting_plan_configs` (nuevo campo `included_mailboxes`).
  - Actualizar pricing en frontend y catálogo.
  - Stripe: nuevo price para el add-on.
  - ~3-4h estimado.

---

## Estado interno reciente

- `275A-3`: hotfix del listado de backups para WordPress/Coolify. El endpoint fallaba con 500 porque `alpine:3.20` usa BusyBox y no soporta `ls --time-style=long-iso`; ahora el listing usa `ls --full-time`, comprueba la existencia del volumen antes de montarlo y el parser acepta timestamps `HH:MM:SS +0000`. Validado con test unitario nuevo y smoke SSH contra el VPS del hosting de prueba.
- `20CA`: reorganización del roadmap (20 julio 2026). 14 tareas pendientes agrupadas por dominio.


## Nota

Esta tarea es para orgnizar, en produccion hubo un fallo sobre el cliente guillermo@nakomi.com, habiamos hecho algo para que se creen unos pagos pendientes personalizados para este cliente y registrar unos hosting a su nombre, esto se perdido porque la base de datos se borro y se volvio a crear, hay que volver a restaurar esto, con el detalle de que el ya pago uno de los hosting que habia pendiente (el hosting de cap.wandori.us ya esta suscrito) asi no se como vincular de nuevo la suscripcion de stripe que ya esta realizada

