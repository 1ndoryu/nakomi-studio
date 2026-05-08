/*
 * Datos de marcas/clientes centralizados.
 * [074A-marketing] Limpiado: se eliminaron marcas ficticias (TechStart, DribbleX, etc.).
 * Solo se mantienen clientes reales confirmados. Añadir nuevos clientes a medida que se consigan.
 * SeccionClientes oculta la sección automáticamente si el array queda vacío.
 */
import {Marca} from '../types/contenido';

export const MARCAS_DATA: Marca[] = [
    {id: 'trust-keith', nombre: 'Trust Keith', url: 'https://trustkeith.com', logo: ''},
    {id: 'onfolk', nombre: 'Onfolk', url: 'https://onfolk.com', logo: ''},
];
