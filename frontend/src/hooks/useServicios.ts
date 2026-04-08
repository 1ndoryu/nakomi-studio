/* [074A-21] Hook de servicios públicos — consume API con fallback a datos estáticos. */
import {useState, useMemo, useEffect} from 'react';
import {SERVICIOS_DATA} from '../data/servicios';
import {Servicio} from '../types/servicios';
import {apiListPublicServices, type PublicService} from '../api/admin-services';

/* Convierte PublicService (API) → Servicio (frontend) */
function convertirServicio(s: PublicService): Servicio {
    return {
        id: s.slug,
        adminId: s.id,
        titulo: s.title,
        descripcion: s.description || '',
        imagen: s.image_url || '',
        categorias: [],
        link: `/servicios/${s.slug}`,
        skills: Array.isArray(s.skills) ? s.skills.map((sk: unknown, i: number) => {
            const obj = sk as Record<string, string>;
            return {id: i, titulo: obj.titulo || '', descripcion: obj.descripcion || ''};
        }) : [],
    };
}

interface UseServiciosProps {
    initialCategory?: string;
    initialSearch?: string;
}

export const useServicios = ({initialCategory = 'todos', initialSearch = ''}: UseServiciosProps = {}) => {
    const [categoriaActiva, setCategoriaActiva] = useState(initialCategory);
    const [busqueda, setBusqueda] = useState(initialSearch);
    const [servicios, setServicios] = useState<Servicio[]>(SERVICIOS_DATA);

    useEffect(() => {
        const controller = new AbortController();
        apiListPublicServices()
            .then(data => {
                if (!controller.signal.aborted && data.length > 0) {
                    setServicios(data.map(convertirServicio));
                }
            })
            .catch(() => { /* mantiene fallback estático */ });
        return () => controller.abort();
    }, []);

    /* Filtrado de servicios según categoría y búsqueda */
    const serviciosFiltrados = useMemo(() => {
        return servicios.filter((servicio: Servicio) => {
            /* Filtro por categoría */
            const coincideCategoria = categoriaActiva === 'todos' || servicio.categorias.includes(categoriaActiva);

            /* Filtro por búsqueda */
            const terminoBusqueda = busqueda.toLowerCase().trim();
            const coincideBusqueda = terminoBusqueda === '' || servicio.titulo.toLowerCase().includes(terminoBusqueda) || servicio.descripcion.toLowerCase().includes(terminoBusqueda);

            return coincideCategoria && coincideBusqueda;
        });
    }, [categoriaActiva, busqueda, servicios]);

    return {
        categoriaActiva,
        setCategoriaActiva,
        busqueda,
        setBusqueda,
        serviciosFiltrados
    };
};
