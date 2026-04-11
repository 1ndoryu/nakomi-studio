/* [164A-9] Hook para gestionar solicitudes de cancelación de órdenes.
 * Empleados crean solicitudes, clientes las aceptan o rechazan. */
import { useState, useCallback } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import {
    apiGetCancellationRequests,
    apiCreateCancellationRequest,
    apiRespondCancellationRequest,
    type CancellationRequestResponse,
} from '../api/wallet';

export function useCancellationRequest(orderId: string) {
    const queryClient = useQueryClient();
    const queryKey = ['cancel-requests', orderId];

    const { data: requests = [], isLoading } = useQuery({
        queryKey,
        queryFn: () => apiGetCancellationRequests(orderId),
        staleTime: 30_000,
    });

    const pendingRequest: CancellationRequestResponse | null =
        requests.find((r) => r.status === 'pending') ?? null;

    const [creating, setCreating] = useState(false);
    const [responding, setResponding] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const createRequest = useCallback(
        async (reason: string) => {
            setCreating(true);
            setError(null);
            try {
                await apiCreateCancellationRequest(orderId, reason);
                await queryClient.invalidateQueries({ queryKey });
                await queryClient.invalidateQueries({ queryKey: ['orders'] });
            } catch (err: unknown) {
                const msg =
                    (err as { response?: { data?: { message?: string } } })
                        ?.response?.data?.message || 'Error al solicitar cancelación';
                setError(msg);
                throw err;
            } finally {
                setCreating(false);
            }
        },
        [orderId, queryClient, queryKey],
    );

    const respond = useCallback(
        async (requestId: string, accept: boolean) => {
            setResponding(true);
            setError(null);
            try {
                await apiRespondCancellationRequest(orderId, requestId, accept);
                await queryClient.invalidateQueries({ queryKey });
                await queryClient.invalidateQueries({ queryKey: ['orders'] });
                await queryClient.invalidateQueries({ queryKey: ['wallet'] });
            } catch (err: unknown) {
                const msg =
                    (err as { response?: { data?: { message?: string } } })
                        ?.response?.data?.message || 'Error al responder solicitud';
                setError(msg);
                throw err;
            } finally {
                setResponding(false);
            }
        },
        [orderId, queryClient, queryKey],
    );

    return {
        requests,
        pendingRequest,
        isLoading,
        creating,
        responding,
        error,
        createRequest,
        respond,
    };
}
