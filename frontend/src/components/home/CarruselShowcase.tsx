/// <reference types="vite/client" />
/*
 * Componente: CarruselShowcase
 * Carrusel infinito de proyectos con desplazamiento automatico y soporte drag.
 * [084A-11] Ahora consume API pública.
 * [094A-20] Sin fallback estático: home debe reflejar el CMS o quedar vacío. */
import React, {useMemo} from 'react';
import {useQuery} from '@tanstack/react-query';
import {Badge} from '../ui/Badge';
import OptimizedImage from '../ui/OptimizedImage';
import {spaClick} from '../../navegacionSPA';
import './CarruselShowcase.css';
import {useCarruselInfinito} from '../../hooks/useCarruselInfinito';
import {mapAdminProjectsToProyectos} from '../../data/showcase';
import {apiListPublicProjects} from '../../api/admin-projects';
import {IMAGENES_SHOWCASE} from '../../hooks/useImagenes';

export const CarruselShowcase: React.FC = () => {
    /* [084A-11] Fetch proyectos publicados del API.
     * [084A-30] Mientras carga no renderizamos un dataset fantasma. */
    const {data: apiProjects} = useQuery({
        queryKey: ['public-projects-showcase'],
        queryFn: apiListPublicProjects,
        staleTime: 5 * 60 * 1000,
        retry: 1,
    });

    const baseProyectos = useMemo(
        () => mapAdminProjectsToProyectos(apiProjects || []),
        [apiProjects]
    );

    const proyectosConImagen = baseProyectos.map((proyecto, idx) => ({
        ...proyecto,
        imagen: proyecto.imagen || (IMAGENES_SHOWCASE.length > 0 ? IMAGENES_SHOWCASE[idx % IMAGENES_SHOWCASE.length] : ''),
    }));

    const itemsTotales = [...proyectosConImagen, ...proyectosConImagen];

    /* [084A-25] Hook SIEMPRE antes de early return — Rules of Hooks.
     * Se llama con totalItems=0 cuando hay 0 proyectos, pero nunca se omite. */
    const {indiceActual, conTransicion, dragOffset, handlers} = useCarruselInfinito({
        totalItems: proyectosConImagen.length,
        tiempoEspera: 8000,
        tiempoTransicion: 800
    });

    if (proyectosConImagen.length === 0) return null;

    return (
        <div className="carruselContenedorPrincipal">
            <div
                className="carruselPista"
                {...handlers}
                style={
                    {
                        transform: `translateX(calc( -1 * (var(--carrusel-item-width) + var(--carrusel-item-gap)) * ${indiceActual} + ${dragOffset}px))`,
                        transition: conTransicion ? 'transform 800ms cubic-bezier(0.25, 1, 0.5, 1)' : 'none',
                        cursor: 'grab',
                        touchAction: 'pan-y'
                    } as React.CSSProperties
                }>
                {itemsTotales.map((proyecto, index) => {
                    const categoriasArr = Array.isArray(proyecto.categorias)
                        ? proyecto.categorias
                        : (proyecto.categorias ? [proyecto.categorias] : []);

                    return (
                        <a key={`proj-${index}`} href={proyecto.link || '#'} className="carruselItem" draggable={false} onClick={e => { if (proyecto.link) spaClick(e, proyecto.link); }}>
                            <div className="carruselImagenWrapper">
                                {proyecto.imagen && (
                                    <OptimizedImage
                                        src={proyecto.imagen}
                                        alt={proyecto.titulo}
                                        className="carruselImagen"
                                        draggable={false}
                                    />
                                )}
                            </div>
                            <div className="carruselContenido">
                                <h3 className="carruselTitulo">{proyecto.titulo}</h3>
                                <div className="carruselTags">
                                    {categoriasArr.slice(0, 3).map(cat => (
                                        <Badge key={cat} label={cat} />
                                    ))}
                                </div>
                            </div>
                        </a>
                    );
                })}
            </div>
            <div className="carruselOverlay" />
        </div>
    );
};
