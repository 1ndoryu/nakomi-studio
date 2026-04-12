/* [064A-72] Panel lateral de info del visitante en una sesión de chat.
 * Muestra IP, user-agent parseado, notas del staff, y permite renombrar. */

import React, {useState} from 'react';
import {Monitor, Globe, StickyNote, Edit3, Send, X} from 'lucide-react';
import {useQuery, useMutation, useQueryClient} from '@tanstack/react-query';
import {
    type ChatSession,
    type ChatSessionNote,
    apiListSessionNotes,
    apiCreateSessionNote,
    apiUpdateVisitorName,
} from '../../api/chat';
import {Button} from '../ui/Button';
import {Input} from '../ui/Input';
import {Textarea} from '../ui/Textarea';
import './ChatInfoPanel.css';

interface ChatInfoPanelProps {
    session: ChatSession;
    onClose: () => void;
}

/* [064A-72] Parsear user-agent en nombre legible (navegador + OS).
 * No es exhaustivo, solo cubre los casos más comunes. */
function parseUserAgent(ua: string): {browser: string; os: string} {
    let browser = 'Desconocido';
    let os = 'Desconocido';

    if (ua.includes('Firefox/')) browser = 'Firefox';
    else if (ua.includes('Edg/')) browser = 'Edge';
    else if (ua.includes('Chrome/') && !ua.includes('Edg/')) browser = 'Chrome';
    else if (ua.includes('Safari/') && !ua.includes('Chrome/')) browser = 'Safari';

    if (ua.includes('Windows')) os = 'Windows';
    else if (ua.includes('Mac OS')) os = 'macOS';
    else if (ua.includes('Linux')) os = 'Linux';
    else if (ua.includes('Android')) os = 'Android';
    else if (ua.includes('iPhone') || ua.includes('iPad')) os = 'iOS';

    return {browser, os};
}

export const ChatInfoPanel: React.FC<ChatInfoPanelProps> = ({session, onClose}) => {
    const queryClient = useQueryClient();
    const [noteInput, setNoteInput] = useState('');
    const [editingName, setEditingName] = useState(false);
    const [nameInput, setNameInput] = useState(session.visitor_name ?? '');

    const {data: notes = []} = useQuery<ChatSessionNote[]>({
        queryKey: ['chatNotes', session.id],
        queryFn: () => apiListSessionNotes(session.id),
    });

    const createNote = useMutation({
        mutationFn: (content: string) => apiCreateSessionNote(session.id, content),
        onSuccess: () => {
            void queryClient.invalidateQueries({queryKey: ['chatNotes', session.id]});
            setNoteInput('');
        },
    });

    const renameMutation = useMutation({
        mutationFn: (name: string) => apiUpdateVisitorName(session.id, name),
        onSuccess: () => {
            void queryClient.invalidateQueries({queryKey: ['chatSessions']});
            setEditingName(false);
        },
    });

    const parsed = session.visitor_user_agent ? parseUserAgent(session.visitor_user_agent) : null;

    return (
        <div className="chatInfoPanel">
            <div className="chatInfoHeader">
                <span className="chatInfoTitulo">Info del visitante</span>
                <Button
                    className="chatInfoBtnCerrar"
                    onClick={onClose}
                    variante="texto"
                    tamano="pequeno"
                    type="button"
                >
                    <X size={16} />
                </Button>
            </div>

            <div className="chatInfoBody">
                {/* Nombre / renombrar */}
                <div className="chatInfoSeccion">
                    <div className="chatInfoLabel">Nombre</div>
                    {editingName ? (
                        <div className="chatInfoEditRow">
                            <Input
                                className="chatInfoInput"
                                value={nameInput}
                                onChange={e => setNameInput(e.target.value)}
                                onKeyDown={e => {
                                    if (e.key === 'Enter' && nameInput.trim()) {
                                        renameMutation.mutate(nameInput.trim());
                                    }
                                }}
                                autoFocus
                            />
                            <Button
                                type="button"
                                variante="texto"
                                tamano="pequeno"
                                onClick={() => {
                                    if (nameInput.trim()) renameMutation.mutate(nameInput.trim());
                                }}
                                disabled={renameMutation.isPending}
                            >
                                <Send size={14} />
                            </Button>
                        </div>
                    ) : (
                        <div className="chatInfoValorRow">
                            <span className="chatInfoValor">
                                {session.visitor_name || 'Visitante anónimo'}
                            </span>
                            <Button
                                type="button"
                                variante="texto"
                                tamano="pequeno"
                                className="chatInfoBtnEditar"
                                onClick={() => setEditingName(true)}
                            >
                                <Edit3 size={14} />
                            </Button>
                        </div>
                    )}
                </div>

                {/* IP */}
                {session.visitor_ip && (
                    <div className="chatInfoSeccion">
                        <div className="chatInfoLabel">
                            <Globe size={14} /> IP
                        </div>
                        <div className="chatInfoValor">{session.visitor_ip}</div>
                    </div>
                )}

                {/* País */}
                {session.visitor_country && (
                    <div className="chatInfoSeccion">
                        <div className="chatInfoLabel">
                            <Globe size={14} /> País
                        </div>
                        <div className="chatInfoValor">{session.visitor_country}</div>
                    </div>
                )}

                {/* Dispositivo */}
                {parsed && (
                    <div className="chatInfoSeccion">
                        <div className="chatInfoLabel">
                            <Monitor size={14} /> Dispositivo
                        </div>
                        <div className="chatInfoValor">
                            {parsed.browser} en {parsed.os}
                        </div>
                    </div>
                )}

                {/* Notas */}
                <div className="chatInfoSeccion chatInfoSeccionNotas">
                    <div className="chatInfoLabel">
                        <StickyNote size={14} /> Notas ({notes.length})
                    </div>
                    <div className="chatInfoNotas">
                        {notes.map(n => (
                            <div key={n.id} className="chatInfoNota">
                                <div className="chatInfoNotaContenido">{n.content}</div>
                                <div className="chatInfoNotaFecha">
                                    {new Date(n.created_at).toLocaleString('es', {
                                        day: '2-digit',
                                        month: 'short',
                                        hour: '2-digit',
                                        minute: '2-digit',
                                    })}
                                </div>
                            </div>
                        ))}
                    </div>
                    <div className="chatInfoNotaInput">
                        {/* [074A-40] Textarea sin botón — Enter envía la nota */}
                        <Textarea
                            className="chatInfoInput"
                            value={noteInput}
                            onChange={e => setNoteInput(e.target.value)}
                            onKeyDown={e => {
                                if (e.key === 'Enter' && !e.shiftKey && noteInput.trim()) {
                                    e.preventDefault();
                                    createNote.mutate(noteInput.trim());
                                }
                            }}
                            placeholder="Agregar nota... (Enter para enviar)"
                            rows={2}
                        />
                    </div>
                </div>
            </div>
        </div>
    );
};
