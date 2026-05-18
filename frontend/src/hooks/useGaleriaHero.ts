import {useCallback, useEffect, useMemo, useState} from 'react';
import type {MouseEvent} from 'react';
import {useQuery} from '@tanstack/react-query';
import {apiListPublicProjects} from '../api/admin-projects';
import type {AdminProject} from '../api/admin-projects';

export interface EntradaGaleriaHero {
    url: string;
    proyecto: AdminProject;
}

export const useGaleriaHero = () => {
    /* [175A-1] Misma queryKey que SeccionShowcase para compartir cach\u00e9 y evitar
     * un segundo request a /api/projects. Un solo fetch sirve ambos componentes. */
    const {data: proyectos} = useQuery({
        queryKey: ['public-projects-showcase'],
        queryFn: apiListPublicProjects,
        staleTime: 5 * 60 * 1000,
        retry: 1,
    });

    const entradas = useMemo((): EntradaGaleriaHero[] => {
        if (!proyectos) return [];
        return proyectos
            .filter(p => p.in_carousel)
            .map(p => ({url: p.gallery_image || p.featured_image, proyecto: p}))
            .filter((entrada): entrada is EntradaGaleriaHero => Boolean(entrada.url));
    }, [proyectos]);

    const [indice, setIndice] = useState(0);
    const [expandido, setExpandido] = useState(false);

    useEffect(() => {
        setIndice(prev => (entradas.length === 0 ? 0 : Math.min(prev, entradas.length - 1)));
    }, [entradas.length]);

    useEffect(() => {
        if (entradas.length <= 1) return undefined;
        const intervalo = setInterval(() => {
            setIndice(prev => (prev + 1) % entradas.length);
            setExpandido(false);
        }, 10000);
        return () => clearInterval(intervalo);
    }, [entradas.length]);

    const toggleExpandido = useCallback((e: MouseEvent<HTMLElement>) => {
        e.preventDefault();
        e.stopPropagation();
        setExpandido(prev => !prev);
    }, []);

    const irAnterior = useCallback(() => {
        if (entradas.length <= 1) return;
        setIndice(prev => (prev - 1 + entradas.length) % entradas.length);
        setExpandido(false);
    }, [entradas.length]);

    const irSiguiente = useCallback(() => {
        if (entradas.length <= 1) return;
        setIndice(prev => (prev + 1) % entradas.length);
        setExpandido(false);
    }, [entradas.length]);

    const actual = entradas[indice] ?? null;
    const href = actual ? `/proyectos/${actual.proyecto.slug}` : '';

    return {
        actual,
        entradas,
        expandido,
        href,
        indice,
        irAnterior,
        irSiguiente,
        toggleExpandido,
    };
};
