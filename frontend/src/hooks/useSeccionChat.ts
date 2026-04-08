/* [054A-10] Hook de la sección de chat del panel.
 * Centraliza estado local, envío, cierre y conexión WS por sesión activa.
 * [064A-68] WS events ahora invalidan React Query cache para updates en tiempo real. */

import {useCallback, useEffect, useRef, useState, type KeyboardEvent} from 'react';
import {useQueryClient} from '@tanstack/react-query';

import {apiCloseSession} from '../api/chat';
import {useChat} from './useChat';
import {useChatWs} from './useChatWs';
import {toast} from '../stores/toastStore';

export function useSeccionChat() {
    const queryClient = useQueryClient();
    const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
    const [input, setInput] = useState('');
    const [closing, setClosing] = useState(false);
    const [messageLimit, setMessageLimit] = useState(100);
    const messagesEndRef = useRef<HTMLDivElement>(null);

    const {sessions, messages, cargandoSesiones, cargandoMensajes, enviarMensaje, enviando} =
        useChat(activeSessionId ?? undefined, messageLimit);
    const ws = useChatWs();

    /* [064A-68] Cuando WS recibe nuevos mensajes, invalidar cache REST para refrescar
     * la UI instantáneamente en vez de esperar el polling de 5s/15s. */
    useEffect(() => {
        if (ws.messages.length > 0) {
            queryClient.invalidateQueries({queryKey: ['chat-messages']});
        }
    }, [ws.messages.length, queryClient]);

    useEffect(() => {
        if (ws.sessions.length > 0) {
            queryClient.invalidateQueries({queryKey: ['chat-sessions']});
        }
    }, [ws.sessions.length, queryClient]);

    useEffect(() => {
        ws.connect();
        return () => ws.disconnect();
    }, [ws.connect, ws.disconnect]);

    useEffect(() => {
        if (activeSessionId) {
            ws.joinSession(activeSessionId);
        }
    }, [activeSessionId, ws.joinSession]);

    useEffect(() => {
        messagesEndRef.current?.scrollIntoView({behavior: 'smooth'});
    }, [messages]);

    const handleSend = useCallback(async () => {
        if (!input.trim() || !activeSessionId || enviando) return;

        const content = input.trim();
        setInput('');

        try {
            await enviarMensaje({sId: activeSessionId, content});
        } catch {
            setInput(content);
        }
    }, [activeSessionId, enviando, enviarMensaje, input]);

    const handleKeyDown = useCallback(
        (event: KeyboardEvent<HTMLTextAreaElement>) => {
            if (event.key === 'Enter' && !event.shiftKey) {
                event.preventDefault();
                void handleSend();
            }
        },
        [handleSend],
    );

    const handleCloseSession = useCallback(async () => {
        if (!activeSessionId || closing) return;

        setClosing(true);
        try {
            await apiCloseSession(activeSessionId);
            toast.success('Conversación cerrada');
            setActiveSessionId(null);
        } catch {
            toast.error('Error al cerrar conversación');
        } finally {
            setClosing(false);
        }
    }, [activeSessionId, closing]);

    const selectSession = useCallback((sessionId: string) => {
        setActiveSessionId(sessionId);
        setMessageLimit(100);
    }, []);

    const clearActiveSession = useCallback(() => {
        setActiveSessionId(null);
        setMessageLimit(100);
    }, []);

    /* [074A-43] Cargar más mensajes antiguos */
    const loadOlderMessages = useCallback(() => {
        setMessageLimit(prev => prev + 100);
    }, []);

    /* Indica si podría haber mensajes más antiguos (si la cantidad actual coincide con el límite) */
    const hasOlderMessages = messages.length >= messageLimit;

    return {
        activeSessionId,
        sessions,
        messages,
        cargandoSesiones,
        cargandoMensajes,
        enviando,
        closing,
        input,
        messagesEndRef,
        typingMap: ws.typingMap,
        showingChat: activeSessionId !== null,
        hasOlderMessages,
        setInput,
        selectSession,
        clearActiveSession,
        loadOlderMessages,
        handleKeyDown,
        handleSend,
        handleCloseSession,
    };
}