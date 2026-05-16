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

## Actualización 2026-05-15 — hosting normal vs WordPress

- El catálogo público distingue dos familias: WordPress conserva los slugs `basico`, `pro`, `ecommerce`; hosting normal usa `normal-basico`, `normal-pro`, `normal-ecommerce`.
- Los planes normales se siembran por migración con precio `CEIL(precio_wordpress * 1.30)`.
- `/soluciones/hosting-wordpress` muestra la oferta WordPress y `/soluciones/hosting` muestra hosting normal. `/soluciones` queda fuera de las rutas SPA válidas para no crear una página intermedia.
- El provisioning de hosting normal usa Nginx + SFTP sin base de datos, WP-CLI ni panel WordPress. El provisioning WordPress mantiene la composición WordPress/MariaDB/SFTP existente.
- El copy público evita el término visible “hosting normal”: comercialmente se presenta como hosting administrado para sitios a medida, landings y frontends, aunque los slugs `normal-*` se conservan por compatibilidad técnica.
- El panel usa copy genérico de “hosting” y solo muestra accesos WordPress cuando el plan no empieza por `normal-`.
- `GLORY_TEST_CHECKOUT_EMAILS` permite cuentas de prueba con checkout bypass: hosting queda `active` sin Stripe real, pero desde `165A-1` además reutiliza el mismo provisioning automático en Coolify que el webhook real de Stripe.
- Los hostings de prueba que habían quedado `active` antes de `165A-1` pero sin `server_uuid`/SSH se autoreparan al volver a cargar el panel del usuario test: `list_subscriptions` y `get_subscription` intentan el provisioning una vez para esas suscripciones huérfanas allowlisted.

## Archivos involucrados

- `frontend/src/hooks/useModalCompra.ts`
- `frontend/src/components/servicios/ModalCompra.tsx`
- `frontend/src/islands/SolucionHostingIsland.tsx`
- `frontend/src/hooks/useHostingCatalog.ts`
- `src/handlers/hosting.rs`
- `src/services/coolify.rs`
- `src/services/test_checkout.rs`
- `migrations/20260515010000_hosting_normal_plans.up.sql`

## Validación

- `npm --prefix frontend run type-check`
- `npm --prefix frontend run build`
- `npm run self-check -- -TareaId 165A-1`

## Pendiente relacionado

El checkout público de hosting ya compra la suscripción correcta, pero la plataforma todavía no soporta prepago de varios meses ni compra de dominio como producto separado. Eso sigue cubierto por tareas pendientes de hosting/dominios del roadmap.