/* [035A-18] Resolver imagen de servicio en un solo punto para reutilizarla
 * en cards de proyectos, pagos y cualquier otro resumen del panel. */

const SERVICE_IMAGE: Record<string, string> = {
    'diseno-web': '/assets/Servicios/diseno web.jpg',
    'desarrollo-apps': '/assets/Servicios/diseno de aplicaciones.jpg',
    'agentes-ia': '/assets/Servicios/agente ia.jpg',
    'branding': '/assets/Servicios/Identidad de marca.jpg',
    'ecommerce': '/assets/Servicios/ecommerce.jpg',
};

const DEFAULT_SERVICE_IMAGE = '/assets/Servicios/diseno web.jpg';

export function getServiceImage(serviceSlug: string): string {
    return SERVICE_IMAGE[serviceSlug] ?? DEFAULT_SERVICE_IMAGE;
}