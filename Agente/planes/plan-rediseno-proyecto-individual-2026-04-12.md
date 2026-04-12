# Plan: Rediseño página individual de proyectos (estilo kontrapunkt.com)

**Fecha:** 2026-04-12
**ID Tarea:** 124A-PROJ1
**Referencia:** https://kontrapunkt.com/work/true-anomaly

## Diseño objetivo

1. **Hero:** Izquierda texto breve (descripción corta), Derecha H1 título
2. **Cover image:** featured_image a 100% ancho
3. **Case Introduction:** 
   - Izquierda: Client, Industry (categorías), Deliveries (tecnologías), Links (solo texto sin iconos)
   - Derecha: descripción completa
4. **Galería:** Layout alternante — imágenes "full" (1/1 = 100% ancho) o "half" (1/2 = 50% ancho, 2 por fila)
5. **Proyectos Relacionados:** Grid de 4 columnas (antes 3)

## Fases

### Fase 1: Backend — Modelo gallery_images con layout metadata
- Migración SQL: convertir `gallery JSONB` de `["url1", ...]` a `[{"url": "url1", "layout": "full"}, ...]`
- Agregar `GalleryImage { url: String, layout: String }` al modelo
- Actualizar `ProjectResponse.gallery` de `Vec<String>` a `Vec<GalleryImage>`
- Actualizar handlers create/update para aceptar nuevo formato
- Actualizar repository (SELECT, INSERT, UPDATE)

### Fase 2: Frontend — Componentes nuevos
- Reescribir `SeccionHeroProyecto` → layout kontrapunkt (izq texto, der título)
- Nueva `SeccionPortada` → imagen featured a 100%
- Nueva `SeccionCaseIntro` → 2 columnas (meta + links | descripción)
- Nueva `SeccionGaleriaProyecto` → grid alternante 1/1 y 1/2
- Actualizar `SeccionProyectosRelacionados` → grid 4 columnas
- Actualizar `ProyectoIndividualIsland` → eliminar skills/CTA, integrar nuevas secciones

### Fase 3: CMS — Editor de galería con layout
- Actualizar el editor de galería para mostrar selector "Full" / "Half" por imagen
- Actualizar API frontend para enviar nuevo formato

## Estado
- [ ] Fase 1
- [ ] Fase 2
- [ ] Fase 3
