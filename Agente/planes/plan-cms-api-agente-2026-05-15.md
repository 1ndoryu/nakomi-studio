# Plan: API CMS privada para el agente en producción
**Tarea:** 035A-34  
**Fecha:** 2026-05-15  
**Estado:** planificado

## Contexto

El sitio usa un sistema de fixtures TOML (`content/`) para gestionar usuarios, servicios, planes, fases, etc. El flujo actual para modificar contenido es:
1. Editar el `.toml` localmente
2. Push al repositorio
3. Deploy via coolify-manager-rs (que hace git pull en el contenedor)
4. El servidor detecta cambio y re-sincroniza los fixtures al inicio

**Problema:** El agente no puede modificar contenido de producción directamente sin un ciclo de código completo. Se necesita una API admin autenticada que permita modificar fixtures en runtime.

## Arquitectura propuesta

Los fixtures viven en dos lugares en producción:
- Los archivos `.toml` en el contenedor (desde git)
- Los datos en PostgreSQL (sincronizados desde los TOMLs)

La API puede operar a dos niveles:
1. **Solo BD** (más rápido, más limitado): modifica registros directamente, sin tocar TOMLs. Si el servidor reinicia, los cambios se sobreescriben por el fixture sync.
2. **BD + TOML** (más persistente, más complejo): modifica la BD Y actualiza el TOML en disco, de modo que los cambios sobreviven reinicios.

**Decisión recomendada:** Nivel 1 (solo BD) con un flag que marque registros como "modificados por API" para que fixture sync los salte. Esto es la opción arquitectónicamente correcta.

## Modelo de seguridad

- Todos los endpoints requieren `UserRole::Admin` (igual que `/admin/services`)
- Autenticación por JWT Bearer (mismo sistema existente)
- Los endpoints usan el prefijo `/api/admin/cms/` para diferenciarlos del CRUD admin normal
- No se expone ningún endpoint sin auth — las queries son preparadas (sin interpolación)

## Alcance de la fase 1 (implementación)

### Servicios y planes
Los endpoints de `admin_services.rs` ya existen para leer y guardar servicios/planes. El agente puede usarlos directamente sin nueva API.

Endpoints ya disponibles:
- `GET /api/admin/services` — lista completa con planes y fases
- `PUT /api/admin/services/:id` — actualizar nombre, descripción, estado
- `PUT /api/admin/services/:id/plans` — guardar/reemplazar planes y fases

**Conclusión fase servicios:** ya implementado, no requiere trabajo nuevo.

### Contenido CMS: textos e imágenes del sitio

Contenido que NO tiene CRUD hoy (solo fixtures TOML):
- Textos de landing/home (hero, about, etc.)
- Configuración del sitio (nombre, redes sociales, etc.)
- Blog posts (`blog.rs` tiene lectura pero no escritura)

Para este contenido se necesitan nuevos endpoints.

### Plan de implementación

#### Fase 1 — Leer la API existente (0 código nuevo)
El agente debe primero verificar qué endpoints ya existen y usarlos. Endpoints admin actuales:
- `/api/admin/services` (CRUD completo)
- `/api/admin/users` (CRUD completo)
- `/api/admin/seed` (seed/reset)
- `/api/admin/fixtures` (status + sync)

#### Fase 2 — Endpoint de contenido de configuración global
Nuevo modelo `site_config` o uso de tabla existente para:
- `title`, `tagline`, `about_text`, `contact_email`
- `social_links` (JSON)

Endpoints nuevos:
- `GET /api/admin/cms/config` — leer config actual
- `PUT /api/admin/cms/config` — actualizar config

#### Fase 3 — Blog posts
- `GET /api/admin/cms/posts` — lista todos (incluyendo drafts)
- `POST /api/admin/cms/posts` — crear post
- `PUT /api/admin/cms/posts/:slug` — editar post
- `DELETE /api/admin/cms/posts/:slug` — archivar

#### Fase 4 — Uploads de imágenes via API
El sistema ya tiene `POST /api/uploads` con soporte multipart. El agente puede:
1. Subir imagen vía `/api/uploads`
2. Obtener la URL resultante
3. Actualizar el campo correspondiente vía PUT de config/services

No requiere nuevos endpoints de upload.

## Decisión de diseño: tabla vs TOML

Para config global y blog, se recomienda usar tablas en BD con fixture sync en una sola dirección (TOML → BD al inicio), pero permitir override vía API que marque el registro como `api_managed = true` y lo salta en el próximo sync.

Alternativamente: usar solo BD sin TOMLs para estas entidades nuevas, y no añadir TOMLs correspondientes. Es más simple y limpio.

**Recomendación:** No crear TOMLs para config/blog. Usar BD directamente. Si el agente quiere persistir cambios cross-deploy, hace commit del TOML actualizado vía git (fuera de banda).

## Orden de prioridad real

1. El agente (copilot) puede ya modificar servicios/planes vía los endpoints admin existentes
2. La autenticación funciona con el mismo JWT del panel
3. El único gap real es config global y blog posts

**Pregunta al usuario antes de implementar Fase 2+:** ¿Qué contenido específico necesitas modificar remotamente? ¿Solo servicios/planes, o también textos del landing, blog, configuración del sitio?

## Próximos pasos

1. [ ] Confirmar con el usuario qué contenido específico necesita modificar via API
2. [ ] Verificar que el agente puede autenticarse con credenciales admin vía JWT
3. [ ] Si se confirma gap en config/blog: implementar Fase 2 (tabla + endpoints)
4. [ ] Documentar el endpoint base URL y cómo autenticarse para el agente en producción
