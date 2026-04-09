# Sistema de Optimización de Imágenes

**Fecha:** 2026-04-09
**Tarea:** 104A-5

## Arquitectura

Proxy on-demand estilo Jetpack Photon CDN. Las imágenes se procesan al vuelo al ser solicitadas, con cache en disco para evitar re-procesamiento.

## Backend

### Ruta: `GET /api/img/{*path}?w={ancho}&q={calidad}&fmt={formato}`

**Parámetros:**
| Param | Tipo | Default | Descripción |
|-------|------|---------|-------------|
| `w` | u32 | original | Ancho objetivo. Se snapea al más cercano de la whitelist |
| `q` | u8 | 80 | Calidad 10-100 (solo JPEG) |
| `fmt` | string | original | Formato de salida: `webp`, `jpeg`, `png` |

**Anchos permitidos (whitelist):** 150, 300, 480, 640, 800, 1024, 1200, 1600, 2400

**Seguridad:**
- Path traversal: doble validación (contains("..") + canonicalize + starts_with(uploads_dir))
- Whitelist de anchos previene cache flooding
- Solo procesa imágenes de `uploads/` (jpg, jpeg, png, webp, gif)
- No procesa SVGs (no son rasterizables de forma segura)

**Cache:** `uploads/.cache/{w}_{q}_{fmt}/{relative_path}`

**Headers:** `Cache-Control: public, max-age=31536000, immutable`

### Archivos clave:
- `src/services/image_processing.rs` — procesamiento de imágenes (resize, compress, encode)
- `src/handlers/image_proxy.rs` — handler HTTP con validación

## Frontend

### `optimizedUrl(src, options)` — `utils/imageUtils.ts`
Genera URL al proxy para imágenes de uploads/. No modifica data URIs, URLs externas ni SVGs.

### `generateSrcSet(src, widths, options)` — `utils/imageUtils.ts`
Genera string srcSet responsive con URLs al proxy.

### `<OptimizedImage>` — `components/ui/OptimizedImage.tsx`
Componente React que renderiza `<picture>` con source WebP + fallback. Props principales:
- `src`, `alt` (obligatorios)
- `sizes` (default: "100vw")
- `quality` (default: 80)
- `noOptimize` (deshabilitar optimización)
- `loading` (default: "lazy")

Para imágenes que NO vienen de uploads (data URIs, URLs externas, SVGs), renderiza un `<img>` simple sin proxy.

## Limitaciones conocidas
1. `image` crate 0.25 solo soporta WebP lossless (no lossy). Esto produce archivos WebP más grandes que lossy pero aún menores que PNG.
2. El cache no tiene TTL ni limpieza automática. Para limpiar: `rm -rf uploads/.cache/`.
3. No hay soporte para GIF animado (se procesa solo el primer frame).
