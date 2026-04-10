import {useEffect, useState} from 'react';

import {
    apiDeletePaymentMethod,
    apiListPaymentMethods,
    type SavedPaymentMethod,
} from '../api/payments';
import {toast} from '../stores/toastStore';

function sortPaymentMethods(methods: SavedPaymentMethod[]): SavedPaymentMethod[] {
    return [...methods].sort((left, right) => {
        if (left.is_default === right.is_default) {
            return Date.parse(right.created_at) - Date.parse(left.created_at);
        }

        return left.is_default ? -1 : 1;
    });
}

export function usePaymentMethods() {
    const [paymentMethods, setPaymentMethods] = useState<SavedPaymentMethod[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [loadError, setLoadError] = useState<string | null>(null);
    const [deletingId, setDeletingId] = useState<string | null>(null);
    const [isAddModalOpen, setIsAddModalOpen] = useState(false);

    const loadPaymentMethods = async (signal?: AbortSignal) => {
        try {
            const methods = await apiListPaymentMethods(signal);
            setPaymentMethods(sortPaymentMethods(methods));
            setLoadError(null);
        } catch (error) {
            if (signal?.aborted) {
                return;
            }

            const message = (error as {response?: {data?: {message?: string}}})?.response?.data?.message
                ?? (error instanceof Error ? error.message : 'No se pudieron cargar tus tarjetas');
            setLoadError(message);
        } finally {
            if (!signal?.aborted) {
                setIsLoading(false);
            }
        }
    };

    useEffect(() => {
        const abortController = new AbortController();

        void loadPaymentMethods(abortController.signal);

        return () => abortController.abort();
    }, []);

    const handlePaymentMethodCreated = (method: SavedPaymentMethod) => {
        setPaymentMethods((currentMethods) => sortPaymentMethods([
            method,
            ...currentMethods.filter((currentMethod) => currentMethod.id !== method.id),
        ]));
        setIsAddModalOpen(false);
        toast.success('Tarjeta guardada correctamente');
    };

    const handleDeletePaymentMethod = async (paymentMethodId: string) => {
        setDeletingId(paymentMethodId);

        try {
            await apiDeletePaymentMethod(paymentMethodId);
            const refreshedMethods = await apiListPaymentMethods();
            setPaymentMethods(sortPaymentMethods(refreshedMethods));
            toast.success('Tarjeta eliminada correctamente');
        } catch (error) {
            const message = (error as {response?: {data?: {message?: string}}})?.response?.data?.message
                ?? (error instanceof Error ? error.message : 'No se pudo eliminar la tarjeta');
            toast.error(message);
        } finally {
            setDeletingId(null);
        }
    };

    const reloadPaymentMethods = async () => {
        setIsLoading(true);
        await loadPaymentMethods();
    };

    return {
        paymentMethods,
        isLoading,
        loadError,
        deletingId,
        isAddModalOpen,
        reloadPaymentMethods,
        openAddModal: () => setIsAddModalOpen(true),
        closeAddModal: () => setIsAddModalOpen(false),
        handlePaymentMethodCreated,
        handleDeletePaymentMethod,
    };
}