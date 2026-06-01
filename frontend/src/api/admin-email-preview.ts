/* [311A-INV] API client para previsualizar plantillas de email.
 * Endpoints:
 *   GET /api/admin/email-templates       → lista de plantillas
 *   GET /api/admin/email-templates/:name → HTML renderizado */

import axiosInstance from './axios-instance';

export interface TemplateMeta {
    id: string;
    label: string;
    description: string;
    category: string;
    recipients: string;
}

export interface TemplatesListResponse {
    templates: TemplateMeta[];
}

/** Obtiene la lista de plantillas disponibles */
export async function apiListTemplates(): Promise<TemplatesListResponse> {
    const { data } = await axiosInstance.get<TemplatesListResponse>('/api/admin/email-templates');
    return data;
}

/** Obtiene el HTML renderizado de una plantilla con datos de muestra */
export async function apiRenderTemplate(name: string): Promise<string> {
    const { data } = await axiosInstance.get(`/api/admin/email-templates/${name}`, {
        responseType: 'text',
    });
    return data as string;
}

/** Categorías para agrupar en UI */
export const CATEGORY_LABELS: Record<string, string> = {
    orders: 'Pedidos',
    payments: 'Pagos',
    chat: 'Chat',
    vps: 'VPS',
    profile: 'Perfil',
};

export const CATEGORY_ORDER = ['orders', 'payments', 'chat', 'vps', 'profile'];
