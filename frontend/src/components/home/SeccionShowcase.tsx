/**
 * Componente: SeccionShowcase
 * Muestra proyectos destacados organizados por categoría.
 * [084A-11] Ahora consume API pública de proyectos, con fallback a datos estáticos. */
import {useMemo} from 'react';
import {useQuery} from '@tanstack/react-query';
import {useTranslation} from 'react-i18next';
import {spaClick} from '../../navegacionSPA';
import {SeccionHeader} from '../ui/SeccionHeader';
import {Badge} from '../ui/Badge';
import {CATEGORIAS_SHOWCASE, buildCategoriasShowcase, mapAdminProjectsToProyectos} from '../../data/showcase';
import {apiListPublicProjects} from '../../api/admin-projects';
import OptimizedImage from '../ui/OptimizedImage';
import './SeccionShowcase.css';

export const SeccionShowcase = (): JSX.Element => {
    const {t} = useTranslation();

    /* [084A-11] Fetch proyectos publicados del API, fallback a datos estáticos
     * [084A-30] isPending evita flash: no mostrar fallback mientras la API resuelve */
    const {data: apiProjects, isPending} = useQuery({
        queryKey: ['public-projects-showcase'],
        queryFn: apiListPublicProjects,
        staleTime: 5 * 60 * 1000,
        retry: 1,
    });

    const categorias = useMemo(() => {
        if (isPending) return [];
        if (!apiProjects || apiProjects.length === 0) return CATEGORIAS_SHOWCASE;
        const proyectos = mapAdminProjectsToProyectos(apiProjects);
        return proyectos.length > 0 ? buildCategoriasShowcase(proyectos) : CATEGORIAS_SHOWCASE;
    }, [apiProjects, isPending]);

    return (
        <section className="seccionShowcase">
            <div className="showcaseContenedor">
                <SeccionHeader titulo={t('sections.selected_work')} />

                {categorias.map((categoria) => (
                    <div className="showcaseFila" key={categoria.titulo}>
                        <div className="showcaseCategoria">
                            <h2 className="showcaseTituloCategoria">{categoria.titulo}</h2>
                        </div>

                        <div className="showcaseGridProyectos">
                            {categoria.proyectos.map(proyecto => (
                                <a key={proyecto.id} href={proyecto.link || '#'} className="proyectoCard" onClick={e => { if (proyecto.link) spaClick(e, proyecto.link); }}>
                                    <div className="proyectoImagenWrapper">
                                        <OptimizedImage src={proyecto.imagen} alt={proyecto.titulo} className="proyectoImagen" sizes="(max-width: 768px) 100vw, 50vw" />
                                    </div>
                                    <div className="proyectoInfo">
                                        <h3 className="proyectoTitulo">
                                            {proyecto.titulo} <span className="proyectoCliente">- {proyecto.cliente}</span>
                                        </h3>
                                        <div className="proyectoTags">
                                            {(Array.isArray(proyecto.categorias) ? proyecto.categorias : [proyecto.categorias]).map(cat => (
                                                <Badge key={cat} label={cat} />
                                            ))}
                                        </div>
                                    </div>
                                </a>
                            ))}
                        </div>
                    </div>
                ))}
            </div>
        </section>
    );
};
