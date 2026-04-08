/* [064A-28+064A-52] ChatWidget: redesign completo. Sin header, burbuja con avatar,
 * fondo semi-transparente, sombra especifica del usuario, animacion suave.
 * [064A-52] Eliminada lógica de pedir nombre — el agente IA lo pide directamente.
 * Input 100% ancho con send button overlaid, bg unset, radius 104px.
 * Avatar de AI al lado de los mensajes en vez de icono Bot. */

import React, {useState, useRef, useEffect} from 'react';
import {useLocation} from 'react-router-dom';
import {Send, User, Minus, Paperclip} from 'lucide-react';
import {useChatWidget} from '../../hooks/useChatWidget';
import {SENDER_LABELS} from '../../api/chat';
import {useChatStore} from '../../stores/chatStore';
import {Input} from '../ui/Input';
import {Button} from '../ui/Button';
import './ChatWidget.css';

const AVATAR_SRC = '/assets/random/85a51ba9a4233272662e744b48f97d67.jpg';

export const ChatWidget: React.FC = () => {
    const abierto = useChatStore(s => s.abierto);
    const abrir = useChatStore(s => s.abrir);
    const cerrar = useChatStore(s => s.cerrar);
    const [input, setInput] = useState('');
    const messagesEndRef = useRef<HTMLDivElement>(null);

    const {
        connected,
        connecting,
        messages,
        typing,
        uploading,
        connect,
        sendMessage,
        sendTyping,
        uploadFile,
    } = useChatWidget();

    /* [064A-67→074A-18] Widget de chat oculto en /panel — el panel tiene su propia SeccionChat */
    const location = useLocation();
    if (location.pathname.startsWith('/panel')) return null;

    /* [064A-52] Al abrir, conectar directamente sin pedir nombre.
     * El agente IA pedirá el nombre al usuario si lo necesita. */
    const handleOpen = () => {
        abrir();
        if (!connected && !connecting) {
            connect();
        }
    };

    const handleMinimize = () => {
        cerrar();
    };

    const handleSend = () => {
        const content = input.trim();
        if (!content || !connected) return;
        sendMessage(content);
        setInput('');
    };

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            handleSend();
        }
    };

    const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        setInput(e.target.value);
        if (e.target.value.trim()) {
            sendTyping(e.target.value);
        }
    };

    return (
        <>
            <Button
                variante="texto"
                tamano="pequeno"
                className={`chatWidgetBubble ${abierto ? 'chatWidgetBubbleOculta' : ''}`}
                onClick={handleOpen}
                aria-label="Abrir chat"
                type="button"
            >
                <img src={AVATAR_SRC} alt="" className="chatWidgetBubbleAvatar" loading="lazy" />
                <span className="chatWidgetBubbleTexto">Chat</span>
            </Button>

            <div className={`chatWidgetPanel ${abierto ? 'chatWidgetPanelAbierto' : ''}`}>
                <Button
                    variante="texto"
                    tamano="pequeno"
                    className="chatWidgetMinimizeBtn"
                    onClick={handleMinimize}
                    aria-label="Minimizar chat"
                    type="button"
                >
                    <Minus size={18} />
                </Button>

                <ChatWidgetMessages
                    messages={messages}
                    typing={typing}
                    messagesEndRef={messagesEndRef}
                />
                <ChatWidgetInput
                    input={input}
                    connected={connected}
                    uploading={uploading}
                    onInputChange={handleInputChange}
                    onKeyDown={handleKeyDown}
                    onSend={handleSend}
                    onUpload={uploadFile}
                />
            </div>
        </>
    );
};

/* Sub-componentes (SRP) */

/* [T-5] Renderiza contenido rico de mensajes: imágenes inline, audio player, enlace PDF.
 * Si message_type es null/text, renderiza texto plano como antes. */
function renderMessageContent(msg: {
    content: string;
    message_type?: string | null;
    metadata?: Record<string, unknown> | null;
}): React.ReactNode {
    const fileUrl = (msg.metadata?.file_url as string) || '';
    const fileName = (msg.metadata?.file_name as string) || 'archivo';

    switch (msg.message_type) {
        case 'image':
            return (
                <div className="chatWidgetMsgRich">
                    <a href={fileUrl} target="_blank" rel="noopener noreferrer">
                        <img
                            src={fileUrl}
                            alt={fileName}
                            className="chatWidgetMsgImage"
                            loading="lazy"
                        />
                    </a>
                    {msg.content && <p className="chatWidgetMsgCaption">{msg.content}</p>}
                </div>
            );
        case 'audio':
            return (
                <div className="chatWidgetMsgRich">
                    <audio controls preload="metadata" className="chatWidgetMsgAudio">
                        <source src={fileUrl} />
                    </audio>
                    {msg.content && <p className="chatWidgetMsgCaption">{msg.content}</p>}
                </div>
            );
        case 'file':
            return (
                <div className="chatWidgetMsgRich">
                    <a
                        href={fileUrl}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="chatWidgetMsgFileLink"
                    >
                        📄 {fileName}
                    </a>
                    {msg.content && <p className="chatWidgetMsgCaption">{msg.content}</p>}
                </div>
            );
        default:
            return <>{msg.content}</>;
    }
}

function ChatWidgetMessages({
    messages,
    typing,
    messagesEndRef,
}: {
    messages: Array<{
        id: string;
        sender_type: string;
        sender_id: string | null;
        content: string;
        created_at: string;
        message_type?: string | null;
        metadata?: Record<string, unknown> | null;
    }>;
    typing: {sender: string; content: string} | null;
    messagesEndRef: React.RefObject<HTMLDivElement>;
}) {
    useEffect(() => {
        messagesEndRef.current?.scrollIntoView({behavior: 'smooth'});
    }, [messages, typing, messagesEndRef]);

    return (
        <div className="chatWidgetMessages">
            {messages.length === 0 && (
                <p className="chatWidgetEmpty">
                    Envía un mensaje para comenzar la conversación.
                    Un asistente te responderá pronto.
                </p>
            )}
            {messages.map((msg) => {
                const isOwn = msg.sender_type === 'visitor' || msg.sender_type === 'client';
                const isAi = msg.sender_type === 'ai';
                const label = SENDER_LABELS[msg.sender_type] || msg.sender_type;

                /* [T-5] Renderizar contenido según message_type */
                const bubbleContent = renderMessageContent(msg);
                const bubble = (
                    <div className={`chatWidgetMsgBubble ${isOwn ? 'chatWidgetMsgBubbleOwn' : 'chatWidgetMsgBubbleOther'}`}>
                        {bubbleContent}
                    </div>
                );

                return (
                    <div
                        key={msg.id}
                        className={`chatWidgetMsg ${isOwn ? 'chatWidgetMsgOwn' : 'chatWidgetMsgOther'}`}
                    >
                        {!isOwn && !isAi && (
                            <span className={`chatWidgetMsgSender chatWidgetSender--${msg.sender_type}`}>
                                <User size={14} /> {label}
                            </span>
                        )}
                        {isAi ? (
                            <div className="chatWidgetAiRow">
                                <img src={AVATAR_SRC} alt="" className="chatWidgetAiAvatar" loading="lazy" />
                                {bubble}
                            </div>
                        ) : (
                            bubble
                        )}
                    </div>
                );
            })}

            {typing && (
                <div className="chatWidgetMsg chatWidgetMsgOther">
                    {typing.sender !== 'ai' && (
                        <span className={`chatWidgetMsgSender chatWidgetSender--${typing.sender}`}>
                            <User size={14} /> {SENDER_LABELS[typing.sender] || typing.sender}
                        </span>
                    )}
                    {typing.sender === 'ai' ? (
                        <div className="chatWidgetAiRow">
                            <img src={AVATAR_SRC} alt="" className="chatWidgetAiAvatar" loading="lazy" />
                            <div className="chatWidgetMsgBubble chatWidgetMsgBubbleOther chatWidgetTyping">
                                <span className="chatWidgetTypingDots">
                                    <span />
                                    <span />
                                    <span />
                                </span>
                            </div>
                        </div>
                    ) : (
                        <div className="chatWidgetMsgBubble chatWidgetMsgBubbleOther chatWidgetTyping">
                            <span className="chatWidgetTypingDots">
                                <span />
                                <span />
                                <span />
                            </span>
                        </div>
                    )}
                </div>
            )}

            <div ref={messagesEndRef} />
        </div>
    );
}

/* [T-5] MIME types aceptados en el file picker */
const CHAT_FILE_ACCEPT = 'image/jpeg,image/png,image/webp,image/gif,audio/mpeg,audio/ogg,audio/wav,audio/webm,audio/mp4,audio/flac,application/pdf';

function ChatWidgetInput({
    input,
    connected,
    uploading,
    onInputChange,
    onKeyDown,
    onSend,
    onUpload,
}: {
    input: string;
    connected: boolean;
    uploading: boolean;
    onInputChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
    onKeyDown: (e: React.KeyboardEvent) => void;
    onSend: () => void;
    onUpload: (file: File) => void;
}) {
    const fileInputRef = useRef<HTMLInputElement>(null);

    const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (file) {
            onUpload(file);
            e.target.value = '';
        }
    };

    return (
        <div className="chatWidgetInputArea">
            <input
                ref={fileInputRef}
                type="file"
                accept={CHAT_FILE_ACCEPT}
                onChange={handleFileChange}
                style={{display: 'none'}}
            />
            <Button
                variante="texto"
                tamano="pequeno"
                className="chatWidgetAttachBtn"
                onClick={() => fileInputRef.current?.click()}
                disabled={!connected || uploading}
                aria-label="Adjuntar archivo"
                type="button"
            >
                <Paperclip size={18} />
            </Button>
            <div className="chatWidgetInputWrapper">
                <Input
                    type="text"
                    placeholder={uploading ? 'Subiendo archivo...' : connected ? 'Escribe un mensaje...' : 'Conectando...'}
                    value={input}
                    onChange={onInputChange}
                    onKeyDown={onKeyDown}
                    disabled={!connected || uploading}
                    className="chatWidgetInput"
                />
                <Button
                    variante="texto"
                    tamano="pequeno"
                    className="chatWidgetSendBtn"
                    onClick={onSend}
                    disabled={!connected || !input.trim() || uploading}
                    aria-label="Enviar mensaje"
                    type="button"
                >
                    <Send size={18} />
                </Button>
            </div>
        </div>
    );
}
