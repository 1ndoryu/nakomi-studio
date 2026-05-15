/* [044A-13] Servicio API de autenticación.
 * Conecta con POST /api/auth/login y POST /api/auth/register del backend Rust.
 * [044A-38 Fase 1] Añadido role/effective_role en AuthResponse y switch-role endpoint.
 * [084A-1] Añadido impersonating: al switchear rol, el admin impersona un usuario real.
 * [155A-1] Añadido email en AuthResponse y endpoints Google OAuth. */
import instance from './axios-instance';

export type UserRole = 'admin' | 'employee' | 'client';

export interface AuthResponse {
    token: string;
    user_id: string;
    email: string;
    role: UserRole;
    effective_role: UserRole;
    impersonating: boolean;
    /* [154A-5] true si el usuario necesita establecer contraseña (quick_register) */
    needs_password: boolean;
}

interface ApiError {
    error: string;
    message?: string;
}

export async function apiLogin(email: string, password: string): Promise<AuthResponse> {
    const {data} = await instance.post<AuthResponse>('/api/auth/login', {email, password});
    return data;
}

export async function apiRegister(email: string, password: string): Promise<AuthResponse> {
    const {data} = await instance.post<AuthResponse>('/api/auth/register', {email, password});
    return data;
}

/* [064A-3] Registro rapido solo con email (flujo de compra).
 * El backend genera password aleatorio; el usuario puede cambiarlo desde el panel.
 * Retorna 409 si el email ya existe — el frontend debe pedir password. */
export async function apiQuickRegister(email: string): Promise<AuthResponse> {
    const {data} = await instance.post<AuthResponse>('/api/auth/quick-register', {email});
    return data;
}

/* [044A-38 Fase 1] Cambia el active_role del admin y devuelve nuevo token */
export async function apiSwitchRole(targetRole: UserRole): Promise<AuthResponse> {
    const {data} = await instance.post<AuthResponse>('/api/auth/switch-role', {role: targetRole});
    return data;
}

/* [154A-5] Establece contraseña para usuarios de quick_register.
 * Requiere autenticación (token JWT en header). */
export async function apiSetPassword(password: string): Promise<void> {
    await instance.put('/api/auth/set-password', {password});
}

/* [155A-1] Google OAuth 2.0 — obtiene URL de redirección a Google */
export async function apiGoogleLoginUrl(): Promise<{url: string}> {
    const {data} = await instance.get<{url: string}>('/api/auth/google/url');
    return data;
}

/* [155A-1] Google OAuth 2.0 — intercambia el `code` de Google por JWT */
export async function apiGoogleLogin(code: string): Promise<AuthResponse> {
    const {data} = await instance.post<AuthResponse>('/api/auth/google/login', {code});
    return data;
}

/* [044A-46] Extrae mensaje de error legible desde respuestas del backend.
 * El backend devuelve { error: "tipo", message: "texto legible" }.
 * Prioriza message (legible) sobre error (tipo técnico). */
export function extraerMensajeError(error: unknown): string {
    if (typeof error === 'object' && error !== null && 'response' in error) {
        const resp = (error as { response?: { data?: ApiError; status?: number } }).response;
        if (resp?.data?.message) return resp.data.message;
        if (resp?.data?.error) return resp.data.error;
        if (resp?.status === 401) return 'Credenciales inválidas';
        if (resp?.status === 409) return 'El email ya está registrado';
        if (resp?.status === 422) return 'Datos de formulario inválidos';
    }
    return 'Error de conexión. Intenta de nuevo.';
}
