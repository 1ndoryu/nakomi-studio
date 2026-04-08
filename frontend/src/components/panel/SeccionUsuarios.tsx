/* [054A-1] Sección admin: gestión de usuarios registrados.
 * Búsqueda por email/nombre, filtros por rol y status, paginación.
 * Acciones: cambiar rol, banear/reactivar. */

import { useState } from 'react';
import { Loader2, AlertCircle, Search, Users, Shield, Ban, UserCheck, ChevronDown } from 'lucide-react';
import { useAdminUsers } from '../../hooks/useAdminUsers';
import { ROLE_LABELS, STATUS_LABELS, STATUS_CLASS } from '../../api/admin-users';
import type { AdminUserItem } from '../../api/admin-users';
import { Input } from '../ui/Input';
import { Modal } from '../ui/Modal';
import { Button } from '../ui/Button';
import { MenuContextual, type MenuContextualItem } from '../ui/ContextMenu';
import './SeccionUsuarios.css';

export function SeccionUsuarios() {
    const {
        users, total, page, totalPages, isLoading, error,
        search, roleFilter, statusFilter,
        setSearch, setRoleFilter, setStatusFilter, setPage,
        changeRole, changeStatus, isChangingRole, isChangingStatus,
    } = useAdminUsers();

    /* [064A-17] Menus de filtro personalizados en vez de <select> nativo */
    const [rolMenuAbierto, setRolMenuAbierto] = useState(false);
    const [statusMenuAbierto, setStatusMenuAbierto] = useState(false);

    const [confirmAction, setConfirmAction] = useState<{
        userId: string;
        type: 'role' | 'status';
        value: string;
        email: string;
    } | null>(null);

    const handleConfirm = () => {
        if (!confirmAction) return;
        if (confirmAction.type === 'role') {
            changeRole({ userId: confirmAction.userId, role: confirmAction.value });
        } else {
            changeStatus({ userId: confirmAction.userId, status: confirmAction.value });
        }
        setConfirmAction(null);
    };

    if (isLoading) {
        return (
            <div className="usuariosVacio">
                <Loader2 className="usuariosSpinner" size={32} />
            </div>
        );
    }

    if (error) {
        return (
            <div className="usuariosError">
                <AlertCircle size={20} />
                <span>Error al cargar usuarios</span>
            </div>
        );
    }

    return (
        <div className="usuariosContenedor">
            {/* [064A-17] Filtros con MenuContextual personalizado */}
            <div className="usuariosFiltros">
                <div className="usuariosBusqueda">
                    <Search size={16} className="usuariosBusquedaIcono" />
                    <Input
                        type="text"
                        className="usuariosBusquedaInput"
                        placeholder="Buscar por email o nombre..."
                        value={search}
                        onChange={(e) => setSearch(e.target.value)}
                    />
                </div>

                <MenuContextual
                    abierto={rolMenuAbierto}
                    onToggle={() => setRolMenuAbierto(prev => !prev)}
                    onCerrar={() => setRolMenuAbierto(false)}
                    ariaLabel="Filtrar por rol"
                    triggerClassName="usuariosFiltroBtn"
                    triggerVariante="outline"
                    triggerTamano="pequeno"
                    triggerContent={<>{roleFilter ? ROLE_LABELS[roleFilter] : 'Todos los roles'} <ChevronDown size={14} /></>}
                    items={[
                        {id: 'all', label: 'Todos los roles', onSelect: () => setRoleFilter('')},
                        {id: 'admin', label: 'Admin', onSelect: () => setRoleFilter('admin')},
                        {id: 'employee', label: 'Empleado', onSelect: () => setRoleFilter('employee')},
                        {id: 'client', label: 'Cliente', onSelect: () => setRoleFilter('client')},
                    ]}
                />

                <MenuContextual
                    abierto={statusMenuAbierto}
                    onToggle={() => setStatusMenuAbierto(prev => !prev)}
                    onCerrar={() => setStatusMenuAbierto(false)}
                    ariaLabel="Filtrar por status"
                    triggerClassName="usuariosFiltroBtn"
                    triggerVariante="outline"
                    triggerTamano="pequeno"
                    triggerContent={<>{statusFilter ? STATUS_LABELS[statusFilter] : 'Todos los status'} <ChevronDown size={14} /></>}
                    items={[
                        {id: 'all', label: 'Todos los status', onSelect: () => setStatusFilter('')},
                        {id: 'active', label: 'Activo', onSelect: () => setStatusFilter('active')},
                        {id: 'banned', label: 'Baneado', onSelect: () => setStatusFilter('banned')},
                        {id: 'suspended', label: 'Suspendido', onSelect: () => setStatusFilter('suspended')},
                    ]}
                />

                <span className="usuariosTotal">
                    <Users size={14} />
                    {total} usuario{total !== 1 ? 's' : ''}
                </span>
            </div>

            {/* Tabla de usuarios */}
            {users.length === 0 ? (
                <div className="usuariosVacio">
                    <Users size={32} />
                    <p>No se encontraron usuarios</p>
                </div>
            ) : (
                <div className="usuariosTablaWrapper">
                    <table className="usuariosTabla">
                        <thead>
                            <tr>
                                <th>Usuario</th>
                                <th>Rol</th>
                                <th>Status</th>
                                <th>Registrado</th>
                                <th>Acciones</th>
                            </tr>
                        </thead>
                        <tbody>
                            {users.map((u) => (
                                <UserRow
                                    key={u.id}
                                    user={u}
                                    onChangeRole={(role) => setConfirmAction({
                                        userId: u.id, type: 'role', value: role, email: u.email,
                                    })}
                                    onChangeStatus={(status) => setConfirmAction({
                                        userId: u.id, type: 'status', value: status, email: u.email,
                                    })}
                                />
                            ))}
                        </tbody>
                    </table>
                </div>
            )}

            {/* Paginación */}
            {totalPages > 1 && (
                <div className="usuariosPaginacion">
                    <Button
                        className="usuariosPaginacionBtn"
                        variante="outline"
                        tamano="pequeno"
                        disabled={page <= 1}
                        onClick={() => setPage(page - 1)}
                        type="button"
                    >
                        ← Anterior
                    </Button>
                    <span className="usuariosPaginacionInfo">
                        Página {page} de {totalPages}
                    </span>
                    <Button
                        className="usuariosPaginacionBtn"
                        variante="outline"
                        tamano="pequeno"
                        disabled={page >= totalPages}
                        onClick={() => setPage(page + 1)}
                        type="button"
                    >
                        Siguiente →
                    </Button>
                </div>
            )}

            {/* Modal de confirmación */}
            <Modal abierto={!!confirmAction} onCerrar={() => setConfirmAction(null)} className="usuariosModal">
                <h3 className="modalTitulo">Confirmar acción</h3>
                <p className="usuariosModalTexto">
                    {confirmAction?.type === 'role'
                        ? `¿Cambiar rol de ${confirmAction.email} a ${ROLE_LABELS[confirmAction.value] || confirmAction.value}?`
                        : `¿Cambiar status de ${confirmAction?.email} a ${STATUS_LABELS[confirmAction?.value ?? ''] || confirmAction?.value}?`
                    }
                </p>
                <div className="modalAcciones">
                    <Button
                        variante="secundario"
                        tamano="pequeno"
                        onClick={() => setConfirmAction(null)}
                        type="button"
                    >
                        Cancelar
                    </Button>
                    <Button
                        variante="primario"
                        tamano="pequeno"
                        onClick={handleConfirm}
                        disabled={isChangingRole || isChangingStatus}
                        type="button"
                    >
                        {(isChangingRole || isChangingStatus) ? 'Procesando...' : 'Confirmar'}
                    </Button>
                </div>
            </Modal>
        </div>
    );
}

/* Fila individual de usuario con menú de acciones */
function UserRow({ user, onChangeRole, onChangeStatus }: {
    user: AdminUserItem;
    onChangeRole: (role: string) => void;
    onChangeStatus: (status: string) => void;
}) {
    const [menuAbierto, setMenuAbierto] = useState(false);

    const rolesDisponibles = ['admin', 'employee', 'client'].filter(r => r !== user.role);
    const isBanned = user.status === 'banned';

    const menuItems: MenuContextualItem[] = [
        ...rolesDisponibles.map(r => ({
            id: `role-${r}`,
            label: `Cambiar a ${ROLE_LABELS[r]}`,
            icon: <Shield size={14} />,
            onSelect: () => onChangeRole(r),
        })),
        isBanned
            ? {
                id: 'reactivar',
                label: 'Reactivar',
                icon: <UserCheck size={14} />,
                onSelect: () => onChangeStatus('active'),
            }
            : {
                id: 'banear',
                label: 'Banear',
                icon: <Ban size={14} />,
                onSelect: () => onChangeStatus('banned'),
                danger: true,
            },
    ];

    return (
        <tr className="usuariosFila">
            <td className="usuariosCeldaUsuario">
                <div className="usuariosAvatar">
                    {user.avatar_url
                        ? <img src={user.avatar_url} alt="" className="usuariosAvatarImg" loading="lazy" />
                        : <span className="usuariosAvatarPlaceholder">
                            {(user.display_name || user.email)[0].toUpperCase()}
                          </span>
                    }
                </div>
                <div className="usuariosInfo">
                    <span className="usuariosNombre">{user.display_name || '—'}</span>
                    <span className="usuariosEmail">{user.email}</span>
                </div>
            </td>
            <td>
                <span className={`usuariosRolBadge usuariosRol${user.role}`}>
                    {ROLE_LABELS[user.role] || user.role}
                </span>
            </td>
            <td>
                <span className={`usuariosStatusBadge ${STATUS_CLASS[user.status] || ''}`}>
                    {STATUS_LABELS[user.status] || user.status}
                </span>
            </td>
            <td className="usuariosFecha">
                {new Date(user.created_at).toLocaleDateString('es-ES', {
                    day: 'numeric', month: 'short', year: 'numeric',
                })}
            </td>
            <td className="usuariosAcciones">
                <MenuContextual
                    abierto={menuAbierto}
                    onToggle={() => setMenuAbierto(prev => !prev)}
                    onCerrar={() => setMenuAbierto(false)}
                    items={menuItems}
                    ariaLabel="Acciones del usuario"
                    triggerClassName="usuariosMenuBtn"
                />
            </td>
        </tr>
    );
}
