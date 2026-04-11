/* [044A-40] Componente base Modal reutilizable.
 * Todos los modales del sistema deben usar este componente como contenedor.
 * Incluye: overlay con click-to-close, Escape listener, focus trap, scroll lock.
 * [114A-10] Usa createPortal para evitar que stacking contexts padre
 * causen desaparición del fondo. Compensa scrollbar width al bloquear scroll. */
import React, {useEffect, useRef} from 'react';
import {createPortal} from 'react-dom';
import {useFocusTrap} from '../../hooks/useFocusTrap';
import './Modal.css';

interface ModalProps {
    abierto: boolean;
    onCerrar: () => void;
    children: React.ReactNode;
    /** Clase CSS adicional para el contenedor interno */
    className?: string;
}

export const Modal: React.FC<ModalProps> = ({abierto, onCerrar, children, className}) => {
    const modalRef = useRef<HTMLDivElement>(null);
    useFocusTrap(modalRef, abierto);

    /* [114A-10] Bloquear scroll + compensar scrollbar para evitar layout shift */
    useEffect(() => {
        if (!abierto) return;

        const handleEscape = (e: KeyboardEvent) => {
            if (e.key === 'Escape') onCerrar();
        };

        const scrollbarWidth = window.innerWidth - document.documentElement.clientWidth;
        document.body.style.overflow = 'hidden';
        if (scrollbarWidth > 0) {
            document.body.style.paddingRight = `${scrollbarWidth}px`;
        }

        document.addEventListener('keydown', handleEscape);

        return () => {
            document.removeEventListener('keydown', handleEscape);
            document.body.style.overflow = '';
            document.body.style.paddingRight = '';
        };
    }, [abierto, onCerrar]);

    if (!abierto) return null;

    const handleOverlayClick = (e: React.MouseEvent<HTMLDivElement>) => {
        if (e.target === e.currentTarget) onCerrar();
    };

    const claseContenedor = className
        ? `modalBaseContenedor ${className}`
        : 'modalBaseContenedor';

    /* [114A-10] Portal a document.body para aislar de stacking contexts padre */
    return createPortal(
        <div className="modalBaseOverlay" onClick={handleOverlayClick}>
            <div className={claseContenedor} ref={modalRef} role="dialog" aria-modal="true">
                {children}
            </div>
        </div>,
        document.body,
    );
};
