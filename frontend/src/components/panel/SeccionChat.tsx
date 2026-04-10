/* [044A-38 Fase 5] Sección "Mensajes" del panel.
 * Lista de conversaciones + chat activo + panel info visitante (064A-72).
 * Staff: ve todas las sesiones. Cliente: solo sus chats de órdenes.
 * [074A-60] Info visitante solo visible para admin. */

import React, {useState} from 'react';
import {MessageCircle, Send, Bot, User, ChevronLeft, XCircle, Info} from 'lucide-react';
import {SENDER_LABELS, SESSION_STATUS_LABELS, type ChatSession, type ChatMessage} from '../../api/chat';
import {useSeccionChat} from '../../hooks/useSeccionChat';
import {useAuthStore} from '../../stores/authStore';
import {ChatInfoPanel} from './ChatInfoPanel';
import {Button} from '../ui/Button';
import OptimizedImage from '../ui/OptimizedImage';
import {Textarea} from '../ui/Textarea';
import {DEFAULT_PROFILE_AVATAR} from '../../hooks/useCurrentProfile';
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
    } = useSeccionChat();

    const [showInfo, setShowInfo] = useState(false);
    const activeSession = sessions.find(s => s.id === activeSessionId) ?? null;
    const isAdmin = useAuthStore(s => s.user?.effectiveRole === 'admin');

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
                             * [074A-60] Solo visible para admin — contiene IP, user-agent, notas */}
                            {isAdmin && (
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
                            {/* [064A-71] Botón para deseleccionar la conversación activa,
                             * sin cerrarla permanentemente. El cierre definitivo (close_session)
                             * requiere acción explícita desde un menú contextual futuro. */}
                            <Button
                                className="chatBtnCerrar"
                                onClick={clearActiveSession}
                                type="button"
                                title="Volver a la lista"
                                variante="texto"
                                tamano="pequeno"
                            >
                                <XCircle size={18} />
                            </Button>
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
             * [074A-60] Solo admin puede ver info sensible (IP, user-agent, notas) */}
            {isAdmin && showInfo && activeSession && (
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

function MessageBubble({message}: {message: ChatMessage}) {
    const isAi = message.sender_type === 'ai';
    const isOwn = message.sender_type === 'client';
    const displayName = message.sender_display_name
        || SENDER_LABELS[message.sender_type]
        || message.sender_type;

    return (
        <div className={`chatBurbuja ${isOwn ? 'chatBurbujaPropia' : ''} ${isAi ? 'chatBurbujaIA' : ''}`}>
            <div className="chatBurbujaHeader">
                {/* [074A-32] Avatar del sender — usa DEFAULT_PROFILE_AVATAR como fallback */}
                <OptimizedImage
                    className="chatBurbujaAvatar"
                    src={message.sender_avatar_url || DEFAULT_PROFILE_AVATAR}
                    alt={displayName}
                />
                <span className={resolveSenderToneClass(message.sender_type)}>{displayName}</span>
                <span className="chatBurbujaHora">
                    {new Date(message.created_at).toLocaleTimeString('es', {
                        hour: '2-digit',
                        minute: '2-digit',
                    })}
                </span>
            </div>
            <div className="chatBurbujaContenido">{message.content}</div>
        </div>
    );
}

function resolveSenderToneClass(senderType: string) {
    switch (senderType) {
        case 'admin':
            return 'chatRemitente chatRemitente--admin';
        case 'employee':
            return 'chatRemitente chatRemitente--employee';
        case 'client':
            return 'chatRemitente chatRemitente--client';
        case 'ai':
            return 'chatRemitente chatRemitente--ai';
        case 'visitor':
            return 'chatRemitente chatRemitente--visitor';
        default:
            return 'chatRemitente chatRemitente--neutral';
    }
}
