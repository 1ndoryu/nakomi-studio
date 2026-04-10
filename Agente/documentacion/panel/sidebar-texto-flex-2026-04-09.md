# Sidebar texto sin recorte prematuro — 2026-04-09

## Objetivo

Corregir el recorte temprano de las etiquetas del sidebar del panel sin modificar el contenido ni el ancho general del sidebar.

## Causa

- `sidebarItem` es un botón flex basado en `Button`.
- El label `sidebarItemTexto` no tenía `flex: 1` ni `min-width: 0`.
- En esa configuración, flexbox impide que el texto consuma correctamente el ancho disponible y la elipsis aparece antes de tiempo.

## Cambios aplicados

- `sidebarItem` ahora declara `min-width: 0`.
- `sidebarItemTexto` ahora declara `flex: 1` y `min-width: 0`.

## Validación

- `npm --prefix frontend run type-check`
- `npm --prefix frontend run build`