/* [026B-1] API client para gestión admin de billing_items (cobros pendientes). */

import axiosInstance from './axios-instance';

export interface AdminBillingItem {
    id: string;
    user_id: string;
    user_email: string;
    resource_type: string;
    resource_id: string | null;
    title: string;
    description: string | null;
    amount_cents: number;
    currency: string;
    billing_period: string;
    status: string;
    due_at: string;
    grace_period_ends_at: string;
    paid_at: string | null;
    created_at: string;
    updated_at: string;
}

export async function apiAdminListBillingItems(status?: string): Promise<AdminBillingItem[]> {
    const params = status ? { status } : {};
    const { data } = await axiosInstance.get<AdminBillingItem[]>('/api/admin/billing-items', { params });
    return data;
}

export async function apiAdminUpdateBillingStatus(
    itemId: string,
    status: string,
): Promise<void> {
    await axiosInstance.patch(`/api/admin/billing-items/${itemId}/status`, { status });
}
