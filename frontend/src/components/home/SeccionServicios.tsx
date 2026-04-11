/**
 * Componente: SeccionServicios
 * Muestra una lista de servicios ofrecidos con imágenes y enlaces.
 * [084A-30] Migrado a useQuery para mostrar servicios del CMS, con fallback estático. */
import React, {useMemo} from 'react';
import {useQuery} from '@tanstack/react-query';
import {useTranslation} from 'react-i18next';
import {SeccionHeader} from '../ui/SeccionHeader';
import {ServiceCard} from '../ui/ServiceCard';
import {SERVICIOS_PRINCIPALES} from '../../data/servicios';
import {apiListPublicServices} from '../../api/admin-services';
import type {Servicio} from '../../types/servicios';
import './SeccionServicios.css';

/* Convierte PublicService → Servicio para ServiceCard */
function toServicio(s: {id: string; slug: string; title: string; description: string | null; image_url: string | null; skills?: unknown[]}): Servicio {
    return {
        id: s.slug,
        adminId: s.id,
        titulo: s.title,
        descripcion: s.description || '',
        imagen: s.image_url || '',
        categorias: [],
        link: `/servicios/${s.slug}`,
        skills: [],
    };
}

export const SeccionServicios: React.FC = () => {
    const {t} = useTranslation();

    /* [084A-30] Fetch servicios del CMS, fallback a datos estáticos sin flash */
    const {data: apiServices, isPending} = useQuery({
        queryKey: ['public-services-home'],
        queryFn: apiListPublicServices,
        staleTime: 5 * 60 * 1000,
        retry: 1,
    });

    const servicios = useMemo(() => {
        if (isPending) return [];
        if (!apiServices || apiServices.length === 0) return SERVICIOS_PRINCIPALES;
        const mapped = apiServices.map(toServicio);
        return mapped.length > 0 ? mapped : SERVICIOS_PRINCIPALES;
    }, [apiServices, isPending]);

    if (servicios.length === 0) return null;

    return (
        <section className="seccionServicios" id="servicios">
            <div className="serviciosContenedor">
                <SeccionHeader titulo={t('sections.services')} />
                <div className="serviciosLista">
                    {servicios.map(servicio => (
                        <ServiceCard key={servicio.id} servicio={servicio} variant="simple" noOverlay />
                    ))}
                </div>
            </div>
        </section>
    );
};
