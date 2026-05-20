# Plan cliente hosting y dominios - 2026-05-20

## Tarea

- **205A-1:** Onboarding de cliente sin cuenta: 4 hostings reales en VPS1, 2 dominios comprados en GoDaddy y cobros pendientes visibles en Mi Hosting.

## Decisiones

- Separar estado tecnico (`hosting_subscriptions.status`) de estado financiero (`billing_items.status`).
- Mantener los hostings existentes como `active`; la facturacion pendiente no debe reprovisionar ni suspender nada.
- No usar fixtures TOML para datos reales de cliente en produccion. El caso se crea con bootstrap admin idempotente y explicito.
- Representar dominios GoDaddy como cobros/activos gestionados manualmente hasta implementar transferencia automatica.

## Estado

- Backend `billing_items` + API + Stripe Checkout: completado.
- UI de pagos pendientes en Mi Hosting + indicador rojo en sidebar: completado.
- Bootstrap admin `POST /api/admin/client-bootstrap/guillermo`: completado.
- Deploy: no ejecutado en este bloque; requiere revision explicita del bootstrap y entorno productivo.