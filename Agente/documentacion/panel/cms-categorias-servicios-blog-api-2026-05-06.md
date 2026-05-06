# CMS servicios + blog API — 2026-05-06

## Resumen
- Los servicios ahora soportan `categories` persistidas en BD (`services.categories` JSONB), expuestas tanto en `/api/services` como en `/api/admin/services`.
- El editor CMS de servicios permite editar categorías como lista separada por comas y el cargador `scripts/push-services-cms.ps1` ya sincroniza ese campo desde `Agente/documentacion/panel/propuesta-servicios-planes-2026-05-05.json`.
- El catálogo público normaliza categorías tanto desde arrays reales como desde valores CSV heredados del CMS de proyectos para construir filtros solo con categorías presentes.

## Contrato nuevo de servicios
- Request admin create/update: `categories?: string[]`
- Response admin/public: `categories: string[]`
- Migración: `migrations/20260506000000_service_categories.*.sql`
- Backend normaliza el campo con `parse_service_categories()` para evitar filtrar contra JSON crudo.

## Operación blog API en producción
- Se verificó el flujo real contra producción:
  1. Crear post en `/api/admin/blog` con `status = draft`
  2. Publicar con `PUT /api/admin/blog/{id}` y `status = published`
  3. Leer el post desde `/api/blog/{slug}`
  4. Borrar el post temporal con `POST /api/admin/blog/{id}/destroy`
- En `studio` la firma válida para esos JWT admin salió del env runtime `SERVICE_PASSWORD_64_JWTSECRET` obtenido desde Coolify; `JWT_SECRET` no autenticó el endpoint admin actual.

## Nota de validación local
- Para `cargo sqlx prepare`, si la base principal tiene checksums viejos de migraciones ya aplicadas, conviene usar una base temporal limpia y correr ahí `cargo sqlx migrate run` + `cargo sqlx prepare`.
