/* [154A-15a] API client para wallet y solicitudes de cancelación.
 * GET /api/wallet — saldo actual
 * GET /api/wallet/transactions — historial paginado
 * POST /api/orders/:id/cancel-request — crear solicitud de cancelación
 * POST /api/orders/:id/cancel-request/:reqId/respond — aceptar/rechazar */
import instance from './axios-instance';

export interface WalletResponse {
    id: string;
    user_id: string;
    balance_cents: number;
    currency: string;
}

export interface WalletTransactionResponse {
    id: string;
    wallet_id: string;
    user_id: string;
    amount_cents: number;
    transaction_type: string;
    reference_type: string | null;
    reference_id: string | null;
    description: string | null;
    balance_after_cents: number;
    created_at: string;
}

export interface WalletTransactionsPage {
    items: WalletTransactionResponse[];
    total: number;
    page: number;
    per_page: number;
}

export interface CancellationRequestResponse {
    id: string;
    order_id: string;
    order_number: number | null;
    requested_by: string;
    requester_name: string | null;
    reason: string;
    status: string;
    resolved_by: string | null;
    resolved_at: string | null;
    created_at: string;
}

/* Helpers para formatear saldo */
export function formatBalance(cents: number, currency = 'USD'): string {
    return new Intl.NumberFormat('es', {
        style: 'currency',
        currency,
        minimumFractionDigits: 2,
    }).format(cents / 100);
}

/* GET /api/wallet */
export async function apiGetBalance(): Promise<WalletResponse> {
    const { data } = await instance.get<WalletResponse>('/api/wallet');
    return data;
}

/* GET /api/wallet/transactions */
export async function apiListTransactions(
    page = 1,
    perPage = 20
): Promise<WalletTransactionsPage> {
    const { data } = await instance.get<WalletTransactionsPage>(
        '/api/wallet/transactions',
        { params: { page, per_page: perPage } }
    );
    return data;
}

/* POST /api/orders/:orderId/cancel-request */
export async function apiCreateCancellationRequest(
    orderId: string,
    reason: string
): Promise<CancellationRequestResponse> {
    const { data } = await instance.post<CancellationRequestResponse>(
        `/api/orders/${orderId}/cancel-request`,
        { reason }
    );
    return data;
}

/* POST /api/orders/:orderId/cancel-request/:requestId/respond */
export async function apiRespondCancellationRequest(
    orderId: string,
    requestId: string,
    accept: boolean
): Promise<CancellationRequestResponse> {
    const { data } = await instance.post<CancellationRequestResponse>(
        `/api/orders/${orderId}/cancel-request/${requestId}/respond`,
        { accept }
    );
    return data;
}
