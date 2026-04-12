/* [124A-SEARCH1] Hook reutilizable para búsqueda en tiempo real en listas del CMS.
 * Filtra items por texto contra múltiples campos (keys).
 * Normaliza acentos y case para búsqueda tolerante. */
import { useState, useMemo } from 'react';

/* Normalizar texto: minúsculas + sin acentos */
function normalizar(texto: string): string {
    return texto.toLowerCase().normalize('NFD').replace(/[\u0300-\u036f]/g, '');
}

export function useBusquedaLista<T>(items: T[], keys: (keyof T)[]) {
    const [busqueda, setBusqueda] = useState('');

    const filtrados = useMemo(() => {
        const termino = normalizar(busqueda.trim());
        if (!termino) return items;

        return items.filter(item =>
            keys.some(key => {
                const valor = item[key];
                if (typeof valor === 'string') {
                    return normalizar(valor).includes(termino);
                }
                if (Array.isArray(valor)) {
                    return valor.some(v => typeof v === 'string' && normalizar(v).includes(termino));
                }
                return false;
            })
        );
    }, [items, keys, busqueda]);

    return { busqueda, setBusqueda, filtrados };
}
