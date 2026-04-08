/* [094A-3] Mutations de hosting extraídas de useSeccionHosting para cumplir límite 120 líneas.
 * Contiene todas las mutaciones CRUD + checkout + self-service subscribe. */

import {useMutation, useQueryClient} from '@tanstack/react-query';
import {
    apiCreateHostingSubscription,
    apiUpdateHostingStatus,
    apiUpdateHostingSubscription,
    apiDeleteHostingSubscription,
    apiRequestCancelHosting,
    apiCreateHostingCheckout,
    apiSelfSubscribe,
    type CreateHostingRequest,
    type UpdateHostingRequest,
    type SelfSubscribeRequest,
} from '../api/hosting';
import {toast} from '../stores/toastStore';

export function useHostingMutations(
    hostingKey: readonly string[],
    onCreateSuccess: () => void,
) {
    const queryClient = useQueryClient();

    const createMutation = useMutation({
        mutationFn: (req: CreateHostingRequest) => apiCreateHostingSubscription(req),
        onSuccess: () => {
            queryClient.invalidateQueries({queryKey: hostingKey});
            toast.success('Suscripción creada');
            onCreateSuccess();
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

    const updateMutation = useMutation({
        mutationFn: ({id, req}: {id: string; req: UpdateHostingRequest}) =>
            apiUpdateHostingSubscription(id, req),
        onSuccess: () => {
            queryClient.invalidateQueries({queryKey: hostingKey});
            toast.success('Suscripción actualizada');
        },
        onError: () => toast.error('Error al actualizar suscripción'),
    });

    const deleteMutation = useMutation({
        mutationFn: (id: string) => apiDeleteHostingSubscription(id),
        onSuccess: () => {
            queryClient.invalidateQueries({queryKey: hostingKey});
            toast.success('Suscripción eliminada');
        },
        onError: () => toast.error('Error al eliminar suscripción'),
    });

    const cancelMutation = useMutation({
        mutationFn: (id: string) => apiRequestCancelHosting(id),
        onSuccess: () => {
            queryClient.invalidateQueries({queryKey: hostingKey});
            toast.success('Suscripción cancelada');
        },
        onError: () => toast.error('Error al cancelar suscripción'),
    });

    const checkoutMutation = useMutation({
        mutationFn: (id: string) => apiCreateHostingCheckout(id),
        onSuccess: (checkoutUrl) => {
            window.location.href = checkoutUrl;
        },
        onError: () => toast.error('Error al iniciar checkout'),
    });

    const subscribeMutation = useMutation({
        mutationFn: (req: SelfSubscribeRequest) => apiSelfSubscribe(req),
        onSuccess: (resp) => {
            queryClient.invalidateQueries({queryKey: hostingKey});
            toast.success('Suscripción creada. Redirigiendo al pago…');
            window.location.href = resp.checkout_url;
        },
        onError: () => toast.error('Error al contratar hosting'),
    });

    return {
        createMutation,
        statusMutation,
        updateMutation,
        deleteMutation,
        cancelMutation,
        checkoutMutation,
        subscribeMutation,
    };
}
