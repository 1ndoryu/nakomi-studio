/* [074A-11] Lista de blog posts en el CMS admin.
 * [114A-7] Menú 3 puntos con archivar/desarchivar/eliminar. */
import React, {useState} from 'react';
import {Plus, Archive, ArchiveRestore, Trash2, Globe} from 'lucide-react';
import {Button} from '../ui/Button';
import OptimizedImage from '../ui/OptimizedImage';
import {MenuContextual, type MenuContextualItem} from '../ui/ContextMenu';
import type {AdminBlogPost} from '../../api/admin-blog';
import './ListaBlog.css';

interface ListaBlogProps {
    posts: AdminBlogPost[];
    cargando: boolean;
    onEditar: (post: AdminBlogPost) => void;
    onCrear: () => void;
    onArchivar: (id: string) => void;
    onDesarchivar?: (id: string) => void;
    onEliminar?: (id: string) => void;
    onPublicar?: (id: string) => void;
}

/* Badge de status con color semántico */
function BadgeStatus({status}: {status: string}) {
    const clase = `listaBlogBadge listaBlogBadge--${status}`;
    const labels: Record<string, string> = {
        published: 'Publicado',
        draft: 'Borrador',
        archived: 'Archivado',
    };
    return <span className={clase}>{labels[status] ?? status}</span>;
}

/* Formatea fecha ISO a formato legible */
function formatearFecha(iso: string): string {
    try {
        return new Date(iso).toLocaleDateString('es', {day: 'numeric', month: 'short', year: 'numeric'});
    } catch {
        return iso;
    }
}

export const ListaBlog: React.FC<ListaBlogProps> = ({
    posts,
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
        return <div className="listaBlogCargando">Cargando posts...</div>;
    }

    return (
        <div className="listaBlog">
            <div className="listaBlogHeader">
                <h3 className="listaBlogTitulo">Blog</h3>
                <Button variante="primario" tamano="pequeno" onClick={onCrear}>
                    <Plus size={14} />
                    Nuevo
                </Button>
            </div>

            <div className="listaBlogGrid">
                {posts.map(post => {
                    const items: MenuContextualItem[] = [];
                    /* [084A-10] Publicar: solo si no está ya publicado */
                    if (post.status !== 'published' && onPublicar) {
                        items.push({id: 'publicar', label: 'Publicar', icon: <Globe size={14} />, onSelect: () => onPublicar(post.id)});
                    }
                    if (post.status !== 'archived') {
                        items.push({id: 'archivar', label: 'Archivar', icon: <Archive size={14} />, onSelect: () => onArchivar(post.id)});
                    }
                    if (post.status === 'archived' && onDesarchivar) {
                        items.push({id: 'desarchivar', label: 'Desarchivar', icon: <ArchiveRestore size={14} />, onSelect: () => onDesarchivar(post.id)});
                    }
                    if (onEliminar) {
                        items.push({id: 'eliminar', label: 'Eliminar', icon: <Trash2 size={14} />, danger: true, onSelect: () => onEliminar(post.id)});
                    }

                    return (
                        <div
                            key={post.id}
                            className={`listaBlogCard ${post.status === 'archived' ? 'listaBlogCard--inactivo' : ''}`}
                            onClick={() => onEditar(post)}
                            role="button"
                            tabIndex={0}
                            onKeyDown={e => { if (e.key === 'Enter') onEditar(post); }}
                        >
                            {post.featured_image && (
                                <div className="listaBlogImagen">
                                    <OptimizedImage src={post.featured_image} alt={post.title} loading="lazy" />
                                </div>
                            )}
                            <div className="listaBlogInfo">
                                <div className="listaBlogCardHeader">
                                    <span className="listaBlogNombre">{post.title}</span>
                                    <BadgeStatus status={post.status} />
                                </div>
                                <span className="listaBlogSlug">/{post.slug}</span>
                                {post.excerpt && (
                                    <span className="listaBlogDesc">{post.excerpt}</span>
                                )}
                                <div className="listaBlogCardFooter">
                                    <span className="listaBlogFecha">
                                        {formatearFecha(post.created_at)}
                                    </span>
                                    {post.tags.length > 0 && (
                                        <span className="listaBlogTags">
                                            {post.tags.slice(0, 3).join(', ')}
                                        </span>
                                    )}
                                </div>
                            </div>
                            <div className="listaBlogMenu" onClick={e => e.stopPropagation()}>
                                <MenuContextual
                                    abierto={menuActivo === post.id}
                                    onToggle={() => setMenuActivo(prev => prev === post.id ? null : post.id)}
                                    onCerrar={() => setMenuActivo(null)}
                                    ariaLabel="Acciones del post"
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
