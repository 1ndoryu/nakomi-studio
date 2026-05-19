# Arquitectura vps.nakomi.studio — 2026-05-10

## Objetivo

`vps.nakomi.studio` debe ser el portal separado para comprar, revisar y operar VPS/infraestructura Nakomi. La web principal `nakomi.studio` conserva marketing general y enlaces; el subdominio VPS presenta la experiencia específica del producto.

## Estado actual corregido

- El frontend de Nakomi mantiene `/soluciones/vps` como página pública/entrada comercial.
- `vps.nakomi.studio` no debe resolverse desde la SPA principal de Nakomi ni como `extraDomain` del stack `studio`.
- El objetivo correcto del subdominio es el frontend propio de `coolify-manager-rs`, publicado como aplicación separada.
- El panel/admin de Nakomi ya contiene lógica de hosting/VPS reutilizable, pero no debe apropiarse del host dedicado del portal operativo.
- El soporte genérico de `extraDomains` en `coolify-manager-rs` sigue siendo válido para otros stacks Rust, pero no define la arquitectura de `vps.nakomi.studio`.

## Routing y deployment

- `nakomi.studio` y `vps.nakomi.studio` son despliegues separados.
- `nakomi.studio` sigue sirviendo la web principal y sus rutas públicas como `/soluciones/vps`.
- `vps.nakomi.studio` debe desplegar el frontend propio de `coolify-manager-rs`, con su backend/API correspondiente o boundary seguro equivalente.
- Queda descartado montar `vps.nakomi.studio` como `extraDomain` del servicio `studio`.
- El deploy del subdominio debe ejecutarse sobre el repo de `coolify-manager-rs` con verificación de health específica del portal.

## Runtime frontend

- El bundle principal de Nakomi no debe cambiar su home por hostname para capturar `vps.nakomi.studio`.
- Desde 2026-05-15, `nakomi.studio` tampoco conserva fallback local `/portal-vps` ni render por `127.0.0.1`; cualquier portal VPS debe salir del despliegue separado.
- Nakomi solo enlaza al portal VPS dedicado mediante CTAs o launcher.
- El portal de `vps.nakomi.studio` debe tener su propio runtime web y su propia política de auth/permisos.

## Integración con backend existente

- Nakomi ya expone catálogo, checkout y operaciones VPS/hosting reutilizables para marketing y panel interno.
- El portal dedicado podrá reutilizar esos flujos o derivarlos a un boundary propio, pero no debe depender de un alias de dominio montado sobre `studio`.
- La separación de runtime debe venir antes de considerar el subdominio como online/cerrado.

## Actualización 2026-05-15 — venta VPS desde agente

- El agente de cuenta puede listar planes VPS, listar VPS del cliente y crear checkout de VPS con tools backend (`list_vps_plans`, `list_my_vps`, `create_vps_checkout`).
- El prompt obliga a confirmar tier antes de cobrar y comunica que la solicitud queda en revisión cuando el plan requiere aprobación.
- Las cuentas incluidas en `GLORY_TEST_CHECKOUT_EMAILS` no pasan por Stripe real: la suscripción VPS queda `pending_approval` y registra evento `test_checkout_bypassed`.
- La ruta pública `/soluciones/vps` se mantiene en Nakomi como entrada comercial; el portal dedicado `vps.nakomi.studio` sigue siendo un despliegue separado pendiente.
- Las acciones de provisioning y operación crítica continúan fuera del alcance del agente cliente; el agente vende/consulta, no ejecuta infraestructura privilegiada.

## Actualización 2026-05-19 — configurador y catálogo VPS

- `/soluciones/vps` ya no abre el modal genérico de compra para VPS: cada plan deriva a `/soluciones/vps/configurar/:tier`.
- El configurador muestra recursos antes de Stripe: CPU, RAM, storage NVMe/SSD, snapshots, región, velocidad de puerto, tráfico y cargo inicial cuando aplica.
- El catálogo VPS queda alineado con la referencia Contabo visible: Cloud VPS 10/20 y VPS 30/40/50/60, con margen operativo de 5% sobre coste base.
- El plan Cloud VPS 10 conserva cargo inicial de puesta en marcha; el resto no añade setup fee.
- Se retiró el campo de “uso previsto” del flujo de compra y del tool schema del agente; el checkout solo confirma tier y hostname opcional.
- La entrega sigue siendo verificada por el equipo antes del provisioning, pero el copy público ya no usa lenguaje de “anti-fraude”.

## Actualización 2026-05-19 — dominios self-service

- Clientes y admins ven la sección `Dominios` en el panel.
- El check de disponibilidad ahora devuelve cotización anual con margen mínimo de 5% para TLDs habilitados (`.com`, `.net`, `.org`, `.studio`, `.io`).
- `POST /api/hosting/domains/checkout` crea una `domain_order` y redirige a Stripe Checkout en modo pago único.
- El webhook `checkout.session.completed` marca la orden como `paid_pending_registration`; el registro final en Contabo queda pendiente para completar handles WHOIS reales con datos correctos.

## Seguridad operativa

- El portal dedicado no debe leer `settings.json`, tokens de Coolify, claves SSH ni secretos desde frontend.
- Si reutiliza backend de Nakomi o de `coolify-manager-rs`, debe hacerlo a través de auth fuerte y DTOs filtrados.
- Las acciones de provisioning y operación siguen detrás de roles admin hasta que exista RBAC más fino.
- El hecho de que exista `/soluciones/vps` en Nakomi no autoriza a servir el panel operativo desde ese mismo bundle.

## Pendientes seguros

- Desplegar el frontend propio de `coolify-manager-rs` bajo `vps.nakomi.studio`.
- Añadir login dedicado visual y logout dentro del portal VPS.
- Separar roles `viewer/operator/customer` si el portal crece fuera del modelo actual.
- Agregar auditoría específica para acciones VPS críticas con actor, target, duración y resultado.
- Definir CSP/CORS y boundary de backend específicos del subdominio dedicado.

