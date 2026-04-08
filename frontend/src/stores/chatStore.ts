/* [064A-5] Store global para controlar la visibilidad del ChatWidget.
 * Permite abrir el chat desde cualquier componente (CTAs, header, etc.)
 * sin prop drilling ni custom events.
 * [084A-28] Campo context para soporte contextual (hosting, servicio, etc.) */
import { create } from 'zustand';

interface ChatState {
    abierto: boolean;
    context: string | null;
    abrir: (context?: string) => void;
    cerrar: () => void;
}

export const useChatStore = create<ChatState>((set) => ({
    abierto: false,
    context: null,
    abrir: (context?: string) => set({ abierto: true, context: context ?? null }),
    cerrar: () => set({ abierto: false, context: null }),
}));
