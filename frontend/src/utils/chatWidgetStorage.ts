import type {ChatMessage} from '../api/chat';

export const CHAT_VISITOR_ID_KEY = 'nakomi_visitor_id';
export const CHAT_SESSION_ID_KEY = 'nakomi_chat_session_id';
export const CHAT_OWNER_KEY = 'nakomi_chat_owner_key';
export const ANONYMOUS_CHAT_OWNER = 'anonymous';

const CHAT_MESSAGES_KEY = 'nakomi_chat_messages';
const MAX_PERSISTED_MESSAGES = 100;

interface PersistedChatMessages {
    sessionId: string;
    messages: ChatMessage[];
}

function canUseStorage(): boolean {
    return typeof window !== 'undefined' && typeof window.localStorage !== 'undefined';
}

function isChatMessage(value: unknown): value is ChatMessage {
    if (!value || typeof value !== 'object') return false;
    const message = value as Partial<ChatMessage>;
    return typeof message.id === 'string'
        && typeof message.session_id === 'string'
        && typeof message.sender_type === 'string'
        && typeof message.content === 'string'
        && typeof message.created_at === 'string';
}

function clearChatSessionData(): void {
    localStorage.removeItem(CHAT_VISITOR_ID_KEY);
    localStorage.removeItem(CHAT_SESSION_ID_KEY);
    localStorage.removeItem(CHAT_MESSAGES_KEY);
}

export function ensureChatStorageOwner(ownerKey: string): boolean {
    if (!canUseStorage()) return false;
    const normalizedOwner = ownerKey || ANONYMOUS_CHAT_OWNER;
    const savedOwner = localStorage.getItem(CHAT_OWNER_KEY);
    if (savedOwner === normalizedOwner) return false;
    clearChatSessionData();
    localStorage.setItem(CHAT_OWNER_KEY, normalizedOwner);
    return true;
}

export function getOrCreateChatVisitorId(ownerKey = ANONYMOUS_CHAT_OWNER): string {
    if (!canUseStorage()) return crypto.randomUUID();
    ensureChatStorageOwner(ownerKey);
    const saved = localStorage.getItem(CHAT_VISITOR_ID_KEY);
    if (saved) return saved;
    const id = crypto.randomUUID();
    localStorage.setItem(CHAT_VISITOR_ID_KEY, id);
    return id;
}

export function getSavedChatSessionId(): string | null {
    if (!canUseStorage()) return null;
    return localStorage.getItem(CHAT_SESSION_ID_KEY);
}

export function saveChatSessionId(id: string): void {
    if (!canUseStorage()) return;
    localStorage.setItem(CHAT_SESSION_ID_KEY, id);
}

export function loadPersistedChatMessages(sessionId: string | null): ChatMessage[] {
    if (!sessionId || !canUseStorage()) return [];
    try {
        const raw = localStorage.getItem(CHAT_MESSAGES_KEY);
        if (!raw) return [];
        const parsed = JSON.parse(raw) as Partial<PersistedChatMessages>;
        if (parsed.sessionId !== sessionId || !Array.isArray(parsed.messages)) return [];
        return parsed.messages.filter(isChatMessage).slice(-MAX_PERSISTED_MESSAGES);
    } catch {
        localStorage.removeItem(CHAT_MESSAGES_KEY);
        return [];
    }
}

export function savePersistedChatMessages(sessionId: string | null, messages: ChatMessage[]): void {
    if (!sessionId || !canUseStorage()) return;
    const payload: PersistedChatMessages = {
        sessionId,
        messages: messages.slice(-MAX_PERSISTED_MESSAGES),
    };
    localStorage.setItem(CHAT_MESSAGES_KEY, JSON.stringify(payload));
}

export function clearChatWidgetStorage(ownerKey?: string): void {
    if (!canUseStorage()) return;
    clearChatSessionData();
    if (ownerKey) {
        localStorage.setItem(CHAT_OWNER_KEY, ownerKey);
    } else {
        localStorage.removeItem(CHAT_OWNER_KEY);
    }
}