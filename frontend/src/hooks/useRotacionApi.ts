/* [114A-12] Hook para gestión de rotación de API keys.
 * Obtiene estado actual y permite toggle desde el panel admin. */

import { useState, useEffect, useCallback } from 'react';
import { apiGetRotacionStatus, apiToggleRotacion, type RotacionStatus } from '../api/configuracion';

export function useRotacionApi() {
    const [status, setStatus] = useState<RotacionStatus | null>(null);
    const [cargando, setCargando] = useState(true);
    const [errorMsg, setErrorMsg] = useState<string | null>(null);

    const cargar = useCallback(async () => {
        try {
            const data = await apiGetRotacionStatus();
            setStatus(data);
            setErrorMsg(null);
        } catch (err: unknown) {
            const msg = err instanceof Error ? err.message : 'Error al obtener estado de rotación';
            setErrorMsg(msg);
        } finally {
            setCargando(false);
        }
    }, []);

    useEffect(() => {
        cargar();
    }, [cargar]);

    const toggle = useCallback(async (enabled: boolean) => {
        setCargando(true);
        setErrorMsg(null);
        try {
            const data = await apiToggleRotacion({ enabled });
            setStatus(data);
        } catch (err: unknown) {
            const msg = err instanceof Error ? err.message : 'Error al cambiar rotación';
            setErrorMsg(msg);
        } finally {
            setCargando(false);
        }
    }, []);

    return { status, cargando, errorMsg, toggle };
}
