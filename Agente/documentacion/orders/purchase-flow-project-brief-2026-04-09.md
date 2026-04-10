# Flujo de compra con brief y fases definibles

Fecha: 2026-04-09

## Resumen

El checkout de servicios ya no crea órdenes con información ambigua en `client_notes`. Ahora separa el brief real del cliente en `orders.project_description`, lo muestra en el panel del pedido y permite editarlo mientras la orden siga abierta.

## Backend

- Se agregó la columna `project_description` a `orders` y una migración que hace backfill desde `client_notes` salvo notas operativas de hosting (`Pago anticipado: ...`).
- `CreateOrderRequest` acepta `project_description` como campo propio.
- `OrderResponse` devuelve `project_description` en todos los flujos de órdenes.
- Se añadieron endpoints:
  - `PATCH /api/orders/{order_id}/project-description`
  - `PATCH /api/orders/{order_id}/phases/{phase_number}`
- Para `PaymentMode::Phased`, la creación de la orden genera:
  - Fase 1: `Definición del proyecto`
  - Fases siguientes: placeholders `Fase N por definir`

## Frontend

- `ModalCompra` pide `Describe tu proyecto` para servicios no-hosting antes de continuar.
- `useModalCompra` bloquea compras sin un brief mínimo y envía `project_description` separado de notas operativas.
- `OrdenDetalle` muestra la descripción del proyecto y permite editarla para cliente/admin mientras la orden esté abierta.
- El chat ya no depende de un botón en `ordenInfoCardAcciones`; se mantiene visible debajo del historial cuando aplica.
- `FaseCard` permite que empleado/admin definan título, descripción, precio, días y revisiones de fases bloqueadas o pendientes de pago.

## Contratos y permisos

- Cliente/Admin: puede actualizar la descripción del proyecto si la orden no está completada/cancelada.
- Empleado asignado/Admin: puede definir fases solo en órdenes `phased` y solo si la fase todavía no comenzó.
- Las validaciones rechazan títulos vacíos, precios/días inválidos y updates vacíos.

## Validación operativa

- Para este cambio fue necesario correr, en este orden:
  1. `cargo sqlx migrate run`
  2. `cargo sqlx prepare`
  3. `cargo check`
  4. `cargo clippy -- -D warnings`
  5. `npm --prefix frontend run type-check`
  6. `npm --prefix frontend run build`
- El rebuild del frontend era obligatorio porque el bundle viejo seguía conteniendo `http://localhost:3000` en `frontend/dist`.