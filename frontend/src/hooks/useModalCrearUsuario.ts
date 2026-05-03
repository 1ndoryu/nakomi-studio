/* [15A-SENT-1] Hook extraído de ModalCrearUsuario: gestiona los 5 useState del form.
 * Extracción requerida por sentinel usestate-excesivo (max 3 por componente).
 * Gotcha: onSubmit es async genérico — el tipo de error se normaliza aquí para no
 * contaminar el componente con manejo de errores de red.
 * [035A-23] Se eliminó FormEvent — handleSubmit ahora sin args para que el componente
 * no necesite un <form> (evita inconsistencia visual con otros modales). */

import { useState } from 'react';

type Role = 'admin' | 'employee' | 'client';

interface Opciones {
    onSubmit: (payload: { email: string; password: string; role?: Role }) => Promise<unknown>;
    onClose: () => void;
    onCreated: () => void;
}

export function useModalCrearUsuario({ onSubmit, onClose, onCreated }: Opciones) {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [role, setRole] = useState<Role>('client');
    const [error, setError] = useState<string | null>(null);
    const [rolMenuAbierto, setRolMenuAbierto] = useState(false);

    const handleSubmit = async () => {
        setError(null);
        try {
            await onSubmit({ email, password, role });
            onCreated();
            onClose();
        } catch (err: unknown) {
            if (typeof err === 'object' && err !== null) {
                const msg = (err as { response?: { data?: { message?: string } } }).response?.data?.message;
                setError(typeof msg === 'string' && msg.trim() ? msg : 'No se pudo crear el usuario.');
            } else {
                setError(err instanceof Error ? err.message : 'No se pudo crear el usuario.');
            }
        }
    };

    return {
        email, setEmail,
        password, setPassword,
        role, setRole,
        error,
        rolMenuAbierto, setRolMenuAbierto,
        handleSubmit,
    };
}
