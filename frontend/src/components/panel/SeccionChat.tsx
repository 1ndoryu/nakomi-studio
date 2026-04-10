/* [044A-38 Fase 5] Sección "Mensajes" del panel.
 * Lista de conversaciones + chat activo + panel info visitante (064A-72).
 * Staff: ve todas las sesiones. Cliente: solo sus chats de órdenes.
 * [074A-60] Info visitante solo visible para admin. */

import React, {useState} from 'react';
import {MessageCircle, Send, Bot, User, ChevronLeft, XCircle, Info} from 'lucide-react';
import {SENDER_LABELS, SESSION_STATUS_LABELS, type ChatSession} from '../../api/chat';
import {useSeccionChat} from '../../hooks/useSeccionChat';
import {useAuthStore} from '../../stores/authStore';
import {ChatInfoPanel} from './ChatInfoPanel';
import {MessageBubble, resolveSenderToneClass} from './ChatBurbujaMessage';
import {Button} from '../ui/Button';
import {Textarea} from '../ui/Textarea';
import './SeccionChat.css';
import './ChatBurbujas.css';

export const SeccionChat: React.FC = () => {
    const {
        activeSessionId,
        sessions,
        messages,
        cargandoSesiones,
        cargandoMensajes,
        enviando,
        input,
        messagesEndRef,
        typingMap,
        showingChat,
        hasOlderMessages,
        setInput,
        selectSession,
        clearActiveSession,
        loadOlderMessages,
        handleKeyDown,
        handleSend,
        handleCloseSession,
        closing,
    } = useSeccionChat();

    const [showInfo, setShowInfo] = useState(false);
    const activeSession = sessions.find(s => s.id === activeSessionId) ?? null;
    /* [104A-36] Info panel visible para admin y empleados (no solo admin) */
    const effectiveRole = useAuthStore(s => s.user?.effectiveRole);
    const isStaff = effectiveRole === 'admin' || effectiveRole === 'employee';

    if (cargandoSesiones) {
        return (
            <div className="chatLoading">
                <div className="chatSpinner" />
                <p>Cargando conversaciones...</p>
            </div>
        );
    }

    return (
        <div className="chatContenedor">
            {/* Lista de sesiones */}
            <div className={`chatListaSesiones ${showingChat ? 'chatListaOculta' : ''}`}>
                <div className="chatListaHeader">
                    <h3>Conversaciones</h3>
                </div>
                {sessions.length === 0 ? (
                    <div className="chatVacio">
                        <MessageCircle size={32} strokeWidth={1.2} />
                        <p>Sin conversaciones activas</p>
                    </div>
                ) : (
                    sessions.map(s => (
                        <SessionItem
                            key={s.id}
                            session={s}
                            active={s.id === activeSessionId}
                            onClick={() => selectSession(s.id)}
                        />
                    ))
                )}
            </div>

            {/* Chat activo */}
            <div className={`chatAreaMensajes ${!showingChat ? 'chatAreaOculta' : ''}`}>
                {activeSessionId ? (
                    <>
                        <div className="chatAreaHeader">
                            <Button
                                className="chatBtnVolver"
                                onClick={clearActiveSession}
                                type="button"
                                variante="texto"
                                tamano="pequeno"
                            >
                                <ChevronLeft size={18} />
                            </Button>
                            <span className="chatAreaTitulo">
                                {activeSession?.order_id
                                    ? `Chat de orden #${activeSession.order_number ?? '...'}`
                                    : activeSession?.visitor_name || 'Chat general'}
                            </span>
                            {/* [064A-72] Botón para abrir/cerrar panel de info
                             * [104A-36] Visible para admin y empleados — contiene IP, user-agent, notas */}
                            {isStaff && (
                                <Button
                                    className="chatBtnInfo"
                                    onClick={() => setShowInfo(prev => !prev)}
                                    type="button"
                                    variante="texto"
                                    tamano="pequeno"
                                    title="Info del visitante"
                                >
                                    <Info size={18} />
                                </Button>
                            )}
                            {/* [104A-36] Cerrar sesión en BD (staff) — antes solo limpiaba UI.
                             * clearActiveSession desselecciona, handleCloseSession cierra via API. */}
                            {isStaff && activeSession?.status !== 'closed' && (
                                <Button
                                    className="chatBtnCerrar"
                                    onClick={() => void handleCloseSession()}
                                    disabled={closing}
                                    type="button"
                                    title="Cerrar conversación"
                                    variante="texto"
                                    tamano="pequeno"
                                >
                                    <XCircle size={18} />
                                </Button>
                            )}
                        </div>

                        <div className="chatMensajes">
                            {cargandoMensajes && (
                                <div className="chatLoading"><div className="chatSpinner" /></div>
                            )}
                            {/* [074A-43] Botón para cargar mensajes antiguos */}
                            {hasOlderMessages && !cargandoMensajes && (
                                <Button
                                    className="chatBtnCargarAnteriores"
                                    onClick={loadOlderMessages}
                                    type="button"
                                    variante="texto"
                                    tamano="pequeno"
                                >
                                    Cargar anteriores
                                </Button>
                            )}
                            {messages.map(m => (
                                <MessageBubble key={m.id} message={m} />
                            ))}
                            {/* [054A-3] Typing indicator desde WebSocket */}
                            {activeSessionId && typingMap[activeSessionId] && (
                                <div className="chatBurbuja chatBurbujaTyping">
                                    <div className="chatBurbujaHeader">
                                        <span className={resolveSenderToneClass(typingMap[activeSessionId].sender)}>
                                            {SENDER_LABELS[typingMap[activeSessionId].sender] || 'Visitante'} está escribiendo...
                                        </span>
                                    </div>
                                    <div className="chatBurbujaContenido chatTypingDots">
                                        <span /><span /><span />
                                    </div>
                                </div>
                            )}
                            <div ref={messagesEndRef} />
                        </div>

                        <div className="chatInputArea">
                            <Textarea
                                className="chatInput"
                                value={input}
                                onChange={e => setInput(e.target.value)}
                                onKeyDown={handleKeyDown}
                                placeholder="Escribe un mensaje..."
                                rows={1}
                            />
                            <Button
                                className="chatBtnEnviar"
                                onClick={() => void handleSend()}
                                disabled={!input.trim() || enviando}
                                type="button"
                                variante="texto"
                                tamano="pequeno"
                            >
                                <Send size={16} />
                            </Button>
                        </div>
                    </>
                ) : (
                    <div className="chatVacio chatVacioCentrado">
                        <MessageCircle size={48} strokeWidth={1.2} />
                        <p>Selecciona una conversación</p>
                    </div>
                )}
            </div>

            {/* [064A-72] Panel lateral de info del visitante
             * [104A-36] Admin y empleados pueden ver info (IP, user-agent, notas) */}
            {isStaff && showInfo && activeSession && (
                <ChatInfoPanel
                    session={activeSession}
                    onClose={() => setShowInfo(false)}
                />
            )}
        </div>
    );
};

function SessionItem({
    session,
    active,
    onClick,
}: {
    session: ChatSession;
    active: boolean;
    onClick: () => void;
}) {
    return (
        <Button
            className={`chatSesionItem ${active ? 'chatSesionActiva' : ''}`}
            onClick={onClick}
            type="button"
            variante="texto"
            tamano="pequeno"
        >
            <div className="chatSesionIcono">
                {session.ai_enabled ? <Bot size={18} /> : <User size={18} />}
            </div>
            <div className="chatSesionInfo">
                <div className="chatSesionTitulo">
                    {session.order_id
                        ? `Orden #${session.order_number ?? '...'}`
                        : 'Chat general'}
                </div>
                <div className="chatSesionPreview">
                    {session.last_message || 'Sin mensajes'}
                </div>
            </div>
            <div className="chatSesionMeta">
                <span className="chatSesionEstado">
                    {SESSION_STATUS_LABELS[session.status] || session.status}
                </span>
            </div>
        </Button>
    );
}

/* [104A-32] MessageBubble, renderMessageContent y resolveSenderToneClass
 * extraidos a ChatBurbujaMessage.tsx para cumplir SRP (max 300 lineas). */
