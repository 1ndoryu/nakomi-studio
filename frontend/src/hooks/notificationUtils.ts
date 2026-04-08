/* [084A-22] Utilidades de notificaciones del navegador.
 * Extraídas de useNotifications.ts para SRP — funciones puras sin hooks. */

/* [084A-26] Genera un bip corto con Web Audio API. No requiere archivo externo. */
export function playNotificationSound() {
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
export function requestNotificationPermission() {
  if (typeof Notification !== 'undefined' && Notification.permission === 'default') {
    Notification.requestPermission();
  }
}

/* [084A-26] Muestra notificación del navegador si la ventana no está en foco. */
export function showBrowserNotification(title: string, body?: string | null) {
  if (typeof Notification === 'undefined' || Notification.permission !== 'granted') return;
  if (document.hasFocus()) return;

  new Notification(title, {
    body: body ?? undefined,
    icon: '/favicon.ico',
  });
}
