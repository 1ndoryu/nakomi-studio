/* [054A-1] Sección admin: gestión de usuarios registrados.
 * Búsqueda por email/nombre, filtros por rol y status, paginación.
 * Acciones: cambiar rol, banear/reactivar.
 * [015A-1] Botón "Nuevo usuario" con modal de creación: email, contraseña, rol. */

import { useState, type FormEvent } from 'react';
import { Loader2, AlertCircle, Search, Users, Shield, Ban, UserCheck, ChevronDown, Trash2, UserPlus } from 'lucide-react';
import { useUsersSection } from '../../hooks/useUsersSection';
import { ROLE_LABELS, STATUS_LABELS, STATUS_CLASS } from '../../api/admin-users';
import type { AdminUserItem } from '../../api/admin-users';
import { Input } from '../ui/Input';
import { Modal } from '../ui/Modal';
import { Button } from '../ui/Button';
import OptimizedImage from '../ui/OptimizedImage';
import { MenuContextual, type MenuContextualItem } from '../ui/ContextMenu';
import './SeccionUsuarios.css';
import './UsuariosAcciones.css';

export function SeccionUsuarios() {
    const {
        users, total, page, totalPages, isLoading, error,
        search, roleFilter, statusFilter,
        setSearch, setRoleFilter, setStatusFilter, setPage,
        currentUserId,
        rolMenuAbierto,
        statusMenuAbierto,
        confirmAction,
        modalError,
        modalCrear,
        isProcessing,
        setRolMenuAbierto,
        setStatusMenuAbierto,
        openRoleConfirm,
        openStatusConfirm,
        openDeleteConfirm,
        closeConfirm,
        handleConfirm,
        openCreateModal,
        closeCreateModal,
        createUser,
        isCreatingUser,
    } = useUsersSection();

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

                <Button
                    variante="primario"
                    tamano="pequeno"
                    onClick={openCreateModal}
                    type="button"
                    className="usuariosNuevoBtn"
                >
                    <UserPlus size={14} />
                    Nuevo usuario
                </Button>
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
                                    onChangeRole={(role) => openRoleConfirm(u.id, u.email, role)}
                                    onChangeStatus={(status) => openStatusConfirm(u.id, u.email, status)}
                                    onDelete={() => openDeleteConfirm(u.id, u.email)}
                                    canDelete={u.id !== currentUserId}
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

            {/* [015A-1] Modal de confirmación de acción */}
            <Modal abierto={!!confirmAction} onCerrar={closeConfirm} className="usuariosModal">
                <h3 className="modalTitulo">Confirmar acción</h3>
                <p className="usuariosModalTexto">
                    {confirmAction?.type === 'role'
                        ? `¿Cambiar rol de ${confirmAction.email} a ${ROLE_LABELS[confirmAction.value] || confirmAction.value}?`
                        : confirmAction?.type === 'status'
                            ? `¿Cambiar status de ${confirmAction.email} a ${STATUS_LABELS[confirmAction.value] || confirmAction.value}?`
                            : `¿Eliminar permanentemente a ${confirmAction?.email}? Solo se podrá borrar si no tiene pedidos, hosting, chats u otras relaciones activas.`
                    }
                </p>
                {modalError && (
                    <div className="usuariosError usuariosError--modal">
                        <AlertCircle size={16} />
                        <span>{modalError}</span>
                    </div>
                )}
                <div className="modalAcciones">
                    <Button
                        variante="secundario"
                        tamano="pequeno"
                        onClick={closeConfirm}
                        disabled={isProcessing}
                        type="button"
                    >
                        Cancelar
                    </Button>
                    <Button
                        variante="primario"
                        tamano="pequeno"
                        onClick={() => void handleConfirm()}
                        disabled={isProcessing}
                        type="button"
                    >
                        {isProcessing ? 'Procesando...' : 'Confirmar'}
                    </Button>
                </div>
            </Modal>

            {/* [015A-1] Modal para crear nuevo usuario */}
            <Modal abierto={modalCrear} onCerrar={closeCreateModal} className="usuariosModal">
                <ModalCrearUsuario
                    onClose={closeCreateModal}
                    onSubmit={createUser}
                    isCreating={isCreatingUser}
                    onCreated={() => setPage(1)}
                />
            </Modal>
        </div>
    );
}

/* Fila individual de usuario con menú de acciones */
function UserRow({ user, onChangeRole, onChangeStatus, onDelete, canDelete }: {
    user: AdminUserItem;
    onChangeRole: (role: string) => void;
    onChangeStatus: (status: string) => void;
    onDelete: () => void;
    canDelete: boolean;
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

    if (canDelete) {
        menuItems.push({
            id: 'eliminar',
            label: 'Eliminar usuario',
            icon: <Trash2 size={14} />,
            onSelect: onDelete,
            danger: true,
        });
    }

    return (
        <tr className="usuariosFila">
            <td className="usuariosCeldaUsuario">
                <div className="usuariosAvatar">
                    {user.avatar_url
                        ? <OptimizedImage src={user.avatar_url} alt="" className="usuariosAvatarImg" loading="lazy" />
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

/* [015A-1] Modal para crear un nuevo usuario desde el panel admin.
 * Form state local (email, password, role). Solo cierra en éxito; muestra error inline.
 * Gotcha: no usar estado global para el form — evita contaminar useUsersSection. */
function ModalCrearUsuario({ onClose, onSubmit, isCreating, onCreated }: {
    onClose: () => void;
    onSubmit: (payload: { email: string; password: string; role?: 'admin' | 'employee' | 'client' }) => Promise<unknown>;
    isCreating: boolean;
    onCreated: () => void;
}) {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [role, setRole] = useState<'admin' | 'employee' | 'client'>('client');
    const [error, setError] = useState<string | null>(null);
    const [rolMenuAbierto, setRolMenuAbierto] = useState(false);

    const handleSubmit = async (e: FormEvent) => {
        e.preventDefault();
        setError(null);
        try {
            await onSubmit({ email, password, role });
            onCreated();
            onClose();
        } catch (err: unknown) {
            if (typeof err === 'object' && err !== null) {
                const msg = (err as {response?: {data?: {message?: string}}}).response?.data?.message;
                setError(typeof msg === 'string' && msg.trim() ? msg : 'No se pudo crear el usuario.');
            } else {
                setError(err instanceof Error ? err.message : 'No se pudo crear el usuario.');
            }
        }
    };

    return (
        <form onSubmit={(e) => void handleSubmit(e)}>
            <div className="usuariosCrearCampo">
                <label className="usuariosCrearLabel" htmlFor="crear-email">Email</label>
                <Input
                    id="crear-email"
                    type="email"
                    value={email}
                    onChange={e => setEmail(e.target.value)}
                    placeholder="usuario@ejemplo.com"
                    required
                    disabled={isCreating}
                />
            </div>

            <div className="usuariosCrearCampo">
                <label className="usuariosCrearLabel" htmlFor="crear-password">Contraseña</label>
                <Input
                    id="crear-password"
                    type="password"
                    value={password}
                    onChange={e => setPassword(e.target.value)}
                    placeholder="Mínimo 8 caracteres"
                    minLength={8}
                    required
                    disabled={isCreating}
                />
            </div>

            <div className="usuariosCrearCampo">
                <label className="usuariosCrearLabel">Rol</label>
                <MenuContextual
                    abierto={rolMenuAbierto}
                    onToggle={() => setRolMenuAbierto(prev => !prev)}
                    onCerrar={() => setRolMenuAbierto(false)}
                    ariaLabel="Seleccionar rol"
                    triggerVariante="outline"
                    triggerTamano="pequeno"
                    triggerClassName="usuariosCrearRolBtn"
                    triggerContent={<>{ROLE_LABELS[role]} <ChevronDown size={14} /></>}
                    items={[
                        { id: 'client', label: 'Cliente', onSelect: () => setRole('client') },
                        { id: 'employee', label: 'Empleado', onSelect: () => setRole('employee') },
                        { id: 'admin', label: 'Admin', onSelect: () => setRole('admin') },
                    ]}
                />
            </div>

            {error && (
                <div className="usuariosError usuariosError--modal">
                    <AlertCircle size={16} />
                    <span>{error}</span>
                </div>
            )}

            <div className="modalAcciones">
                <Button
                    variante="secundario"
                    tamano="pequeno"
                    onClick={onClose}
                    disabled={isCreating}
                    type="button"
                >
                    Cancelar
                </Button>
                <Button
                    variante="primario"
                    tamano="pequeno"
                    disabled={isCreating}
                    type="submit"
                >
                    {isCreating ? 'Creando...' : 'Crear usuario'}
                </Button>
            </div>
        </form>
    );
}
