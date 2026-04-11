/**
 * Componente: SeccionServiciosRelacionados
 * Descripcion: Muestra 3 servicios relacionados excluyendo el actual.
 */
import React, {useMemo} from 'react';
import {useQuery} from '@tanstack/react-query';
import {useTranslation} from 'react-i18next';
import {SeccionHeader} from '../ui/SeccionHeader';
import {ServiceCard} from '../ui/ServiceCard';
import {apiListPublicServices, type PublicService} from '../../api/admin-services';
import type {Servicio} from '../../types/servicios';
import './SeccionServiciosRelacionados.css';

function convertirServicio(servicio: PublicService): Servicio {
    return {
        id: servicio.slug,
        adminId: servicio.id,
        titulo: servicio.title,
        descripcion: servicio.description || '',
        imagen: servicio.image_url || '',
        categorias: [],
        link: `/servicios/${servicio.slug}`,
        skills: Array.isArray(servicio.skills)
            ? servicio.skills.map((skill: unknown, index: number) => {
                const data = skill as Record<string, string>;
                return {
                    id: index,
                    titulo: data.titulo || '',
                    descripcion: data.descripcion || '',
                };
            })
            : [],
    };
}

interface SeccionServiciosRelacionadosProps {
    servicioActualId?: string;
}

export const SeccionServiciosRelacionados: React.FC<SeccionServiciosRelacionadosProps> = ({servicioActualId}) => {
    const {t} = useTranslation();
    const {data: apiData} = useQuery({
        queryKey: ['public-services'],
        queryFn: apiListPublicServices,
        staleTime: 5 * 60 * 1000,
        retry: 1,
    });

    const servicios = useMemo(() => {
        const slugActual = servicioActualId?.split('/').filter(Boolean).pop() || servicioActualId;

        return (apiData || [])
            .map(convertirServicio)
            .filter(servicio => {
                const servicioSlug = servicio.link.split('/').filter(Boolean).pop() || servicio.id;
                return servicioSlug !== slugActual && servicio.id !== slugActual;
            })
            .slice(0, 3);
    }, [apiData, servicioActualId]);

    if (servicios.length === 0) return null;

    return (
        <section className="seccionServiciosRelacionados">
            <div className="relacionadosContenedor">
                <SeccionHeader titulo={t('sections.more_services')} />

                <div className="relacionadosLista">
                    {servicios.map(servicio => (
                        <ServiceCard key={servicio.id} servicio={servicio} variant="simple" noOverlay />
                    ))}
                </div>
            </div>
        </section>
    );
};
