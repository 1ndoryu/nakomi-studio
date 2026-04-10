# Estado vacío consistente en Mis Proyectos — 2026-04-09

## Problema
- `SeccionProyectos` tenía dos vacíos distintos.
- Si no existía ninguna orden, se renderizaba un estado vacío completo con icono, título y descripción.
- Si la tab `Activas` o `Historial` quedaba vacía, solo aparecía un párrafo suelto, rompiendo la jerarquía visual del panel.

## Solución aplicada
- Se extrajo `EstadoVacioProyectos` dentro de `SeccionProyectos.tsx`.
- El mismo bloque visual ahora cubre tres casos:
  - sin órdenes totales
  - sin proyectos activos
  - sin historial todavía

## Validación
- `npm --prefix frontend run type-check`
- `npm --prefix frontend run build`