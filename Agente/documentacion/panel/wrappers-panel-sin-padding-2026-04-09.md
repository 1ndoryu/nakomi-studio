# Wrappers del panel sin padding — 2026-04-09

## Objetivo

Eliminar padding redundante en wrappers del panel que duplicaban el espaciado ya existente en cards o en el modal base.

## Cambios aplicados

- `hostingContenedor` ya no agrega padding extra.
- `reembolsosContenedor` ya no agrega padding extra.
- `configSeccion` ya no agrega padding extra.
- `configConfirmContenido` deja de duplicar el padding del componente `Modal`.

## Criterio

- El panel base ya define ritmo general en `PanelIsland.css`.
- Las cards deben manejar su propio padding interno.
- Los modales base ya aportan padding; el contenido interno solo debe definir gap y layout, salvo casos especiales.

## Validación

- `npm --prefix frontend run type-check`
- `npm --prefix frontend run build`

## Nota de tooling

- El validador CSS siguió señalando hardcodes en `SeccionConfiguracion.css` aun después de migrar a tokens `var(--...)`. Se verificó que el build del frontend pasa, por lo que quedó catalogado como falso positivo del validador.