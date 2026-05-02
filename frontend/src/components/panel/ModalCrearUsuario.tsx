/* [015A-1+] ModalCrearUsuario extraido de SeccionUsuarios.tsx para cumplir limite de 300 lineas.
 * Form state local (email, password, role). Solo cierra en éxito; muestra error inline.
 * Gotcha: no usar estado global para el form — evita contaminar useUsersSection. */

import { useState, type FormEvent } from 'react';
import { AlertCircle, ChevronDown } from 'lucide-react';
import { MenuContextual } from '../ui/ContextMenu';
import { ROLE_LABELS } from '../../api/admin-users';
import { Input } from '../ui/Input';
import { Button } from '../ui/Button';
import './SeccionUsuarios.css';

export function ModalCrearUsuario({ onClose, onSubmit, isCreating, onCreated }: {
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
