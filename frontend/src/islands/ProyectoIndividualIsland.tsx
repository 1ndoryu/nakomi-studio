/**
 * Componente: ProyectoIndividualIsland
 * Página de detalle de un proyecto individual.
 * Estructura: Hero -> Galería -> Skills -> CTA -> Relacionados -> Footer
 * sentinel-disable-file componente-sin-hook: Lógica minimal (fetch API + data lookup)
 * no justifica hook separado; datos estáticos vienen de imports directos.
 */
import React, {useState, useEffect} from 'react';
import {useTranslation} from 'react-i18next';
import {GitBranch, Globe, ExternalLink, Package} from 'lucide-react';
import {spaClick} from '../navegacionSPA';
import {useChatStore} from '../stores/chatStore';
import '../styles/variables.css';
import './ProyectoIndividualIsland.css';
import {LayoutPagina} from '../components/layout/LayoutPagina';
import {SEOHead} from '../components/seo/SEOHead';
import {SeccionSkillsServicio} from '../components/servicios/SeccionSkillsServicio';
import {SeccionGaleriaServicio} from '../components/servicios/SeccionGaleriaServicio';
import {SeccionCta} from '../components/ui/SeccionCta';
import {SeccionContacto} from '../components/home/SeccionContacto';
import {PROYECTOS_DATA} from '../data/showcase';
import {Proyecto} from '../types/contenido';
import {SeccionHeader} from '../components/ui/SeccionHeader';
import {apiGetProjectBySlug, apiListPublicProjects, AdminProject} from '../api/admin-projects';

interface ProyectoIndividualIslandProps {
    titulo?: string;
    descripcion?: string;
    cliente?: string;
    categorias?: string;
    imagen?: string;
    slug?: string;
}

/* [064A-8] Icono según tipo de enlace */
const ICONOS_ENLACE: Record<string, typeof GitBranch> = {
    github: GitBranch,
    web: Globe,
    npm: Package,
    demo: ExternalLink
};

/* Tarjeta de proyecto relacionado */
const TarjetaRelacionado: React.FC<{proyecto: Proyecto}> = ({proyecto}) => (
    <a href={proyecto.link || '#'} className="proyectoRelacionadoCard" onClick={e => { if (proyecto.link) spaClick(e, proyecto.link); }}>
        <div className="proyectoRelacionadoImagen">{proyecto.imagen && <img src={proyecto.imagen} alt={proyecto.titulo} loading="lazy" />}</div>
        <div className="proyectoRelacionadoInfo">
            <h4 className="proyectoRelacionadoTitulo">{proyecto.titulo}</h4>
            <span className="proyectoRelacionadoCliente">{proyecto.cliente}</span>
        </div>
    </a>
);

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
        enlaces: p.links.map(l => ({tipo: l.tipo as 'github' | 'web' | 'npm' | 'demo', url: l.url, etiqueta: l.etiqueta}))
    };
}

export const ProyectoIndividualIsland = ({titulo = 'Proyecto', descripcion = '', cliente = '', categorias = '', slug = ''}: ProyectoIndividualIslandProps): JSX.Element => {
    const {t} = useTranslation();
    const abrirChat = useChatStore(s => s.abrir);

    /* [074A-12] Proyecto desde API con fallback a datos estáticos */
    const fallback = PROYECTOS_DATA.find(p => p.titulo.toLowerCase() === titulo.toLowerCase() || String(p.id) === slug);
    const [proyectoContexto, setProyectoContexto] = useState<Proyecto | undefined>(fallback);
    const [todosProyectos, setTodosProyectos] = useState<Proyecto[]>(PROYECTOS_DATA);

    useEffect(() => {
        const controller = new AbortController();
        if (slug) {
            apiGetProjectBySlug(slug)
                .then(data => {
                    if (!controller.signal.aborted) {
                        setProyectoContexto(convertirProyecto(data));
                    }
                })
                .catch(() => { /* mantiene fallback estático */ });
        }
        /* Cargar todos para los relacionados */
        apiListPublicProjects()
            .then(data => {
                if (!controller.signal.aborted && data.length > 0) {
                    setTodosProyectos(data.map(convertirProyecto));
                }
            })
            .catch(() => { /* mantiene PROYECTOS_DATA */ });
        return () => controller.abort();
    }, [slug]);

    const skills = proyectoContexto?.skills || [];
    const desc = descripcion || proyectoContexto?.descripcion || '';
    const cats = categorias || (Array.isArray(proyectoContexto?.categorias) ? proyectoContexto.categorias.join(', ') : proyectoContexto?.categorias || '');
    const galeria = proyectoContexto?.galeria || [];
    const tecnologias = proyectoContexto?.tecnologias || [];
    const enlaces = proyectoContexto?.enlaces || [];

    /* Proyectos relacionados: misma categoría, excluyendo el actual */
    const categoriasArray = cats.split(',').map(c => c.trim().toLowerCase());
    const relacionados = todosProyectos.filter(p => {
        if (String(p.id) === slug || p.titulo === titulo) return false;
        const pCats = Array.isArray(p.categorias) ? p.categorias : [p.categorias];
        return pCats.some(c => categoriasArray.includes(c.toLowerCase()));
    }).slice(0, 3);

    return (
        <LayoutPagina className="proyectoIndividualMain" id="paginaProyecto">
            <SEOHead
                title={titulo}
                description={desc}
                path={`/proyectos/${slug || ''}`}
            />
            {/* Hero del proyecto */}
            <section className="proyectoHero">
                <div className="proyectoHeroContenido">
                    <div className="proyectoHeroMeta">
                        <span className="proyectoHeroCliente">{cliente}</span>
                        <span className="proyectoHeroCategorias">{cats}</span>
                    </div>
                    <h1 className="proyectoHeroTitulo">{titulo}</h1>
                    {desc && <p className="proyectoHeroDescripcion">{desc}</p>}

                    {/* [064A-8] Detalles técnicos: tecnologías y enlaces */}
                    {(tecnologias.length > 0 || enlaces.length > 0) && (
                        <div className="proyectoHeroDetalles">
                            {tecnologias.length > 0 && (
                                <div className="proyectoHeroTecnologias">
                                    {tecnologias.map(tech => (
                                        <span key={tech} className="proyectoHeroTechTag">{tech}</span>
                                    ))}
                                </div>
                            )}
                            {enlaces.length > 0 && (
                                <div className="proyectoHeroEnlaces">
                                    {enlaces.map(enlace => {
                                        const Icono = ICONOS_ENLACE[enlace.tipo] || ExternalLink;
                                        return (
                                            <a key={enlace.url} href={enlace.url} target="_blank" rel="noopener noreferrer" className="proyectoHeroEnlace">
                                                <Icono size={16} />
                                                <span>{enlace.etiqueta || enlace.tipo}</span>
                                            </a>
                                        );
                                    })}
                                </div>
                            )}
                        </div>
                    )}
                </div>
            </section>

            {/* Galería de imágenes — usa galería del proyecto si está disponible */}
            <SeccionGaleriaServicio imagenes={galeria} />

            {/* Skills del proyecto */}
            {skills.length > 0 && <SeccionSkillsServicio skills={skills} />}

            {/* CTA */}
            <SeccionCta descripcion={[t('project_detail.cta_1'), t('project_detail.cta_2')]} textoBotonPrimario={t('project_detail.cta_start')} onBotonPrimarioClick={abrirChat} textoBotonSecundario={t('project_detail.cta_more')} linkBotonSecundario="/proyectos/" />

            {/* Proyectos relacionados */}
            {relacionados.length > 0 && (
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
            )}

            <SeccionContacto />
        </LayoutPagina>
    );
};

export default ProyectoIndividualIsland;
