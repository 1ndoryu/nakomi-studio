/* [104A-5] Componente de imagen optimizada con soporte WebP y responsive.
 * Usa el proxy /api/img/ para servir imágenes locales optimizadas on-demand.
 * Genera <picture> con source WebP + fallback <img>. */

import type {SyntheticEvent} from 'react';

import {useOptimizedImage} from '../../hooks/useOptimizedImage';

interface OptimizedImageProps {
    src: string;
    alt: string;
    width?: number;
    height?: number;
    className?: string;
    /** Hint manual para el navegador cuando el layout ya es conocido. */
    sizes?: string;
    /** Calidad de compresión 10-100 (default: 80) */
    quality?: number;
    /** Deshabilitar optimización (servir original) */
    noOptimize?: boolean;
    loading?: 'lazy' | 'eager';
    /** [124A-IMG] Prioridad de fetch para LCP. 'high' para la imagen above the fold. */
    fetchPriority?: 'high' | 'low' | 'auto';
    draggable?: boolean;
    onClick?: () => void;
    onError?: (event: SyntheticEvent<HTMLImageElement>) => void;
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
    fetchPriority,
    draggable,
    onClick,
    onError,
}: OptimizedImageProps) {
    const {
        pictureRef,
        shouldOptimize,
        resolvedSizes,
        fallbackSrc,
        fallbackSrcSet,
        webpSrcSet,
    } = useOptimizedImage({
        src,
        width,
        sizes,
        quality,
        noOptimize,
    });

    /* Si no hay src, no renderizar nada */
    if (!src) return null;

    /* Sin optimización: renderizar img simple */
    if (!shouldOptimize) {
        return (
            <img
                src={src}
                alt={alt}
                width={width}
                height={height}
                className={className}
                loading={loading}
                fetchPriority={fetchPriority}
                draggable={draggable}
                onClick={onClick}
                onError={onError}
            />
        );
    }

    return (
        <picture ref={pictureRef}>
            {webpSrcSet && <source type="image/webp" srcSet={webpSrcSet} sizes={resolvedSizes} />}
            <img
                src={fallbackSrc}
                srcSet={fallbackSrcSet || undefined}
                sizes={fallbackSrcSet ? resolvedSizes : undefined}
                alt={alt}
                width={width}
                height={height}
                className={className}
                loading={loading}
                fetchPriority={fetchPriority}
                draggable={draggable}
                onClick={onClick}
                onError={onError}
            />
        </picture>
    );
}
