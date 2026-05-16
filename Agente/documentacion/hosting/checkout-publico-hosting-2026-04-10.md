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

## Actualización 2026-05-16 — contrato de create service con Coolify

- El bypass y el backfill sí estaban disparando el provisioning, pero Coolify devolvía `500 Internal Server Error` al crear el servicio, así que la suscripción seguía `active` sin `server_uuid` ni credenciales SSH/SFTP.
- La causa real no era el frontend ni el gating por email: el cliente Rust de `src/services/coolify.rs` estaba creando stacks compose con un payload incompleto respecto al cliente operativo de `coolify-manager-rs`.
- El `POST /api/v1/services` ahora incluye `instant_deploy: true`, alineado con el manager. Además, si Coolify vuelve a fallar, el backend preserva el body de la respuesta dentro del error para que `coolify_provision_failed` no quede reducido a un `500` opaco.
- Con esto, el siguiente refresh del panel/detalle del hosting test vuelve a intentar el provisioning con el contrato correcto hacia Coolify.

## Actualización 2026-05-16 — compose SSH válido para Coolify

- El siguiente bloqueo real apareció ya con el body preservado: Coolify no podía parsear el compose cerca de `FROM lscr.io/linuxserver/openssh-server:9.9_p2-r0-ls190`, dentro del servicio SSH/SFTP.
- El problema estaba en la generación del YAML para `dockerfile_inline`: los builders de SSH se reescribieron con literales multi-línea más claros y consistentes, y el builder estático además quedó saneado de texto basura de diff que habría roto el hosting normal.
- El backend ahora tiene pruebas que parsean el compose completo con `serde_yaml` para ambos caminos (`basico` y `normal-basico`), de modo que un YAML inválido falla en CI/local antes de llegar a producción.
- Con este cambio, el provisioning vuelve a depender de la respuesta real de Coolify y ya no de un compose roto generado por el backend.
- Verificación real en producción: `POST /api/hosting/subscribe` para `test@test.com` devolvió `201` con la suscripción `87052c71-2dcb-42bf-86ad-4fd7b46638a6` ya provisionada (`server_uuid = ugje5wfljvs6llpw3w2pqh6u`, `server_ip = 173.249.50.44`, `sftp_user = wp_afb12mdbvi`, `sftp_port = 17770`). En el mismo chequeo quedaron reparadas dos suscripciones de prueba previas que seguían activas pero sin infraestructura visible.

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