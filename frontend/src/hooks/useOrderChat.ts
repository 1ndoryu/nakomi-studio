/* [064A-31] Hook para chat dentro de un pedido.
 * Crea/recupera sesión de chat vinculada a la orden y gestiona mensajes
 * via REST polling (5s). El empleado asignado recibe los mensajes en
 * SeccionChat via el broadcast WS existente del backend. */

import {useState, useCallback, useRef} from 'react';
import {useQuery, useMutation, useQueryClient} from '@tanstack/react-query';
import {
    apiCreateChatSession,
    apiGetMessages,
    apiSendMessage,
    type ChatSession,
} from '../api/chat';

export function useOrderChat(orderId: string) {
    const queryClient = useQueryClient();
    const [sessionId, setSessionId] = useState<string | null>(null);
    const [session, setSession] = useState<ChatSession | null>(null);
    const [creando, setCreando] = useState(false);
    const inicializado = useRef(false);

    /* Crear o recuperar sesión al montar */
    const iniciarSesion = useCallback(async () => {
        if (inicializado.current || creando) return;
        inicializado.current = true;
        setCreando(true);
        try {
            const s = await apiCreateChatSession(orderId);
            setSessionId(s.id);
            setSession(s);
        } catch (err) {
            console.error('Error creando sesión de chat de orden:', err);
            inicializado.current = false;
        } finally {
            setCreando(false);
        }
    }, [orderId, creando]);

    /* Polling de mensajes cada 5s cuando hay sesión activa */
    const {data: mensajes = []} = useQuery({
        queryKey: ['order-chat-messages', sessionId],
        queryFn: () => apiGetMessages(sessionId!, 100, 0),
        enabled: !!sessionId,
        refetchInterval: 5_000,
    });

    /* Enviar mensaje */
    const {mutateAsync: enviarMensajeAsync, isPending: enviando} = useMutation({
        mutationFn: (content: string) => apiSendMessage(sessionId!, content),
        onSuccess: () => {
            queryClient.invalidateQueries({queryKey: ['order-chat-messages', sessionId]});
        },
    });

    const enviarMensaje = useCallback(
        async (content: string) => {
            if (!sessionId || !content.trim()) return;
            await enviarMensajeAsync(content.trim());
        },
        [sessionId, enviarMensajeAsync],
    );

    return {
        sessionId,
        session,
        mensajes,
        enviando,
        creando,
        iniciarSesion,
        enviarMensaje,
    };
}
