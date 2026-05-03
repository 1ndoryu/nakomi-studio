import type {UserRole} from '../api/auth';
import type {NotificationResponse} from '../api/notifications';
import {seccionInicialPorRol, type SeccionPanel} from '../data/panel';

const PROJECT_SECTIONS = new Set<SeccionPanel>(['proyectos', 'asignados', 'todos-ordenes']);

function readSearchParams(): URLSearchParams {
    return new URLSearchParams(window.location.search);
}

function replaceSearch(mutator: (params: URLSearchParams) => void): void {
    const url = new URL(window.location.href);
    mutator(url.searchParams);
    const next = `${url.pathname}${url.search}${url.hash}`;
    window.history.replaceState({}, '', next);
}

export function getPanelOrderIdFromUrl(): string | null {
    const params = readSearchParams();
    return params.get('order')
        ?? (params.get('seccion') === 'ordenes' ? params.get('id') : null);
}

export function getPanelHostingIdFromUrl(): string | null {
    const params = readSearchParams();
    return params.get('hostingId')
        ?? ((params.get('seccion') === 'hosting' || params.get('seccion') === 'hostings')
            ? params.get('id')
            : null);
}

export function getPanelChatIdFromUrl(): string | null {
    const url = new URL(window.location.href);
    return url.searchParams.get('chat')
        ?? (url.pathname === '/panel/chat' ? url.searchParams.get('session') : null);
}

export function resolvePanelSectionFromUrl(
    role: UserRole,
    validSections: readonly SeccionPanel[],
): SeccionPanel | null {
    if (getPanelOrderIdFromUrl()) return seccionInicialPorRol(role);
    if (getPanelHostingIdFromUrl()) return 'hosting';
    if (getPanelChatIdFromUrl()) return 'mensajes';

    const seccion = readSearchParams().get('seccion') as SeccionPanel | null;
    if (seccion && validSections.includes(seccion)) return seccion;
    return null;
}

export function syncPanelSectionInUrl(seccion: SeccionPanel): void {
    replaceSearch(params => {
        params.set('seccion', seccion);
        params.delete('id');

        if (!PROJECT_SECTIONS.has(seccion)) {
            params.delete('order');
        }
        if (seccion !== 'hosting') {
            params.delete('hostingId');
        }
        if (seccion !== 'mensajes') {
            params.delete('chat');
        }
    });
}

export function syncPanelOrderInUrl(orderId: string | null, seccion: SeccionPanel): void {
    replaceSearch(params => {
        params.set('seccion', seccion);
        params.delete('id');
        if (orderId) {
            params.set('order', orderId);
        } else {
            params.delete('order');
        }
    });
}

export function syncPanelHostingInUrl(hostingId: string | null): void {
    replaceSearch(params => {
        params.set('seccion', 'hosting');
        params.delete('id');
        if (hostingId) {
            params.set('hostingId', hostingId);
        } else {
            params.delete('hostingId');
        }
    });
}

export function syncPanelChatInUrl(chatId: string | null): void {
    replaceSearch(params => {
        params.set('seccion', 'mensajes');
        if (chatId) {
            params.set('chat', chatId);
        } else {
            params.delete('chat');
        }
    });
}

function normalizePanelLink(rawLink: string): string {
    const url = new URL(rawLink, window.location.origin);
    const sessionId = url.searchParams.get('session');
    const legacySection = url.searchParams.get('seccion');
    const legacyId = url.searchParams.get('id');

    if (url.pathname === '/panel/chat') {
        return sessionId ? `/panel?seccion=mensajes&chat=${sessionId}` : '/panel?seccion=mensajes';
    }

    if (url.pathname === '/panel' && legacySection === 'ordenes' && legacyId) {
        return `/panel?order=${legacyId}`;
    }

    if (url.pathname === '/panel' && legacySection === 'hosting' && legacyId) {
        return `/panel?seccion=hosting&hostingId=${legacyId}`;
    }

    return `${url.pathname}${url.search}${url.hash}`;
}

export function buildPanelNotificationTarget(notification: NotificationResponse): string | null {
    if (notification.link) {
        return normalizePanelLink(notification.link);
    }

    switch (notification.reference_type) {
        case 'order':
            return notification.reference_id ? `/panel?order=${notification.reference_id}` : '/panel';
        case 'hosting_subscription':
            return notification.reference_id
                ? `/panel?seccion=hosting&hostingId=${notification.reference_id}`
                : '/panel?seccion=hosting';
        case 'chat_session':
            return notification.reference_id
                ? `/panel?seccion=mensajes&chat=${notification.reference_id}`
                : '/panel?seccion=mensajes';
        case 'refund':
            return '/panel?seccion=reembolsos';
        case 'withdrawal':
            return '/panel?seccion=retiros';
        case 'order_problem':
            return '/panel?seccion=problemas';
        case 'delegation':
            return '/panel?seccion=delegaciones';
        default:
            return null;
    }
}