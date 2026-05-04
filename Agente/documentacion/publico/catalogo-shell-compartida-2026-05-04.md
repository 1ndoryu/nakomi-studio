# Catalogo publico: shell compartida para servicios y proyectos

Fecha: 2026-05-04

## Cambio

Servicios y proyectos publicos ahora comparten una misma shell estructural (`CatalogPageShell`).

## Decisiones

- El hero, el contenedor principal y los espaciados se movieron a `frontend/src/components/layout/CatalogPageShell.tsx` y `CatalogPageShell.css`.
- Se usaron clases nuevas `catalogPage*` para no mezclar esta receta con clases legacy como `serviciosContenedor` o `proyectosContenedor`, que ya tenian otros consumidores.
- `ServiciosIsland` y `ProyectosIsland` solo conservan logica y estilos propios del contenido de cada catalogo.

## Efecto esperado

- Ambas paginas comparten la misma jerarquia visual y el mismo comportamiento responsive.
- El contenedor interno del catalogo deja de sumar padding lateral extra en servicios.
- Los futuros cambios de hero/layout del catalogo se hacen en un solo punto.

## Alcance

La shell compartida cubre estructura publica. No modifica las cards ni la logica de carga de datos de cada pagina.
