# Auditoría Flujo de Compra/Provisioning Hosting
**Fecha:** 2026-04-30  
**Tarea:** 304A-3 (Tarea 8)  
**Alcance:** Trazado completo del flujo desde checkout hasta panel del cliente.

---

## 1. Flujo principal: cliente compra hosting desde el panel

```
[Cliente] → Panel → "Contratar WordPress Hosting"
    → HostingPlanSelector (elige plan + dominio)
    → POST /api/hosting/self-subscribe (auth requerida)
        → HostingRepository::create (status: "pending_payment")
    → GET /api/hosting/subscriptions/:id/checkout
        → HostingStripeService::create_checkout_session
            → Stripe API: crea session con metadata {hosting_id, plan, domain}
        → retorna {checkout_url}
    → Cliente redirigido a Stripe Checkout
    → [Stripe procesa el pago]
    → Stripe envía webhook → POST /api/webhooks/stripe
        → PaymentService::verify_webhook_signature (HMAC-SHA256)
        → Deduplicación por event_id
        → PaymentService::handle_webhook (event_type routing)
            → checkout.session.completed:
                → HostingStripeService::handle_checkout_completed
                    → HostingRepository::find_by_id (metadata.hosting_id)
                    → HostingRepository::set_stripe_subscription_id
                    → HostingRepository::update_status("active")
                    → CoolifyService::provision_site (crea WP en VPS)
                    → HostingRepository::set_coolify_site_name + server_ip
                    → HostingRepository::log_event("provisioned")
    → Cliente redirigido a /panel?hosting=success
    → Panel muestra hosting en estado "active"
```

## 2. Flujo admin: asignar hosting a cliente registrado

```
[Admin] → Panel → Hosting → Menú contextual card → "Asignar a cliente"
    → Modal: ingresa email del cliente
    → PATCH /api/hosting/subscriptions/:id/assign (admin only)
        → UserRepository::find_by_email (busca usuario registrado)
        → HostingRepository::assign_user (UPDATE user_id = found.id)
        → HostingRepository::log_event("user_assigned")
    → Frontend: toast "Hosting asignado a {email}"
    → Card muestra badge verde "vinculado"
```

## 3. Flujo admin: generar link de pago para cliente

```
[Admin] → Panel → Hosting → Menú contextual → "Link de pago"
    → GET /api/hosting/subscriptions/:id/checkout (admin role bypasses self-check)
    → Frontend: adminCheckoutMutation copia URL al portapapeles
    → Admin comparte URL manualmente con el cliente
    → Flujo continúa desde punto 1.4 (Stripe Checkout)
```

## 4. Variables de entorno críticas

| Variable | Requerida en | Estado local | Estado prod |
|----------|-------------|-------------|-------------|
| `STRIPE_SECRET_KEY` / `GLORY_STRIPE_SECRET_KEY` | hosting checkout | ✅ | ✅ |
| `STRIPE_WEBHOOK_SECRET` / `GLORY_STRIPE_WEBHOOK_SECRET` | webhook verify | ✅ | ✅ |
| `COOLIFY_BASE_URL` | provision | ✅ | ⚠️ pendiente verificar |
| `COOLIFY_API_TOKEN` | provision | ✅ | ⚠️ pendiente verificar |
| `COOLIFY_SERVER_UUID` | provision | ✅ | ⚠️ pendiente verificar |
| `CONTABO_CLIENT_ID` | dominios | ✅ | ⚠️ pendiente verificar |
| `CONTABO_CLIENT_SECRET` | dominios | ✅ | ⚠️ pendiente verificar |
| `CONTABO_API_USER` | dominios | ✅ | ⚠️ pendiente verificar |
| `CONTABO_API_PASSWORD` | dominios | ✅ | ⚠️ pendiente verificar |

**Nota prod:** Las vars de Coolify y Contabo están configuradas vía Coolify Dashboard
(Settings → Environment Variables del servicio). Si `GET /api/hosting/domains` retorna
error 500 en prod, verificar en Coolify UI que esas vars estén presentes.

## 5. Puntos de fallo detectados

| Punto | Riesgo | Mitigación actual |
|-------|--------|-------------------|
| Webhook no llega (Stripe no configurado) | Alta | Debe configurarse endpoint en Stripe Dashboard: `https://nakomi.studio/api/webhooks/stripe` |
| Coolify provision falla en prod | Media | `status` permanece `"active"` pero sin site. Admin debe reprovision manual vía panel. |
| `hosting_id` no en metadata de Stripe session | Alta | `create_checkout_session` lo agrega explícitamente como metadata. |
| Usuario no encontrado al asignar | Baja | Endpoint retorna 404 con mensaje claro. |
| Token Contabo expirado al listar dominios | Media | Contabo API renueva tokens por OAuth2 client credentials — se regeneran en cada llamada. |

## 6. Checklist de auditoría (manual, ejecutar post-deploy)

- [ ] Registrar hosting en Stripe Dashboard → Webhook → `https://nakomi.studio/api/webhooks/stripe`
- [ ] Eventos a escuchar: `checkout.session.completed`, `invoice.paid`, `payment_intent.succeeded`
- [ ] Hacer una compra de prueba con tarjeta `4242 4242 4242 4242`
- [ ] Verificar que el status cambia a `active` en la DB tras el webhook
- [ ] Verificar que Coolify creó el sitio WordPress (endpoint `/api/hosting/subscriptions/:id/details`)
- [ ] Verificar `GET /api/hosting/domains` retorna dominios (si Contabo env vars están en prod)
- [ ] Test de asignación: crear hosting → asignar a cliente → verificar badge en panel

## 7. Mejoras futuras pendientes (no urgentes)

- Email de bienvenida al cliente cuando su hosting queda activo
- Notificación push cuando el provisioning completa (puede tardar 5-10 min)
- Formulario de compra de nuevos dominios desde el panel (actualmente solo se listan)
- Webhook `customer.subscription.deleted` para marcar hosting como `cancelled` automáticamente
