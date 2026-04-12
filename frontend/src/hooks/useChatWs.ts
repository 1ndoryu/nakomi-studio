/* [084A-22] Hook de WebSocket en tiempo real para staff panel.
 * Extraído de useChat.ts para SRP — el hook REST y el WS son independientes.
 * [054A-4] Typing indicator: mapa session_id → info del que escribe.
 * [104A-40] visitorOnlineMap: session_id → {online, lastConnectedAt} */

import {useState, useCallback, useRef, useEffect} from 'react';
import {
    buildStaffWsUrl,
    type ChatMessage,
    type ChatSession,
    type WsServerMessage,
} from '../api/chat';
import {useAuthStore} from '../stores/authStore';

export interface TypingInfo {
    sender: string;
    content: string;
}

/* [104A-40] Estado de presencia del visitante por sesión */
export interface VisitorOnlineInfo {
    online: boolean;
    lastConnectedAt: string | null;
}

export function useChatWs() {
    const token = useAuthStore(s => s.token);
    const [connected, setConnected] = useState(false);
    const [sessions, setSessions] = useState<ChatSession[]>([]);
    const [messages, setMessages] = useState<ChatMessage[]>([]);
    const [typingMap, setTypingMap] = useState<Record<string, TypingInfo>>({});
    /* [104A-40] Presencia del visitante: session_id → {online, lastConnectedAt} */
    const [visitorOnlineMap, setVisitorOnlineMap] = useState<Record<string, VisitorOnlineInfo>>({});
    const wsRef = useRef<WebSocket | null>(null);
    const typingTimersRef = useRef<Record<string, ReturnType<typeof setTimeout>>>({});

    const connect = useCallback(() => {
        if (!token || wsRef.current) return;
        const url = buildStaffWsUrl(token);
        const ws = new WebSocket(url);
        wsRef.current = ws;

        ws.onopen = () => setConnected(true);
        ws.onclose = () => {
            setConnected(false);
            wsRef.current = null;
        };

        ws.onmessage = (event) => {
            try {
                const msg: WsServerMessage = JSON.parse(event.data);

                switch (msg.type) {
                    case 'init':
                        if (msg.sessions) setSessions(msg.sessions);
                        break;
                    case 'message':
                        if (msg.id && msg.session_id && msg.content) {
                            setMessages(prev => [
                                ...prev,
                                {
                                    id: msg.id!,
                                    session_id: msg.session_id!,
                                    sender_type: msg.sender || 'unknown',
                                    sender_id: msg.sender_id ?? null,
                                    content: msg.content!,
                                    created_at: msg.created_at || new Date().toISOString(),
                                    sender_avatar_url: null,
                                    sender_display_name: null,
                                },
                            ]);
                        }
                        break;
                    case 'session_new':
                        if (msg.session) {
                            setSessions(prev => [msg.session!, ...prev]);
                        }
                        break;
                    case 'session_closed':
                        if (msg.session_id) {
                            setSessions(prev =>
                                prev.filter(s => s.id !== msg.session_id),
                            );
                        }
                        break;
                    case 'status':
                        if (msg.session_id && msg.value) {
                            setSessions(prev =>
                                prev.map(s => {
                                    if (s.id !== msg.session_id) return s;
                                    const updated = {...s};
                                    /* [BUGFIX] No sobreescribir session.status con valores de
                                     * gestión de IA — solo actualizar ai_enabled.
                                     * Antes: updated.status = msg.value corrompia el status real
                                     * ("open") con "ai_handling"/"staff_handling". */
                                    if (msg.value !== 'ai_handling' && msg.value !== 'staff_handling') {
                                        updated.status = msg.value!;
                                    }
                                    /* [124A-CHAT1] Sincronizar ai_enabled con el status de IA */
                                    if (msg.value === 'ai_handling') updated.ai_enabled = true;
                                    else if (msg.value === 'staff_handling') updated.ai_enabled = false;
                                    return updated;
                                }),
                            );
                        }
                        break;
                    /* [054A-4] Typing preview: muestra indicador 3s, auto-limpia */
                    case 'typing':
                        if (msg.session_id && msg.sender) {
                            const sid = msg.session_id;
                            setTypingMap(prev => ({
                                ...prev,
                                [sid]: {sender: msg.sender!, content: msg.content || ''},
                            }));
                            if (typingTimersRef.current[sid]) {
                                clearTimeout(typingTimersRef.current[sid]);
                            }
                            typingTimersRef.current[sid] = setTimeout(() => {
                                setTypingMap(prev => {
                                    const next = {...prev};
                                    delete next[sid];
                                    return next;
                                });
                                delete typingTimersRef.current[sid];
                            }, 3000);
                        }
                        break;
                    /* [104A-40] Presencia del visitante: online/offline + última conexión */
                    case 'visitor_status':
                        if (msg.session_id !== undefined) {
                            setVisitorOnlineMap(prev => ({
                                ...prev,
                                [msg.session_id!]: {
                                    online: msg.online ?? false,
                                    lastConnectedAt: msg.last_connected_at ?? null,
                                },
                            }));
                        }
                        break;
                }
            } catch {
                /* Mensaje no JSON, ignorar */
            }
        };
    }, [token]);

    const disconnect = useCallback(() => {
        if (wsRef.current) {
            wsRef.current.close();
            wsRef.current = null;
        }
        /* [054A-4] Limpiar timers de typing al desconectar */
        for (const timer of Object.values(typingTimersRef.current)) {
            clearTimeout(timer);
        }
        typingTimersRef.current = {};
        setTypingMap({});
    }, []);

    const sendWsMessage = useCallback((data: Record<string, unknown>) => {
        if (wsRef.current?.readyState === WebSocket.OPEN) {
            wsRef.current.send(JSON.stringify(data));
        }
    }, []);

    const joinSession = useCallback(
        (sessionId: string) => sendWsMessage({type: 'join', session_id: sessionId}),
        [sendWsMessage],
    );

    const toggleAi = useCallback(
        (sessionId: string, enable: boolean) => {
            sendWsMessage({type: 'toggle_ai', session_id: sessionId, enable});
            /* [BUGFIX] Optimistic update: reflejar el cambio inmediatamente sin esperar
             * el status WS del servidor. Evita el lag visual y el caso donde el broadcast
             * no llega (suscripción al canal de sesión aún no activa). */
            setSessions(prev =>
                prev.map(s => (s.id === sessionId ? {...s, ai_enabled: enable} : s)),
            );
        },
        [sendWsMessage],
    );

    /* [104A-40] Enviar typing del staff al visitante — requiere session_id */
    const sendTyping = useCallback(
        (sessionId: string, content: string) =>
            sendWsMessage({type: 'typing', session_id: sessionId, content}),
        [sendWsMessage],
    );

    /* Limpiar al desmontar */
    useEffect(() => () => disconnect(), [disconnect]);

    return {
        connected,
        sessions,
        messages,
        typingMap,
        visitorOnlineMap,
        connect,
        disconnect,
        joinSession,
        toggleAi,
        sendWsMessage,
        sendTyping,
    };
}
