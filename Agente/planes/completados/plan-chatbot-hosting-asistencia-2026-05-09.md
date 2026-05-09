# Plan: Chatbot como asistente de hosting — 095A-8

> Creado: 2026-05-09
> Estado: Completado en fase accionable 2026-05-09
> Alcance: chatbot publico + clientes registrados, sin reemplazar controles admin de infraestructura.

## Cierre 2026-05-09

- Se agregaron tools reales para listar planes, consultar hostings propios y crear checkout mensual de hosting.
- El prompt ahora exige confirmar plan/dominio, pedir login cuando corresponde y no simular activaciones ni operaciones admin.
- La tarjeta visual de pago reutiliza el rich message de factura con titulo especifico de hosting.
- Se cubrieron guardrails con tests de login requerido y Stripe requerido.

## Diagnostico inicial

El chatbot ya tiene memoria de visitante, captura de email, factura generica Stripe, escalacion humana y contexto pasivo de hosting para clientes registrados. Tambien existe el flujo backend de hosting self-service (`/api/hosting/subscribe`) y checkout Stripe mensual.

Huecos detectados:

1. No hay tools de IA para listar planes de hosting desde la BD.
2. No hay tool para crear checkout de hosting mensual desde conversacion.
3. El prompt no distingue entre factura generica y suscripcion mensual de hosting.
4. El bot puede hablar del estado de hosting si el cliente esta registrado, pero no puede consultarlo bajo demanda via tool.
5. Activar/provisionar/reiniciar/detener hostings debe seguir siendo admin-only; el bot solo debe explicar estado, generar ticket o escalar.

## Fase 1 — Asesoria y compra guiada (implementar ahora)

- Agregar tool `list_hosting_plans` para leer `hosting_plan_configs` y devolver precios/recursos reales.
- Agregar tool `create_hosting_checkout` para cliente registrado: crea suscripcion pending y checkout Stripe mensual.
- Agregar tool `list_my_hostings` para cliente registrado: devuelve suscripciones y estado.
- Actualizar prompt base con flujo de hosting: asesorar plan, pedir dominio opcional, pedir login si no hay `user_id`, crear checkout solo con confirmacion clara.
- Reutilizar RichMessage tipo `invoice` para mostrar boton de pago sin crear UI nueva.

## Fase 2 — Gestion y soporte seguro

- El bot no ejecuta provisioning, restart, stop ni start directamente.
- Para activacion o problema tecnico: crear `create_support_ticket` con categoria `hosting_issue` y escalar si es urgente.
- En contexto `hosting:{id}`, responder con plan/dominio/estado y pedir descripcion del problema.

## Fase 3 — Cobro operativo avanzado

- Revisar si `create_invoice` debe diferenciar servicios one-off vs hosting mensual.
- Si se requiere cobrar hosting manualmente a visitantes anonimos, usar factura generica solo despues de confirmar email, pero preferir checkout mensual de hosting para clientes registrados.

## Fase 4 — Tests y observabilidad

- Tests unitarios de tool definitions para nuevas tools.
- Tests de `create_hosting_checkout` sin user_id y sin Stripe configurado.
- Logs claros cuando una tool de hosting no puede ejecutarse por falta de login, Stripe o plan invalido.

## Criterio de cierre

- El bot puede asesorar planes con datos reales.
- Un cliente registrado puede pedir contratar hosting y recibir un boton de pago Stripe mensual desde el chat.
- Un cliente registrado puede preguntar por sus hostings y recibir estados reales.
- El bot no simula activaciones ni ejecuta operaciones admin peligrosas.