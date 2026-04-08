/* [084A-29] Overlay de administración para contenido editable.
 * Envuelve cualquier elemento de contenido (tarjeta, card, etc.) y muestra
 * un botón de 3 puntos en la esquina superior derecha al hacer hover.
 * Solo visible si el usuario logueado es admin. */

import { useState, type ReactNode } from 'react';
import { Pencil, Archive, Trash2 } from 'lucide-react';
import { useAuthStore } from '../../stores/authStore';
import { useAdminEditStore, type AdminContentType } from '../../stores/adminEditStore';
import { MenuContextual } from './ContextMenu';
import './AdminOverlay.css';

interface AdminOverlayProps {
    contentType: AdminContentType;
    itemId: string;
    children: ReactNode;
    className?: string;
}

export function AdminOverlay({ contentType, itemId, children, className }: AdminOverlayProps) {
    const isAdmin = useAuthStore(s => s.user?.effectiveRole === 'admin');
    const requestEdit = useAdminEditStore(s => s.requestEdit);
    const requestArchive = useAdminEditStore(s => s.requestArchive);
    const requestDelete = useAdminEditStore(s => s.requestDelete);
    const [menuOpen, setMenuOpen] = useState(false);

    if (!isAdmin) return <>{children}</>;

    return (
        <div className={`adminOverlay ${className || ''}`}>
            {children}
            <div className="adminOverlayAcciones">
                <MenuContextual
                    abierto={menuOpen}
                    onToggle={() => setMenuOpen(!menuOpen)}
                    onCerrar={() => setMenuOpen(false)}
                    ariaLabel="Opciones de administración"
                    triggerTamano="pequeno"
                    triggerClassName="adminOverlayTrigger"
                    panelClassName="adminOverlayPanel"
                    items={[
                        {
                            id: 'edit',
                            label: 'Editar',
                            icon: <Pencil size={14} />,
                            onSelect: () => requestEdit(contentType, itemId),
                        },
                        {
                            id: 'archive',
                            label: 'Archivar',
                            icon: <Archive size={14} />,
                            onSelect: () => requestArchive(contentType, itemId),
                        },
                        {
                            id: 'delete',
                            label: 'Eliminar',
                            icon: <Trash2 size={14} />,
                            danger: true,
                            onSelect: () => requestDelete(contentType, itemId),
                        },
                    ]}
                />
            </div>
        </div>
    );
}
