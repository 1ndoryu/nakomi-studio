/* [154A-15a] API client para wallet y solicitudes de cancelación.
 * GET /api/wallet — saldo actual
 * GET /api/wallet/transactions — historial paginado
 * GET /api/orders/:id/cancel-request — listar solicitudes de cancelación
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

/* --- Withdrawal (retiro de saldo) --- */

export interface WithdrawalRequestResponse {
    id: string;
    user_id: string;
    amount_cents: number;
    status: string;
    payment_method: string | null;
    payment_details: string | null;
    admin_notes: string | null;
    resolved_by: string | null;
    created_at: string;
    resolved_at: string | null;
}

export interface WithdrawalRequestsPage {
    items: WithdrawalRequestResponse[];
    total: number;
    page: number;
    per_page: number;
}

export interface CreateWithdrawalRequest {
    amount_cents: number;
    payment_method?: string;
    payment_details?: string;
}

export interface ResolveWithdrawalRequest {
    approve: boolean;
    admin_notes?: string;
}

/* POST /api/wallet/withdraw */
export async function apiCreateWithdrawal(body: CreateWithdrawalRequest): Promise<WithdrawalRequestResponse> {
    const { data } = await instance.post<WithdrawalRequestResponse>('/api/wallet/withdraw', body);
    return data;
}

/* GET /api/wallet/withdrawals */
export async function apiListWithdrawals(page = 1, perPage = 10): Promise<WithdrawalRequestsPage> {
    const { data } = await instance.get<WithdrawalRequestsPage>(
        '/api/wallet/withdrawals',
        { params: { page, per_page: perPage } }
    );
    return data;
}

/* GET /api/admin/withdrawals */
export async function apiAdminListWithdrawals(page = 1, perPage = 20): Promise<WithdrawalRequestsPage> {
    const { data } = await instance.get<WithdrawalRequestsPage>(
        '/api/admin/withdrawals',
        { params: { page, per_page: perPage } }
    );
    return data;
}

/* PATCH /api/admin/withdrawals/:id */
export async function apiResolveWithdrawal(id: string, body: ResolveWithdrawalRequest): Promise<WithdrawalRequestResponse> {
    const { data } = await instance.patch<WithdrawalRequestResponse>(`/api/admin/withdrawals/${id}`, body);
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

/* GET /api/orders/:orderId/cancel-request — solicitudes pendientes */
export async function apiGetCancellationRequests(
    orderId: string
): Promise<CancellationRequestResponse[]> {
    const { data } = await instance.get<CancellationRequestResponse[]>(
        `/api/orders/${orderId}/cancel-request`
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
