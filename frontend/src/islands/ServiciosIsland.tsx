/**
 * Componente: ServiciosIsland
 * Página completa de Servicios con grid filtrable.
 * Categorías centralizadas en data/navegacion.ts (DRY).
 */
import '../styles/variables.css';
import {useState} from 'react';
import {useTranslation} from 'react-i18next';
import {useNavigate} from 'react-router-dom';
import {ListTree} from 'lucide-react';
import {CatalogPageShell} from '../components/layout/CatalogPageShell';
import {BarraFiltros} from '../components/servicios/BarraFiltros';
import {GridServicios} from '../components/servicios/GridServicios';
import {MenuContextual} from '../components/ui/ContextMenu';
import {useServicios} from '../hooks/useServicios';
import './ServiciosIsland.css';

interface ServiciosIslandProps {
    titulo?: string;
}

export const ServiciosIsland = ({titulo}: ServiciosIslandProps): JSX.Element => {
    const {t} = useTranslation();
    const navigate = useNavigate();
    const [menuServiciosAbierto, setMenuServiciosAbierto] = useState(false);
    const {servicios, categoriaActiva, categoriasDisponibles, setCategoriaActiva, busqueda, setBusqueda, serviciosFiltrados} = useServicios();

    return (
        <CatalogPageShell
            id="paginaServicios"
            seoTitle="Servicios"
            seoDescription="Descubre nuestros servicios de desarrollo web, diseño UI/UX y soluciones digitales a medida."
            path="/servicios"
            title={titulo || t('services_page.title')}
            description={t('services_page.description')}
        >
            {servicios.length > 0 && (
                <div className="serviciosAccesos">
                    <MenuContextual
                        abierto={menuServiciosAbierto}
                        onToggle={() => setMenuServiciosAbierto(prev => !prev)}
                        onCerrar={() => setMenuServiciosAbierto(false)}
                        ariaLabel="Ir a un servicio"
                        triggerClassName="serviciosAccesosTrigger"
                        triggerContent={<ListTree size={16} aria-hidden="true" />}
                        items={servicios.map(servicio => ({
                            id: servicio.id,
                            label: t(`content.services.${servicio.id}.titulo`, servicio.titulo),
                            onSelect: () => navigate(servicio.link),
                        }))}
                    />
                </div>
            )}
            <BarraFiltros categorias={categoriasDisponibles} categoriaActiva={categoriaActiva} busqueda={busqueda} onCategoriaChange={setCategoriaActiva} onBusquedaChange={setBusqueda} />
            <GridServicios servicios={serviciosFiltrados} />
        </CatalogPageShell>
    );
};

export default ServiciosIsland;
