/* [311A-1] API client para consultar trazabilidad de correos enviados (admin).
 * Endpoint: GET /api/admin/email-logs?template=&limit=&offset= */

import axiosInstance from './axios-instance';

export interface EmailLogItem {
    id: string;
    to_email: string;
    subject: string;
    template: string;
    template_label: string;
    reference_type: string | null;
    reference_id: string | null;
    status: string;
    status_color: string;
    sent_at: string;
    created_at: string;
}

export interface EmailLogsResponse {
    logs: EmailLogItem[];
    total: number;
}

/* [311A-1] Mapa de template → etiqueta amigable para el Select de filtro.
 * Refleja EmailLogRow::template_label() del backend. */
export const TEMPLATE_LABELS: Record<string, string> = {
    order_confirmation: 'Confirmación al cliente',
    new_order_admin: 'Nueva orden (admin)',
    payment_received_admin: 'Pago recibido (admin)',
    order_completed_client: 'Orden completada (cliente)',
    order_cancelled_client: 'Orden cancelada (cliente)',
    phase_delivered_client: 'Fase entregada (cliente)',
    problem_reported_client: 'Problema reportado (cliente)',
    escalation: 'Escalación de chat',
    chat_invoice_paid_client: 'Factura chat pagada (cliente)',
    chat_invoice_paid_admin: 'Factura chat pagada (admin)',
    vps_pending_approval: 'VPS pendiente (admin)',
    vps_approved: 'VPS aprobado (cliente)',
    vps_rejected: 'VPS rechazado (cliente)',
    profile_email_changed_new: 'Email cambiado (nuevo)',
    profile_email_changed_old: 'Email cambiado (anterior)',
    profile_password_changed: 'Contraseña cambiada',
};

export const TEMPLATE_OPTIONS = Object.entries(TEMPLATE_LABELS).map(([value, label]) => ({
    value,
    label,
}));

export async function apiListEmailLogs(params?: {
    template?: string;
    limit?: number;
    offset?: number;
}): Promise<EmailLogsResponse> {
    const query = new URLSearchParams();
    if (params?.template) query.set('template', params.template);
    if (params?.limit) query.set('limit', String(params.limit));
    if (params?.offset) query.set('offset', String(params.offset));

    const url = `/api/admin/email-logs${query.toString() ? `?${query.toString()}` : ''}`;
    const { data } = await axiosInstance.get<EmailLogsResponse>(url);
    return data;
}
