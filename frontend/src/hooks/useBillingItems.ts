import {useMutation, useQuery, useQueryClient} from '@tanstack/react-query';

import {
    apiCreateBillingCheckout,
    apiListBillingItems,
    type BillingCheckoutMode,
} from '../api/billing';
import {toast} from '../stores/toastStore';

export const BILLING_ITEMS_KEY = ['billing-items'] as const;

export function useBillingItems() {
    const queryClient = useQueryClient();
    const query = useQuery({
        queryKey: BILLING_ITEMS_KEY,
        queryFn: apiListBillingItems,
        staleTime: 30_000,
    });

    const checkoutMutation = useMutation({
        mutationFn: ({itemIds, mode}: {itemIds?: string[]; mode: BillingCheckoutMode}) =>
            apiCreateBillingCheckout({item_ids: itemIds, mode}),
        onSuccess: (checkoutUrl) => {
            void queryClient.invalidateQueries({queryKey: BILLING_ITEMS_KEY});
            window.location.href = checkoutUrl;
        },
        onError: () => toast.error('No se pudo iniciar el pago pendiente'),
    });

    return {
        billingItems: query.data ?? [],
        isLoading: query.isLoading,
        error: query.error,
        checkoutMutation,
    };
}