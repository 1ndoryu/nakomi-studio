import {useCallback, useMemo, useState} from 'react';
import {useAuthStore} from '../stores/authStore';
import {useOrdenes} from './useOrdenes';
import type {OrderResponse, OrderStatus} from '../api/orders';

const STATUS_PRIORITY: Record<string, number> = {
    in_progress: 0,
    under_review: 1,
    awaiting_assignment: 2,
    payment_held: 3,
    pending_payment: 3,
    disputed: 5,
    completed: 6,
    cancelled: 7,
};

const HISTORY_STATUSES: Set<OrderStatus> = new Set(['completed', 'cancelled']);

function sortByStatusPriority(a: OrderResponse, b: OrderResponse): number {
    return (STATUS_PRIORITY[a.status] ?? 99) - (STATUS_PRIORITY[b.status] ?? 99);
}

export function useSeccionProyectos() {
    const effectiveRole = useAuthStore(s => s.user?.effectiveRole) || 'client';
    const isAdmin = effectiveRole === 'admin';
    const [tabActiva, setTabActiva] = useState<'activas' | 'historial'>('activas');
    const [busqueda, setBusqueda] = useState('');
    const [filtroEmpleado, setFiltroEmpleado] = useState<string>('');
    const [empleadoMenuAbierto, setEmpleadoMenuAbierto] = useState(false);
    const {
        ordenes, cargando, detalle, cargandoDetalle,
        ordenSeleccionada, error, seleccionarOrden, recargar,
        cancelarOrden, aprobarFase, solicitarRevision,
        actualizarDescripcionProyecto, actualizarFase,
        cancelando, actualizandoDescripcion, actualizandoFase,
    } = useOrdenes();

    const handleVolver = useCallback(() => seleccionarOrden(null), [seleccionarOrden]);
    const handleCancelar = useCallback(async (orderId: string, reason?: string) => {
        await cancelarOrden({orderId, reason});
    }, [cancelarOrden]);
    const handleAprobar = useCallback(async (orderId: string, phase: number) => {
        await aprobarFase({orderId, phase});
    }, [aprobarFase]);
    const handleRevision = useCallback(async (orderId: string, phase: number) => {
        await solicitarRevision({orderId, phase});
    }, [solicitarRevision]);
    const handleActualizarDescripcion = useCallback(async (orderId: string, projectDescription: string) => {
        await actualizarDescripcionProyecto({orderId, projectDescription});
    }, [actualizarDescripcionProyecto]);
    const handleActualizarFase = useCallback(async (
        orderId: string,
        phase: number,
        req: {
            title?: string;
            description?: string;
            price_cents?: number;
            estimated_days?: number;
            max_revisions?: number;
        },
    ) => {
        await actualizarFase({orderId, phase, req});
    }, [actualizarFase]);

    const activas = useMemo(() =>
        ordenes.filter(o => !HISTORY_STATUSES.has(o.status)).sort(sortByStatusPriority),
    [ordenes]);
    const historial = useMemo(() =>
        ordenes.filter(o => HISTORY_STATUSES.has(o.status)),
    [ordenes]);
    const listaBase = tabActiva === 'activas' ? activas : historial;

    const empleadosUnicos = useMemo(() => {
        if (!isAdmin) return [];
        const mapa = new Map<string, string>();
        for (const orden of ordenes) {
            if (orden.assigned_employee_id && orden.assigned_employee_name) {
                mapa.set(orden.assigned_employee_id, orden.assigned_employee_name);
            }
        }
        return Array.from(mapa, ([id, nombre]) => ({id, nombre}));
    }, [isAdmin, ordenes]);

    const listaActual = useMemo(() => {
        if (!isAdmin) return listaBase;
        let resultado = listaBase;
        if (busqueda.trim()) {
            const query = busqueda.toLowerCase();
            resultado = resultado.filter(orden => orden.service_title.toLowerCase().includes(query));
        }
        if (filtroEmpleado) {
            resultado = resultado.filter(orden => orden.assigned_employee_id === filtroEmpleado);
        }
        return resultado;
    }, [isAdmin, listaBase, busqueda, filtroEmpleado]);

    return {
        effectiveRole,
        isAdmin,
        tabActiva,
        setTabActiva,
        busqueda,
        setBusqueda,
        filtroEmpleado,
        setFiltroEmpleado,
        empleadoMenuAbierto,
        setEmpleadoMenuAbierto,
        ordenes,
        cargando,
        detalle,
        cargandoDetalle,
        ordenSeleccionada,
        error,
        seleccionarOrden,
        recargar,
        cancelando,
        actualizandoDescripcion,
        actualizandoFase,
        activas,
        historial,
        empleadosUnicos,
        listaActual,
        handleVolver,
        handleCancelar,
        handleAprobar,
        handleRevision,
        handleActualizarDescripcion,
        handleActualizarFase,
    };
}