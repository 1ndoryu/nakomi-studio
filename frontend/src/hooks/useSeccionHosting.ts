/* [074A-63] Hook de estado para SeccionHosting.
 * Extrae los 4 useState + queries + mutations para cumplir regla max 3 useState. */

import {useState, useMemo} from 'react';
import {useQuery, useMutation, useQueryClient} from '@tanstack/react-query';
import {
    apiListHostingSubscriptions,
    apiCreateHostingSubscription,
    apiUpdateHostingStatus,
    apiUpdateHostingSubscription,
    apiDeleteHostingSubscription,
    apiRequestCancelHosting,
    type HostingSubscription,
    type CreateHostingRequest,
    type UpdateHostingRequest,
} from '../api/hosting';
import {toast} from '../stores/toastStore';
import {useAuthStore} from '../stores/authStore';

const ACTIVE_STATUSES = new Set(['pending', 'provisioning', 'active']);

export function useSeccionHosting() {
    const queryClient = useQueryClient();
    const [showCreateModal, setShowCreateModal] = useState(false);
    const [selectedSub, setSelectedSub] = useState<HostingSubscription | null>(null);
    const [showEvents, setShowEvents] = useState(false);
    const [tabActiva, setTabActiva] = useState<'activos' | 'inactivos'>('activos');

    const effectiveRole = useAuthStore(s => s.user?.effectiveRole) ?? 'client';
    const isAdmin = effectiveRole === 'admin';

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
        selectedSub,
        setSelectedSub,
        showEvents,
        setShowEvents,
        createMutation,
        statusMutation,
        updateMutation,
        deleteMutation,
        cancelMutation,
    };
}
