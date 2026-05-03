# Deeplinks internos del panel y notificaciones

Fecha: 2026-05-03

## Objetivo

Hacer que el panel pueda reabrir detalles internos desde la URL, compartirlos entre usuarios y permitir que una notificación abra el recurso exacto en vez de depender de `storage` local o de links legacy.

## Contrato activo

- `?seccion=...` sigue identificando la tab activa del panel.
- `?order={order_id}` abre el detalle de la orden y resuelve la sección correcta por rol (`proyectos`, `asignados` o `todos-ordenes`).
- `?hostingId={hosting_id}` abre el detalle de hosting dentro de `hosting`.
- `?chat={session_id}` abre la sesión dentro de `mensajes`.
- Se mantiene compatibilidad con links legacy guardados en notificaciones:
  - `/panel/chat?session=...` se normaliza a `?seccion=mensajes&chat=...`
  - `/panel?seccion=ordenes&id=...` se normaliza a `?order=...`
- `frontend/src/App.tsx` también expone el alias `/panel/chat` para que esos links legacy no caigan en la página 404 antes de que la SPA pueda normalizarlos.

## Implementación

- `frontend/src/utils/panelUrlState.ts` centraliza parseo, normalización y escritura de query params del panel.
- `PanelIsland` ahora observa `location.search` y puede rehidratar `seccionActiva` desde la URL, no solo desde `localStorage`.
- `useSeccionProyectos`, `useSeccionHosting` y `useSeccionChat` hidratan su selección desde la URL y la vuelven a persistir cuando cambia.
- `NotificationBell` dejó de renderizar items muertos: ahora resuelve un target desde `link` o desde `reference_type/reference_id` y navega a él.
- `frontend/src/App.tsx` monta `PanelIsland` también en `/panel/chat` para cubrir notificaciones legacy ya persistidas.
- `src/handlers/chat/rest_messages.rs` genera `/panel?order={order_id}` para mensajes de chats de órdenes; si la sesión no pertenece a una orden, cae a `/panel?seccion=mensajes&chat={session_id}`.

## Validación real

- Cliente autenticado abrió `http://localhost:5173/panel?order=a0000001-0001-4000-8000-000000000001` y el panel cargó el detalle de la orden `#1`.
- Cliente autenticado abrió `http://localhost:5173/panel?seccion=hosting&hostingId=d1db9f36-3c58-450c-80e6-2a435d3597dd`, recargó la página y el detalle de hosting permaneció abierto.
- Se generó una notificación real de `new_message` con contenido `copilot-notif-order-link-1777789638436` y al hacer clic abrió la orden `#1` en `?order=...&seccion=proyectos`.
- Un link legacy `http://localhost:5173/panel/chat?session=bf23c932-6c5e-439e-9ec2-ddf823788a58` dejó de caer en 404 y ahora abre la sección `mensajes` con esa conversación seleccionada.