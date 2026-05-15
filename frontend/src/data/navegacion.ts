/*
 * Configuración centralizada de navegación.
 * Header, Footer, filtros: toda la estructura de links del sitio.
 */
import {EnlaceNavegacion, EnlaceFooter, FiltroCategoria} from '../types/navegacion';
import {SERVICIOS_DATA} from './servicios';

/* Enlaces del Header - Navegación principal del sitio */
export const ENLACES_HEADER: EnlaceNavegacion[] = [
    {label: 'Inicio', href: '/'},
    {
        label: 'Servicios',
        href: '/servicios/',
        hasDropdown: true,
        subEnlaces: SERVICIOS_DATA.map(servicio => ({label: servicio.titulo, href: servicio.link})),
    },
    {label: 'Proyectos', href: '/proyectos/'},
    {label: 'Nosotros', href: '/nosotros/'},
    /* [155A-14] Soluciones vuelve como agrupador contextual sin ruta propia. */
    {
        label: 'Soluciones',
        hasDropdown: true,
        subEnlaces: [
            {label: 'Hosting WordPress', href: '/soluciones/hosting-wordpress/'},
            {label: 'Hosting', href: '/soluciones/hosting/'},
            {label: 'VPS', href: '/soluciones/vps/'},
        ],
    },
];

/* Enlaces del Footer */
export const ENLACES_FOOTER: EnlaceFooter[] = [
    {label: 'Inicio', href: '/'},
    {label: 'Servicios', href: '/servicios/'},
    {label: 'Proyectos', href: '/proyectos/'},
    {label: 'Nosotros', href: '/nosotros/'},
    {label: 'Contacto', href: '/contacto/'},
    {label: 'Política de Privacidad', href: '/politica-privacidad/'}
];

/* Categorías de filtrado para la página de servicios */
export const CATEGORIAS_SERVICIOS: FiltroCategoria[] = [
    {id: 'todos', label: 'Todos'},
    {id: 'web', label: 'Diseño Web'},
    {id: 'software', label: 'Software'},
    {id: 'ai', label: 'Inteligencia Artificial'},
    {id: 'branding', label: 'Branding'}
];

/* Categorías de filtrado para la página de proyectos */
export const CATEGORIAS_PROYECTOS: FiltroCategoria[] = [
    {id: 'todos', label: 'Todos'},
    {id: 'web', label: 'Web'},
    {id: 'branding', label: 'Branding'},
    {id: 'app', label: 'Aplicaciones'},
    {id: 'ai', label: 'IA'}
];
