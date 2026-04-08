/**
 * Componente: ProyectosIsland
 * Página de portfolio/proyectos con grid filtrable.
 */
import React, {useState, useMemo, useEffect} from 'react';
import {useTranslation} from 'react-i18next';
import {spaClick} from '../navegacionSPA';
import '../styles/variables.css';
import './ProyectosIsland.css';
import {LayoutPagina} from '../components/layout/LayoutPagina';
import {SEOHead} from '../components/seo/SEOHead';
import {PROYECTOS_DATA} from '../data/showcase';
import {CATEGORIAS_PROYECTOS} from '../data/navegacion';
import {BarraFiltros} from '../components/servicios/BarraFiltros';
import {Badge} from '../components/ui/Badge';
import {AdminOverlay} from '../components/ui/AdminOverlay';
import {obtenerImagenShowcase} from '../hooks/useImagenes';
import {Proyecto} from '../types/contenido';
import {apiListPublicProjects, AdminProject} from '../api/admin-projects';

interface ProyectosIslandProps {
    titulo?: string;
}

/* [064A-64] TarjetaProyecto traduce descripcion y cliente via content translations */
const TarjetaProyecto: React.FC<{proyecto: Proyecto; indice: number}> = ({proyecto, indice}) => {
    const {t} = useTranslation();
    const categorias = Array.isArray(proyecto.categorias) ? proyecto.categorias : [proyecto.categorias];

    /* Fallback a imagen showcase si el backend no resolvió la URL */
    const imagenSrc = proyecto.imagen || obtenerImagenShowcase(indice);

    return (
        <AdminOverlay contentType="project" itemId={proyecto.adminId || String(proyecto.id)}>
            <a href={proyecto.link || '#'} className="tarjetaProyecto" onClick={e => { if (proyecto.link) spaClick(e, proyecto.link); }}>
                <div className="tarjetaProyectoImagen">
                    <img src={imagenSrc} alt={proyecto.titulo} loading="lazy" />
                </div>
                <div className="tarjetaProyectoInfo">
                    <h3 className="tarjetaProyectoTitulo">{proyecto.titulo}</h3>
                    <span className="tarjetaProyectoCliente">{t(`content.projects.${proyecto.id}.cliente`, proyecto.cliente)}</span>
                    <div className="tarjetaProyectoTags">
                        {categorias.map(cat => (
                            <Badge key={cat} label={cat} />
                        ))}
                    </div>
                </div>
            </a>
        </AdminOverlay>
    );
};

/* [074A-12] Convierte AdminProject (API) → Proyecto (frontend) */
function convertirProyecto(p: AdminProject): Proyecto {
    return {
        id: p.slug,
        adminId: p.id,
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

export const ProyectosIsland = ({titulo}: ProyectosIslandProps): JSX.Element => {
    const {t} = useTranslation();
    const [categoriaActiva, setCategoriaActiva] = useState('todos');
    const [busqueda, setBusqueda] = useState('');
    const [proyectos, setProyectos] = useState<Proyecto[]>(PROYECTOS_DATA);

    /* [074A-12] Fetch proyectos desde API, fallback a datos estáticos */
    useEffect(() => {
        const controller = new AbortController();
        apiListPublicProjects()
            .then(data => {
                if (!controller.signal.aborted && data.length > 0) {
                    setProyectos(data.map(convertirProyecto));
                }
            })
            .catch(() => { /* mantiene PROYECTOS_DATA como fallback */ });
        return () => controller.abort();
    }, []);

    const proyectosFiltrados = useMemo(() => {
        return proyectos.filter(p => {
            const coincideCategoria = categoriaActiva === 'todos' ||
                (Array.isArray(p.categorias) ? p.categorias.includes(categoriaActiva) : p.categorias === categoriaActiva);
            const coincideBusqueda = !busqueda ||
                p.titulo.toLowerCase().includes(busqueda.toLowerCase()) ||
                p.cliente.toLowerCase().includes(busqueda.toLowerCase());
            return coincideCategoria && coincideBusqueda;
        });
    }, [categoriaActiva, busqueda]);

    return (
        <LayoutPagina className="proyectosMain" id="paginaProyectos">
            <SEOHead
                title="Proyectos"
                description="Portfolio de proyectos de desarrollo web y diseño digital de Nakomi Studio."
                path="/proyectos"
            />
            <section className="proyectosHero">
                <div className="heroContenido">
                    <div>
                        <h1 className="heroTitulo">{titulo || t('projects_page.title')}</h1>
                    </div>
                    <div className="heroDescripcion">
                        <p>{t('projects_page.description')}</p>
                    </div>
                </div>
            </section>

            <section className="proyectosContenido">
                <div className="proyectosContenedor">
                    <BarraFiltros
                        categorias={CATEGORIAS_PROYECTOS}
                        categoriaActiva={categoriaActiva}
                        busqueda={busqueda}
                        onCategoriaChange={setCategoriaActiva}
                        onBusquedaChange={setBusqueda}
                    />
                    <div className="proyectosGrid">
                        {proyectosFiltrados.map((proyecto, i) => (
                            <TarjetaProyecto key={proyecto.id} proyecto={proyecto} indice={i} />
                        ))}
                    </div>
                    {proyectosFiltrados.length === 0 && (
                        <p className="proyectosSinResultados">{t('projects_page.empty')}</p>
                    )}
                </div>
            </section>
        </LayoutPagina>
    );
};

export default ProyectosIsland;
