/* [074A-63] Hook de estado para SeccionHosting.
 * [094A-2] Ahora incluye selectedHostingId para navegación lista→detalle.
 * 3 useState: showCreateModal, tabActiva, selectedHostingId. */

import {useState, useMemo, useEffect} from 'react';
import {useQuery, useMutation, useQueryClient} from '@tanstack/react-query';
import {
    apiListHostingSubscriptions,
    apiCreateHostingSubscription,
    apiUpdateHostingStatus,
    apiUpdateHostingSubscription,
    apiDeleteHostingSubscription,
    apiRequestCancelHosting,
    apiCreateHostingCheckout,
    type CreateHostingRequest,
    type UpdateHostingRequest,
} from '../api/hosting';
import {toast} from '../stores/toastStore';
import {useAuthStore} from '../stores/authStore';

const ACTIVE_STATUSES = new Set(['pending', 'provisioning', 'active']);

export function useSeccionHosting() {
    const queryClient = useQueryClient();
    const [showCreateModal, setShowCreateModal] = useState(false);
    const [tabActiva, setTabActiva] = useState<'activos' | 'inactivos' | 'servidores'>('activos');
    const [selectedHostingId, setSelectedHostingId] = useState<string | null>(null);

    const effectiveRole = useAuthStore(s => s.user?.effectiveRole) ?? 'client';
    const isAdmin = effectiveRole === 'admin';

    /* [084A-24] Detectar retorno de Stripe Checkout (?hosting=success/cancelled) */
    useEffect(() => {
        const params = new URLSearchParams(window.location.search);
        const hostingResult = params.get('hosting');
        if (hostingResult === 'success') {
            toast.success('¡Pago completado! Tu hosting se activará en breve.');
        } else if (hostingResult === 'cancelled') {
            toast.warning('Checkout cancelado. Puedes intentarlo de nuevo.');
        }
        if (hostingResult) {
            const url = new URL(window.location.href);
            url.searchParams.delete('hosting');
            url.searchParams.delete('session_id');
            window.history.replaceState({}, '', url.pathname + url.search);
        }
    }, []);

    const hostingKey = ['hosting-subscriptions', effectiveRole] as const;

    const {data: subscriptions = [], isLoading} = useQuery({
        queryKey: hostingKey,
        queryFn: apiListHostingSubscriptions,
    });

    const activos = useMemo(
        () => subscriptions.filter(s => ACTIVE_STATUSES.has(s.status)),
        [subscriptions],
    );
    const inactivos = useMemo(
        () => subscriptions.filter(s => !ACTIVE_STATUSES.has(s.status)),
        [subscriptions],
    );
    const listaActual = tabActiva === 'activos' ? activos : inactivos;

    const createMutation = useMutation({
        mutationFn: (req: CreateHostingRequest) => apiCreateHostingSubscription(req),
        onSuccess: () => {
            queryClient.invalidateQueries({queryKey: hostingKey});
            toast.success('Suscripción creada');
            setShowCreateModal(false);
        },
        onError: () => toast.error('Error al crear suscripción'),
    });

    const statusMutation = useMutation({
        mutationFn: ({id, status}: {id: string; status: string}) =>
            apiUpdateHostingStatus(id, status),
        onSuccess: () => {
            queryClient.invalidateQueries({queryKey: hostingKey});
            toast.success('Status actualizado');
        },
        onError: () => toast.error('Error al actualizar status'),
    });

    /* [074A-65] Editar suscripción (plan, dominio) */
    const updateMutation = useMutation({
        mutationFn: ({id, req}: {id: string; req: UpdateHostingRequest}) =>
            apiUpdateHostingSubscription(id, req),
        onSuccess: () => {
            queryClient.invalidateQueries({queryKey: hostingKey});
            toast.success('Suscripción actualizada');
        },
        onError: () => toast.error('Error al actualizar suscripción'),
    });

    /* [074A-65] Eliminar suscripción */
    const deleteMutation = useMutation({
        mutationFn: (id: string) => apiDeleteHostingSubscription(id),
        onSuccess: () => {
            queryClient.invalidateQueries({queryKey: hostingKey});
            toast.success('Suscripción eliminada');
        },
        onError: () => toast.error('Error al eliminar suscripción'),
    });

    /* [084A-4] Cancelar suscripción (cliente o admin) */
    const cancelMutation = useMutation({
        mutationFn: (id: string) => apiRequestCancelHosting(id),
        onSuccess: () => {
            queryClient.invalidateQueries({queryKey: hostingKey});
            toast.success('Suscripción cancelada');
        },
        onError: () => toast.error('Error al cancelar suscripción'),
    });

    /* [084A-24] Checkout Stripe — redirige al cliente a Stripe para pagar */
    const checkoutMutation = useMutation({
        mutationFn: (id: string) => apiCreateHostingCheckout(id),
        onSuccess: (checkoutUrl) => {
            window.location.href = checkoutUrl;
        },
        onError: () => toast.error('Error al iniciar checkout'),
    });

    return {
        subscriptions,
        isLoading,
        isAdmin,
        tabActiva,
        setTabActiva,
        activos,
        inactivos,
        listaActual,
        showCreateModal,
        setShowCreateModal,
        selectedHostingId,
        setSelectedHostingId,
        createMutation,
        statusMutation,
        updateMutation,
        deleteMutation,
        cancelMutation,
        checkoutMutation,
    };
}
