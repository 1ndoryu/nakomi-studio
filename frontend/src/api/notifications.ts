/* [044A-38 Fase 9] API client de notificaciones.
 * REST + WebSocket para notificaciones en tiempo real. */

import axiosInstance, {getApiHost} from './axios-instance';

/* Types */

export interface NotificationResponse {
  id: string;
  user_id: string;
  notification_type: string;
  title: string;
  body: string | null;
  link: string | null;
  read: boolean;
  reference_type: string | null;
  reference_id: string | null;
  created_at: string;
}

export interface UnreadCountResponse {
  count: number;
}

export interface MarkReadBody {
  ids: string[];
}

/* Tipos de notificación (mirrors backend) */

export const NOTIF_TYPES = {
  new_order: { label: 'Nueva orden', icon: 'ShoppingCart' },
  order_assigned: { label: 'Orden asignada', icon: 'UserCheck' },
  order_completed: { label: 'Orden completada', icon: 'CheckCircle' },
  order_cancelled: { label: 'Orden cancelada', icon: 'XCircle' },
  payment_received: { label: 'Pago recibido', icon: 'DollarSign' },
  payment_released: { label: 'Pago liberado', icon: 'CreditCard' },
  phase_delivered: { label: 'Fase entregada', icon: 'Package' },
  phase_approved: { label: 'Fase aprobada', icon: 'ThumbsUp' },
  revision_requested: { label: 'Revisión solicitada', icon: 'RotateCcw' },
  refund_requested: { label: 'Reembolso solicitado', icon: 'AlertCircle' },
  refund_resolved: { label: 'Reembolso resuelto', icon: 'CheckCircle' },
  new_review: { label: 'Nueva reseña', icon: 'Star' },
  review_response: { label: 'Respuesta a reseña', icon: 'MessageSquare' },
  delegation_received: { label: 'Delegación recibida', icon: 'ArrowRightLeft' },
  delegation_resolved: { label: 'Delegación resuelta', icon: 'ArrowRightLeft' },
  new_message: { label: 'Nuevo mensaje', icon: 'MessageCircle' },
  problem_reported: { label: 'Problema reportado', icon: 'AlertTriangle' },
  problem_resolved: { label: 'Problema atendido', icon: 'CheckCircle' },
} as const;

export type NotificationType = keyof typeof NOTIF_TYPES;

/* REST Functions */

export async function apiListNotifications(
  limit = 20,
  offset = 0
): Promise<NotificationResponse[]> {
  const { data } = await axiosInstance.get<NotificationResponse[]>(
    '/api/notifications',
    { params: { limit, offset } }
  );
  return data;
}

export async function apiGetUnreadCount(): Promise<UnreadCountResponse> {
  const { data } = await axiosInstance.get<UnreadCountResponse>(
    '/api/notifications/unread-count'
  );
  return data;
}

export async function apiMarkRead(ids: string[]): Promise<{ marked: number }> {
  const { data } = await axiosInstance.patch<{ marked: number }>(
    '/api/notifications/read',
    { ids }
  );
  return data;
}

export async function apiMarkAllRead(): Promise<{ marked: number }> {
  const { data } = await axiosInstance.patch<{ marked: number }>(
    '/api/notifications/read-all'
  );
  return data;
}

/* WebSocket URL builder */

export function buildNotificationsWsUrl(token: string): string {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const host = getApiHost();
  return `${protocol}//${host}/ws/notifications?token=${encodeURIComponent(token)}`;
}
