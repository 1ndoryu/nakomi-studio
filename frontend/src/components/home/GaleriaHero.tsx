/* GaleriaHero: muestra una imagen a la vez de los proyectos marcados con in_carousel.
 * Usa gallery_image si existe, si no usa featured_image.
 * Las imágenes rotan con crossfade. Proporción 1200x600 con bordes redondeados.
 * Overlay inferior-derecho: nombre del proyecto (enlace al detalle) + icono info que
 * expande descripción y enlaces del proyecto. Estilo pill inspirado en chatWidgetBubble. */
import React, {useState, useEffect, useMemo, useCallback} from 'react';
import {useQuery} from '@tanstack/react-query';
import {Info, X} from 'lucide-react';
import OptimizedImage from '../ui/OptimizedImage';
import {Button} from '../ui/Button';
import {spaClick} from '../../navegacionSPA';
import {apiListPublicProjects} from '../../api/admin-projects';
import type {AdminProject} from '../../api/admin-projects';
import './GaleriaHero.css';

interface EntradaGaleria {
    url: string;
    proyecto: AdminProject;
}

export const GaleriaHero: React.FC = () => {
    const {data: proyectos} = useQuery({
        queryKey: ['public-projects-gallery'],
        queryFn: apiListPublicProjects,
        staleTime: 5 * 60 * 1000,
        retry: 1,
    });

    /* Filtrar proyectos con in_carousel=true y que tengan imagen */
    const entradas = useMemo((): EntradaGaleria[] => {
        if (!proyectos) return [];
        return proyectos
            .filter(p => p.in_carousel)
            .map(p => ({ url: p.gallery_image || p.featured_image, proyecto: p }))
            .filter((e): e is EntradaGaleria => !!e.url);
    }, [proyectos]);

    const [indice, setIndice] = useState(0);
    const [expandido, setExpandido] = useState(false);

    useEffect(() => {
        if (entradas.length <= 1) return;
        const intervalo = setInterval(() => {
            setIndice(prev => (prev + 1) % entradas.length);
            setExpandido(false);
        }, 10000);
        return () => clearInterval(intervalo);
    }, [entradas.length]);

    const toggleExpandido = useCallback((e: React.MouseEvent) => {
        e.preventDefault();
        e.stopPropagation();
        setExpandido(prev => !prev);
    }, []);

    if (entradas.length === 0) return null;

    const actual = entradas[indice];
    const slug = actual.proyecto.slug;
    const href = `/proyectos/${slug}`;

    return (
        <div className="galeriaHeroContenedor">
            {entradas.map(({url, proyecto}, i) => (
                <OptimizedImage
                    key={url}
                    src={url}
                    alt={proyecto.title}
                    className={`galeriaHeroImagen ${i === indice ? 'galeriaHeroImagenActiva' : ''}`}
                    fixedWidth={1600}
                    quality={80}
                    loading={i === 0 ? 'eager' : 'lazy'}
                />
            ))}

            {/* Overlay inferior-derecho */}
            <div className="galeriaHeroOverlay">
                {expandido && (
                    <div className="galeriaHeroDetalle">
                        {actual.proyecto.description && (
                            <p className="galeriaHeroDescripcion">{actual.proyecto.description}</p>
                        )}
                        {actual.proyecto.links.length > 0 && (
                            <div className="galeriaHeroEnlaces">
                                {actual.proyecto.links.map(enlace => (
                                    <a
                                        key={enlace.url}
                                        href={enlace.url}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        className="galeriaHeroEnlace"
                                    >
                                        {enlace.etiqueta || enlace.tipo}
                                    </a>
                                ))}
                            </div>
                        )}
                    </div>
                )}
                <div className="galeriaHeroPill">
                    <a
                        href={href}
                        className="galeriaHeroPillTitulo"
                        onClick={e => spaClick(e, href)}
                    >
                        {actual.proyecto.title}
                    </a>
                    <Button
                        variante="texto"
                        className="galeriaHeroPillInfo"
                        onClick={toggleExpandido}
                        aria-label={expandido ? 'Cerrar información' : 'Ver información del proyecto'}
                        aria-expanded={expandido}
                    >
                        {expandido ? <X size={14} /> : <Info size={14} />}
                    </Button>
                </div>
            </div>
        </div>
    );
};
