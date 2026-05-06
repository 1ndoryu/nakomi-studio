/* [074A-21] Hook de servicios públicos.
 * [084A-30] Migrado a useQuery + isPending para eliminar flash de contenido.
 * [094A-24] Sin fallback estático: el catálogo visible debe reflejar exactamente
 * los servicios que el backend publica para evitar compras contra slugs fantasma. */
import {useState, useMemo} from 'react';
import {useQuery} from '@tanstack/react-query';
import {Servicio} from '../types/servicios';
import {apiListPublicServices, type PublicService} from '../api/admin-services';
import {buildFilterCategories, extractCategoryIds} from '../utils/catalogCategories';

/* Convierte PublicService (API) → Servicio (frontend) */
function convertirServicio(s: PublicService): Servicio {
    return {
        id: s.slug,
        adminId: s.id,
        titulo: s.title,
        descripcion: s.description || '',
        imagen: s.image_url || '',
        categorias: extractCategoryIds(s.categories),
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

    /* [084A-30] useQuery elimina flash: no muestra fallback mientras API resuelve */
    const {data: apiData, isPending} = useQuery({
        queryKey: ['public-services'],
        queryFn: apiListPublicServices,
        staleTime: 5 * 60 * 1000,
        retry: 1,
    });

    const servicios = useMemo(
        () => (apiData || []).map(convertirServicio),
        [apiData]
    );

    const categoriasDisponibles = useMemo(
        () => buildFilterCategories(servicios.flatMap(servicio => servicio.categorias)),
        [servicios]
    );

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
        categoriasDisponibles,
        setCategoriaActiva,
        busqueda,
        setBusqueda,
        serviciosFiltrados,
        isPending,
    };
};
