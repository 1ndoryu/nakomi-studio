/* [124A-SOUND] Sonido de notificación de mensajes.
 * Usa Web Audio API para generar un bip suave sin depender de archivos de audio.
 * Gotcha: AudioContext requiere interacción del usuario previa — silenciar si no está disponible.
 * No usar en SSR (verificar window). */

export function playNotificationSound(): void {
    try {
        const AudioCtx = window.AudioContext || (window as unknown as {webkitAudioContext: typeof AudioContext}).webkitAudioContext;
        if (!AudioCtx) return;

        const ctx = new AudioCtx();
        const oscillator = ctx.createOscillator();
        const gain = ctx.createGain();

        oscillator.connect(gain);
        gain.connect(ctx.destination);

        oscillator.type = 'sine';
        oscillator.frequency.setValueAtTime(880, ctx.currentTime);
        oscillator.frequency.exponentialRampToValueAtTime(660, ctx.currentTime + 0.15);

        gain.gain.setValueAtTime(0.25, ctx.currentTime);
        gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.35);

        oscillator.start(ctx.currentTime);
        oscillator.stop(ctx.currentTime + 0.35);

        /* Limpiar el contexto al terminar */
        oscillator.onended = () => void ctx.close();
    } catch {
        /* Silenciar si el browser bloquea AudioContext (política autoplay) */
    }
}
