/* [044A-38 Fase 9] Hook de notificaciones: REST polling + WebSocket real-time.
 * Combina React Query para lista/conteo con WS para push instantáneo.
 * [084A-26] Sonido + notificación del navegador al recibir push. */

import { useCallback, useEffect, useRef } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import {
  apiGetUnreadCount,
  apiListNotifications,
  apiMarkAllRead,
  apiMarkRead,
  buildNotificationsWsUrl,
  type NotificationResponse,
} from '../api/notifications';
import { useAuthStore } from '../stores/authStore';

const NOTIF_KEY = ['notifications'] as const;
const UNREAD_KEY = ['notifications', 'unread'] as const;

/* [084A-26] Genera un bip corto con Web Audio API. No requiere archivo externo. */
function playNotificationSound() {
  try {
    const ctx = new AudioContext();
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();

    osc.type = 'sine';
    osc.frequency.setValueAtTime(880, ctx.currentTime);
    osc.frequency.setValueAtTime(660, ctx.currentTime + 0.1);
    gain.gain.setValueAtTime(0.3, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.3);

    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.start(ctx.currentTime);
    osc.stop(ctx.currentTime + 0.3);

    osc.onended = () => ctx.close();
  } catch {
    /* AudioContext no disponible (SSR, permisos, etc.) */
  }
}

/* [084A-26] Solicita permiso para notificaciones del navegador (una sola vez). */
function requestNotificationPermission() {
  if (typeof Notification !== 'undefined' && Notification.permission === 'default') {
    Notification.requestPermission();
  }
}

/* [084A-26] Muestra notificación del navegador si la ventana no está en foco. */
function showBrowserNotification(title: string, body?: string | null) {
  if (typeof Notification === 'undefined' || Notification.permission !== 'granted') return;
  if (document.hasFocus()) return;

  new Notification(title, {
    body: body ?? undefined,
    icon: '/favicon.ico',
  });
}

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

/* Hook de WebSocket para notificaciones push en tiempo real */
export function useNotificationWs() {
  const queryClient = useQueryClient();
  const token = useAuthStore((s) => s.token);
  const wsRef = useRef<WebSocket | null>(null);

  /* [084A-26] Pedir permiso para notificaciones del navegador al montar */
  useEffect(() => {
    requestNotificationPermission();
  }, []);

  const connect = useCallback(() => {
    if (!token) return;

    const url = buildNotificationsWsUrl(token);
    const ws = new WebSocket(url);

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);

        if (msg.type === 'notification') {
          queryClient.invalidateQueries({ queryKey: NOTIF_KEY });
          queryClient.invalidateQueries({ queryKey: UNREAD_KEY });

          /* [084A-26] Sonido + notificación browser al recibir push */
          playNotificationSound();
          showBrowserNotification(msg.title ?? 'Nueva notificación', msg.body);
        } else if (msg.type === 'unread_count') {
          queryClient.setQueryData(UNREAD_KEY, { count: msg.count });
        }
      } catch {
        /* Ignorar mensajes no parseables */
      }
    };

    ws.onclose = () => {
      wsRef.current = null;
      /* Reconectar tras 5s si aún hay token */
      setTimeout(() => {
        if (useAuthStore.getState().token) {
          connect();
        }
      }, 5000);
    };

    wsRef.current = ws;
  }, [token, queryClient]);

  useEffect(() => {
    connect();
    return () => {
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
    };
  }, [connect]);
}
