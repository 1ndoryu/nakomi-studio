/**
 * Componente: BarraFiltros
 * Barra de búsqueda y filtrado para la página de servicios.
 * Tipo FiltroCategoria centralizado en types/navegacion.ts (DRY).
 */
import React from 'react';
import {useTranslation} from 'react-i18next';
import {Search} from 'lucide-react';
import {Input} from '../ui/Input';
import {Button} from '../ui/Button';
import {FiltroCategoria} from '../../types/navegacion';
import './BarraFiltros.css';

interface BarraFiltrosProps {
    categorias: FiltroCategoria[];
    categoriaActiva: string;
    busqueda: string;
    onCategoriaChange: (id: string) => void;
    onBusquedaChange: (valor: string) => void;
}

export const BarraFiltros: React.FC<BarraFiltrosProps> = ({categorias, categoriaActiva, busqueda, onCategoriaChange, onBusquedaChange}) => {
    const {t} = useTranslation();

    const CAT_KEYS: Record<string, string> = {
        todos: 'categories.all',
        web: 'categories.web',
        software: 'categories.software',
        ai: 'categories.ai',
        branding: 'categories.branding',
    };

    return (
        <div className="barraFiltros">
            <div className="filtrosCategorias">
                {categorias.map(cat => (
                    <Button variante="texto" key={cat.id} className={`filtroBoton ${categoriaActiva === cat.id ? 'activo' : ''}`} onClick={() => onCategoriaChange(cat.id)}>
                        {CAT_KEYS[cat.id] ? t(CAT_KEYS[cat.id]) : cat.label}
                    </Button>
                ))}
            </div>
            <div className="filtroBusqueda">
                <Search className="busquedaIcono" />
                <Input type="text" className="busquedaInput" placeholder={t('services_page.search_placeholder')} value={busqueda} onChange={e => onBusquedaChange(e.target.value)} />
            </div>
        </div>
    );
};
