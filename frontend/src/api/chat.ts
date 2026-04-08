/* [044A-38 Fase 5] API client de chat: REST + WebSocket para sesiones y mensajes.
 * Soporta chat de orden (autenticado) y chat pre-venta (visitante anónimo). */

import axiosInstance, {getApiHost} from './axios-instance';

/*    TIPOS */

export interface ChatSession {
    id: string;
    order_id: string | null;
    order_number: number | null;
    status: string;
    ai_enabled: boolean;
    assigned_staff_id: string | null;
    last_message: string | null;
    last_message_at: string | null;
    created_at: string;
    /* [064A-72] Metadata del visitante (panel de info) */
    visitor_name: string | null;
    visitor_ip: string | null;
    visitor_user_agent: string | null;
}

export interface ChatMessage {
    id: string;
    session_id: string;
    sender_type: string;
    sender_id: string | null;
    content: string;
    created_at: string;
    /* [064A-70] Datos del sender enriquecidos por el backend */
    sender_avatar_url: string | null;
    sender_display_name: string | null;
}

export interface WsServerMessage {
    type: 'message' | 'typing' | 'status' | 'session_new' | 'session_closed' | 'error' | 'init';
    /* message */
    id?: string;
    session_id?: string;
    sender?: string;
    sender_id?: string | null;
    content?: string;
    created_at?: string;
    /* status */
    value?: string;
    /* session_new */
    session?: ChatSession;
    /* init */
    sessions?: ChatSession[];
    /* error */
    message?: string;
}

/*    REST API */

export async function apiListChatSessions(): Promise<ChatSession[]> {
    const {data} = await axiosInstance.get<ChatSession[]>('/api/chat/sessions');
    return data;
}

export async function apiGetMessages(
    sessionId: string,
    limit = 50,
    offset = 0,
): Promise<ChatMessage[]> {
    const {data} = await axiosInstance.get<ChatMessage[]>(
        `/api/chat/sessions/${sessionId}/messages`,
        {params: {limit, offset}},
    );
    return data;
}

export async function apiCreateChatSession(
    orderId?: string,
): Promise<ChatSession> {
    const {data} = await axiosInstance.post<ChatSession>('/api/chat/sessions', {
        order_id: orderId ?? null,
    });
    return data;
}

export async function apiSendMessage(
    sessionId: string,
    content: string,
): Promise<ChatMessage> {
    const {data} = await axiosInstance.post<ChatMessage>(
        `/api/chat/sessions/${sessionId}/messages`,
        {content},
    );
    return data;
}

/* [054A-9] Cerrar sesión de chat (staff/admin) */
export async function apiCloseSession(sessionId: string): Promise<void> {
    await axiosInstance.post(`/api/chat/sessions/${sessionId}/close`);
}

/*    [064A-72] NOTAS DE SESIÓN Y RENOMBRAR VISITANTE */

export interface ChatSessionNote {
    id: string;
    session_id: string;
    author_id: string;
    content: string;
    created_at: string;
}

export async function apiListSessionNotes(sessionId: string): Promise<ChatSessionNote[]> {
    const {data} = await axiosInstance.get<ChatSessionNote[]>(
        `/api/chat/sessions/${sessionId}/notes`,
    );
    return data;
}

export async function apiCreateSessionNote(
    sessionId: string,
    content: string,
): Promise<ChatSessionNote> {
    const {data} = await axiosInstance.post<ChatSessionNote>(
        `/api/chat/sessions/${sessionId}/notes`,
        {content},
    );
    return data;
}

export async function apiUpdateVisitorName(
    sessionId: string,
    name: string,
): Promise<void> {
    await axiosInstance.patch(`/api/chat/sessions/${sessionId}/visitor-name`, {name});
}

/*    WEBSOCKET HELPERS */

/** Construye URL de WebSocket para visitante */
export function buildVisitorWsUrl(visitorId: string, visitorName?: string): string {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = getApiHost();
    let url = `${protocol}//${host}/ws/chat/visitor?visitor_id=${encodeURIComponent(visitorId)}`;
    if (visitorName) {
        url += `&visitor_name=${encodeURIComponent(visitorName)}`;
    }
    return url;
}

/** Construye URL de WebSocket para staff */
export function buildStaffWsUrl(token: string): string {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = getApiHost();
    return `${protocol}//${host}/ws/chat/staff?token=${encodeURIComponent(token)}`;
}

/*    CONSTANTES UI */

/* [T-7] Labels no revelan IA. 'ai' se muestra como 'Nakomi' (nombre del agente del equipo). */
export const SENDER_LABELS: Record<string, string> = {
    client: 'Cliente',
    ai: 'Nakomi',
    employee: 'Empleado',
    admin: 'Admin',
    visitor: 'Visitante',
};

export const SENDER_COLORS: Record<string, string> = {
    client: '#60a5fa',
    ai: '#a78bfa',
    employee: '#34d399',
    admin: '#f97316',
    visitor: '#60a5fa',
};

export const SESSION_STATUS_LABELS: Record<string, string> = {
    active: 'Activa',
    ai_handling: 'Equipo',
    staff_handling: 'Staff',
    closed: 'Cerrada',
};
