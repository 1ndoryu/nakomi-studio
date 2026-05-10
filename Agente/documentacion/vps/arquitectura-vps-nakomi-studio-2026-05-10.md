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
- Nakomi solo enlaza al portal VPS dedicado mediante CTAs o launcher.
- El portal de `vps.nakomi.studio` debe tener su propio runtime web y su propia política de auth/permisos.

## Integración con backend existente

- Nakomi ya expone catálogo, checkout y operaciones VPS/hosting reutilizables para marketing y panel interno.
- El portal dedicado podrá reutilizar esos flujos o derivarlos a un boundary propio, pero no debe depender de un alias de dominio montado sobre `studio`.
- La separación de runtime debe venir antes de considerar el subdominio como online/cerrado.

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

