/* [084A-22] Hook de WebSocket para notificaciones push en tiempo real.
 * Extraído de useNotifications.ts para SRP.
 * [084A-26] Sonido + notificación del navegador al recibir push. */

import { useCallback, useEffect, useRef } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { buildNotificationsWsUrl } from '../api/notifications';
import { useAuthStore } from '../stores/authStore';
import { playNotificationSound, requestNotificationPermission, showBrowserNotification } from './notificationUtils';

const NOTIF_KEY = ['notifications'] as const;
const UNREAD_KEY = ['notifications', 'unread'] as const;

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
        /* Mensaje no JSON, ignorar */
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
