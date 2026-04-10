/**
 * Datos y tipos para el Panel de usuario.
 * Configuracion de tabs por rol, helper de usuario actual.
 * [044A-38 Fase 1] Tabs dinámicos por rol (admin/employee/client).
 */
import type {UserRole} from '../api/auth';

/* [064A-34] Eliminados: servicios, empleados, config-servicios (páginas innecesarias).
 * [064A-62] Añadido 'configuracion' para tab admin de herramientas dev. */
export type SeccionPanel =
    | 'proyectos' | 'pagos' | 'perfil' | 'metodos-pago' | 'mensajes'
    | 'asignados' | 'disponibles' | 'delegaciones'
    | 'todos-ordenes' | 'reembolsos' | 'usuarios' | 'hosting' | 'configuracion'
    | 'contenido' | 'problemas';

export interface TabConfig {
    id: SeccionPanel;
    label: string;
    descripcion: string;
}

/* [104A-15] El flujo publico de compra fuerza esta tab al volver al panel para
 * que la orden recien creada quede visible aunque el usuario hubiera dejado otra
 * seccion persistida en sessionStorage. */
export const PANEL_TAB_KEY = 'panel-tab';

/* [044A-38 Fase 1] Tabs para cliente
 * [064A-34] Tab 'servicios' eliminado. */
const TABS_CLIENT: TabConfig[] = [
    {
        id: 'proyectos',
        label: 'Mis Proyectos',
        descripcion: 'Aqui podras ver el estado de tus proyectos en progreso y los finalizados. Seguimiento en tiempo real del avance de cada servicio contratado.'
    },
    {
        id: 'pagos',
        label: 'Historial de Pagos',
        descripcion: 'Historial completo de pagos realizados. Facturas, recibos y metodos de pago asociados a tu cuenta.'
    },
    {
        id: 'perfil',
        label: 'Configurar Perfil',
        descripcion: 'Personaliza tu perfil publico: nombre, imagen, descripcion y redes sociales.'
    },
    {
        id: 'metodos-pago',
        label: 'Metodos de Pago',
        descripcion: 'Administra tus tarjetas de credito, direccion de facturacion y metodos de pago registrados.'
    },
    {
        id: 'mensajes',
        label: 'Mensajes',
        descripcion: 'Conversaciones con soporte, asistente IA y seguimiento de tus ordenes por chat.'
    },
    /* [064A-32] Hosting visible para clientes con suscripciones */
    {
        id: 'hosting',
        label: 'Mi Hosting',
        descripcion: 'Estado de tus suscripciones de hosting, plan activo y eventos recientes.'
    }
];

/* [044A-38 Fase 1] Tabs para empleado */
const TABS_EMPLOYEE: TabConfig[] = [
    {
        id: 'asignados',
        label: 'Asignados',
        descripcion: 'Ordenes asignadas a ti. Revisa el estado, entrega fases y comunica con el cliente.'
    },
    {
        id: 'disponibles',
        label: 'Disponibles',
        descripcion: 'Ordenes sin asignar disponibles para tomar. Revisa requisitos y acepta nuevos proyectos.'
    },
    {
        id: 'delegaciones',
        label: 'Delegaciones',
        descripcion: 'Solicitudes de delegación y ayuda entre empleados. Acepta, rechaza o delega órdenes.'
    },
    {
        id: 'mensajes',
        label: 'Mensajes',
        descripcion: 'Conversaciones con clientes, delegaciones y chat de soporte en tiempo real.'
    },
    {
        id: 'perfil',
        label: 'Configurar Perfil',
        descripcion: 'Personaliza tu perfil publico: nombre, imagen, descripcion y redes sociales.'
    }
];

/* [044A-38 Fase 1] Tabs para admin: todo el sistema
 * [064A-34] Tabs 'empleados' y 'config-servicios' eliminados. */
const TABS_ADMIN: TabConfig[] = [
    {
        id: 'todos-ordenes',
        label: 'Todas las Ordenes',
        descripcion: 'Vista completa de todas las ordenes del sistema. Filtra por estado, cliente o empleado.'
    },
    {
        id: 'reembolsos',
        label: 'Reembolsos',
        descripcion: 'Gestiona solicitudes de reembolso: revisa, aprueba o rechaza pedidos de clientes.'
    },
    {
        id: 'usuarios',
        label: 'Usuarios',
        descripcion: 'Busca, filtra y gestiona los usuarios registrados: cambia roles, banea o reactiva cuentas.'
    },
    {
        id: 'hosting',
        label: 'Hosting',
        descripcion: 'Gestiona suscripciones de hosting: planes, dominios, estados y eventos de cada cliente.'
    },
    {
        id: 'mensajes',
        label: 'Mensajes',
        descripcion: 'Todas las conversaciones de soporte, chat IA y mensajes de ordenes.'
    },
    {
        id: 'perfil',
        label: 'Configurar Perfil',
        descripcion: 'Personaliza tu perfil publico: nombre, imagen, descripcion y redes sociales.'
    },
    /* [064A-62] Tab de configuración/herramientas dev */
    {
        id: 'configuracion',
        label: 'Configuración',
        descripcion: 'Herramientas de desarrollo: recrear o borrar datos de prueba, configuraciones del sistema.'
    },
    /* [074A-7] Tab de contenido editorial — CMS admin */
    {
        id: 'contenido',
        label: 'Contenido',
        descripcion: 'Gestiona servicios, blog, proyectos y equipo. Editor de contenido con texto enriquecido.'
    },
    /* [104A-28] Tab de problemas reportados — gestión de tickets */
    {
        id: 'problemas',
        label: 'Problemas',
        descripcion: 'Reportes de problemas en órdenes. Revisa, resuelve o descarta tickets de clientes y empleados.'
    }
];

/* [044A-38 Fase 1] Devuelve las tabs correspondientes al rol efectivo */
export function obtenerTabsPorRol(role: UserRole): TabConfig[] {
    switch (role) {
        case 'admin': return TABS_ADMIN;
        case 'employee': return TABS_EMPLOYEE;
        default: return TABS_CLIENT;
    }
}

/* Legacy: mantiene compat con imports existentes que aún usen TABS_PANEL */
export const TABS_PANEL: TabConfig[] = TABS_CLIENT;

/* [044A-38 Fase 1] Devuelve la seccion por defecto según el rol */
export function seccionInicialPorRol(role: UserRole): SeccionPanel {
    switch (role) {
        case 'admin': return 'todos-ordenes';
        case 'employee': return 'asignados';
        default: return 'proyectos';
    }
}

export interface UsuarioActual {
    id: number;
    nombre: string;
    email: string;
    avatar: string;
    rol: string;
}

/* [044A-1] TO-DO: Conectar con API Rust para obtener usuario actual */
export function obtenerUsuarioActual(): UsuarioActual | null {
    return null;
}
