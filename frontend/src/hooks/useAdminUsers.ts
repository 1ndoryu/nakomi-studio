/* [054A-1] Hook para gestión de usuarios admin.
 * Búsqueda, paginación, cambio de rol y status con React Query. */

import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import {
  apiChangeRole,
  apiChangeStatus,
  apiDeleteUser,
  apiListUsers,
  type ListUsersParams,
  type PaginatedUsers,
} from '../api/admin-users';

const USERS_KEY = 'admin-users';

export function useAdminUsers() {
  const queryClient = useQueryClient();

  const [page, setPage] = useState(1);
  const [search, setSearch] = useState('');
  const [roleFilter, setRoleFilter] = useState('');
  const [statusFilter, setStatusFilter] = useState('');

  const params: ListUsersParams = {
    page,
    per_page: 15,
    ...(search && { search }),
    ...(roleFilter && { role: roleFilter }),
    ...(statusFilter && { status: statusFilter }),
  };

  const {
    data,
    isLoading,
    error,
  } = useQuery<PaginatedUsers>({
    queryKey: [USERS_KEY, params],
    queryFn: () => apiListUsers(params),
  });

  const changeRoleMut = useMutation({
    mutationFn: ({ userId, role }: { userId: string; role: string }) =>
      apiChangeRole(userId, role),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [USERS_KEY] });
    },
  });

  const changeStatusMut = useMutation({
    mutationFn: ({ userId, status }: { userId: string; status: string }) =>
      apiChangeStatus(userId, status),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [USERS_KEY] });
    },
  });

  const deleteUserMut = useMutation({
    mutationFn: ({ userId }: { userId: string }) => apiDeleteUser(userId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [USERS_KEY] });
    },
  });

  const totalPages = data ? Math.ceil(data.total / data.per_page) : 0;

  return {
    users: data?.users ?? [],
    total: data?.total ?? 0,
    page,
    totalPages,
    isLoading,
    error,
    search,
    roleFilter,
    statusFilter,
    setPage,
    setSearch: (val: string) => { setSearch(val); setPage(1); },
    setRoleFilter: (val: string) => { setRoleFilter(val); setPage(1); },
    setStatusFilter: (val: string) => { setStatusFilter(val); setPage(1); },
    changeRole: changeRoleMut.mutateAsync,
    changeStatus: changeStatusMut.mutateAsync,
    deleteUser: deleteUserMut.mutateAsync,
    isChangingRole: changeRoleMut.isPending,
    isChangingStatus: changeStatusMut.isPending,
    isDeletingUser: deleteUserMut.isPending,
  };
}
