# Chatbot + Hosting — Asistencia operacional

Fecha: 2026-05-09

## Estado antes del cambio

El chatbot de Nakomi ya podia:

- Responder sobre servicios y precios base.
- Capturar nombre, email e informacion de proyecto.
- Crear facturas genericas de Stripe con tarjeta visual de pago.
- Crear tickets de soporte.
- Leer contexto pasivo de hostings del cliente registrado desde el prompt.

Pero no podia asistir bien en hosting porque no tenia herramientas de dominio especificas. En particular, no podia consultar los planes reales de `hosting_plan_configs`, crear checkout mensual de hosting ni listar hostings del cliente como accion concreta.

## Decision de arquitectura

El chatbot debe usar herramientas reales para acciones concretas y nunca simularlas en texto.

Se define esta frontera:

- Permitido: asesorar planes, listar planes reales, crear checkout de hosting para cliente registrado, listar hostings del cliente y crear tickets de soporte.
- No permitido: provisionar, activar, reiniciar, detener o borrar infraestructura desde el chat. Esas operaciones siguen en endpoints admin/panel y requieren staff.
- Si el usuario pide activacion o gestion tecnica, el bot explica el estado y crea ticket/escalacion.

## Flujo esperado

1. Usuario pregunta por hosting.
2. El modelo llama `list_hosting_plans` si necesita precios/recursos reales.
3. El bot recomienda un plan segun necesidad.
4. Si el usuario confirma compra:
   - Si no esta registrado/autenticado, pide iniciar sesion o crear cuenta.
   - Si esta registrado, llama `create_hosting_checkout` con plan y dominio opcional.
5. El chat muestra una tarjeta de pago con Stripe Checkout.
6. Tras pago, el flujo existente de hosting/Stripe sincroniza la suscripcion.
7. Para activacion o problema operativo, el bot crea ticket/escalacion; no ejecuta provisioning directo.

## Riesgos controlados

- Duplicacion de suscripciones: `HostingStripeService` usa idempotency key por subscription_id, pero cada tool call crea una suscripcion nueva. El prompt debe pedir confirmacion clara antes de llamar la tool.
- Usuarios anonimos: `create_hosting_checkout` devuelve `requires_login` y no crea nada.
- Stripe ausente: la tool falla con mensaje accionable y no deja checkout parcial.
- Activacion: sigue fuera del chatbot para evitar operaciones admin inseguras.
