# Plan: Compra y Gestión de Dominios

> Creado: 2026-04-07
> Estado: **Implementado parcialmente (~80%)** — Contabo integrado con endpoints `/hosting/domains/*`, `domain_orders` table, frontend SeccionDominios, DNS TXT verification. Gaps: auto-renew Stripe subscriptions, 30-day expiry reminders, visual DNS config UI. [Verificado 2026-05-23]

## Objetivo

Ofrecer compra de dominios como servicio complementario al hosting administrado. Los clientes podrían registrar un dominio directamente desde la plataforma al contratar hosting.

## Requisitos previos

1. **Proveedor DNS validado**: Definir proveedor (Contabo DNS, Cloudflare Registrar, Namecheap API, etc.)
2. **API de registrar dominios**: El proveedor debe tener API REST para registrar/renovar/transferir dominios
3. **VPS2 operativo**: Para apuntar nameservers personalizados (ns1/ns2.nakomi.dev)
4. **Precios definidos**: Markup sobre coste de registro del proveedor por TLD

## Flujo propuesto

### Registro de dominio nuevo
1. Cliente selecciona plan de hosting o ya tiene suscripción activa
2. En el panel → hosting → "Agregar dominio"
3. Formulario: buscador de disponibilidad por TLD (.com, .dev, .io, etc.)
4. Si disponible → checkout con Stripe (pago anual)
5. Backend registra dominio via API del proveedor
6. Configura DNS automáticamente (apuntar al servidor Coolify del cliente)
7. SSL via Let's Encrypt (automático en Coolify)

### Renovación
- Stripe subscription anual con auto-renovación
- Webhook Stripe → backend → renovar en proveedor
- Notificación al cliente 30 días antes del vencimiento

### Transferencia
- Opción para transferir dominio existente a nuestra plataforma
- Validación de auth code
- Proceso asíncrono (3-7 días dependiendo del TLD)

## Modelo de datos

```sql
-- Futura migración
CREATE TABLE domains (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hosting_subscription_id UUID REFERENCES hosting_subscriptions(id),
    user_id UUID NOT NULL REFERENCES users(id),
    domain_name VARCHAR(255) NOT NULL UNIQUE,
    tld VARCHAR(20) NOT NULL,         -- 'com', 'dev', 'io'
    registrar VARCHAR(100) NOT NULL,  -- 'cloudflare', 'contabo', etc.
    registrar_domain_id VARCHAR(255), -- ID en el proveedor
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    -- pending → registering → active → expired → transferred
    registered_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    auto_renew BOOLEAN NOT NULL DEFAULT true,
    dns_configured BOOLEAN NOT NULL DEFAULT false,
    stripe_subscription_id VARCHAR(255), -- Suscripción Stripe anual
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE domain_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    domain_id UUID NOT NULL REFERENCES domains(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## Backend endpoints (futuro)

| Método | Ruta | Descripción |
|--------|------|-------------|
| GET | `/api/domains/check/:name` | Verificar disponibilidad |
| POST | `/api/domains` | Registrar dominio |
| GET | `/api/domains` | Listar mis dominios |
| GET | `/api/domains/:id` | Detalle dominio |
| PATCH | `/api/domains/:id/auto-renew` | Toggle auto-renovación |
| POST | `/api/domains/:id/transfer` | Iniciar transferencia |
| GET | `/api/domains/:id/events` | Historial eventos |

## Frontend

- Buscador de dominios integrado en la página de hosting
- Sección "Mis Dominios" en el panel del cliente (tab o parte de hosting)
- Indicadores de estado (activo, expirando, expirado)
- Configuración DNS visual (records A, CNAME, MX)

## Decisiones adoptadas (2026-05-23)

1. **Proveedor**: Contabo (integrado vía `contabo_domains.rs`)
2. **Pricing**: Markup fijo por TLD en `base_domain_cost_cents()` para .com/.net/.org/.studio/.io
3. **Scope MVP**: .com/.net/.org/.studio/.io — precios hardcodeados
4. **Auto-config DNS**: TXT verification + Traefik routing automático
5. **Bundling**: No implementado todavía
6. **Pago**: One-time Stripe `mode=payment` (no subscriptions)

## Decisiones pendientes (todavía sin resolver)

1. **Auto-renew**: No hay Stripe subscriptions anuales para renovación automática
2. **Expiry reminders**: No hay sistema de notificaciones 30 días antes del vencimiento
3. **Visual DNS config UI**: No hay interfaz visual para records A/CNAME/MX
4. **VPS2 custom nameservers**: ns1/ns2.nakomi.dev no implementado

## Dependencias

- 064A-32 completada (página de hosting funcional) ✅
- VPS2 configurado con Coolify (bloqueado por infra)
- API del proveedor DNS validada (bloqueado por decisión de proveedor)
- Stripe Subscriptions integrado (parcialmente listo por hosting)
