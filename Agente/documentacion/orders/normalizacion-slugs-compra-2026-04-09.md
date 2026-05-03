# Normalización de slugs al crear órdenes — 2026-04-09

## Problema
- El modal de compra enviaba `service_slug` y `plan_slug` usando IDs del catálogo visual (`diseno-web`, `web-basico`, `apps-medio`).
- El backend resuelve servicios y planes con slugs canónicos de base de datos (`diseno-de-sitios-web`, `basico`, `medio`, `avanzado`, `personalizado`).
- Esa deriva hacía que `POST /api/orders` respondiera `404` al no encontrar el plan o el servicio esperado.

## Solución aplicada
- `frontend/src/api/orders.ts` normaliza el payload antes de llamar a `/api/orders`.
- `service_slug` pasa por un mapa de alias legacy → canónico.
- `plan_slug` elimina prefijos de catálogo (`web-`, `apps-`, `ia-`, `branding-`, `ecommerce-`, `seo-`, `mkt-`) cuando el sufijo final coincide con un slug real del backend.

## Alcance
- El fix cubre el flujo de compra público sin exigir reiniciar el backend.
- También protege otros callers del frontend que reutilicen `apiCreateOrder()`.

## Ajuste adicional [094A-24]
- El 404 persistente no venía ya de los slugs canónicos para los 4 servicios publicados, sino de páginas/frontend que seguían ofreciendo servicios fantasma (`ecommerce`, `seo`, `marketing-digital`) ausentes en el backend.
- `ServicioIndividualIsland`, `App.tsx`, `SeccionPlanesServicio`, `SeccionServiciosRelacionados` y `useServicios` dejaron de depender del catálogo estático para la compra.
- El detalle individual ahora carga el servicio real desde `/api/services/:slug`, usa `apiData.plans` como fuente de verdad y devuelve 404 visual si el slug no existe en la API pública.
- Con esto, el usuario ya no puede abrir un checkout contra un servicio inexistente y disparar `Servicio '...' no encontrado` al crear la orden.

## Ajuste adicional [035A-11]
- El alias de `service_slug` para `diseno-web` quedó invertido y seguía enviando `diseno-de-sitios-web` aunque el backend actual, los fixtures y la URL pública ya usan `diseno-web`.
- `frontend/src/api/orders.ts` volvió a tratar `diseno-web` como slug canónico y dejó `diseno-de-sitios-web` solo como alias legacy de entrada.
- Validación real: el flujo público local de `Diseño de Sitios Web` dejó de fallar con `404` en `POST /api/orders` y avanzó hasta el modal de Stripe.

## Validación
- `npm --prefix frontend run type-check`
- `npm --prefix frontend run build`
- Sonda HTTP autenticada: servicios reales (`diseno-de-sitios-web`, `desarrollo-apps`, `agentes-ia`, `branding`) crean órdenes con `201`; servicios fantasma (`ecommerce`, `seo`, `marketing-digital`) reproducían el `404` antes del ajuste.