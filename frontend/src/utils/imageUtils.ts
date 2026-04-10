/* [104A-5] Utilidades de optimización de imágenes.
 * Genera URLs al proxy de imágenes del backend (/api/img/) con parámetros de optimización.
 * Similar a Jetpack Photon CDN: las imágenes se procesan on-demand con cache del lado del servidor. */

/* Anchos predefinidos que coinciden con los permitidos en el backend */
const ALLOWED_WIDTHS = [150, 300, 480, 640, 800, 1024, 1200, 1600, 2400] as const;

const OPTIMIZABLE_PREFIXES = ['/uploads/', '/assets/'] as const;

type AllowedWidth = typeof ALLOWED_WIDTHS[number];

interface OptimizeOptions {
    width?: AllowedWidth | number;
    quality?: number;
    format?: 'webp' | 'jpeg' | 'png';
}

function isOptimizableSrc(src: string): boolean {
    return OPTIMIZABLE_PREFIXES.some(prefix => src.startsWith(prefix));
}

function toProxyRelativePath(src: string): string {
    if (src.startsWith('/uploads/')) {
        return src.replace(/^\/uploads\//, '');
    }

    return src.replace(/^\//, '');
}

/* Genera la URL optimizada para una imagen.
 * Si la src ya es una URL externa o un data URI, la retorna sin modificar.
 * Si no tiene parámetros de optimización, retorna la URL original. */
export function optimizedUrl(src: string, options: OptimizeOptions = {}): string {
    if (!src) return src;

    /* No procesar data URIs, URLs externas o SVGs */
    if (src.startsWith('data:') || src.startsWith('http') || src.endsWith('.svg')) {
        return src;
    }

    /* Solo procesar imágenes locales conocidas */
    if (!isOptimizableSrc(src)) {
        return src;
    }

    /* Si no hay opciones de optimización, retornar original */
    const { width, quality, format } = options;
    if (!width && !quality && !format) {
        return src;
    }

    /* Construir ruta relativa para el proxy */
    const relativePath = toProxyRelativePath(src);

    /* Construir query params */
    const params = new URLSearchParams();
    if (width) params.set('w', String(width));
    if (quality) params.set('q', String(quality));
    if (format) params.set('fmt', format);

    return `/api/img/${relativePath}?${params.toString()}`;
}

/* Genera un srcSet para imágenes responsive.
 * Retorna pares ancho→URL para los breakpoints especificados. */
export function generateSrcSet(
    src: string,
    widths: AllowedWidth[] = [300, 640, 1024, 1600],
    options: Omit<OptimizeOptions, 'width'> = {},
): string {
    if (!src || src.startsWith('data:') || src.startsWith('http') || src.endsWith('.svg')) {
        return '';
    }

    if (!isOptimizableSrc(src)) {
        return '';
    }

    return widths
        .map(w => `${optimizedUrl(src, { ...options, width: w })} ${w}w`)
        .join(', ');
}

/* Genera srcSet en formato WebP para usar con <picture> <source> */
export function generateWebPSrcSet(
    src: string,
    widths: AllowedWidth[] = [300, 640, 1024, 1600],
    quality = 80,
): string {
    return generateSrcSet(src, widths, { format: 'webp', quality });
}
