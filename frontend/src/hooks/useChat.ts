/* [044A-38 Fase 5] Hook de chat REST: sesiones, mensajes, envío.
 * [084A-22] WebSocket extraído a useChatWs.ts (SRP). */

import {useQuery, useMutation, useQueryClient} from '@tanstack/react-query';
import {
    apiListChatSessions,
    apiGetMessages,
    apiCreateChatSession,
    apiSendMessage,
} from '../api/chat';

/*    HOOK: Sesiones y mensajes REST */

export function useChat(sessionId?: string, limit = 100) {
    const queryClient = useQueryClient();

    const {data: sessions = [], isLoading: cargandoSesiones} = useQuery({
        queryKey: ['chat-sessions'],
        queryFn: apiListChatSessions,
        refetchInterval: 15_000,
    });

    /* [074A-43] limit parametrizable para carga diferida */
    const {data: messages = [], isLoading: cargandoMensajes} = useQuery({
        queryKey: ['chat-messages', sessionId, limit],
        queryFn: () => apiGetMessages(sessionId!, limit, 0),
        enabled: !!sessionId,
        refetchInterval: 5_000,
    });

    const {mutateAsync: crearSesion} = useMutation({
        mutationFn: (orderId?: string) => apiCreateChatSession(orderId),
        onSuccess: () => {
            queryClient.invalidateQueries({queryKey: ['chat-sessions']});
        },
    });

    const {mutateAsync: enviarMensaje, isPending: enviando} = useMutation({
        mutationFn: ({sId, content}: {sId: string; content: string}) =>
            apiSendMessage(sId, content),
        onSuccess: () => {
            queryClient.invalidateQueries({queryKey: ['chat-messages', sessionId]});
        },
    });

    return {
        sessions,
        messages,
        cargandoSesiones,
        cargandoMensajes,
        crearSesion,
        enviarMensaje,
        enviando,
    };
}