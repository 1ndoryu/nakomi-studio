/* [154A-15a] Hook de wallet: saldo + transacciones con React Query.
 * Separado del hook de órdenes para SRP.
 * [T1-withdrawal] Añadidos hooks de retiros: useWithdrawals, useCreateWithdrawal,
 * useAdminWithdrawals, useResolveWithdrawal. */
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
    apiGetBalance, apiListTransactions,
    apiCreateWithdrawal, apiListWithdrawals,
    apiAdminListWithdrawals, apiResolveWithdrawal,
} from '../api/wallet';
import type {
    WalletResponse, WalletTransactionsPage,
    CreateWithdrawalRequest, ResolveWithdrawalRequest, WithdrawalRequestsPage,
} from '../api/wallet';
import { useAuthStore } from '../stores/authStore';

export function useWallet() {
    const logueado = useAuthStore(s => s.logueado);

    const {
        data: wallet,
        isLoading: cargandoSaldo,
    } = useQuery<WalletResponse>({
        queryKey: ['wallet'],
        queryFn: apiGetBalance,
        enabled: logueado,
    });

    return { wallet, cargandoSaldo };
}

export function useWalletTransactions(page = 1, perPage = 20) {
    const logueado = useAuthStore(s => s.logueado);

    const {
        data,
        isLoading: cargando,
    } = useQuery<WalletTransactionsPage>({
        queryKey: ['wallet-transactions', page, perPage],
        queryFn: () => apiListTransactions(page, perPage),
        enabled: logueado,
    });

    return {
        transacciones: data?.items ?? [],
        total: data?.total ?? 0,
        page: data?.page ?? 1,
        perPage: data?.per_page ?? perPage,
        cargando,
    };
}

/* Hook para listar solicitudes de retiro del usuario actual */
export function useWithdrawals(page = 1, perPage = 10) {
    const logueado = useAuthStore(s => s.logueado);

    const { data, isLoading: cargando } = useQuery<WithdrawalRequestsPage>({
        queryKey: ['withdrawals', page, perPage],
        queryFn: () => apiListWithdrawals(page, perPage),
        enabled: logueado,
    });

    return {
        solicitudes: data?.items ?? [],
        total: data?.total ?? 0,
        page: data?.page ?? 1,
        perPage: data?.per_page ?? perPage,
        cargando,
    };
}

/* Mutación para crear una solicitud de retiro */
export function useCreateWithdrawal() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: (body: CreateWithdrawalRequest) => apiCreateWithdrawal(body),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['withdrawals'] });
            queryClient.invalidateQueries({ queryKey: ['wallet'] });
        },
    });
}

/* Hook para listar retiros pendientes (admin) */
export function useAdminWithdrawals(page = 1, perPage = 20) {
    const logueado = useAuthStore(s => s.logueado);
    const role = useAuthStore(s => s.user?.effectiveRole);

    const { data, isLoading: cargando } = useQuery<WithdrawalRequestsPage>({
        queryKey: ['admin-withdrawals', page, perPage],
        queryFn: () => apiAdminListWithdrawals(page, perPage),
        enabled: logueado && role === 'admin',
    });

    return {
        solicitudes: data?.items ?? [],
        total: data?.total ?? 0,
        page: data?.page ?? 1,
        perPage: data?.per_page ?? perPage,
        cargando,
    };
}

/* Mutación admin para aprobar/rechazar retiro */
export function useResolveWithdrawal() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: ({ id, body }: { id: string; body: ResolveWithdrawalRequest }) =>
            apiResolveWithdrawal(id, body),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['admin-withdrawals'] });
            queryClient.invalidateQueries({ queryKey: ['withdrawals'] });
        },
    });
}
