# Checkout público de hosting

## Fecha
2026-04-10

## Problema

La página pública de hosting abría el mismo `ModalCompra` usado por servicios tradicionales. Ese modal creaba una orden vía `/api/orders` con `service_slug = "hosting"`, pero el catálogo real de órdenes no tiene un servicio `hosting`, así que la compra fallaba con `404` antes de llegar al cobro.

## Solución aplicada

- El flujo público de hosting dejó de usar órdenes + `PaymentIntent`.
- `useModalCompra` ahora detecta el caso `servicioSlug === "hosting"` y llama a `/api/hosting/subscribe`, que es el endpoint real de suscripciones de hosting.
- Los planes visuales `hosting-basico`, `hosting-pro` y `hosting-ecommerce` se normalizan a los slugs backend `basico`, `pro` y `ecommerce`.
- El modal ya no muestra pago anticipado por meses para hosting, porque el backend actual vende una suscripción mensual recurrente en Stripe.
- Se agregó un campo opcional de dominio para enviar contexto útil al alta de la suscripción.
- Antes de redirigir a Stripe Checkout, el flujo fija `panel-tab = hosting` para que el retorno del usuario aterrice en la sección correcta del panel.

## Archivos involucrados

- `frontend/src/hooks/useModalCompra.ts`
- `frontend/src/components/servicios/ModalCompra.tsx`

## Validación

- `npm --prefix frontend run type-check`
- `npm --prefix frontend run build`

## Pendiente relacionado

El checkout público de hosting ya compra la suscripción correcta, pero la plataforma todavía no soporta prepago de varios meses ni compra de dominio como producto separado. Eso sigue cubierto por tareas pendientes de hosting/dominios del roadmap.