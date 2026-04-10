# Checkout inmediato con fallback al panel

Fecha: 2026-04-10

## Resumen

La compra publica ya no crea la orden para despues mandar al cliente al panel a recien iniciar otro checkout. Ahora `ModalCompra` crea la orden, inicia el `PaymentIntent` de Stripe en el mismo flujo y abre `CheckoutModal` inmediatamente con el `client_secret` ya resuelto.

Si el cliente cancela o cierra ese checkout, el sistema lo lleva al panel con la tab `Mis Proyectos` forzada para que vea la orden pendiente de pago sin depender de la ultima seccion persistida en `sessionStorage`.

## Cambios de flujo

1. `useModalCompra` crea la orden.
2. `useModalCompra` llama `apiInitiatePayment()` en el mismo momento.
3. `ModalCompra` deja de cerrar y pasa a renderizar `CheckoutModal` con `clientSecret` preexistente.
4. `CheckoutModal` reutiliza ese `clientSecret` y no crea un segundo `PaymentIntent`.
5. Si Stripe requiere redireccion, `PANEL_TAB_KEY=proyectos` queda presembrada antes de abrir checkout.
6. Si el cliente cierra el checkout, navega al panel con la orden en `pending_payment`.

## Contrato frontend

- `CheckoutModal` ahora acepta `clientSecret?: string`.
- Si `clientSecret` existe, omite `apiInitiatePayment()` y renderiza `PaymentElement` directamente.
- Si no existe, conserva el comportamiento anterior del panel y crea el intent en ese momento.

## Validacion operativa

- `npm --prefix frontend run type-check`
- `npm --prefix frontend run build`
- Backend local levantado con `target/debug/glory-backend.exe`
- Prueba por API del nuevo tramo publico:
  - `POST /api/auth/quick-register`
  - `POST /api/orders`
  - `POST /api/orders/{id}/pay`
  - `GET /api/orders/{id}` -> status `pending_payment`
  - respuesta de pago con `client_secret` presente

## Gotchas

- Sin reutilizar el `client_secret`, el usuario podia crear un `PaymentIntent` en el flujo publico y otro distinto al entrar al panel.
- Como la tab del panel persiste en `sessionStorage`, navegar a `/panel` sin fijar `PANEL_TAB_KEY` podia esconder la orden recien creada en otra seccion no relacionada.