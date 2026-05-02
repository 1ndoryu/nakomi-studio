/* [054A-1] API client para gestión de usuarios desde panel admin.
 * Endpoints: listar paginado con búsqueda/filtros, cambiar rol, cambiar status. */

import axiosInstance from './axios-instance';

/* Types */

export interface AdminUserItem {
  id: string;
  email: string;
  role: 'admin' | 'employee' | 'client';
  status: string;
  email_verified: boolean;
  avatar_url: string | null;
  display_name: string | null;
  created_at: string;
}

export interface PaginatedUsers {
  users: AdminUserItem[];
  total: number;
  page: number;
  per_page: number;
}

export interface ListUsersParams {
  page?: number;
  per_page?: number;
  search?: string;
  role?: string;
  status?: string;
}

/* API functions */

export async function apiListUsers(params: ListUsersParams = {}): Promise<PaginatedUsers> {
  const { data } = await axiosInstance.get('/api/admin/users', { params });
  return data;
}

export async function apiChangeRole(userId: string, role: string): Promise<void> {
  await axiosInstance.patch(`/api/admin/users/${userId}/role`, { role });
}

export async function apiChangeStatus(userId: string, status: string): Promise<void> {
  await axiosInstance.patch(`/api/admin/users/${userId}/status`, { status });
}

export async function apiDeleteUser(userId: string): Promise<void> {
  await axiosInstance.delete(`/api/admin/users/${userId}`);
}

/* [015A-1] Crear usuario desde panel admin */
export interface AdminCreateUserPayload {
  email: string;
  password: string;
  role?: 'admin' | 'employee' | 'client';
}

export async function apiCreateUser(payload: AdminCreateUserPayload): Promise<AdminUserItem> {
  const { data } = await axiosInstance.post('/api/admin/users', payload);
  return data;
}

/* Constantes de UI */

export const ROLE_LABELS: Record<string, string> = {
  admin: 'Admin',
  employee: 'Empleado',
  client: 'Cliente',
};

export const STATUS_LABELS: Record<string, string> = {
  active: 'Activo',
  banned: 'Baneado',
  suspended: 'Suspendido',
};

export const STATUS_CLASS: Record<string, string> = {
  active: 'statusActivo',
  banned: 'statusBaneado',
  suspended: 'statusSuspendido',
};
