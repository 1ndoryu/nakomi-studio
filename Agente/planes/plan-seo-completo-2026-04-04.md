# Plan SEO Completo — Nakomi Studio
**ID:** 044A-28 | **Fecha:** 2026-04-04

## Diagnóstico
SPA pura (React + Vite) sin SSR, sin meta tags, sin robots.txt/sitemap, sin structured data, sin hreflang. Los crawlers ven un HTML vacío.

## Fases

### Fase 1 — Quick wins (implementables ahora, impacto inmediato)
1. ✅ Instalar `react-helmet-async` para gestión dinámica de `<head>`
2. ✅ Crear componente `SEOHead` reutilizable (title, description, OG, Twitter, canonical)
3. ✅ Agregar `SEOHead` a cada island/página con meta tags específicos
4. ✅ `<html lang>` dinámico basado en i18n
5. ✅ Endpoint `/robots.txt` en Axum
6. ✅ Endpoint `/sitemap.xml` en Axum (URLs estáticas + dinámicas si hay BD)
7. ✅ Structured data JSON-LD: Organization, WebSite, Service, BlogPosting
8. ✅ Favicon
9. ✅ Cache-Control headers para assets estáticos
10. ✅ Fix: trailing slashes consistentes, página 404 real, ruta privacidad

### Fase 2 — Imágenes y rendimiento
11. ✅ Bundle splitting (manualChunks en Vite — react-core, query, editor, stripe, i18n)
12. ✅ `loading="lazy"` en todas las imágenes (13 arregladas + 1 eager para hero)
13. ✅ Preconnect/dns-prefetch para Stripe en index.html
14. Pendiente: WebP para assets estáticos en public/ (23 JPGs) — requiere herramienta de conversión
15. Pendiente: `<picture>` / srcset para responsive images — requiere pipeline de imágenes
16. Pendiente: width/height explícitos en `<img>` — compensado parcialmente por CSS aspect-ratio

### Fase 3 — Pre-rendering para crawlers (impacto máximo)
16. Middleware Axum que detecta crawlers por User-Agent
17. Pre-rendering con headless browser o alternativa estática
18. Servir HTML pre-renderizado a crawlers, SPA a usuarios

## Estado actual
- [ ] Fase 1 en progreso
- [ ] Fase 2 pendiente
- [ ] Fase 3 pendiente
