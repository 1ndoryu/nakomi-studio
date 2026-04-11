/**
 * Componente: SeccionProyectosRelacionados
 * Descripcion: Muestra hasta 3 proyectos relacionados por categoría, excluyendo el actual.
 * [114A-15] Extraído de ProyectoIndividualIsland para consistencia con SeccionServiciosRelacionados.
 * Usa react-query para fetch autónomo, igual que su equivalente en servicios.
 */
import React, {useMemo} from 'react';
import {useQuery} from '@tanstack/react-query';
import {useTranslation} from 'react-i18next';
import {spaClick} from '../../navegacionSPA';
import {SeccionHeader} from '../ui/SeccionHeader';
import OptimizedImage from '../ui/OptimizedImage';
import {apiListPublicProjects, type AdminProject} from '../../api/admin-projects';
import type {Proyecto, EnlaceProyecto} from '../../types/contenido';
import {PROYECTOS_DATA} from '../../data/showcase';
import './SeccionProyectosRelacionados.css';

/* [074A-12] Convierte AdminProject (API) → Proyecto (frontend) */
function convertirProyecto(p: AdminProject): Proyecto {
    return {
        id: p.slug,
        titulo: p.title,
        cliente: p.client || '',
        categorias: p.categories,
        imagen: p.featured_image || '',
        descripcion: p.description,
        link: `/proyectos/${p.slug}`,
        skills: p.skills.map((s, i) => ({id: i, titulo: s.titulo, descripcion: s.descripcion})),
        galeria: p.gallery,
        tecnologias: p.technologies,
        enlaces: p.links.map(l => ({tipo: l.tipo as EnlaceProyecto['tipo'], url: l.url, etiqueta: l.etiqueta}))
    };
}

/* Tarjeta de proyecto relacionado */
const TarjetaRelacionado: React.FC<{proyecto: Proyecto}> = ({proyecto}) => (
    <a href={proyecto.link || '#'} className="proyectoRelacionadoCard" onClick={e => { if (proyecto.link) spaClick(e, proyecto.link); }}>
        <div className="proyectoRelacionadoImagen">{proyecto.imagen && <OptimizedImage src={proyecto.imagen} alt={proyecto.titulo} sizes="(max-width: 768px) 100vw, 33vw" />}</div>
        <div className="proyectoRelacionadoInfo">
            <h4 className="proyectoRelacionadoTitulo">{proyecto.titulo}</h4>
            <span className="proyectoRelacionadoCliente">{proyecto.cliente}</span>
        </div>
    </a>
);

interface SeccionProyectosRelacionadosProps {
    slugActual: string;
    tituloActual: string;
    categorias: string;
}

export const SeccionProyectosRelacionados: React.FC<SeccionProyectosRelacionadosProps> = ({slugActual, tituloActual, categorias}) => {
    const {t} = useTranslation();

    const {data: apiData} = useQuery({
        queryKey: ['public-projects'],
        queryFn: apiListPublicProjects,
        staleTime: 5 * 60 * 1000,
        retry: 1,
    });

    const relacionados = useMemo(() => {
        const todos: Proyecto[] = apiData && apiData.length > 0
            ? apiData.map(convertirProyecto)
            : PROYECTOS_DATA;

        const categoriasArray = categorias.split(',').map(c => c.trim().toLowerCase());

        return todos.filter(p => {
            if (String(p.id) === slugActual || p.titulo === tituloActual) return false;
            const pCats = Array.isArray(p.categorias) ? p.categorias : [p.categorias];
            return pCats.some(c => categoriasArray.includes(c.toLowerCase()));
        }).slice(0, 3);
    }, [apiData, slugActual, tituloActual, categorias]);

    if (relacionados.length === 0) return null;

    return (
        <section className="proyectoRelacionados">
            <div className="proyectoRelacionadosContenedor">
                <SeccionHeader titulo={t('sections.related_projects')} />
                <div className="proyectoRelacionadosGrid">
                    {relacionados.map(p => (
                        <TarjetaRelacionado key={p.id} proyecto={p} />
                    ))}
                </div>
            </div>
        </section>
    );
};
