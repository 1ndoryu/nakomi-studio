/* [154A-5] Hook para la lógica del banner de establecer contraseña.
 * Extraído de BannerPassword.tsx para cumplir SRP (max 3 useState en componente). */
import {useState, useCallback} from 'react';
import {apiSetPassword, extraerMensajeError} from '../api/auth';
import {useAuthStore} from '../stores/authStore';
import {toast} from '../stores/toastStore';

export function useBannerPassword() {
    const needsPassword = useAuthStore(s => s.user?.needsPassword);
    const marcarPasswordEstablecida = useAuthStore(s => s.marcarPasswordEstablecida);
    const [password, setPassword] = useState('');
    const [confirmar, setConfirmar] = useState('');
    const [cargando, setCargando] = useState(false);
    const [expandido, setExpandido] = useState(false);

    const handleSubmit = useCallback(async (e: React.FormEvent) => {
        e.preventDefault();
        if (password.length < 8) {
            toast.error('La contraseña debe tener al menos 8 caracteres.');
            return;
        }
        if (password !== confirmar) {
            toast.error('Las contraseñas no coinciden.');
            return;
        }
        setCargando(true);
        try {
            await apiSetPassword(password);
            marcarPasswordEstablecida();
            toast.success('Contraseña establecida correctamente.');
        } catch (err) {
            toast.error(extraerMensajeError(err));
        } finally {
            setCargando(false);
        }
    }, [password, confirmar, marcarPasswordEstablecida]);

    return {
        needsPassword: !!needsPassword,
        password,
        setPassword,
        confirmar,
        setConfirmar,
        cargando,
        expandido,
        setExpandido,
        handleSubmit,
    };
}
