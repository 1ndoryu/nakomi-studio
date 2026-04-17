/* GaleriaHero: muestra una imagen a la vez de los proyectos marcados con in_carousel.
 * Usa gallery_image si existe, si no usa featured_image.
 * Las imágenes rotan con crossfade. Proporción 1200x600 con bordes redondeados. */
import React, {useState, useEffect, useMemo} from 'react';
import {useQuery} from '@tanstack/react-query';
import OptimizedImage from '../ui/OptimizedImage';
import {apiListPublicProjects} from '../../api/admin-projects';
import './GaleriaHero.css';

export const GaleriaHero: React.FC = () => {
    const {data: proyectos} = useQuery({
        queryKey: ['public-projects-gallery'],
        queryFn: apiListPublicProjects,
        staleTime: 5 * 60 * 1000,
        retry: 1,
    });

    /* Filtrar proyectos con in_carousel=true y que tengan imagen */
    const imagenes = useMemo(() => {
        if (!proyectos) return [];
        return proyectos
            .filter(p => p.in_carousel)
            .map(p => p.gallery_image || p.featured_image)
            .filter((url): url is string => !!url);
    }, [proyectos]);

    const [indice, setIndice] = useState(0);

    useEffect(() => {
        if (imagenes.length <= 1) return;
        const intervalo = setInterval(() => {
            setIndice(prev => (prev + 1) % imagenes.length);
        }, 5000);
        return () => clearInterval(intervalo);
    }, [imagenes.length]);

    if (imagenes.length === 0) return null;

    return (
        <div className="galeriaHeroContenedor">
            {imagenes.map((url, i) => (
                <OptimizedImage
                    key={url}
                    src={url}
                    alt=""
                    className={`galeriaHeroImagen ${i === indice ? 'galeriaHeroImagenActiva' : ''}`}
                    fixedWidth={1200}
                    quality={80}
                    loading={i === 0 ? 'eager' : 'lazy'}
                />
            ))}
        </div>
    );
};
