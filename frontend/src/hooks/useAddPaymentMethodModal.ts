import {useEffect, useState} from 'react';

import {apiCreatePaymentMethodSetupIntent} from '../api/payments';

export function useAddPaymentMethodModal(open: boolean) {
    const [clientSecret, setClientSecret] = useState<string | null>(null);
    const [isLoadingIntent, setIsLoadingIntent] = useState(false);
    const [loadError, setLoadError] = useState<string | null>(null);

    useEffect(() => {
        if (!open) {
            setClientSecret(null);
            setLoadError(null);
            setIsLoadingIntent(false);
            return;
        }

        let cancelled = false;

        const prepareSetupIntent = async () => {
            setIsLoadingIntent(true);
            setLoadError(null);

            try {
                const response = await apiCreatePaymentMethodSetupIntent();
                if (!cancelled) {
                    setClientSecret(response.client_secret);
                }
            } catch (error) {
                if (!cancelled) {
                    const message = (error as {response?: {data?: {message?: string}}})?.response?.data?.message
                        ?? (error instanceof Error ? error.message : 'No se pudo preparar el guardado de la tarjeta');
                    setLoadError(message);
                }
            } finally {
                if (!cancelled) {
                    setIsLoadingIntent(false);
                }
            }
        };

        void prepareSetupIntent();

        return () => {
            cancelled = true;
        };
    }, [open]);

    return {
        clientSecret,
        isLoadingIntent,
        loadError,
    };
}