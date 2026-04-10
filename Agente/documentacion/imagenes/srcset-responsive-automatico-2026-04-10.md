# Srcset responsive automático — 2026-04-10

## Problema

Las imágenes sí pasaban por el proxy `/api/img/`, pero el frontend generaba siempre el mismo `srcset` fijo (`300, 640, 1024, 1600`) y, cuando un uso no declaraba `sizes`, el navegador asumía `100vw`.

Eso hacía que logos, avatars, cards pequeñas y otros bloques sin hint explícito descargaran variantes más grandes de lo necesario aunque el backend ya soportaba más buckets.

## Corrección

- `frontend/src/utils/imageUtils.ts`
  - Se dejaron todos los buckets permitidos por backend como default.
  - Se agregaron `resolveResponsiveWidths()` y `resolveBestWidth()` para elegir anchos según ancho renderizado + device pixel ratio.
- `frontend/src/hooks/useOptimizedImage.ts`
  - Nuevo hook para medir el ancho real del `picture` con `ResizeObserver`.
  - Si el caller no pasa `sizes`, el hook genera un `sizes` en píxeles con el ancho medido en vez de caer en `100vw`.
- `frontend/src/components/ui/OptimizedImage.tsx`
  - El componente quedó solo como render y delega toda la lógica responsive al hook.

## Validación

- `npm --prefix frontend run type-check`
- `npm --prefix frontend run build`
- Probe puntual de `imageUtils`:
  - avatar `48px @2x` → `[150]`
  - card `320px @2x` → `[150, 300, 480, 640, 800, 1024]`
  - hero `1180px @1x` → `[800, 1024, 1200, 1600, 2400]`
  - fallback `320px @2x` → `640`

## Gotcha

El probe temporal con `tsc` aislado mostró errores de resolución en tipos de terceros por compilar fuera del flujo normal del proyecto, pero el `type-check` y el `build` reales del frontend quedaron limpios.