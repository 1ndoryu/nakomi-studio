/* [044A-38 Fase 9] Hook de notificaciones REST: lista, conteo, marcar leídas.
 * [084A-22] WebSocket extraído a useNotificationWs.ts, utils a notificationUtils.ts. */

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import {
  apiGetUnreadCount,
  apiListNotifications,
  apiMarkAllRead,
  apiMarkRead,
  type NotificationResponse,
} from '../api/notifications';
import { useAuthStore } from '../stores/authStore';

const NOTIF_KEY = ['notifications'] as const;
const UNREAD_KEY = ['notifications', 'unread'] as const;

/* Hook principal de notificaciones */
export function useNotifications() {
  const queryClient = useQueryClient();
  const token = useAuthStore((s) => s.token);

  /* Lista paginada de notificaciones */
  const {
    data: notifications = [],
    isLoading,
    error,
  } = useQuery<NotificationResponse[]>({
    queryKey: NOTIF_KEY,
    queryFn: () => apiListNotifications(30, 0),
    enabled: !!token,
    refetchInterval: 60_000,
  });

  /* Conteo de no leídas */
  const { data: unreadData } = useQuery({
    queryKey: UNREAD_KEY,
    queryFn: apiGetUnreadCount,
    enabled: !!token,
    refetchInterval: 30_000,
  });

  const unreadCount = unreadData?.count ?? 0;

  /* Marcar como leídas */
  const markReadMut = useMutation({
    mutationFn: (ids: string[]) => apiMarkRead(ids),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: NOTIF_KEY });
      queryClient.invalidateQueries({ queryKey: UNREAD_KEY });
    },
  });

  /* Marcar todas como leídas */
  const markAllReadMut = useMutation({
    mutationFn: apiMarkAllRead,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: NOTIF_KEY });
      queryClient.invalidateQueries({ queryKey: UNREAD_KEY });
    },
  });

  return {
    notifications,
    unreadCount,
    isLoading,
    error,
    marcarLeidas: markReadMut.mutate,
    marcarTodasLeidas: markAllReadMut.mutate,
  };
}
