import {useEffect, useRef, useState} from 'react';

import {
    generateSrcSet,
    optimizedUrl,
    resolveBestWidth,
    resolveResponsiveWidths,
} from '../utils/imageUtils';

interface UseOptimizedImageParams {
    src: string;
    width?: number;
    sizes?: string;
    quality: number;
    noOptimize: boolean;
}

interface UseOptimizedImageResult {
    pictureRef: React.RefObject<HTMLPictureElement>;
    shouldOptimize: boolean;
    resolvedSizes: string;
    fallbackSrc: string;
    fallbackSrcSet: string;
    webpSrcSet: string;
}

export function useOptimizedImage({
    src,
    width,
    sizes,
    quality,
    noOptimize,
}: UseOptimizedImageParams): UseOptimizedImageResult {
    const pictureRef = useRef<HTMLPictureElement>(null);
    const [renderedWidth, setRenderedWidth] = useState<number | undefined>(width);
    const shouldOptimize = Boolean(src)
        && !noOptimize
        && !src.startsWith('data:')
        && !src.startsWith('http')
        && !src.endsWith('.svg')
        && (src.startsWith('/uploads/') || src.startsWith('/assets/'));

    useEffect(() => {
        if (!shouldOptimize) {
            setRenderedWidth(width);
            return undefined;
        }

        const pictureElement = pictureRef.current;
        if (!pictureElement) {
            return undefined;
        }

        const updateWidth = (nextWidth?: number) => {
            const measuredWidth = Math.round(nextWidth ?? pictureElement.getBoundingClientRect().width);

            if (measuredWidth > 0) {
                setRenderedWidth((currentWidth) => (currentWidth === measuredWidth ? currentWidth : measuredWidth));
            }
        };

        updateWidth(width);

        if (typeof ResizeObserver === 'undefined') {
            const handleResize = () => updateWidth();
            window.addEventListener('resize', handleResize);
            return () => window.removeEventListener('resize', handleResize);
        }

        const resizeObserver = new ResizeObserver((entries) => {
            const firstEntry = entries[0];
            updateWidth(firstEntry?.contentRect.width);
        });

        resizeObserver.observe(pictureElement);

        return () => resizeObserver.disconnect();
    }, [shouldOptimize, src, width]);

    /* [104A-13] Si el caller no declara sizes, medimos el ancho real para no caer en 100vw.
     * Esto evita que logos, avatars y cards pequeñas descarguen variantes más grandes de lo necesario. */
    const devicePixelRatio = typeof window === 'undefined' ? 1 : window.devicePixelRatio || 1;
    const targetWidth = renderedWidth ?? width;
    const responsiveWidths = resolveResponsiveWidths(targetWidth, devicePixelRatio);
    const fallbackWidth = resolveBestWidth(targetWidth, devicePixelRatio);
    const resolvedSizes = sizes ?? (targetWidth ? `${Math.round(targetWidth)}px` : '100vw');

    return {
        pictureRef,
        shouldOptimize,
        resolvedSizes,
        fallbackSrc: optimizedUrl(src, fallbackWidth ? {quality, width: fallbackWidth} : {quality}),
        fallbackSrcSet: generateSrcSet(src, responsiveWidths, {quality}),
        /* [154A-IMG] WebP desactivado: image-webp 0.2 solo soporta lossless (archivos enormes).
         * JPEG lossy con quality 80 produce ~80% menos peso. Reactivar cuando haya lossy WebP. */
        webpSrcSet: '',
    };
}