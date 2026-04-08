/* [074A-13] API client para CMS de equipo.
 * Mismo patrón que admin-projects.ts. */

import instance from './axios-instance';

export interface AdminTeamMember {
    id: string;
    name: string;
    slug: string;
    role: string;
    bio: string;
    avatar: string | null;
    linkedin: string | null;
    twitter: string | null;
    github: string | null;
    status: string;
    sort_order: number;
    created_at: string;
    updated_at: string;
}

export interface CreateTeamMemberBody {
    name: string;
    slug: string;
    role?: string;
    bio?: string;
    avatar?: string;
    linkedin?: string;
    twitter?: string;
    github?: string;
    status?: string;
    sort_order?: number;
}

export interface UpdateTeamMemberBody {
    name?: string;
    slug?: string;
    role?: string;
    bio?: string;
    avatar?: string;
    linkedin?: string;
    twitter?: string;
    github?: string;
    status?: string;
    sort_order?: number;
}

export async function apiListAdminTeamMembers(): Promise<AdminTeamMember[]> {
    const { data } = await instance.get<AdminTeamMember[]>('/api/admin/team');
    return data;
}

export async function apiCreateTeamMember(body: CreateTeamMemberBody): Promise<AdminTeamMember> {
    const { data } = await instance.post<AdminTeamMember>('/api/admin/team', body);
    return data;
}

export async function apiUpdateTeamMember(id: string, body: UpdateTeamMemberBody): Promise<AdminTeamMember> {
    const { data } = await instance.put<AdminTeamMember>(`/api/admin/team/${id}`, body);
    return data;
}

export async function apiArchiveTeamMember(id: string): Promise<void> {
    await instance.delete(`/api/admin/team/${id}`);
}

/* [084A-10] Eliminación permanente de un miembro de equipo */
export async function apiDestroyTeamMember(id: string): Promise<void> {
    await instance.post(`/api/admin/team/${id}/destroy`);
}

export async function apiListPublicTeamMembers(): Promise<AdminTeamMember[]> {
    const { data } = await instance.get<AdminTeamMember[]>('/api/team');
    return data;
}
