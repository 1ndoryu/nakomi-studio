import axiosInstance from './axios-instance';

export type BillingItemStatus = 'pending' | 'paid' | 'cancelled';
export type BillingItemKind = 'hosting' | 'domain' | 'other';
export type BillingPeriod = 'month' | 'year' | 'one_time';
export type BillingCheckoutMode = 'subscription' | 'prepay_year';

export interface BillingItem {
    id: string;
    user_id: string;
    resource_type: BillingItemKind;
    resource_id: string | null;
    title: string;
    description: string | null;
    amount_cents: number;
    currency: string;
    billing_period: BillingPeriod;
    status: BillingItemStatus;
    due_at: string;
    grace_period_ends_at: string;
    paid_at: string | null;
    stripe_session_id: string | null;
    metadata: Record<string, unknown> | null;
    created_at: string;
    updated_at: string;
}

export interface CreateBillingCheckoutRequest {
    item_ids?: string[];
    mode: BillingCheckoutMode;
}

export interface BillingCheckoutResponse {
    checkout_url: string;
}

export async function apiListBillingItems(): Promise<BillingItem[]> {
    const {data} = await axiosInstance.get<BillingItem[]>('/api/billing/items');
    return data;
}

export async function apiCreateBillingCheckout(
    req: CreateBillingCheckoutRequest,
): Promise<string> {
    const {data} = await axiosInstance.post<BillingCheckoutResponse>('/api/billing/checkout', req);
    return data.checkout_url;
}

export function billingItemPeriodLabel(period: BillingPeriod): string {
    switch (period) {
    case 'month':
        return '/mes';
    case 'year':
        return '/año';
    default:
        return '';
    }
}