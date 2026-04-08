/* [074A-12] Lista de proyectos en el CMS admin.
 * [114A-7] Menú 3 puntos con archivar/desarchivar/eliminar. */
import React, {useState} from 'react';
import { Plus, Archive, ArchiveRestore, Trash2, Globe } from 'lucide-react';
import { Button } from '../ui/Button';
import { MenuContextual, type MenuContextualItem } from '../ui/ContextMenu';
import type { AdminProject } from '../../api/admin-projects';
import './ListaProyectos.css';

interface ListaProyectosProps {
    proyectos: AdminProject[];
    cargando: boolean;
    onEditar: (proyecto: AdminProject) => void;
    onCrear: () => void;
    onArchivar: (id: string) => void;
    onDesarchivar?: (id: string) => void;
    onEliminar?: (id: string) => void;
    onPublicar?: (id: string) => void;
}

function BadgeStatus({ status }: { status: string }) {
    const clase = `listaProyectosBadge listaProyectosBadge--${status}`;
    const labels: Record<string, string> = {
        published: 'Publicado',
        draft: 'Borrador',
        archived: 'Archivado',
    };
    return <span className={clase}>{labels[status] ?? status}</span>;
}

export const ListaProyectos: React.FC<ListaProyectosProps> = ({
    proyectos,
    cargando,
    onEditar,
    onCrear,
    onArchivar,
    onDesarchivar,
    onEliminar,
    onPublicar,
}) => {
    const [menuActivo, setMenuActivo] = useState<string | null>(null);

    if (cargando) {
        return <div className="listaProyectosCargando">Cargando proyectos...</div>;
    }

    return (
        <div className="listaProyectos">
            <div className="listaProyectosHeader">
                <h3 className="listaProyectosTitulo">Proyectos</h3>
                <Button variante="primario" tamano="pequeno" onClick={onCrear}>
                    <Plus size={14} />
                    Nuevo
                </Button>
            </div>

            <div className="listaProyectosGrid">
                {proyectos.map(proyecto => {
                    const items: MenuContextualItem[] = [];
                    /* [084A-10] Publicar: solo si no está ya publicado */
                    if (proyecto.status !== 'published' && onPublicar) {
                        items.push({id: 'publicar', label: 'Publicar', icon: <Globe size={14} />, onSelect: () => onPublicar(proyecto.id)});
                    }
                    if (proyecto.status !== 'archived') {
                        items.push({id: 'archivar', label: 'Archivar', icon: <Archive size={14} />, onSelect: () => onArchivar(proyecto.id)});
                    }
                    if (proyecto.status === 'archived' && onDesarchivar) {
                        items.push({id: 'desarchivar', label: 'Desarchivar', icon: <ArchiveRestore size={14} />, onSelect: () => onDesarchivar(proyecto.id)});
                    }
                    if (onEliminar) {
                        items.push({id: 'eliminar', label: 'Eliminar', icon: <Trash2 size={14} />, danger: true, onSelect: () => onEliminar(proyecto.id)});
                    }

                    return (
                        <div
                            key={proyecto.id}
                            className={`listaProyectosCard ${proyecto.status === 'archived' ? 'listaProyectosCard--inactivo' : ''}`}
                            onClick={() => onEditar(proyecto)}
                            role="button"
                            tabIndex={0}
                            onKeyDown={e => { if (e.key === 'Enter') onEditar(proyecto); }}
                        >
                            {proyecto.featured_image && (
                                <div className="listaProyectosImagen">
                                    <img src={proyecto.featured_image} alt={proyecto.title} loading="lazy" />
                                </div>
                            )}
                            <div className="listaProyectosInfo">
                                <div className="listaProyectosCardHeader">
                                    <span className="listaProyectosNombre">{proyecto.title}</span>
                                    <BadgeStatus status={proyecto.status} />
                                </div>
                                {proyecto.client && (
                                    <span className="listaProyectosCliente">{proyecto.client}</span>
                                )}
                                <span className="listaProyectosSlug">/{proyecto.slug}</span>
                                <div className="listaProyectosCardFooter">
                                    {proyecto.categories.length > 0 && (
                                        <span className="listaProyectosCategorias">
                                            {proyecto.categories.slice(0, 3).join(', ')}
                                        </span>
                                    )}
                                    {proyecto.technologies.length > 0 && (
                                        <span className="listaProyectosTech">
                                            {proyecto.technologies.slice(0, 3).join(', ')}
                                        </span>
                                    )}
                                </div>
                            </div>
                            <div className="listaProyectosMenu" onClick={e => e.stopPropagation()}>
                                <MenuContextual
                                    abierto={menuActivo === proyecto.id}
                                    onToggle={() => setMenuActivo(prev => prev === proyecto.id ? null : proyecto.id)}
                                    onCerrar={() => setMenuActivo(null)}
                                    ariaLabel="Acciones del proyecto"
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
