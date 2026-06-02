/* [026B-1] Hook para gestión admin de billing_items (cobros pendientes). */

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import {
    apiAdminListBillingItems,
    apiAdminUpdateBillingStatus,
} from '../api/admin-billing';

const ADMIN_BILLING_KEY = 'admin-billing-items';

export function useAdminBilling(statusFilter?: string) {
    const queryClient = useQueryClient();

    const { data: items = [], isLoading, error } = useQuery({
        queryKey: [ADMIN_BILLING_KEY, statusFilter],
        queryFn: () => apiAdminListBillingItems(statusFilter),
    });

    const toggleStatusMut = useMutation({
        mutationFn: ({ itemId, newStatus }: { itemId: string; newStatus: string }) =>
            apiAdminUpdateBillingStatus(itemId, newStatus),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: [ADMIN_BILLING_KEY] });
            /* Invalidar también el billing_items del cliente en el sidebar */
            queryClient.invalidateQueries({ queryKey: ['billing-items'] });
        },
    });

    return { items, isLoading, error, toggleStatusMut };
}
