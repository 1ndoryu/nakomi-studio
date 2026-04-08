/* [044A-38 Fase 3] Hook de pagos: React Query para historial + mutation para iniciar pago.
 * Integra con Stripe Elements para confirmar el pago en el frontend.
 * [074A-59] Query key incluye effectiveRole para invalidar cache al cambiar rol. */

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { apiInitiatePayment, apiListPayments } from '../api/payments';
import type { InitiatePaymentRequest } from '../api/payments';
import { useAuthStore } from '../stores/authStore';
import { isAxiosError } from 'axios';

function extraerError(err: unknown): string {
    if (isAxiosError(err) && err.response?.data) {
        const d = err.response.data as Record<string, unknown>;
        if (typeof d.message === 'string') return d.message;
    }
    return err instanceof Error ? err.message : 'Error desconocido';
}

export function usePagos(orderId: string | null) {
    const queryClient = useQueryClient();
    const effectiveRole = useAuthStore(s => s.user?.effectiveRole ?? s.user?.role ?? 'client');

    const {
        data: pagos,
        isLoading: cargandoPagos,
        error: errorPagos,
    } = useQuery({
        queryKey: ['payments', orderId, effectiveRole],
        queryFn: () => apiListPayments(orderId!),
        enabled: !!orderId,
    });

    const iniciarPago = useMutation({
        mutationFn: (req: { orderId: string } & InitiatePaymentRequest) =>
            apiInitiatePayment(req.orderId, { phase_number: req.phase_number }),
        onSuccess: () => {
            if (orderId) {
                void queryClient.invalidateQueries({ queryKey: ['payments', orderId, effectiveRole] });
                void queryClient.invalidateQueries({ queryKey: ['order', orderId] });
            }
            void queryClient.invalidateQueries({ queryKey: ['orders'] });
        },
    });

    return {
        pagos: pagos ?? [],
        cargandoPagos,
        errorPagos: errorPagos ? extraerError(errorPagos) : null,
        iniciarPago,
    };
}
