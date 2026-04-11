/**
 * Componente: ProyectoIndividualIsland
 * Página de detalle de un proyecto individual.
 * Estructura: Hero -> Galería -> Skills -> CTA -> Relacionados -> Contacto -> Footer
 * [114A-15] Refactorizado para usar componentes extraídos (consistencia con ServicioIndividualIsland).
 * sentinel-disable-file componente-sin-hook: Lógica minimal (fetch API + data lookup)
 * no justifica hook separado; datos estáticos vienen de imports directos.
 */
import {useState, useEffect} from 'react';
import {useTranslation} from 'react-i18next';
import {useChatStore} from '../stores/chatStore';
import '../styles/variables.css';
import './ProyectoIndividualIsland.css';
import {LayoutPagina} from '../components/layout/LayoutPagina';
import {SEOHead} from '../components/seo/SEOHead';
import {SeccionHeroProyecto} from '../components/proyectos/SeccionHeroProyecto';
import {SeccionProyectosRelacionados} from '../components/proyectos/SeccionProyectosRelacionados';
import {SeccionSkillsServicio} from '../components/servicios/SeccionSkillsServicio';
import {SeccionGaleriaServicio} from '../components/servicios/SeccionGaleriaServicio';
import {SeccionCta} from '../components/ui/SeccionCta';
import {SeccionContacto} from '../components/home/SeccionContacto';
import {PROYECTOS_DATA} from '../data/showcase';
import type {Proyecto, EnlaceProyecto} from '../types/contenido';
import {apiGetProjectBySlug, type AdminProject} from '../api/admin-projects';

interface ProyectoIndividualIslandProps {
    titulo?: string;
    descripcion?: string;
    cliente?: string;
    categorias?: string;
    imagen?: string;
    slug?: string;
}

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

export const ProyectoIndividualIsland = ({titulo = 'Proyecto', descripcion = '', cliente = '', categorias = '', slug = ''}: ProyectoIndividualIslandProps): JSX.Element => {
    const {t} = useTranslation();
    const abrirChat = useChatStore(s => s.abrir);

    /* [074A-12] Proyecto desde API con fallback a datos estáticos */
    const fallback = PROYECTOS_DATA.find(p => p.titulo.toLowerCase() === titulo.toLowerCase() || String(p.id) === slug);
    const [proyectoContexto, setProyectoContexto] = useState<Proyecto | undefined>(fallback);

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
        return () => controller.abort();
    }, [slug]);

    const skills = proyectoContexto?.skills || [];
    const desc = descripcion || proyectoContexto?.descripcion || '';
    const cats = categorias || (Array.isArray(proyectoContexto?.categorias) ? proyectoContexto.categorias.join(', ') : proyectoContexto?.categorias || '');
    const galeria = proyectoContexto?.galeria || [];
    const tecnologias = proyectoContexto?.tecnologias || [];
    const enlaces = proyectoContexto?.enlaces || [];

    return (
        <LayoutPagina className="proyectoIndividualMain" id="paginaProyecto">
            <SEOHead
                title={titulo}
                description={desc}
                path={`/proyectos/${slug || ''}`}
            />
            {/* Hero del proyecto */}
            <SeccionHeroProyecto
                titulo={titulo}
                descripcion={desc}
                cliente={cliente}
                categorias={cats}
                tecnologias={tecnologias}
                enlaces={enlaces}
            />

            {/* Galería de imágenes — usa galería del proyecto si está disponible */}
            <SeccionGaleriaServicio imagenes={galeria} />

            {/* Skills del proyecto */}
            {skills.length > 0 && <SeccionSkillsServicio skills={skills} />}

            {/* CTA */}
            <SeccionCta descripcion={[t('project_detail.cta_1'), t('project_detail.cta_2')]} textoBotonPrimario={t('project_detail.cta_start')} onBotonPrimarioClick={abrirChat} textoBotonSecundario={t('project_detail.cta_more')} linkBotonSecundario="/proyectos/" />

            {/* Proyectos relacionados */}
            <SeccionProyectosRelacionados
                slugActual={slug}
                tituloActual={titulo}
                categorias={cats}
            />

            <SeccionContacto />
        </LayoutPagina>
    );
};

export default ProyectoIndividualIsland;
