# Facturacion pendiente para hosting y dominios - 2026-05-20

## Contexto

El caso Guillermo necesita mostrar 4 hostings ya existentes en VPS1 y 2 dominios comprados en GoDaddy sin reprovisionar ni mover infraestructura. Dos hostings ya estan pagados, dos hostings deben activar cobro mensual y los dominios `wandori.us` no se cobran.

## Decision

- `hosting_subscriptions.status` sigue siendo estado tecnico (`active`, `pending`, `suspended`, etc.).
- `billing_items.status` representa el cobro pendiente (`pending`, `paid`, `cancelled`).
- Un hosting puede estar `active` y tener un `billing_items.pending` asociado.
- Stripe Checkout cobra items pendientes; el webhook `checkout.session.completed` marca esos items como `paid` sin ejecutar provisioning.

## GoDaddy

Los dominios `materialdepadel.es` y `guillechatbots.es` se representan como items de dominio gestionado manualmente con metadata `registrar = GoDaddy`, `renewal_date = 2027-01-14` y `transfer_status = manual_review`.

La transferencia automatica queda fuera de este corte: el panel muestra el cobro y deja trazabilidad para que admin revise el traspaso antes de enviar credenciales al cliente.

## Preview local

- Usuario objetivo: `guillermo@nakomi.com`
- Produccion no usa TOML para este caso. Un admin debe ejecutar `POST /api/admin/client-bootstrap/guillermo` con `temporary_password` para crear/vincular solo cliente, hostings existentes y cobros pendientes.
- Hostings: `materialdepadel.es`, `guillechatbots.es`, `cap.wandori.us`, `restaurante.wandori.us`
- Cobros pendientes: hosting `guillechatbots.es`, hosting `cap.wandori.us`, dominio `materialdepadel.es`, dominio `guillechatbots.es`
- Renovacion dominios GoDaddy: `2027-01-14`, $15 por dominio/año. Los dominios `wandori.us` no se cobran.