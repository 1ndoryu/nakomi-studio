/* [074A-9] Lista de servicios en el CMS admin.
 * [114A-7] Menú 3 puntos con archivar/desarchivar/eliminar.
 * [124A-CMS7] Convertido de grid a lista vertical, mismo patrón que ListaProyectos. */
import React, {useState} from 'react';
import {Plus, Archive, ArchiveRestore, Trash2, Globe, Eye, EyeOff} from 'lucide-react';
import {Button} from '../ui/Button';
import OptimizedImage from '../ui/OptimizedImage';
import {MenuContextual, type MenuContextualItem} from '../ui/ContextMenu';
import type {AdminService} from '../../api/admin-services';
import './ListaServicios.css';

interface ListaServiciosProps {
    servicios: AdminService[];
    cargando: boolean;
    onEditar: (servicio: AdminService) => void;
    onCrear: () => void;
    onArchivar: (id: string) => void;
    onDesarchivar?: (id: string) => void;
    onEliminar?: (id: string) => void;
    onPublicar?: (id: string) => void;
    onToggleHome?: (id: string, visible: boolean) => void;
}

/* Badge de status con color semántico */
function BadgeStatus({status}: {status: string}) {
    const clase = `listaServiciosBadge listaServiciosBadge--${status}`;
    const labels: Record<string, string> = {
        published: 'Publicado',
        draft: 'Borrador',
        archived: 'Archivado',
    };
    return <span className={clase}>{labels[status] ?? status}</span>;
}

export const ListaServicios: React.FC<ListaServiciosProps> = ({
    servicios,
    cargando,
    onEditar,
    onCrear,
    onArchivar,
    onDesarchivar,
    onEliminar,
    onPublicar,
    onToggleHome,
}) => {
    const [menuActivo, setMenuActivo] = useState<string | null>(null);

    if (cargando) {
        return <div className="listaServiciosCargando">Cargando servicios...</div>;
    }

    return (
        <div className="listaServicios">
            <div className="listaServiciosHeader">
                <h3 className="listaServiciosTitulo">Servicios</h3>
                <Button variante="primario" tamano="pequeno" onClick={onCrear}>
                    <Plus size={14} />
                    Nuevo
                </Button>
            </div>

            <div className="listaServiciosLista">
                {servicios.map(svc => {
                    const items: MenuContextualItem[] = [];
                    /* [124A-CMS4] Toggle visibilidad en home (is_active) */
                    if (onToggleHome) {
                        items.push({
                            id: 'toggle-home',
                            label: svc.is_active ? 'Ocultar del Home' : 'Mostrar en Home',
                            icon: svc.is_active ? <EyeOff size={14} /> : <Eye size={14} />,
                            onSelect: () => onToggleHome(svc.id, !svc.is_active),
                        });
                    }
                    if (svc.status !== 'published' && onPublicar) {
                        items.push({id: 'publicar', label: 'Publicar', icon: <Globe size={14} />, onSelect: () => onPublicar(svc.id)});
                    }
                    if (svc.status !== 'archived') {
                        items.push({id: 'archivar', label: 'Archivar', icon: <Archive size={14} />, onSelect: () => onArchivar(svc.id)});
                    }
                    if (svc.status === 'archived' && onDesarchivar) {
                        items.push({id: 'desarchivar', label: 'Desarchivar', icon: <ArchiveRestore size={14} />, onSelect: () => onDesarchivar(svc.id)});
                    }
                    if (onEliminar) {
                        items.push({id: 'eliminar', label: 'Eliminar', icon: <Trash2 size={14} />, danger: true, onSelect: () => onEliminar(svc.id)});
                    }

                    return (
                        <div
                            key={svc.id}
                            className={`listaServiciosFila ${!svc.is_active ? 'listaServiciosFila--inactivo' : ''}`}
                        >
                            {svc.image_url && (
                                <div className="listaServiciosMiniatura">
                                    <OptimizedImage src={svc.image_url} alt={svc.title} loading="lazy" />
                                </div>
                            )}

                            <div
                                className="listaServiciosFilaInfo"
                                onClick={() => onEditar(svc)}
                                role="button"
                                tabIndex={0}
                                onKeyDown={e => { if (e.key === 'Enter') onEditar(svc); }}
                            >
                                <span className="listaServiciosNombre">{svc.title}</span>
                                <BadgeStatus status={svc.status} />
                                <span className="listaServiciosPrecio">
                                    ${(svc.base_price_cents / 100).toFixed(0)} {svc.currency}
                                </span>
                                <span className="listaServiciosPlanes">
                                    {svc.plans.length} plan{svc.plans.length !== 1 ? 'es' : ''}
                                </span>
                            </div>

                            <div className="listaServiciosMenu" onClick={e => e.stopPropagation()}>
                                <MenuContextual
                                    abierto={menuActivo === svc.id}
                                    onToggle={() => setMenuActivo(prev => prev === svc.id ? null : svc.id)}
                                    onCerrar={() => setMenuActivo(null)}
                                    ariaLabel="Acciones del servicio"
                                    items={items}
                                />
                            </div>
                        </div>
                    );
                })}
            </div>
        </div>
    );
};
