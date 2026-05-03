/* [054A-10] Hook de la sección de chat del panel.
 * Centraliza estado local, envío, cierre y conexión WS por sesión activa.
 * [064A-68] WS events ahora invalidan React Query cache para updates en tiempo real. */

import {useCallback, useEffect, useRef, useState, type KeyboardEvent} from 'react';
import {useQueryClient} from '@tanstack/react-query';
import {useLocation} from 'react-router-dom';

import {apiCloseSession, apiMarkSessionViewed, apiUploadChatFile} from '../api/chat';
import {useChat} from './useChat';
import {useChatWs} from './useChatWs';
import {toast} from '../stores/toastStore';
import {playNotificationSound} from '../utils/notificationSound';
import {getPanelChatIdFromUrl, syncPanelChatInUrl} from '../utils/panelUrlState';

export function useSeccionChat() {
    const queryClient = useQueryClient();
    const location = useLocation();
    const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
    const [input, setInput] = useState('');
    const [closing, setClosing] = useState(false);
    const [uploading, setUploading] = useState(false);
    const [messageLimit, setMessageLimit] = useState(100);
    const messagesEndRef = useRef<HTMLDivElement>(null);

    const {sessions, messages, cargandoSesiones, cargandoMensajes, enviarMensaje, enviando} =
        useChat(activeSessionId ?? undefined, messageLimit);
    const ws = useChatWs();

    /* [064A-68] Cuando WS recibe nuevos mensajes, invalidar cache REST para refrescar
     * la UI instantáneamente en vez de esperar el polling de 5s/15s.
     * [124A-SOUND] Sonar al recibir mensajes del visitante en el panel del admin. */
    const prevWsMsgCountRef = useRef(0);
    useEffect(() => {
        if (ws.messages.length > prevWsMsgCountRef.current) {
            queryClient.invalidateQueries({queryKey: ['chat-messages']});
            /* Sonar solo si el mensaje más reciente es del visitante */
            const lastMsg = ws.messages[ws.messages.length - 1];
            if (lastMsg && (lastMsg.sender_type === 'visitor' || lastMsg.sender_type === 'client')) {
                playNotificationSound();
            }
        }
        prevWsMsgCountRef.current = ws.messages.length;
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
            syncPanelChatInUrl(null);
        } catch {
            toast.error('Error al cerrar conversación');
        } finally {
            setClosing(false);
        }
    }, [activeSessionId, closing]);

    const selectSession = useCallback((sessionId: string) => {
        setActiveSessionId(sessionId);
        setMessageLimit(100);
        syncPanelChatInUrl(sessionId);
        /* [104A-39] Marcar como vista para limpiar el badge del ChatBell.
         * [104A-41] Invalidar cache ['chat-sessions'] inmediatamente para que ChatBell
         * refleje el nuevo last_viewed_at sin esperar el polling de 15s. */
        void apiMarkSessionViewed(sessionId)
            .then(() => queryClient.invalidateQueries({queryKey: ['chat-sessions']}))
            .catch(() => { /* silenciar — no crítico */ });
    }, [queryClient]);

    /* [154A-12] FIX: Auto-select desde ChatBell ahora usa selectSession (que llama
     * apiMarkSessionViewed + invalidateQueries). Antes usaba setActiveSessionId directo
     * y la sesión nunca se marcaba como leída → badge nunca bajaba. */
    useEffect(() => {
        const target = sessionStorage.getItem('PANEL_CHAT_TARGET') ?? getPanelChatIdFromUrl();
        if (target && sessions.length > 0) {
            sessionStorage.removeItem('PANEL_CHAT_TARGET');
            const exists = sessions.some(s => s.id === target);
            if (exists && activeSessionId !== target) selectSession(target);
            return;
        }
        if (!target && activeSessionId) {
            setActiveSessionId(null);
        }
    }, [location.search, sessions, selectSession, activeSessionId]);

    const clearActiveSession = useCallback(() => {
        setActiveSessionId(null);
        setMessageLimit(100);
        syncPanelChatInUrl(null);
    }, []);

    /* [074A-43] Cargar más mensajes antiguos */
    const loadOlderMessages = useCallback(() => {
        setMessageLimit(prev => prev + 100);
    }, []);

    /* Indica si podría haber mensajes más antiguos (si la cantidad actual coincide con el límite) */
    const hasOlderMessages = messages.length >= messageLimit;

    /* [114A-13] Subir archivo adjunto en la sesión activa (staff) */
    const handleUpload = useCallback(async (file: File) => {
        if (!activeSessionId || uploading) return;
        setUploading(true);
        try {
            await apiUploadChatFile(activeSessionId, file);
            queryClient.invalidateQueries({queryKey: ['chat-messages']});
        } catch {
            toast.error('Error al subir archivo');
        } finally {
            setUploading(false);
        }
    }, [activeSessionId, uploading, queryClient]);

    /* [124A-CHAT1] Merge de ai_enabled desde WS en tiempo real.
     * ws.sessions tiene el estado live (actualizado por WS status messages),
     * sessions (React Query) puede estar stale hasta el próximo poll de 15s. */
    const wsSessionAiEnabled = (activeSessionId
        ? ws.sessions.find(s => s.id === activeSessionId)?.ai_enabled
        : undefined);

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
        /* [104A-40] Presencia del visitante para indicador online/offline */
        visitorOnlineMap: ws.visitorOnlineMap,
        sendTyping: ws.sendTyping,
        /* [124A-CHAT1] Toggle IA por sesión — staff puede activar/desactivar la IA */
        toggleAi: ws.toggleAi,
        /* [124A-CHAT1] ai_enabled en tiempo real (WS) para el botón toggle — fallback a sessions */
        wsSessionAiEnabled,
        showingChat: activeSessionId !== null,
        hasOlderMessages,
        setInput,
        selectSession,
        clearActiveSession,
        loadOlderMessages,
        handleKeyDown,
        handleSend,
        handleCloseSession,
        handleUpload,
        uploading,
    };
}