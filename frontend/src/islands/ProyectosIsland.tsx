/**
 * Componente: ProyectosIsland
 * Página de portfolio/proyectos con grid filtrable.
 * [094A-20] Usa la misma fuente pública del CMS que la home, sin fallback estático.
 */
import React, {useState, useMemo} from 'react';
import {useQuery} from '@tanstack/react-query';
import {useTranslation} from 'react-i18next';
import {spaClick} from '../navegacionSPA';
import '../styles/variables.css';
import './ProyectosIsland.css';
import {CatalogPageShell} from '../components/layout/CatalogPageShell';
import {mapAdminProjectsToProyectos} from '../data/showcase';
import {CATEGORIAS_PROYECTOS} from '../data/navegacion';
import {BarraFiltros} from '../components/servicios/BarraFiltros';
import {Badge} from '../components/ui/Badge';
import {AdminOverlay} from '../components/ui/AdminOverlay';
import OptimizedImage from '../components/ui/OptimizedImage';
import {obtenerImagenShowcase} from '../hooks/useImagenes';
import {Proyecto} from '../types/contenido';
import {apiListPublicProjects} from '../api/admin-projects';

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
                    <OptimizedImage src={imagenSrc} alt={proyecto.titulo} width={640} height={480} sizes="(max-width: 768px) 100vw, (max-width: 1024px) 50vw, 33vw" />
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

export const ProyectosIsland = ({titulo}: ProyectosIslandProps): JSX.Element => {
    const {t} = useTranslation();
    const [categoriaActiva, setCategoriaActiva] = useState('todos');
    const [busqueda, setBusqueda] = useState('');

    const {data: apiProjects} = useQuery({
        queryKey: ['public-projects-showcase'],
        queryFn: apiListPublicProjects,
        staleTime: 5 * 60 * 1000,
        retry: 1,
    });

    const proyectos = useMemo<Proyecto[]>(
        () => mapAdminProjectsToProyectos(apiProjects || []),
        [apiProjects]
    );

    const proyectosFiltrados = useMemo(() => {
        return proyectos.filter(p => {
            const coincideCategoria = categoriaActiva === 'todos' ||
                (Array.isArray(p.categorias) ? p.categorias.includes(categoriaActiva) : p.categorias === categoriaActiva);
            const coincideBusqueda = !busqueda ||
                p.titulo.toLowerCase().includes(busqueda.toLowerCase()) ||
                p.cliente.toLowerCase().includes(busqueda.toLowerCase());
            return coincideCategoria && coincideBusqueda;
        });
    }, [busqueda, categoriaActiva, proyectos]);

    return (
        <CatalogPageShell
            id="paginaProyectos"
            seoTitle="Proyectos"
            seoDescription="Portfolio de proyectos de desarrollo web y diseño digital de Nakomi Studio."
            path="/proyectos"
            title={titulo || t('projects_page.title')}
            description={t('projects_page.description')}
        >
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
        </CatalogPageShell>
    );
};

export default ProyectosIsland;
