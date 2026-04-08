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

/* [074A-65] Actualizar suscripción (plan, dominio) — admin o dueño de la suscripción
 * [084A-4] Ya no es solo admin — clientes pueden editar sus propias suscripciones */
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

/* [084A-4] Solicitar cancelación — cliente o admin.
 * Cambia status a 'cancelled' y registra evento de auditoría. */
export async function apiRequestCancelHosting(id: string): Promise<void> {
    await axiosInstance.post(`/api/hosting/subscriptions/${id}/cancel`);
}

/* [084A-24] Iniciar checkout Stripe para suscripción de hosting.
 * Retorna la URL de Stripe Checkout a la que redirigir al cliente. */
export async function apiCreateHostingCheckout(id: string): Promise<string> {
    const { data } = await axiosInstance.post<{ checkout_url: string }>(
        `/api/hosting/subscriptions/${id}/checkout`,
    );
    return data.checkout_url;
}

export async function apiListHostingEvents(id: string): Promise<HostingEvent[]> {
    const {data} = await axiosInstance.get<HostingEvent[]>(`/api/hosting/subscriptions/${id}/events`);
    return data;
}

/* [094A-3] Self-service: cliente selecciona plan y recibe URL de Stripe Checkout */
export interface SelfSubscribeRequest {
    plan: string;
    domain?: string;
}

export interface SelfSubscribeResponse {
    subscription: HostingSubscription;
    checkout_url: string;
}

export async function apiSelfSubscribe(req: SelfSubscribeRequest): Promise<SelfSubscribeResponse> {
    const {data} = await axiosInstance.post<SelfSubscribeResponse>('/api/hosting/subscribe', req);
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

/* [094A-3] Info de planes para el selector de contratación self-service */
export interface HostingPlanInfo {
    id: string;
    label: string;
    priceCents: number;
    storageMb: number;
    features: string[];
}

export const HOSTING_PLANS: HostingPlanInfo[] = [
    {
        id: 'basico',
        label: 'Básico',
        priceCents: 500,
        storageMb: 5120,
        features: ['5 GB almacenamiento', 'SSL gratuito', '1 dominio', 'Soporte por email'],
    },
    {
        id: 'pro',
        label: 'Profesional',
        priceCents: 1000,
        storageMb: 20480,
        features: ['20 GB almacenamiento', 'SSL gratuito', '3 dominios', 'Soporte prioritario', 'Backups diarios'],
    },
    {
        id: 'ecommerce',
        label: 'E-commerce',
        priceCents: 1500,
        storageMb: 51200,
        features: ['50 GB almacenamiento', 'SSL gratuito', '5 dominios', 'Soporte 24/7', 'Backups diarios', 'CDN incluido'],
    },
];

/* [084A-24] VPS stats — proxy a Contabo API (admin only) */

/* [094A-8] Stats reales de una suscripción de hosting */
export interface HostingStatsData {
    storage_limit_mb: number;
    storage_used_mb: number | null;
    bandwidth_limit_gb: number;
    bandwidth_used_gb: number | null;
    uptime_percent: number;
    active_since: string | null;
    total_events: number;
    last_event_at: string | null;
    monitoring_available: boolean;
}

export async function apiGetHostingStats(id: string): Promise<HostingStatsData> {
    const {data} = await axiosInstance.get<HostingStatsData>(`/api/hosting/subscriptions/${id}/stats`);
    return data;
}

export interface VpsSummary {
    instance_id: number;
    name: string;
    ip: string;
    status: string;
    region: string;
    cpu_cores: number;
    ram_mb: number;
    disk_mb: number;
}

export async function apiListVps(): Promise<VpsSummary[]> {
    const { data } = await axiosInstance.get<{ data: VpsSummary[] }>('/api/hosting/vps');
    return data.data;
}

export async function apiGetVps(instanceId: number): Promise<VpsSummary> {
    const { data } = await axiosInstance.get<{ data: VpsSummary }>(`/api/hosting/vps/${instanceId}`);
    return data.data;
}
