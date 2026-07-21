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

## Contexto

Proyecto migrado de WordPress a Rust (Axum) + React SPA. El frontend React se integra en frontend/src/. El backend Rust sirve API + SPA.

## Estado interno reciente

- `275A-3`: hotfix del listado de backups para WordPress/Coolify. El endpoint fallaba con 500 porque `alpine:3.20` usa BusyBox y no soporta `ls --time-style=long-iso`; ahora el listing usa `ls --full-time`, comprueba la existencia del volumen antes de montarlo y el parser acepta timestamps `HH:MM:SS +0000`. Validado con test unitario nuevo y smoke SSH contra el VPS del hosting de prueba.

---

## Tareas pendientes

### 🟦 Producto de Correo para Hosting

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

--------------

Ve un problema, automaticamente cuando se haga un pedido, tiene que asigarse a mi, no importa que ya tenga pedidos asignados, no hay limite para el administrador

el modal para asignar un empleado se ve mal no se porque no es ve como los otros modales

hay un problema, no veo que despues de que tenga una orden asignada no pueda cancelar el pedido, o cambiar el empleado

Donde dice "Empleado asignado" debería de decir "Freelancer asignado"

otro problema grave es la cuestion de que el chat en los pedidos no funciona en tiempo real, no hubo una notificación a mi cuando probe enviar un mensaje como cliente, tambien debería llegar un correo cuando un mensaje pasa 20 minutos sin responderse, y debería mostrar un punto rojo cuando hay mensajes nuevos en el boton de sidebar de mensajes

hay un problema visual con las notificaciones, el texto esta centrado, no debería

otra cosa es que veo que los correos estan duplicados en el codigo para los envio y preview ¿porque? me parece mal a nivel codigo, deberia estar centralizado en plantillas, a demás de que se esta duplicando codigo innecesario

--------------

## 20/07

Ha pasado algo de tiempo con el proyecto inactivo, necesito confirmar varias cosas.

Comprobar que en nakomi los pagos funcionen: comprobe, que ya no hay el problema de antes sobre de que sin pago se creaban las ordenes, bien, ya no se crean ordenes sin pagos, pero, se crean cuentas sin ordenes, eso no debería de pasar, que no se creen cuentas al menso que se haya hecho el pago del servicio; tambien neecesitamos comprobar que los pagos de servicios funcionan como esperan, no he tenido mi primer pago de servicio asi que no puedo saber aun si realmente funciona. 

Algunos detalles más

Comprobar que el chat funciona bien, que el bot redirige al whatsapp, y que cada vez que haya un conversación me llegue un correo y un whatsapp, mi correo es andoryyu@gmail.com y mi whatsapp es +1 (608) 466-8134, esto es importante ya no quiero que las cosas sucedan a ciega, tambien debe llegarme un whatsapp y un correo cuando se haga un pedido, lo de los correo creo que ya funcionaba pero hay verificar que siga funcionando. 

Subir un poco la resolucion a galeriaHeroContenedor y a proyectoGaleriaItem, un 10% mas

En el gestor de contenido Nakomi no puedo agregar comas, lo que impide pues crear varios tag y cosas, mal ahi

