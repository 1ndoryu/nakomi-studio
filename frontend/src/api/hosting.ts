/* [054A-2] API client de hosting: suscripciones y eventos.
 * Endpoints bajo /api/hosting/. Requiere JWT. */

import axiosInstance from './axios-instance';

/*    TIPOS */

export interface HostingSubscription {
    id: string;
    user_id: string | null;
    client_name: string;
    client_email: string;
    plan: string;
    domain: string | null;
    coolify_site_name: string | null;
    status: string;
    monthly_price_cents: number;
    storage_limit_mb: number;
    created_at: string;
    updated_at: string;
}

export interface HostingEvent {
    id: string;
    subscription_id: string;
    event_type: string;
    details: Record<string, unknown> | null;
    created_at: string;
}

export interface CreateHostingRequest {
    client_name: string;
    client_email: string;
    plan: string;
    domain?: string;
}

/*    REST API */

export async function apiListHostingSubscriptions(): Promise<HostingSubscription[]> {
    const {data} = await axiosInstance.get<HostingSubscription[]>('/api/hosting/subscriptions');
    return data;
}

export async function apiGetHostingSubscription(id: string): Promise<HostingSubscription> {
    const {data} = await axiosInstance.get<HostingSubscription>(`/api/hosting/subscriptions/${id}`);
    return data;
}

export async function apiCreateHostingSubscription(
    req: CreateHostingRequest,
): Promise<HostingSubscription> {
    const {data} = await axiosInstance.post<HostingSubscription>('/api/hosting/subscriptions', req);
    return data;
}

export async function apiUpdateHostingStatus(
    id: string,
    status: string,
    reason?: string,
): Promise<void> {
    await axiosInstance.patch(`/api/hosting/subscriptions/${id}/status`, {status, reason});
}

/* [074A-65] Actualizar suscripción (plan, dominio) — solo admin */
export interface UpdateHostingRequest {
    plan: string;
    domain?: string;
}

export async function apiUpdateHostingSubscription(
    id: string,
    req: UpdateHostingRequest,
): Promise<HostingSubscription> {
    const {data} = await axiosInstance.put<HostingSubscription>(`/api/hosting/subscriptions/${id}`, req);
    return data;
}

/* [074A-65] Eliminar suscripción — solo admin */
export async function apiDeleteHostingSubscription(id: string): Promise<void> {
    await axiosInstance.delete(`/api/hosting/subscriptions/${id}`);
}

export async function apiListHostingEvents(id: string): Promise<HostingEvent[]> {
    const {data} = await axiosInstance.get<HostingEvent[]>(`/api/hosting/subscriptions/${id}/events`);
    return data;
}

/*    CONSTANTES */

export const HOSTING_PLAN_LABELS: Record<string, string> = {
    basico: 'Básico',
    pro: 'Profesional',
    ecommerce: 'E-commerce',
    custom: 'Custom',
};

export const HOSTING_STATUS_LABELS: Record<string, string> = {
    pending: 'Pendiente',
    provisioning: 'Provisionando',
    active: 'Activo',
    suspended: 'Suspendido',
    cancelled: 'Cancelado',
};

export const HOSTING_STATUS_COLORS: Record<string, string> = {
    pending: '#f59e0b',
    provisioning: '#3b82f6',
    active: '#22c55e',
    suspended: '#ef4444',
    cancelled: '#6b7280',
};

export const HOSTING_STATUS_CLASS: Record<string, string> = {
    pending: 'hostingStatus--pending',
    provisioning: 'hostingStatus--provisioning',
    active: 'hostingStatus--active',
    suspended: 'hostingStatus--suspended',
    cancelled: 'hostingStatus--cancelled',
};
