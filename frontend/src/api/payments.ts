/* [044A-38 Fase 3] API client para pagos Stripe.
 * Conecta con endpoints de /api/orders/:id/pay y /payments.
 * Tipos alineados con PaymentResponse del backend Rust. */
import instance from './axios-instance';
import type { PaymentMode } from './orders';

export type PaymentStatus = 'pending' | 'held' | 'released' | 'refunded' | 'failed';

export interface PaymentIntentResponse {
    payment_id: string;
    client_secret: string;
    amount_cents: number;
    currency: string;
    bypassed?: boolean;
}

export interface PaymentResponse {
    id: string;
    order_id: string;
    phase_number: number | null;
    amount_cents: number;
    currency: string;
    status: PaymentStatus;
    payment_mode: PaymentMode;
    description: string | null;
    bypassed: boolean;
    created_at: string;
}

export interface InitiatePaymentRequest {
    phase_number?: number;
}

export interface SavedPaymentMethod {
    id: string;
    brand: string;
    last_four: string;
    exp_month: number;
    exp_year: number;
    is_default: boolean;
    created_at: string;
}

export interface SetupIntentClientSecretResponse {
    client_secret: string;
}

export interface SavePaymentMethodRequest {
    setup_intent_id: string;
}

/* [064A-57] Corregido: faltaba /api prefix en ambos endpoints. */
export async function apiInitiatePayment(
    orderId: string,
    req: InitiatePaymentRequest
): Promise<PaymentIntentResponse> {
    const { data } = await instance.post<PaymentIntentResponse>(
        `/api/orders/${orderId}/pay`,
        req
    );
    return data;
}

export async function apiListPayments(orderId: string): Promise<PaymentResponse[]> {
    const { data } = await instance.get<PaymentResponse[]>(
        `/api/orders/${orderId}/payments`
    );
    return data;
}

export async function apiCreatePaymentMethodSetupIntent(): Promise<SetupIntentClientSecretResponse> {
    const { data } = await instance.post<SetupIntentClientSecretResponse>(
        '/api/payment-methods/setup-intent'
    );
    return data;
}

export async function apiListPaymentMethods(signal?: AbortSignal): Promise<SavedPaymentMethod[]> {
    const { data } = await instance.get<SavedPaymentMethod[]>('/api/payment-methods', {signal});
    return data;
}

export async function apiSavePaymentMethod(
    req: SavePaymentMethodRequest
): Promise<SavedPaymentMethod> {
    const { data } = await instance.post<SavedPaymentMethod>('/api/payment-methods', req);
    return data;
}

export async function apiDeletePaymentMethod(paymentMethodId: string): Promise<void> {
    await instance.delete(`/api/payment-methods/${paymentMethodId}`);
}

export const PAYMENT_STATUS_LABELS: Record<PaymentStatus, string> = {
    pending: 'Pendiente',
    held: 'Retenido',
    released: 'Liberado',
    refunded: 'Reembolsado',
    failed: 'Fallido',
};

export const PAYMENT_STATUS_CLASS: Record<PaymentStatus, string> = {
    pending: 'pagoEstado--pendiente',
    held: 'pagoEstado--retenido',
    released: 'pagoEstado--liberado',
    refunded: 'pagoEstado--reembolsado',
    failed: 'pagoEstado--fallido',
};
