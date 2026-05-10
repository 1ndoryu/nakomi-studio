/**
 * Componente: SeccionShowcase
 * Muestra proyectos destacados organizados por categoría.
 * [084A-11] Ahora consume API pública de proyectos.
 * [094A-20] Sin fallback estático: la home debe reflejar el CMS real. */
import {useMemo} from 'react';
import {useQuery} from '@tanstack/react-query';
import {useTranslation} from 'react-i18next';
import {spaClick} from '../../navegacionSPA';
import {SeccionHeader} from '../ui/SeccionHeader';
import {Badge} from '../ui/Badge';
import {buildCategoriasShowcase, mapAdminProjectsToProyectos} from '../../data/showcase';
import {apiListPublicProjects} from '../../api/admin-projects';
import OptimizedImage from '../ui/OptimizedImage';
import './SeccionShowcase.css';

export const SeccionShowcase = (): JSX.Element | null => {
    const {t} = useTranslation();

    /* [084A-11] Fetch proyectos publicados del API.
     * [084A-30] Mientras carga no renderizamos contenido estático inventado. */
    const {data: apiProjects} = useQuery({
        queryKey: ['public-projects-showcase'],
        queryFn: apiListPublicProjects,
        staleTime: 5 * 60 * 1000,
        retry: 1,
    });

    /* [124A-CMS2] Solo proyectos marcados como featured aparecen en el home.
     * Si ninguno está featured, se muestran todos los publicados (backward compatible). */
    const categorias = useMemo(() => {
        const allProjects = mapAdminProjectsToProyectos(apiProjects || []);
        const featured = (apiProjects || []).filter(p => p.is_featured);
        const source = featured.length > 0
            ? mapAdminProjectsToProyectos(featured)
            : allProjects;
        return buildCategoriasShowcase(source);
    }, [apiProjects]);

    if (categorias.length === 0) return null;

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
                                        <OptimizedImage src={proyecto.imagen} alt={proyecto.titulo} className="proyectoImagen" width={800} height={600} sizes="(min-width: 1024px) min(45vw, 800px), (min-width: 768px) 50vw, 100vw" quality={86} />
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
