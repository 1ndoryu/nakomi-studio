# Arquitectura vps.nakomi.studio — 2026-05-10

## Objetivo

`vps.nakomi.studio` debe ser el portal separado para comprar, revisar y operar VPS/infraestructura Nakomi. La web principal `nakomi.studio` conserva marketing general y enlaces; el subdominio VPS presenta la experiencia específica del producto.

## Estado inicial implementado

- El frontend ya tiene `SolucionVpsIsland` con catálogo real desde `/api/vps/public-plans` y checkout con aprobación manual.
- El root de `vps.nakomi.studio` se resuelve a esa experiencia VPS en la SPA.
- `/soluciones/vps` deja de mostrar placeholder y usa la página real de VPS.
- El panel ya incluye infraestructura admin: despliegues Coolify multi-VPS, servidores Contabo y suscripciones VPS.
- `coolify-manager-rs` soporta `extraDomains` para enrutar subdominios extra al mismo stack Rust sin duplicar base de datos.

## Routing y deployment

- Dominio principal del servicio Rust: `nakomi.studio`.
- Dominio extra del mismo servicio: `vps.nakomi.studio`.
- El template Rust genera routers Traefik HTTPS/HTTP adicionales por cada `extraDomains`.
- `vps.nakomi.studio` debe apuntar por DNS a VPS1 `66.94.100.241`.
- El deploy se hace con `coolify-manager-rs deploy-service --name studio`, que sincroniza compose con Coolify y verifica health.

## Runtime frontend

- `window.location.hostname === 'vps.nakomi.studio'` activa la home VPS.
- El mismo bundle sigue sirviendo `nakomi.studio` sin cambiar la home principal.
- API relativa: `/api/...`, por lo que el subdominio consume el mismo backend del stack.

## Backend disponible

- `GET /api/vps/public-plans`: catálogo público.
- `POST /api/vps/subscribe`: checkout Stripe de VPS con aprobación manual.
- `GET /api/vps/subscriptions`: suscripciones del usuario o todas para admin/employee.
- `POST /api/admin/vps/subscriptions/{id}/approve`: provisioning Contabo, solo admin.
- `POST /api/admin/vps/subscriptions/{id}/reject`: rechazo y cancelación, solo admin.
- `GET /api/hosting/deployments`: despliegues Coolify multi-VPS para admin.
- `GET /api/hosting/vps`: inventario Contabo para admin.

## Seguridad operativa

- No se expone `settings.json` ni secretos al navegador.
- El subdominio comparte backend autenticado existente: JWT en endpoints de panel, roles `admin`, `employee`, `client`.
- Las acciones de provisioning VPS siguen detrás de rutas admin.
- El primer corte público no habilita restart/redeploy desde el portal; esas operaciones quedan en `coolify-manager-rs` o panel admin protegido.

## Pendientes seguros

- Añadir login dedicado visual dentro de `vps.nakomi.studio` para reducir dependencia del panel general.
- Separar más adelante roles `viewer/operator/customer` si el portal crece fuera del modelo actual.
- Agregar auditoría específica para acciones VPS críticas con actor, target, duración y resultado.
- Diseñar dashboard read-only compacto para clientes VPS, separado de Hosting WordPress.
- Evaluar CSP/CORS específicos del subdominio cuando se separe en servicio dedicado.

