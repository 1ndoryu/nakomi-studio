/* [064A-31] Chat dentro de pedidos: comunicación cliente ↔ empleado asignado.
 * Se abre inline en OrdenDetalle. Usa REST polling (5s) via useOrderChat.
 * El empleado ve estos mensajes en SeccionChat del panel staff. */

import React, {useState, useRef, useEffect} from 'react';
import {Send} from 'lucide-react';
import {useOrderChat} from '../../hooks/useOrderChat';
import {useAuthStore} from '../../stores/authStore';
import {Input} from '../ui/Input';
import {Button} from '../ui/Button';
import OptimizedImage from '../ui/OptimizedImage';
import {DEFAULT_PROFILE_AVATAR} from '../../hooks/useCurrentProfile';
import './OrderChat.css';

interface OrderChatProps {
    orderId: string;
}

export const OrderChat: React.FC<OrderChatProps> = ({orderId}) => {
    const [input, setInput] = useState('');
    const messagesContainerRef = useRef<HTMLDivElement>(null);
    const userId = useAuthStore(s => s.user?.userId);
    const {sessionId, session, mensajes, enviando, creando, iniciarSesion, enviarMensaje} =
        useOrderChat(orderId);

    /* Iniciar sesión al montar */
    useEffect(() => {
        iniciarSesion();
    }, [iniciarSesion]);

    /* [074A-52] Auto-scroll interno: usar scrollTop del contenedor, no scrollIntoView
     * que mueve el scroll de la página entera */
    useEffect(() => {
        const el = messagesContainerRef.current;
        if (el) el.scrollTop = el.scrollHeight;
    }, [mensajes]);

    const handleSend = async () => {
        const content = input.trim();
        if (!content || enviando) return;
        setInput('');
        await enviarMensaje(content);
    };

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            handleSend();
        }
    };

    if (creando && !sessionId) {
        return (
            <div className="orderChatPanel">
                <p className="orderChatCargando">Conectando chat...</p>
            </div>
        );
    }

    /* Nombres de participantes visibles en la cabecera */
    const participantes = [
        session?.client_name,
        session?.employee_name,
    ].filter(Boolean);

    return (
        <div className="orderChatPanel">
            {participantes.length > 0 && (
                <div className="orderChatParticipantes">
                    {participantes.map(nombre => (
                        <span key={nombre} className="orderChatParticipante">{nombre}</span>
                    ))}
                </div>
            )}
            <div className="orderChatMensajes" ref={messagesContainerRef}>
                {mensajes.length === 0 && (
                    <p className="orderChatVacio">
                        Envía un mensaje para iniciar la conversación.
                    </p>
                )}
                {mensajes.map(msg => {
                    const esPropio = msg.sender_id === userId;
                    /* [T-10] Mensajes de IA intermediaria con estilo diferente */
                    const esIntermediaria = msg.sender_type === 'ai_intermediary';
                    const clsBurbuja = esIntermediaria
                        ? 'orderChatBurbuja orderChatBurbuja--intermediaria'
                        : `orderChatBurbuja ${esPropio ? 'orderChatBurbuja--propia' : 'orderChatBurbuja--otra'}`;
                    return (
                        <div key={msg.id} className={clsBurbuja}>
                            {/* [074A-38] Solo mostrar avatar del otro usuario, no el propio */}
                            {!esPropio && (
                                <OptimizedImage
                                    className="orderChatBurbujaAvatar"
                                    src={msg.sender_avatar_url || DEFAULT_PROFILE_AVATAR}
                                    alt={msg.sender_display_name || ''}
                                />
                            )}
                            <div className="orderChatBurbujaContenido">
                                {esIntermediaria && (
                                    <span className="orderChatLabelIntermed">Asistente</span>
                                )}
                                {!esPropio && !esIntermediaria && msg.sender_display_name && (
                                    <span className="orderChatNombreSender">{msg.sender_display_name}</span>
                                )}
                                <div className="orderChatBurbujaTexto">{msg.content}</div>
                                <span className="orderChatBurbujaHora">
                                {new Date(msg.created_at).toLocaleTimeString('es-ES', {
                                    hour: '2-digit',
                                    minute: '2-digit',
                                })}
                                </span>
                            </div>
                        </div>
                    );
                })}
            </div>
            <div className="orderChatInputArea">
                <Input
                    value={input}
                    onChange={e => setInput(e.target.value)}
                    onKeyDown={handleKeyDown}
                    placeholder="Escribe un mensaje..."
                    disabled={!sessionId || enviando}
                />
                <Button
                    variante="texto"
                    tamano="pequeno"
                    type="button"
                    onClick={handleSend}
                    disabled={!input.trim() || enviando || !sessionId}
                    className="orderChatEnviar"
                >
                    <Send size={16} />
                </Button>
            </div>
        </div>
    );
};
