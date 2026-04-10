# Plan: Hosting Completo — 084A-24 (Tarea Titánica)

**Fecha:** 2026-04-08  
**Prioridad:** Máxima  

---

## Contexto

El backend+frontend CRUD de hosting están completos. Falta: integración Contabo, stats reales, Stripe checkout, provisioning automático. El usuario pide empezar simple: "que el cliente pueda ver la info de su hosting".

## Fases (por complejidad descendente)

### Fase A — Diagnóstico (esta sesión)

1. ✅ Verificar Contabo API con las credenciales del .env
2. ✅ Diagnosticar por qué el panel cliente "no funciona" o está "bloqueado"
3. ✅ Verificar conexión a VPS existentes

### Fase B — Panel cliente mejorado (esta sesión)

4. Agregar vista de recursos al panel del cliente (storage, status, dominio, plan details)
5. Simular estadísticas de uso (CPU, storage, bandwidth) mientras no haya integración real
6. Mejorar UX del HostingCard para clientes: info clara, acciones disponibles

### Fase C — Stripe hosting (próxima sesión)

7. Crear Stripe Products/Prices para los 3 planes de hosting
8. Integrar checkout de suscripción Stripe en el flujo de compra
9. Webhook Stripe → crear/actualizar hosting_subscription

### Fase D — Provisioning (futuro)

10. Endpoint `POST /api/hosting/provision` semi-automático
11. Integración coolify-manager-rs como subprocess
12. Health checks automáticos

## Estado

- Fase A: En progreso
