/* [104A-34] Botón de chat en header con dropdown de sesiones activas.
 * Al hacer clic en una sesión, navega al panel → sección mensajes → abre ese chat.
 * Usa sessionStorage para pasar el target a SeccionChat (evita race conditions con mount). */

import {useCallback, useState} from 'react';
import {MessageSquare, Bot, User} from 'lucide-react';

import {useChat} from '../../hooks/useChat';
import {Button} from '../ui/Button';
import {MenuContextual} from '../ui/ContextMenu';
import './ChatBell.css';

const PANEL_CHAT_TARGET = 'PANEL_CHAT_TARGET';

export default function ChatBell() {
    const {sessions, cargandoSesiones} = useChat();
    const [open, setOpen] = useState(false);

    /* Sesiones activas (no cerradas/archivadas) ordenadas por último mensaje */
    const activeSessions = sessions
        .filter(s => s.status !== 'closed')
        .sort((a, b) => {
            const ta = a.last_message_at ?? a.created_at;
            const tb = b.last_message_at ?? b.created_at;
            return new Date(tb).getTime() - new Date(ta).getTime();
        })
        .slice(0, 15);

    /* [104A-39] Badge = sesiones con mensajes no leídos (last_message_at > last_viewed_at).
     * Si nunca se vio (last_viewed_at null) y tiene mensajes, cuenta como no leída. */
    const unreadCount = activeSessions.filter(s => {
        if (!s.last_message_at) return false;
        if (!s.last_viewed_at) return true;
        return new Date(s.last_message_at) > new Date(s.last_viewed_at);
    }).length;

    const activeCount = activeSessions.length;

    const toggleDropdown = useCallback(() => {
        setOpen(prev => !prev);
    }, []);

    const closeDropdown = useCallback(() => {
        setOpen(false);
    }, []);

    /* Navegar al panel → sección mensajes → chat seleccionado */
    const handleSelectSession = useCallback((sessionId: string) => {
        sessionStorage.setItem(PANEL_CHAT_TARGET, sessionId);
        window.dispatchEvent(new CustomEvent('panel-cambiar-tab', {detail: 'mensajes'}));
        setOpen(false);
    }, []);

    /* Formatear fecha relativa */
    const formatTime = (iso: string | null): string => {
        if (!iso) return '';
        const diff = Date.now() - new Date(iso).getTime();
        const mins = Math.floor(diff / 60000);
        if (mins < 1) return 'ahora';
        if (mins < 60) return `hace ${mins}m`;
        const hours = Math.floor(mins / 60);
        if (hours < 24) return `hace ${hours}h`;
        const days = Math.floor(hours / 24);
        return `hace ${days}d`;
    };

    return (
        <MenuContextual
            abierto={open}
            onToggle={toggleDropdown}
            onCerrar={closeDropdown}
            ariaLabel={`Chats${activeCount > 0 ? ` (${activeCount} activos)` : ''}`}
            className="chatBell"
            triggerClassName="chatBell__trigger"
            panelClassName="chatBell__dropdown"
            triggerContent={(
                <>
                    <MessageSquare size={20} />
                    {unreadCount > 0 && (
                        <span className="chatBell__badge">
                            {unreadCount > 99 ? '99+' : unreadCount}
                        </span>
                    )}
                </>
            )}
        >
            <div className="chatBell__header">
                <h3 className="chatBell__title">Conversaciones</h3>
            </div>

            <div className="chatBell__list">
                {cargandoSesiones ? (
                    <p className="chatBell__empty">Cargando...</p>
                ) : activeSessions.length === 0 ? (
                    <p className="chatBell__empty">Sin conversaciones activas</p>
                ) : (
                    activeSessions.map(s => (
                        <Button
                            key={s.id}
                            type="button"
                            variante="texto"
                            className="chatBell__item"
                            onClick={() => handleSelectSession(s.id)}
                        >
                            <span className="chatBell__itemIcon">
                                {s.ai_enabled ? <Bot size={16} /> : <User size={16} />}
                            </span>
                            <div className="chatBell__itemContent">
                                <span className="chatBell__itemTitle">
                                    {s.order_number ? `Orden #${s.order_number}` : (s.visitor_name ?? 'Chat general')}
                                </span>
                                {s.last_message && (
                                    <span className="chatBell__itemPreview">{s.last_message}</span>
                                )}
                                <span className="chatBell__itemTime">
                                    {formatTime(s.last_message_at ?? s.created_at)}
                                </span>
                            </div>
                        </Button>
                    ))
                )}
            </div>
        </MenuContextual>
    );
}
