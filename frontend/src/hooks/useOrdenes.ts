/* [044A-38 Fase 2] Hook para gestión de órdenes en el panel.
 * Usa React Query para fetching + cache + invalidación.
 * Encapsula toda la lógica de estado de órdenes (SRP).
 * Max 3 useState → estado de UI mínimo, el resto en React Query.
 * [054A-1] CRÍTICO: incluir effectiveRole en la query key para que el cache se invalide
 * automáticamente al cambiar de rol (admin→client). Sin esto, el cache del admin (todas las
 * órdenes) persiste en vista cliente y produce 403 al abrir órdenes ajenas. */
import {useState, useCallback, useEffect} from 'react';
import {useQuery, useMutation, useQueryClient} from '@tanstack/react-query';
import {useAuthStore} from '../stores/authStore';
import {
    apiListOrders,
    apiGetOrder,
    apiCreateOrder,
    apiCancelOrder,
    apiApprovePhase,
    apiRequestRevision,
    apiUpdateOrderPhaseDefinition,
    apiUpdateOrderProjectDescription,
    type OrderResponse,
    type OrderDetailResponse,
    type CreateOrderRequest,
    type UpdateOrderPhaseDefinitionRequest,
} from '../api/orders';

const ordersKey = (role: string) => ['orders', role] as const;
const orderDetailKey = (role: string, id: string) => ['order', role, id] as const;

export function useOrdenes() {
    const queryClient = useQueryClient();
    const effectiveRole = useAuthStore(s => s.user?.effectiveRole) ?? 'client';
    const [ordenSeleccionada, setOrdenSeleccionada] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        setOrdenSeleccionada(null);
        setError(null);
    }, [effectiveRole]);

    /* Lista de órdenes del usuario — la key incluye effectiveRole para invalidar cache al cambiar rol */
    const {
        data: ordenes = [],
        isLoading: cargando,
        refetch: recargar,
    } = useQuery<OrderResponse[]>({
        queryKey: ordersKey(effectiveRole),
        queryFn: apiListOrders,
    });

    /* Detalle de orden seleccionada */
    const {
        data: detalle,
        isLoading: cargandoDetalle,
    } = useQuery<OrderDetailResponse>({
        queryKey: ordenSeleccionada ? orderDetailKey(effectiveRole, ordenSeleccionada) : ['order', effectiveRole, 'none'],
        queryFn: () => apiGetOrder(ordenSeleccionada!),
        enabled: !!ordenSeleccionada,
    });

    /* Invalidar cache tras mutaciones */
    const invalidar = useCallback(() => {
        queryClient.invalidateQueries({queryKey: ordersKey(effectiveRole)});
        if (ordenSeleccionada) {
            queryClient.invalidateQueries({queryKey: orderDetailKey(effectiveRole, ordenSeleccionada)});
        }
    }, [queryClient, ordenSeleccionada, effectiveRole]);

    /* Crear orden */
    const crearMutation = useMutation({
        mutationFn: (req: CreateOrderRequest) => apiCreateOrder(req),
        onSuccess: () => {
            invalidar();
            setError(null);
        },
        onError: (err: unknown) => {
            setError(extraerError(err));
        },
    });

    /* [104A-28] Cancelar orden — optimistic update + razón opcional.
     * Empleados deben proporcionar razón obligatoriamente. */
    const cancelarMutation = useMutation({
        mutationFn: ({orderId, reason}: {orderId: string; reason?: string}) =>
            apiCancelOrder(orderId, reason),
        onMutate: async ({orderId}) => {
            await queryClient.cancelQueries({queryKey: ordersKey(effectiveRole)});
            const prev = queryClient.getQueryData<OrderResponse[]>(ordersKey(effectiveRole));
            queryClient.setQueryData<OrderResponse[]>(ordersKey(effectiveRole), old =>
                old?.map(o => o.id === orderId ? {...o, status: 'cancelled' as const} : o),
            );
            return {prev};
        },
        onSuccess: () => {
            invalidar();
            setError(null);
        },
        onError: (err: unknown, _orderId, context) => {
            if (context?.prev) {
                queryClient.setQueryData(ordersKey(effectiveRole), context.prev);
            }
            setError(extraerError(err));
        },
    });

    /* Aprobar fase (cliente) */
    const aprobarFaseMutation = useMutation({
        mutationFn: ({orderId, phase}: {orderId: string; phase: number}) =>
            apiApprovePhase(orderId, phase),
        onSuccess: () => {
            invalidar();
            setError(null);
        },
        onError: (err: unknown) => {
            setError(extraerError(err));
        },
    });

    /* Solicitar revisión (cliente) */
    const revisionMutation = useMutation({
        mutationFn: ({orderId, phase}: {orderId: string; phase: number}) =>
            apiRequestRevision(orderId, phase),
        onSuccess: () => {
            invalidar();
            setError(null);
        },
        onError: (err: unknown) => {
            setError(extraerError(err));
        },
    });

    const actualizarDescripcionMutation = useMutation({
        mutationFn: ({orderId, projectDescription}: {orderId: string; projectDescription: string}) =>
            apiUpdateOrderProjectDescription(orderId, {project_description: projectDescription}),
        onSuccess: () => {
            invalidar();
            setError(null);
        },
        onError: (err: unknown) => {
            setError(extraerError(err));
        },
    });

    const actualizarFaseMutation = useMutation({
        mutationFn: ({
            orderId,
            phase,
            req,
        }: {
            orderId: string;
            phase: number;
            req: UpdateOrderPhaseDefinitionRequest;
        }) => apiUpdateOrderPhaseDefinition(orderId, phase, req),
        onSuccess: () => {
            invalidar();
            setError(null);
        },
        onError: (err: unknown) => {
            setError(extraerError(err));
        },
    });

    /* [044A-38 Fase 6] entregarFase eliminada de useOrdenes:
     * la entrega ahora se maneja en useDeliverables con upload multipart. */

    return {
        ordenes,
        cargando,
        detalle,
        cargandoDetalle,
        ordenSeleccionada,
        error,
        seleccionarOrden: setOrdenSeleccionada,
        recargar,
        crearOrden: crearMutation.mutateAsync,
        cancelarOrden: cancelarMutation.mutateAsync,
        aprobarFase: aprobarFaseMutation.mutateAsync,
        solicitarRevision: revisionMutation.mutateAsync,
        actualizarDescripcionProyecto: actualizarDescripcionMutation.mutateAsync,
        actualizarFase: actualizarFaseMutation.mutateAsync,
        creando: crearMutation.isPending,
        cancelando: cancelarMutation.isPending,
        actualizandoDescripcion: actualizarDescripcionMutation.isPending,
        actualizandoFase: actualizarFaseMutation.isPending,
    };
}

/* Extrae mensaje de error del response axios */
function extraerError(err: unknown): string {
    if (typeof err === 'object' && err !== null && 'response' in err) {
        const resp = (err as {response?: {data?: {message?: string; error?: string}}}).response;
        if (resp?.data?.message) return resp.data.message;
        if (resp?.data?.error) return resp.data.error;
    }
    return 'Error de conexión';
}
