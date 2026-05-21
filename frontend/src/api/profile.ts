/* [044A-43] API de perfil de usuario.
 * GET /api/profile — obtiene perfil del usuario autenticado.
 * POST /api/profile/avatar — sube imagen de avatar (multipart).
 * [074A-23] PATCH /api/profile — actualiza display_name y campos extendidos.
 * [205A-2] PUT /api/profile/password — cambia contraseña desde configuración. */
import instance from './axios-instance';

export interface PerfilResponse {
    id: string;
    email: string;
    role: string;
    effective_role: string;
    avatar_url: string | null;
    display_name: string | null;
    created_at: string;
}

export interface AvatarResponse {
    avatar_url: string;
}

export interface UpdateProfilePayload {
    email?: string;
    display_name?: string;
    bio?: string;
    linkedin?: string;
    twitter?: string;
    website?: string;
}

export interface ChangeProfilePasswordPayload {
    current_password: string;
    new_password: string;
}

export async function obtenerPerfil(): Promise<PerfilResponse> {
    const {data} = await instance.get<PerfilResponse>('/api/profile');
    return data;
}

export async function actualizarPerfil(payload: UpdateProfilePayload): Promise<PerfilResponse> {
    const {data} = await instance.patch<PerfilResponse>('/api/profile', payload);
    return data;
}

export async function cambiarPasswordPerfil(payload: ChangeProfilePasswordPayload): Promise<void> {
    await instance.put('/api/profile/password', payload);
}

export async function subirAvatar(archivo: File): Promise<AvatarResponse> {
    const formData = new FormData();
    formData.append('avatar', archivo);
    const {data} = await instance.post<AvatarResponse>('/api/profile/avatar', formData, {
        headers: {'Content-Type': 'multipart/form-data'}
    });
    return data;
}
