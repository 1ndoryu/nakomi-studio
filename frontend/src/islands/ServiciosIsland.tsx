/**
 * Componente: ServiciosIsland
 * Página completa de Servicios con grid filtrable.
 * Categorías centralizadas en data/navegacion.ts (DRY).
 */
import '../styles/variables.css';
import {useTranslation} from 'react-i18next';
import {CatalogPageShell} from '../components/layout/CatalogPageShell';
import {BarraFiltros} from '../components/servicios/BarraFiltros';
import {GridServicios} from '../components/servicios/GridServicios';
import {useServicios} from '../hooks/useServicios';
import {CATEGORIAS_SERVICIOS} from '../data/navegacion';

interface ServiciosIslandProps {
    titulo?: string;
}

export const ServiciosIsland = ({titulo}: ServiciosIslandProps): JSX.Element => {
    const {t} = useTranslation();
    const {categoriaActiva, setCategoriaActiva, busqueda, setBusqueda, serviciosFiltrados} = useServicios();

    return (
        <CatalogPageShell
            id="paginaServicios"
            seoTitle="Servicios"
            seoDescription="Descubre nuestros servicios de desarrollo web, diseño UI/UX y soluciones digitales a medida."
            path="/servicios"
            title={titulo || t('services_page.title')}
            description={t('services_page.description')}
        >
            <BarraFiltros categorias={CATEGORIAS_SERVICIOS} categoriaActiva={categoriaActiva} busqueda={busqueda} onCategoriaChange={setCategoriaActiva} onBusquedaChange={setBusqueda} />
            <GridServicios servicios={serviciosFiltrados} />
        </CatalogPageShell>
    );
};

export default ServiciosIsland;
