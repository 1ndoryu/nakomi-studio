# Imágenes del carrusel home — ancho fijo de optimización

## Fecha
2026-04-16

## Problema

El carrusel principal del inicio seguía pudiendo pedir variantes mayores a las deseadas aunque se ajustara la calidad, porque `OptimizedImage` medía el ancho renderizado y lo combinaba con DPR para derivar buckets responsive.

Para este bloque el objetivo era explícito: servir imágenes del proxy con `w=1200&q=80` y evitar que pantallas densas empujaran la descarga a 1600 o 2400.

## Cambio aplicado

- `OptimizedImage` ahora acepta `fixedWidth`.
- Si `fixedWidth` existe, `useOptimizedImage`:
  - no activa `ResizeObserver`
  - no genera `srcSet` responsive
  - usa `fixedWidth` como ancho exacto para `optimizedUrl()`
- `CarruselShowcase` usa `fixedWidth={1200}` y `quality={80}`.

## Cuándo usarlo

Solo en bloques donde la intención sea una variante exacta y estable del proxy.

No usar `fixedWidth` por defecto en imágenes normales del sitio. Para cards, galerías y thumbnails sigue siendo preferible el modo responsive con `sizes` o medición real.

## Validación

- `npx tsc --noEmit` en `frontend/`
- `npm --prefix frontend run build`
- Revisión del componente: `CarruselShowcase` ya no pasa por el cálculo de buckets responsive para estas imágenes