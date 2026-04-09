/* [104A-5] Componente de imagen optimizada con soporte WebP y responsive.
 * Usa el proxy /api/img/ para servir imágenes optimizadas on-demand.
 * Genera <picture> con source WebP + fallback <img>. */

import { generateSrcSet, generateWebPSrcSet, optimizedUrl } from '../../utils/imageUtils';

interface OptimizedImageProps {
    src: string;
    alt: string;
    width?: number;
    height?: number;
    className?: string;
    /** Anchos para srcSet responsive (default: [300, 640, 1024, 1600]) */
    sizes?: string;
    /** Calidad de compresión 10-100 (default: 80) */
    quality?: number;
    /** Deshabilitar optimización (servir original) */
    noOptimize?: boolean;
    loading?: 'lazy' | 'eager';
    draggable?: boolean;
    onClick?: () => void;
}

export default function OptimizedImage({
    src,
    alt,
    width,
    height,
    className,
    sizes = '100vw',
    quality = 80,
    noOptimize = false,
    loading = 'lazy',
    draggable,
    onClick,
}: OptimizedImageProps) {
    /* Si no hay src, no renderizar nada */
    if (!src) return null;

    /* Sin optimización: renderizar img simple */
    if (noOptimize || src.startsWith('data:') || src.startsWith('http') || src.endsWith('.svg') || !src.startsWith('/uploads/')) {
        return (
            <img
                src={src}
                alt={alt}
                width={width}
                height={height}
                className={className}
                loading={loading}
                draggable={draggable}
                onClick={onClick}
            />
        );
    }

    const webpSrcSet = generateWebPSrcSet(src, undefined, quality);
    const fallbackSrcSet = generateSrcSet(src, undefined, { quality });
    const fallbackSrc = optimizedUrl(src, { quality });

    return (
        <picture>
            {webpSrcSet && <source type="image/webp" srcSet={webpSrcSet} sizes={sizes} />}
            <img
                src={fallbackSrc}
                srcSet={fallbackSrcSet || undefined}
                sizes={fallbackSrcSet ? sizes : undefined}
                alt={alt}
                width={width}
                height={height}
                className={className}
                loading={loading}
                draggable={draggable}
                onClick={onClick}
            />
        </picture>
    );
}
