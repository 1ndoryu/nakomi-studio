# Plan hosting/VPS/test checkout — 2026-05-15

## Tareas

- 155A-11: usuario test de produccion sin pago real en servicios, hosting y VPS.
- 155A-12: agente capaz de asesorar, listar y cobrar VPS.
- 155A-13: separar hosting WordPress y hosting normal, con hosting normal 30% mas caro.
- 155A-14: navegacion con menus contextuales para Soluciones y Servicios.
- 155A-15: eliminar render/redirect indebido al portal VPS desde la SPA principal.
- 155A-16: eliminar flash de contenido legacy en detalle/relacionados de proyectos.

## Decisiones

- Mantener `hosting_plan_configs` como fuente de recursos y precios; diferenciar hosting normal con slugs `normal-basico`, `normal-pro`, `normal-ecommerce` para evitar una migracion amplia de suscripciones.
- El hosting WordPress conserva slugs legacy `basico`, `pro`, `ecommerce`; el hosting normal usa Nginx + SFTP en Coolify y no debe mostrar copy de WordPress en catalogo/panel.
- El bypass de pago no se hardcodea a cualquier cuenta: se activa con env `GLORY_TEST_CHECKOUT_EMAILS`, comma-separated. Produccion debe incluir `test@test.com`.
- VPS reutiliza su dominio existente: catalogo publico, `vps_subscriptions`, checkout Stripe y estado `pending_approval` tras pago/bypass.

## Fases

1. Catalogo y navegacion: rutas de tres soluciones, planes normales + precios 30%, panel con lenguaje generico.
2. Provisioning: compose Coolify de hosting normal con Nginx + SFTP; WordPress queda intacto.
3. Agente: tools VPS y prompt actualizado.
4. Bypass test: servicios, hosting, VPS y respuesta frontend sin Stripe.
5. Rutas y flicker: Nakomi no renderiza el portal VPS por hostname y proyectos no precargan PROYECTOS_DATA antes del CMS.
6. Validacion, deploy, creacion/sync segura de usuario test en produccion.

## Riesgos

- SQLx offline: preferir migraciones que agreguen filas y runtime queries solo donde el dominio ya las usa.
- El usuario test requiere password indicada por el usuario; si no esta disponible tras compaction, debe ingresarla directamente en terminal, nunca en chat.

## Estado final

- Completado en el bloque `155A-11`.
- Servicios y VPS quedaron validados sin Stripe en produccion.
- Hosting comparte el mismo bypass runtime; el reintento directo quedo bloqueado temporalmente por rate limit 429.