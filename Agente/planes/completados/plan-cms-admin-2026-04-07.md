# Plan: Edición de Contenidos desde Admin Panel

> **ID tarea:** 064A-63
> **Fecha:** 2026-04-07
> **Objetivo:** Hacer editables desde el panel admin: servicios, blog, proyectos y equipo, con modal + tabs laterales.

---

## Estado actual

### Qué existe en BD
- `services` + `service_plans` + `service_plan_phases`: catálogo de servicios con planes de precios (creados por seed, read-only desde UI).
- No existen tablas para: blog, proyectos/portfolio, equipo.

### Qué existe en frontend
- Datos estáticos en `data/servicios.ts`, `data/showcase.ts`, `data/miembros.ts`, `data/planes/*.ts`.
- Panel admin tiene secciones para órdenes, chat, usuarios, pagos, reembolsos, pero NADA de contenido editorial.
- `notes` es el único CRUD simple existente (modelo a seguir).

### Endpoints existentes
- `GET /api/services` y `GET /api/services/:slug` (solo lectura).
- No hay CRUD de servicios, blog, proyectos ni equipo.

---

## Arquitectura propuesta

### Nuevo sistema de contenido
Cada tipo de contenido sigue el mismo patron:
1. **Tabla en PostgreSQL** con campos comunes (`id`, `title`, `slug`, `content`, `status`, `created_at`, `updated_at`) + campos específicos.
2. **Endpoints REST** (CRUD completo, admin-only para write).
3. **Sección admin** "Contenido" con tabs laterales por tipo.
4. **Modal editor** con tabs internas para organizar campos.

### Patrón del Modal Editor
```
┌──────────────────────────────────────────┐
│  [General] [Media] [SEO] [Opciones]      │ ← tabs superiores
│─────────────────────────────────────────-│
│  Título:    [________________]            │
│  Slug:      [________________] (auto)     │
│  Descripción: [textarea_____________]     │
│  Contenido:   [rich text editor_____]     │
│                                           │
│           [Guardar borrador] [Publicar]   │
└──────────────────────────────────────────┘
```

---

## Fase 1 — Servicios admin (CRUD completo)

### 1.1 Backend
**Migración:** `alter table services` — añadir campos faltantes:
- `meta_title VARCHAR(255)`, `meta_description TEXT` (SEO)
- `image_url VARCHAR(500)` (imagen principal)
- `gallery JSONB DEFAULT '[]'` (galería de imágenes)
- `skills JSONB DEFAULT '[]'` (habilidades/features del servicio)
- `sort_order INT DEFAULT 0` (orden en listado)

**Endpoints nuevos (admin-only):**
| Método | Ruta | Acción |
|--------|------|--------|
| GET | `/api/admin/services` | Listar todos (incluso inactivos) |
| POST | `/api/admin/services` | Crear servicio |
| PUT | `/api/admin/services/:id` | Actualizar servicio |
| DELETE | `/api/admin/services/:id` | Archivar (soft delete) |
| PUT | `/api/admin/services/:id/plans` | Actualizar planes del servicio |

**Modelos:**
- `CreateServiceRequest { title, slug, description, status, image_url, meta_title, meta_description, categories[], skills[] }`
- `UpdateServiceRequest` (mismo, todos opcionales)

### 1.2 Frontend
**Componentes:**
- `SeccionContenido.tsx` — sección admin con tabs laterales: Servicios | Blog | Proyectos | Equipo
- `EditorServicio.tsx` — modal editor con tabs: General | Media | Planes | SEO
- `useContenidoServicios.ts` — hook: listar, crear, editar, eliminar servicios

**Flujo:**
1. Admin abre panel → tab "Contenido" → sub-tab "Servicios"
2. Grid/tabla de servicios con status (publicado/borrador/archivado)
3. Click "+" o click servicio existente → modal `EditorServicio`
4. Tabs del modal:
   - **General:** título, slug (auto-generado), descripción, categorías (multi-select)
   - **Media:** imagen principal (upload), galería (drag & drop)
   - **Planes:** editar planes y características inline (tabla editable)
   - **SEO:** meta title, meta description, vista previa Google
5. Guardar → PUT `/api/admin/services/:id` → invalidar cache → cerrar modal

### 1.3 Migración de datos
- Run seed que copie los datos de `servicios.ts` a la tabla `services` del BD.
- Frontend consume API en vez de datos estáticos.
- Fallback: si API falla, usar datos estáticos como respaldo.

---

## Fase 2 — Blog

### 2.1 Backend
**Migración nueva:** `create table blog_posts`
```sql
CREATE TABLE blog_posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    author_id UUID NOT NULL REFERENCES users(id),
    title VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    excerpt TEXT,
    content TEXT NOT NULL,
    featured_image VARCHAR(500),
    status VARCHAR(20) NOT NULL DEFAULT 'draft', -- draft, published, archived
    tags JSONB DEFAULT '[]',
    meta_title VARCHAR(255),
    meta_description TEXT,
    published_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_blog_posts_status ON blog_posts(status);
CREATE INDEX idx_blog_posts_slug ON blog_posts(slug);
CREATE INDEX idx_blog_posts_published ON blog_posts(published_at DESC);
```

**Endpoints:**
| Método | Ruta | Rol | Acción |
|--------|------|-----|--------|
| GET | `/api/blog` | Público | Listar publicados (paginado) |
| GET | `/api/blog/:slug` | Público | Detalle de post |
| GET | `/api/admin/blog` | Admin | Listar todos (incluso borradores) |
| POST | `/api/admin/blog` | Admin | Crear post |
| PUT | `/api/admin/blog/:id` | Admin | Actualizar post |
| DELETE | `/api/admin/blog/:id` | Admin | Archivar post |

### 2.2 Frontend
**Componentes admin:**
- `EditorBlog.tsx` — modal con tabs: Contenido | Media | SEO | Opciones
- `useContenidoBlog.ts` — hook CRUD
- Rich text editor: usar `@tiptap/react` (liviano, extensible, no requiere backend)

**Componentes públicos:**
- `BlogIsland.tsx` — lista de posts con paginación y tags
- `BlogPostIsland.tsx` — detalle de post individual
- Rutas: `/blog/` (lista), `/blog/:slug` (detalle)

### 2.3 Contenido inicial
- 3-5 posts seed sobre temas relevantes (desarrollo web, IA, branding).

---

## Fase 3 — Proyectos/Portfolio

### 3.1 Backend
**Migración:** `create table projects`
```sql
CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    client VARCHAR(255),
    description TEXT NOT NULL,
    featured_image VARCHAR(500),
    gallery JSONB DEFAULT '[]',
    categories JSONB DEFAULT '[]',
    technologies JSONB DEFAULT '[]',
    links JSONB DEFAULT '[]',  -- [{tipo: 'web', url: '...'}, ...]
    skills JSONB DEFAULT '[]', -- [{titulo: '...', descripcion: '...'}]
    status VARCHAR(20) NOT NULL DEFAULT 'published',
    sort_order INT DEFAULT 0,
    meta_title VARCHAR(255),
    meta_description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Endpoints:** Misma estructura que servicios (admin CRUD + público read-only).

### 3.2 Frontend
- `EditorProyecto.tsx` — modal con tabs: General | Media | Tech | SEO
- Migrar `showcase.ts` a datos de BD.
- ProyectosIsland consume API en vez de datos estáticos.

---

## Fase 4 — Equipo

### 4.1 Backend
**Migración:** `create table team_members`
```sql
CREATE TABLE team_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    role VARCHAR(255) NOT NULL,
    bio TEXT NOT NULL,
    avatar VARCHAR(500),
    linkedin VARCHAR(500),
    github VARCHAR(500),
    sort_order INT DEFAULT 0,
    visible BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Endpoints:** Admin CRUD + público GET.

### 4.2 Frontend
- `EditorMiembro.tsx` — modal simple: datos + avatar upload
- Migrar `miembros.ts` a datos de BD.

---

## Fase 5 — Infraestructura transversal

### 5.1 Upload de archivos
- Endpoint genérico `POST /api/admin/uploads` → guarda en `uploads/` con hash.
- Return URL relativa. Componente `UploadImage` reutilizable con preview + drag-and-drop.
- Para galería: componente `GalleryEditor` con reordenamiento.

### 5.2 Rich text editor
- `@tiptap/react` con extensiones: heading, bold, italic, link, image, code block, list.
- Sanitizar HTML en backend con `ammonia` (Rust) antes de guardar.
- Componente `RichTextEditor.tsx` reutilizable.

### 5.3 Slug auto-generation
- Frontend: genera slug desde título (lowercase, remove accents, replace spaces with dashes).
- Backend: valida unicidad, sugiere alternativa si duplicado.

### 5.4 Caché e invalidación
- React Query con `queryKey` por tipo de contenido.
- Mutaciones invalidan cache automáticamente.
- Fallback a datos estáticos si API no disponible (graceful degradation).

---

## Orden de implementación

| Prioridad | Fase | Estimación | Dependencia |
|-----------|------|------------|-------------|
| 1 | Fase 5 (infra) | — | Ninguna |
| 2 | Fase 1 (servicios) | — | Fase 5 |
| 3 | Fase 2 (blog) | — | Fase 5 |
| 4 | Fase 3 (proyectos) | — | Fase 5 |
| 5 | Fase 4 (equipo) | — | Fase 5 |

Cada fase es independiente (excepto infra). Se pueden paralelizar si hay múltiples agentes.

---

## Decisiones pendientes

1. **Rich text vs Markdown:** ¿Blog usa rich text (tiptap) o Markdown? Rich text es mejor UX para no-técnicos. Markdown es más portable. **Recomendación:** Tiptap con export a HTML.
2. **Image storage:** ¿Local (`uploads/`) o servicio externo (S3, Cloudflare R2)? **Recomendación:** Local para MVP, migrar a R2 después.
3. **Versionado de contenido:** ¿Guardar historial de ediciones? **Recomendación:** No para MVP, añadir después si se necesita.
4. **i18n de contenido:** ¿Contenido en múltiples idiomas? **Recomendación:** Campo `locale` en cada entry + tabs de idioma en el editor. Implementar después de MVP.
5. **Permisos:** ¿Solo admin edita o también empleados? **Recomendación:** Solo admin para MVP. Role `editor` futuro.

---

## Componentes reutilizables a crear

| Componente | Uso |
|-----------|-----|
| `ContentSection.tsx` | Layout base con tabs laterales por tipo de contenido |
| `ContentEditor.tsx` | Modal base con tabs superiores (reutilizable para todos los tipos) |
| `RichTextEditor.tsx` | Editor tiptap con toolbar |
| `UploadImage.tsx` | Upload de imagen con preview y crop |
| `GalleryEditor.tsx` | Gestión de galería con reordenamiento |
| `SlugInput.tsx` | Input de slug con auto-generación desde título |
| `TagsInput.tsx` | Input de tags con autocompletado |
| `StatusBadge.tsx` | Badge de estado (draft/published/archived) |
| `SEOPreview.tsx` | Vista previa estilo Google de meta title/description |
