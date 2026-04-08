# Plan Consolidado de Pendientes

> **Fecha:** 2026-04-07
> **Objetivo:** Resumen unificado de todos los planes activos y sus pendientes.

---

## 1. CMS Admin — Edición de contenidos (`plan-cms-admin-2026-04-07.md`)
**Estado:** Planificación completada, implementación NO iniciada.

**Pendientes (5 fases):**
- Fase 5 (Infra): Uploads, rich text (tiptap), slugs, caché
- Fase 1: CRUD servicios (admin endpoints + editor modal)
- Fase 2: Blog completo (tabla, CRUD, tiptap, páginas públicas)
- Fase 3: Proyectos/portfolio (migrar datos estáticos a BD)
- Fase 4: Equipo (migrar miembros.ts a BD)

**Bloqueadores:** Ninguno. Depende solo de tiempo de implementación.

---

## 2. Compra de Servicios — Flujo restructurado (`plan-compra-servicios-2026-04-05.md`)
**Estado:** Parcialmente implementado. Modal de compra y modos de pago ya existen.

**Pendientes:**
- Fase 2: Guest checkout (comprar sin cuenta, auth lazy)
- Fase 3: Mejoras UX (progress indicator, email confirmación, historial)
- Validación e2e del flujo Stripe completo (captura manual de PaymentIntent)

**Bloqueadores:** Stripe testing en modo live requiere cuenta verificada.

---

## 3. Dominios — Servicio complementario (`plan-dominios-2026-04-07.md`)
**Estado:** Pendiente. Solo plan escrito.

**Pendientes:**
- Decisión de proveedor DNS (Cloudflare, Namecheap API, etc.)
- Modelo de pricing (margen sobre costo, incluido en hosting, etc.)
- Implementación completa: API integración, UI panel, checkout

**Bloqueadores:** Requiere decisión de negocio sobre proveedor y pricing.

---

## 4. Hosting Coolify — Infraestructura (`plan-hosting-coolify-2026-04-04.md`)
**Estado:** Fases 3-4 completadas. Fases 1-2-5 bloqueadas.

**Completado:**
- ✅ Fase 3: Panel admin hosting
- ✅ Fase 4: Panel cliente hosting

**Pendientes:**
- Fase 1: Provisioning automatizado (requiere VPS2 con Coolify)
- Fase 2: DNS automation (requiere proveedor DNS + API)
- Fase 5: Google Drive backups (requiere OAuth setup)

**Bloqueadores:** Infraestructura externa (VPS2, DNS provider, Google Drive OAuth).

---

## 5. SEO — Optimización SPA (`plan-seo-completo-2026-04-04.md`)
**Estado:** Fase 1 completada (10/10 items). Fases 2-3 pendientes.

**Completado:**
- ✅ Fase 1: react-helmet, SEOHead, robots.txt, sitemap.xml, JSON-LD

**Pendientes:**
- Fase 2: Performance (Core Web Vitals, lazy loading, image optimization)
- Fase 3: Analytics e indexación (Google Search Console, schema.org avanzado)

**Bloqueadores:** Ninguno técnico. Performance es prioridad baja.

---

## Priorización recomendada

| Prioridad | Plan | Razón |
|-----------|------|-------|
| 1 | CMS Admin | Mayor impacto funcional, sin bloqueadores |
| 2 | SEO Fase 2 | Performance afecta UX y ranking |
| 3 | Compra Servicios | Flujo de monetización, parcialmente hecho |
| 4 | Dominios | Genera revenue, pero bloqueado por decisiones |
| 5 | Hosting Coolify | Bloqueado por infraestructura externa |
