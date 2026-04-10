/* [044A-13] Store de autenticación con Zustand.
 * Maneja token JWT, estado de sesión y persistencia en localStorage.
 * El token se guarda en localStorage para que axios-instance lo inyecte automáticamente.
 * [044A-38 Fase 1] Añadido role/effectiveRole para panel dinámico por rol.
 * [084A-1] Añadido impersonating + userId dinámico para impersonación real.
 * [154A-5] Añadido needsPassword para usuarios de quick_register sin contraseña propia. */
import {create} from 'zustand';
import type {UserRole} from '../api/auth';

interface AuthUser {
    userId: string;
    email: string;
    role: UserRole;
    effectiveRole: UserRole;
    impersonating: boolean;
    /* [154A-5] true si el usuario necesita establecer contraseña */
    needsPassword: boolean;
}

interface AuthState {
    token: string | null;
    user: AuthUser | null;
    logueado: boolean;
    login: (token: string, userId: string, email: string, role: UserRole, effectiveRole: UserRole, needsPassword?: boolean) => void;
    logout: () => void;
    inicializar: () => void;
    /* [084A-1] Actualiza token, roles, userId y estado de impersonación tras switch-role */
    actualizarRol: (token: string, userId: string, role: UserRole, effectiveRole: UserRole, impersonating: boolean) => void;
    /* [154A-5] Marca que el usuario ya estableció su contraseña */
    marcarPasswordEstablecida: () => void;
}

export const useAuthStore = create<AuthState>((set, get) => ({
    token: null,
    user: null,
    logueado: false,

    login: (token, userId, email, role, effectiveRole, needsPassword = false) => {
        localStorage.setItem('token', token);
        localStorage.setItem('auth_user', JSON.stringify({userId, email, role, effectiveRole, impersonating: false, needsPassword}));
        set({token, user: {userId, email, role, effectiveRole, impersonating: false, needsPassword}, logueado: true});
    },

    logout: () => {
        localStorage.removeItem('token');
        localStorage.removeItem('auth_user');
        set({token: null, user: null, logueado: false});
    },

    /* Restaura sesión desde localStorage al cargar la app */
    inicializar: () => {
        const token = localStorage.getItem('token');
        const userData = localStorage.getItem('auth_user');
        if (token && userData) {
            try {
                const user = JSON.parse(userData) as AuthUser;
                /* [044A-38 Fase 1] Backward compat: si no tiene role, asumir client */
                if (!user.role) {
                    user.role = 'client';
                    user.effectiveRole = 'client';
                }
                if (user.impersonating === undefined) {
                    user.impersonating = false;
                }
                if (user.needsPassword === undefined) {
                    user.needsPassword = false;
                }
                set({token, user, logueado: true});
            } catch {
                localStorage.removeItem('token');
                localStorage.removeItem('auth_user');
            }
        }
    },

    /* [084A-1] Switch role: actualiza token, userId, roles e impersonación */
    actualizarRol: (token, userId, role, effectiveRole, impersonating) => {
        const currentUser = get().user;
        if (!currentUser) return;
        const updatedUser = {...currentUser, userId, role, effectiveRole, impersonating};
        localStorage.setItem('token', token);
        localStorage.setItem('auth_user', JSON.stringify(updatedUser));
        set({token, user: updatedUser});
    },

    /* [154A-5] Marca que el usuario ya estableció su contraseña */
    marcarPasswordEstablecida: () => {
        const currentUser = get().user;
        if (!currentUser) return;
        const updatedUser = {...currentUser, needsPassword: false};
        localStorage.setItem('auth_user', JSON.stringify(updatedUser));
        set({user: updatedUser});
    },
}));
