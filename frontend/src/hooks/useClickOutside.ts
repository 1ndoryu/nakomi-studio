/* [124A-SENT-R7] Hook para detectar clicks fuera de un elemento.
 * Reemplaza los useEffect manuales de addEventListener en SidebarPanel e IconPickerEnlace.
 * Escucha mousedown + touchstart para soporte móvil. */
import { useEffect, type RefObject } from 'react';

export function useClickOutside<T extends HTMLElement>(
    ref: RefObject<T | null>,
    handler: () => void,
    enabled = true,
): void {
    useEffect(() => {
        if (!enabled) return;
        const onOutside = (e: MouseEvent | TouchEvent) => {
            if (ref.current && !ref.current.contains(e.target as Node)) {
                handler();
            }
        };
        document.addEventListener('mousedown', onOutside);
        document.addEventListener('touchstart', onOutside);
        return () => {
            document.removeEventListener('mousedown', onOutside);
            document.removeEventListener('touchstart', onOutside);
        };
    }, [ref, handler, enabled]);
}
