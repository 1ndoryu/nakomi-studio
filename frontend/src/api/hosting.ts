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
    /* [104A-42] Datos reales del servidor Coolify */
    server_uuid: string | null;
    server_ip: string | null;
    /* [104A-18] Credenciales SFTP generadas al provisionar */
    sftp_user: string | null;
    sftp_password: string | null;
    sftp_port: number | null;
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

/* [154A-11] Provisionar hosting: crea servicio Nginx real en Coolify (admin only). */
export async function apiProvisionHosting(id: string): Promise<HostingSubscription> {
    const {data} = await axiosInstance.post<HostingSubscription>(
        `/api/hosting/subscriptions/${id}/provision`,
    );
    return data;
}

/* [154A-16] Verificación DNS: comprueba si el dominio apunta al servidor correcto. */
export interface DnsCheckResult {
    configured: boolean;
    domain?: string;
    resolved?: boolean;
    resolved_ips?: string[];
    expected_ip?: string;
    points_to_server?: boolean;
    ssl_provider?: string;
    error?: string;
    message?: string;
}

export async function apiDnsCheck(id: string): Promise<DnsCheckResult> {
    const {data} = await axiosInstance.get<DnsCheckResult>(
        `/api/hosting/subscriptions/${id}/dns-check`,
    );
    return data;
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

/* [094A-3] Info de planes para el selector de contratación self-service
 * [114A-5] Especialización WordPress: features WP-CLI, WordPress pre-instalado */
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
        features: ['WordPress pre-instalado', '5 GB almacenamiento', 'SSL gratuito', '1 dominio', 'WP-CLI vía SSH'],
    },
    {
        id: 'pro',
        label: 'Profesional',
        priceCents: 1000,
        storageMb: 20480,
        features: ['WordPress pre-instalado', '20 GB almacenamiento', 'SSL gratuito', '3 dominios', 'WP-CLI vía SSH', 'Backups diarios'],
    },
    {
        id: 'ecommerce',
        label: 'E-commerce',
        priceCents: 1500,
        storageMb: 51200,
        features: ['WordPress + WooCommerce', '50 GB almacenamiento', 'SSL gratuito', '5 dominios', 'WP-CLI vía SSH', 'Backups diarios', 'Caché avanzada'],
    },
];

/* [154A-9] Control de servicio: restart / stop / start */
export async function apiRestartHosting(id: string): Promise<void> {
    await axiosInstance.post(`/api/hosting/subscriptions/${id}/restart`);
}

export async function apiStopHosting(id: string): Promise<void> {
    await axiosInstance.post(`/api/hosting/subscriptions/${id}/stop`);
}

export async function apiStartHosting(id: string): Promise<void> {
    await axiosInstance.post(`/api/hosting/subscriptions/${id}/start`);
}

/* [154A-14] Admin test subscribe: crea hosting sin Stripe */
export async function apiAdminTestSubscribe(req: SelfSubscribeRequest): Promise<{subscription: HostingSubscription; message: string}> {
    const {data} = await axiosInstance.post<{subscription: HostingSubscription; message: string}>('/api/hosting/admin-test-subscribe', req);
    return data;
}

/* [084A-24] VPS stats — proxy a Contabo API (admin only) */

/* [094A-8] Stats reales de una suscripción de hosting */
/* [114A-15+] Añadidos campos de CPU/RAM y stats por contenedor */
export interface ContainerStatsData {
    name: string;
    cpu_percent: number;
    mem_used_mb: number;
    mem_limit_mb: number;
    net_input_mb: number;
    net_output_mb: number;
}

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
    cpu_percent: number | null;
    ram_used_mb: number | null;
    ram_limit_mb: number | null;
    containers: ContainerStatsData[] | null;
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

/* [154A-1] Contabo Domain Management API */

export interface ContaboDomain {
    sld: string | null;
    tld: string | null;
    status: string | null;
    handles: DomainHandles | null;
    nameservers: DomainNameserver[] | null;
    createdDate: string | null;
    paidUntil: string | null;
}

export interface DomainHandles {
    owner: string | null;
    admin: string | null;
    tech: string | null;
    zone: string | null;
}

export interface DomainNameserver {
    hostname: string | null;
    ipv4: string | null;
    ipv6: string | null;
}

export interface DomainAvailability {
    domain: string;
    available: boolean;
}

export interface DnsZone {
    zoneName: string | null;
}

export interface DnsRecord {
    recordId: number | null;
    name: string | null;
    type: string | null;
    ttl: number | null;
    prio: number | null;
    data: string | null;
    port: number | null;
    weight: number | null;
    flag: number | null;
    tag: string | null;
}

export interface ContaboHandle {
    handleId: string | null;
    handleType: string | null;
    firstName: string | null;
    lastName: string | null;
    organization: string | null;
    email: string | null;
    gender: string | null;
    address: HandleAddress | null;
    phone: HandlePhone | null;
}

export interface HandleAddress {
    street: string | null;
    streetNumber: string | null;
    city: string | null;
    country: string | null;
    zipCode: string | null;
}

export interface HandlePhone {
    prefix: string | null;
    number: string | null;
}

/* Domains */

export async function apiCheckDomainAvailability(domain: string): Promise<DomainAvailability> {
    const { data } = await axiosInstance.get<DomainAvailability>(
        `/api/hosting/domains/check/${encodeURIComponent(domain)}`,
    );
    return data;
}

export async function apiListDomains(): Promise<ContaboDomain[]> {
    const { data } = await axiosInstance.get<ContaboDomain[]>('/api/hosting/domains');
    return data;
}

export async function apiGetDomain(domain: string): Promise<ContaboDomain> {
    const { data } = await axiosInstance.get<ContaboDomain>(
        `/api/hosting/domains/${encodeURIComponent(domain)}`,
    );
    return data;
}

export interface OrderDomainRequest {
    domain: string;
    auth_code?: string;
    handles: DomainHandles;
    nameservers: DomainNameserver[];
}

export async function apiOrderDomain(req: OrderDomainRequest): Promise<ContaboDomain> {
    const { data } = await axiosInstance.post<ContaboDomain>('/api/hosting/domains', req);
    return data;
}

export async function apiUpdateDomain(
    domain: string,
    body: { nameservers?: DomainNameserver[]; handles?: DomainHandles },
): Promise<ContaboDomain> {
    const { data } = await axiosInstance.patch<ContaboDomain>(
        `/api/hosting/domains/${encodeURIComponent(domain)}`,
        body,
    );
    return data;
}

export async function apiCancelDomain(domain: string, reason?: string): Promise<void> {
    await axiosInstance.post(`/api/hosting/domains/${encodeURIComponent(domain)}/cancel`, { reason });
}

export async function apiGetDomainAuthCode(domain: string): Promise<{ domain: string; auth_code: string }> {
    const { data } = await axiosInstance.post<{ domain: string; auth_code: string }>(
        `/api/hosting/domains/${encodeURIComponent(domain)}/auth-code`,
    );
    return data;
}

/* Handles */

export async function apiListHandles(): Promise<ContaboHandle[]> {
    const { data } = await axiosInstance.get<ContaboHandle[]>('/api/hosting/handles');
    return data;
}

export async function apiCreateHandle(handle: Omit<ContaboHandle, 'handleId'>): Promise<ContaboHandle> {
    const { data } = await axiosInstance.post<ContaboHandle>('/api/hosting/handles', handle);
    return data;
}

/* DNS Zones */

export async function apiListDnsZones(): Promise<DnsZone[]> {
    const { data } = await axiosInstance.get<DnsZone[]>('/api/hosting/dns/zones');
    return data;
}

export async function apiCreateDnsZone(zoneName: string): Promise<DnsZone> {
    const { data } = await axiosInstance.post<DnsZone>('/api/hosting/dns/zones', { zone_name: zoneName });
    return data;
}

export async function apiDeleteDnsZone(zoneName: string): Promise<void> {
    await axiosInstance.delete(`/api/hosting/dns/zones/${encodeURIComponent(zoneName)}`);
}

/* DNS Records */

export async function apiListDnsRecords(zoneName: string): Promise<DnsRecord[]> {
    const { data } = await axiosInstance.get<DnsRecord[]>(
        `/api/hosting/dns/zones/${encodeURIComponent(zoneName)}/records`,
    );
    return data;
}

export interface CreateDnsRecordRequest {
    name?: string;
    type: string;
    ttl: number;
    prio: number;
    data: string;
    port?: number;
    weight?: number;
    flag?: number;
    tag?: string;
}

export async function apiCreateDnsRecord(zoneName: string, record: CreateDnsRecordRequest): Promise<DnsRecord> {
    const { data } = await axiosInstance.post<DnsRecord>(
        `/api/hosting/dns/zones/${encodeURIComponent(zoneName)}/records`,
        record,
    );
    return data;
}

export async function apiUpdateDnsRecord(
    zoneName: string,
    recordId: number,
    record: Omit<CreateDnsRecordRequest, 'name'>,
): Promise<DnsRecord> {
    const { data } = await axiosInstance.patch<DnsRecord>(
        `/api/hosting/dns/zones/${encodeURIComponent(zoneName)}/records/${recordId}`,
        record,
    );
    return data;
}

export async function apiDeleteDnsRecord(zoneName: string, recordId: number): Promise<void> {
    await axiosInstance.delete(
        `/api/hosting/dns/zones/${encodeURIComponent(zoneName)}/records/${recordId}`,
    );
}

/* ── Client DNS (por suscripción) ─────── */

export async function apiClientListDnsRecords(subId: string): Promise<DnsRecord[]> {
    const {data} = await axiosInstance.get<DnsRecord[]>(
        `/api/hosting/subscriptions/${subId}/dns`,
    );
    return data;
}

export async function apiClientCreateDnsRecord(
    subId: string,
    req: CreateDnsRecordRequest,
): Promise<DnsRecord> {
    const {data} = await axiosInstance.post<DnsRecord>(
        `/api/hosting/subscriptions/${subId}/dns`,
        req,
    );
    return data;
}

export async function apiClientUpdateDnsRecord(
    subId: string,
    recordId: number,
    req: {type: string; ttl: number; prio: number; data: string},
): Promise<DnsRecord> {
    const {data} = await axiosInstance.patch<DnsRecord>(
        `/api/hosting/subscriptions/${subId}/dns/${recordId}`,
        req,
    );
    return data;
}

export async function apiClientDeleteDnsRecord(subId: string, recordId: number): Promise<void> {
    await axiosInstance.delete(`/api/hosting/subscriptions/${subId}/dns/${recordId}`);
}
