/* [154A-15a] Hook de wallet: saldo + transacciones con React Query.
 * Separado del hook de órdenes para SRP. */
import { useQuery } from '@tanstack/react-query';
import { apiGetBalance, apiListTransactions } from '../api/wallet';
import type { WalletResponse, WalletTransactionsPage } from '../api/wallet';
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
