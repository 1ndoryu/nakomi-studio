/* [044A-38 Fase 2] Sección "Mis Proyectos" del panel.
 * Muestra lista de órdenes con progreso visual + detalle con fases.
 * Vista lista ↔ detalle. Detalle extraído a OrdenDetalle.tsx (SRP).
 * [064A-50] Tabs Activas/Historial. Activas ordenadas por prioridad de status.
 * Canceladas y completadas van a Historial.
 * [084A-1] Admin: búsqueda, filtro por empleado, empleado en card footer.
 * sentinel-disable-file componente-sin-hook: la lógica son constantes de ordenamiento y filtros
 * que no son reutilizables; data ya está en useOrdenes.
 * sentinel-disable-file usestate-excesivo: 4 useState — tab, búsqueda, filtroEmpleado,
 * menuAbierto son estados UI independientes del componente lista. */
import React, {useCallback, useMemo, useState} from 'react';
import {FolderOpen, Search, ChevronDown} from 'lucide-react';
import {useAuthStore} from '../../stores/authStore';
import {useOrdenes} from '../../hooks/useOrdenes';
import {OrdenDetalle} from './OrdenDetalle';
import {Button} from '../ui/Button';
import {Input} from '../ui/Input';
import {MenuContextual} from '../ui/ContextMenu';
import {
    ORDER_STATUS_LABELS,
    PAYMENT_MODE_LABELS,
    formatPrice,
    type OrderResponse,
    type OrderStatus,
} from '../../api/orders';
import './SeccionProyectos.css';

/* [064A-50] Prioridad de status para ordenar (menor = más arriba).
 * in_progress es lo más urgente, completed/cancelled al fondo. */
const STATUS_PRIORITY: Record<string, number> = {
    in_progress: 0,
    under_review: 1,
    awaiting_assignment: 2,
    payment_held: 3,
    pending_payment: 4,
    disputed: 5,
    completed: 6,
    cancelled: 7,
};

/* Status que van al tab "Historial" */
const HISTORY_STATUSES: Set<OrderStatus> = new Set(['completed', 'cancelled']);

function sortByStatusPriority(a: OrderResponse, b: OrderResponse): number {
    return (STATUS_PRIORITY[a.status] ?? 99) - (STATUS_PRIORITY[b.status] ?? 99);
}

/* Mapa status → clase CSS (evita inline style) */
const STATUS_CLASS: Record<string, string> = {
    pending_payment: 'ordenBadge--pendingPayment',
    payment_held: 'ordenBadge--paymentHeld',
    awaiting_assignment: 'ordenBadge--awaiting',
    in_progress: 'ordenBadge--inProgress',
    under_review: 'ordenBadge--underReview',
    completed: 'ordenBadge--completed',
    cancelled: 'ordenBadge--cancelled',
    disputed: 'ordenBadge--disputed',
};

export const SeccionProyectos: React.FC = () => {
    const effectiveRole = useAuthStore(s => s.user?.effectiveRole) || 'client';
    const isAdmin = effectiveRole === 'admin';
    const [tabActiva, setTabActiva] = useState<'activas' | 'historial'>('activas');
    /* [084A-1] Búsqueda y filtro por empleado — solo admin */
    const [busqueda, setBusqueda] = useState('');
    const [filtroEmpleado, setFiltroEmpleado] = useState<string>('');
    const [empleadoMenuAbierto, setEmpleadoMenuAbierto] = useState(false);
    const {
        ordenes, cargando, detalle, cargandoDetalle,
        ordenSeleccionada, error, seleccionarOrden, recargar,
        cancelarOrden, aprobarFase, solicitarRevision, cancelando,
    } = useOrdenes();

    const handleVolver = useCallback(() => seleccionarOrden(null), [seleccionarOrden]);
    const handleCancelar = useCallback(async (orderId: string) => {
        await cancelarOrden(orderId);
    }, [cancelarOrden]);
    const handleAprobar = useCallback(async (orderId: string, phase: number) => {
        await aprobarFase({orderId, phase});
    }, [aprobarFase]);
    const handleRevision = useCallback(async (orderId: string, phase: number) => {
        await solicitarRevision({orderId, phase});
    }, [solicitarRevision]);

    /* [064A-50] Filtrar y ordenar por tab — debe estar antes de early returns para cumplir Rules of Hooks */
    const activas = useMemo(() =>
        ordenes.filter(o => !HISTORY_STATUSES.has(o.status)).sort(sortByStatusPriority),
        [ordenes]
    );
    const historial = useMemo(() =>
        ordenes.filter(o => HISTORY_STATUSES.has(o.status)),
        [ordenes]
    );
    const listaBase = tabActiva === 'activas' ? activas : historial;

    /* [084A-1] Filtrado admin: búsqueda por título + filtro por empleado.
     * [084A-4] Movido antes de early returns para cumplir Rules of Hooks. */
    const empleadosUnicos = useMemo(() => {
        if (!isAdmin) return [];
        const mapa = new Map<string, string>();
        for (const o of ordenes) {
            if (o.assigned_employee_id && o.assigned_employee_name) {
                mapa.set(o.assigned_employee_id, o.assigned_employee_name);
            }
        }
        return Array.from(mapa, ([id, nombre]) => ({id, nombre}));
    }, [isAdmin, ordenes]);

    const listaActual = useMemo(() => {
        if (!isAdmin) return listaBase;
        let resultado = listaBase;
        if (busqueda.trim()) {
            const q = busqueda.toLowerCase();
            resultado = resultado.filter(o => o.service_title.toLowerCase().includes(q));
        }
        if (filtroEmpleado) {
            resultado = resultado.filter(o => o.assigned_employee_id === filtroEmpleado);
        }
        return resultado;
    }, [isAdmin, listaBase, busqueda, filtroEmpleado]);

    if (cargando) {
        return <div className="proyectosLoading"><div className="proyectosSpinner" /><p>Cargando proyectos...</p></div>;
    }
    if (error) {
        return <div className="proyectosError">{error}</div>;
    }

    /* Vista detalle */
    if (ordenSeleccionada && detalle) {
        return (
            <OrdenDetalle
                order={detalle.order}
                phases={detalle.phases}
                effectiveRole={effectiveRole}
                onVolver={handleVolver}
                onCancelar={handleCancelar}
                onAprobar={handleAprobar}
                onRevision={handleRevision}
                onPagoExitoso={recargar}
                cancelando={cancelando}
            />
        );
    }
    if (ordenSeleccionada && cargandoDetalle) {
        return <div className="proyectosLoading"><div className="proyectosSpinner" /><p>Cargando detalle...</p></div>;
    }

    /* Lista vacía (todas las órdenes, no solo la tab) */
    if (ordenes.length === 0) {
        return (
            <div className="proyectosVacio">
                <FolderOpen size={48} className="proyectosVacioIcono" strokeWidth={1.2} />
                <h3 className="proyectosVacioTitulo">Sin proyectos aún</h3>
                <p className="proyectosVacioTexto">
                    Cuando contrates un servicio, aparecerá aquí con seguimiento en tiempo real.
                </p>
            </div>
        );
    }

    /* Lista de órdenes con tabs */
    return (
        <div className="proyectosContenedor">
            {/* [084A-1] Barra de búsqueda y filtro — solo admin */}
            {isAdmin && (
                <div className="proyectosFiltros">
                    <div className="proyectosBusqueda">
                        <Search size={16} className="proyectosBusquedaIcono" />
                        <Input
                            type="text"
                            className="proyectosBusquedaInput"
                            placeholder="Buscar por servicio..."
                            value={busqueda}
                            onChange={e => setBusqueda(e.target.value)}
                        />
                    </div>
                    {empleadosUnicos.length > 0 && (
                        <MenuContextual
                            abierto={empleadoMenuAbierto}
                            onToggle={() => setEmpleadoMenuAbierto(prev => !prev)}
                            onCerrar={() => setEmpleadoMenuAbierto(false)}
                            ariaLabel="Filtrar por empleado"
                            triggerClassName="proyectosFiltroEmpleado"
                            triggerVariante="outline"
                            triggerTamano="pequeno"
                            triggerContent={<>{filtroEmpleado ? empleadosUnicos.find(e => e.id === filtroEmpleado)?.nombre ?? 'Empleado' : 'Todos los empleados'} <ChevronDown size={14} /></>}
                            items={[
                                {id: 'all', label: 'Todos los empleados', onSelect: () => setFiltroEmpleado('')},
                                ...empleadosUnicos.map(emp => ({
                                    id: emp.id,
                                    label: emp.nombre,
                                    onSelect: () => setFiltroEmpleado(emp.id),
                                })),
                            ]}
                        />
                    )}
                </div>
            )}

            <div className="proyectosTabs">
                <Button
                    type="button"
                    variante="texto"
                    className={`proyectosTab ${tabActiva === 'activas' ? 'proyectosTab--activa' : ''}`}
                    onClick={() => setTabActiva('activas')}
                >
                    Activas ({activas.length})
                </Button>
                <Button
                    type="button"
                    variante="texto"
                    className={`proyectosTab ${tabActiva === 'historial' ? 'proyectosTab--activa' : ''}`}
                    onClick={() => setTabActiva('historial')}
                >
                    Historial ({historial.length})
                </Button>
            </div>

            {listaActual.length === 0 ? (
                <div className="proyectosVacio">
                    <p className="proyectosVacioTexto">
                        {tabActiva === 'activas'
                            ? 'No tienes proyectos activos en este momento.'
                            : 'No tienes órdenes en el historial.'}
                    </p>
                </div>
            ) : (
                <div className="proyectosLista">
                    {listaActual.map(orden => (
                        <OrdenCard
                            key={orden.id}
                            orden={orden}
                            onClick={() => seleccionarOrden(orden.id)}
                            mostrarEmpleado={isAdmin}
                        />
                    ))}
                </div>
            )}
        </div>
    );
};

/* [054A-6] Mapa slug → imagen estática del servicio.
 * Los servicios sin imagen en /public/assets/Servicios/ usan un fallback genérico. */
const SERVICE_IMAGE: Record<string, string> = {
    'diseno-web': '/assets/Servicios/diseno web.jpg',
    'desarrollo-apps': '/assets/Servicios/diseno de aplicaciones.jpg',
    'agentes-ia': '/assets/Servicios/agente ia.jpg',
    'branding': '/assets/Servicios/Identidad de marca.jpg',
    'ecommerce': '/assets/Servicios/ecommerce.jpg',
};

/* Sub-componente: card de orden en la lista */
/* [054A-6] Rediseño: quita numeración, plan_name, barra de progreso.
 * Agrega imagen del servicio resuelta por slug.
 * [084A-1] Vista admin muestra empleado responsable en footer.
 * [074A-59] Admin/empleado ven client_name en footer. */
function OrdenCard({orden, onClick, mostrarEmpleado}: {orden: OrderResponse; onClick: () => void; mostrarEmpleado?: boolean}) {
    const imgSrc = SERVICE_IMAGE[orden.service_slug] || '/assets/Servicios/diseno web.jpg';

    return (
        <Button className="ordenCard" onClick={onClick} type="button" variante="texto">
            <img
                className="ordenCardImagen"
                src={imgSrc}
                alt={orden.service_title}
                loading="lazy"
            />
            <div className="ordenCardBody">
                <div className="ordenCardHeader">
                    <h3 className="ordenCardTitulo">{orden.service_title}</h3>
                    <span className={`ordenCardBadge ${STATUS_CLASS[orden.status] || ''}`}>
                        {ORDER_STATUS_LABELS[orden.status]}
                    </span>
                </div>
                <div className="ordenCardFooter">
                    <span className="ordenCardPrecio">{formatPrice(orden.final_price_cents, orden.currency)}</span>
                    <span className="ordenCardModo">{PAYMENT_MODE_LABELS[orden.payment_mode]}</span>
                    {mostrarEmpleado && orden.client_name && (
                        <span className="ordenCardCliente">{orden.client_name}</span>
                    )}
                    {mostrarEmpleado && (
                        <span className="ordenCardEmpleado">
                            {orden.assigned_employee_name ?? 'Sin asignar'}
                        </span>
                    )}
                    <span className="ordenCardFecha">{new Date(orden.created_at).toLocaleDateString('es')}</span>
                </div>
            </div>
        </Button>
    );
}
