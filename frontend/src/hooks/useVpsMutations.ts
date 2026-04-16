import {useMutation, useQueryClient} from '@tanstack/react-query';
import {apiApproveVpsSubscription, apiRejectVpsSubscription} from '../api/hosting';
import {toast} from '../stores/toastStore';

export function useVpsMutations(vpsKey: readonly string[]) {
    const queryClient = useQueryClient();

    const approveMutation = useMutation({
        mutationFn: (id: string) => apiApproveVpsSubscription(id),
        onSuccess: () => {
            queryClient.invalidateQueries({queryKey: vpsKey});
            toast.success('VPS aprobado y provisionado');
        },
        onError: () => toast.error('Error al aprobar el VPS'),
    });

    const rejectMutation = useMutation({
        mutationFn: ({id, reason}: {id: string; reason: string}) => apiRejectVpsSubscription(id, reason),
        onSuccess: () => {
            queryClient.invalidateQueries({queryKey: vpsKey});
            toast.success('Solicitud VPS rechazada');
        },
        onError: () => toast.error('Error al rechazar el VPS'),
    });

    return {
        approveMutation,
        rejectMutation,
    };
}