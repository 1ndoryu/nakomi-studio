# Proyectos públicos sincronizados con CMS — 2026-04-09

## Resumen
- Home y `/proyectos` ahora usan exclusivamente los proyectos publicados que devuelve `GET /api/projects`.
- Se eliminó el fallback silencioso a `PROYECTOS_DATA` cuando la API devuelve vacío o cuando el mapeo produce `[]`.
- El mapeo público quedó centralizado en `mapAdminProjectsToProyectos()` para que carrusel, showcase y listado compartan la misma transformación.

## Archivos implicados
- `frontend/src/data/showcase.ts`
- `frontend/src/components/home/CarruselShowcase.tsx`
- `frontend/src/components/home/SeccionShowcase.tsx`
- `frontend/src/islands/ProyectosIsland.tsx`

## Notas técnicas
- `buildCategoriasShowcase()` ahora distingue entre `undefined` y `[]`: ausencia de argumento sigue usando fallback local, pero un array vacío del CMS se respeta como estado real.
- `ProyectosIsland` dejó de mantener un estado local inicial con datos demo y consume el mismo query público que la home.
- El filtro memorizado de `/proyectos` volvió a depender de `proyectos`, evitando que la UI se quedara mostrando datos viejos hasta tocar un filtro.
- Actualización 2026-05-15: el detalle `/proyectos/:slug` ya no recibe `PROYECTOS_DATA` desde el router ni lo pinta como fallback inicial. Mientras el CMS responde muestra loading; si falla, muestra 404. Los proyectos relacionados tampoco caen a `PROYECTOS_DATA` durante el fetch.

## Validación
- `npm --prefix frontend run type-check`
- `npm --prefix frontend run build`