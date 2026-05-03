/* [074A-9] API client para CRUD de servicios desde panel admin.
 * Conecta con endpoints /api/admin/services del backend. */
import instance from './axios-instance';

export interface AdminServicePlan {
    id: string;
    slug: string;
    name: string;
    price_cents: number;
    description: string | null;
    features: unknown;
    is_highlighted: boolean;
    is_custom: boolean;
    phases: AdminPlanPhase[];
}

export interface AdminPlanPhase {
    phase_number: number;
    title: string;
    description: string | null;
    percentage_of_total: number;
    estimated_days: number;
    max_revisions: number;
}

export interface AdminService {
    id: string;
    slug: string;
    title: string;
    description: string | null;
    base_price_cents: number;
    currency: string;
    is_active: boolean;
    sort_order: number;
    image_url: string | null;
    gallery: string[];
    skills: string[];
    content: string | null;
    meta_title: string | null;
    meta_description: string | null;
    status: string;
    created_at: string;
    updated_at: string;
    plans: AdminServicePlan[];
}

export interface CreateServiceBody {
    title: string;
    slug: string;
    description?: string;
    base_price_cents?: number;
    currency?: string;
    image_url?: string;
    gallery?: string[];
    skills?: string[];
    content?: string;
    meta_title?: string;
    meta_description?: string;
    status?: string;
    sort_order?: number;
}

export interface UpdateServiceBody {
    title?: string;
    slug?: string;
    description?: string;
    base_price_cents?: number;
    currency?: string;
    is_active?: boolean;
    image_url?: string;
    gallery?: string[];
    skills?: string[];
    content?: string;
    meta_title?: string;
    meta_description?: string;
    status?: string;
    sort_order?: number;
}

export async function apiListAdminServices(): Promise<AdminService[]> {
    const {data} = await instance.get<AdminService[]>('/api/admin/services');
    return data;
}

export async function apiCreateService(body: CreateServiceBody): Promise<AdminService> {
    const {data} = await instance.post<AdminService>('/api/admin/services', body);
    return data;
}

export async function apiUpdateService(id: string, body: UpdateServiceBody): Promise<AdminService> {
    const {data} = await instance.put<AdminService>(`/api/admin/services/${id}`, body);
    return data;
}

export async function apiArchiveService(id: string): Promise<void> {
    await instance.delete(`/api/admin/services/${id}`);
}

/* [084A-10] Eliminación permanente de un servicio (409 si tiene órdenes) */
export async function apiDestroyService(id: string): Promise<void> {
    await instance.post(`/api/admin/services/${id}/destroy`);
}

/* [074A-66] Guardar (reemplazar) planes de un servicio */
export interface SavePlanBody {
    id?: string;
    slug: string;
    name: string;
    price_cents: number;
    description: string | null;
    features: string[];
    is_highlighted: boolean;
    is_custom: boolean;
    sort_order: number;
    phases: SavePhaseBody[];
}

export interface SavePhaseBody {
    phase_number: number;
    title: string;
    description: string | null;
    percentage_of_total: number;
    estimated_days: number;
    max_revisions: number;
}

export async function apiSaveServicePlans(serviceId: string, plans: SavePlanBody[]): Promise<AdminServicePlan[]> {
    const {data} = await instance.put<AdminServicePlan[]>(
        `/api/admin/services/${serviceId}/plans`,
        {plans},
    );
    return data;
}

/* [124A-CMS10] Reordenar servicios en batch */
export async function apiReorderServices(items: {id: string; sort_order: number}[]): Promise<void> {
    await instance.put('/api/admin/services/reorder', {items});
}

/* [074A-21] Endpoints públicos de servicios (sin auth)
 * [084A-6] Añadidos content, gallery, meta_title, meta_description
 * para que el frontend público reciba todo lo que el admin edita. */
export interface PublicServicePlan {
    id: string;
    slug: string;
    name: string;
    price_cents: number;
    description: string | null;
    features: unknown;
    is_highlighted: boolean;
    is_custom: boolean;
    phases: AdminPlanPhase[];
}

export interface PublicService {
    id: string;
    slug: string;
    title: string;
    description: string | null;
    image_url: string | null;
    base_price_cents: number;
    skills: unknown[];
    content: string | null;
    gallery: unknown;
    meta_title: string | null;
    meta_description: string | null;
    plans: PublicServicePlan[];
}

export async function apiListPublicServices(): Promise<PublicService[]> {
    const {data} = await instance.get<PublicService[]>('/api/services');
    return data;
}

export async function apiGetServiceBySlug(slug: string): Promise<PublicService> {
    const {data} = await instance.get<PublicService>(`/api/services/${slug}`);
    return data;
}
