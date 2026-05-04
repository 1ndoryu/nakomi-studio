/* [074A-9] Hook para gestión de servicios en panel CMS.
 * Encapsula: listado, creación, actualización, archivado. */
import {useState, useCallback, useEffect} from 'react';
import {isAxiosError} from 'axios';
import {
    AdminService,
    CreateServiceBody,
    UpdateServiceBody,
    apiListAdminServices,
    apiCreateService,
    apiUpdateService,
    apiArchiveService,
    apiDestroyService,
    apiReorderServices,
} from '../api/admin-services';
import {toast} from '../stores/toastStore';

/* [045A-4] El CMS de servicios debe exponer el mensaje real del backend.
 * Si el API devuelve un 409/422 con `message`, no lo degradamos a "Request failed...". */
function extraerMensajeServicios(err: unknown, fallback: string): string {
    if (isAxiosError(err) && err.response?.data) {
        const data = err.response.data as Record<string, unknown>;
        if (typeof data.message === 'string' && data.message.trim().length > 0) {
            return data.message;
        }
    }

    return err instanceof Error ? err.message : fallback;
}

export function useContenidoServicios() {
    const [servicios, setServicios] = useState<AdminService[]>([]);
    const [cargando, setCargando] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [guardando, setGuardando] = useState(false);

    const cargar = useCallback(async () => {
        setCargando(true);
        setError(null);
        try {
            const data = await apiListAdminServices();
            setServicios(data);
        } catch (err: unknown) {
            const msg = extraerMensajeServicios(err, 'Error al cargar servicios');
            setError(msg);
        } finally {
            setCargando(false);
        }
    }, []);

    useEffect(() => {
        cargar();
    }, [cargar]);

    const crear = useCallback(async (body: CreateServiceBody): Promise<AdminService | null> => {
        setGuardando(true);
        setError(null);
        try {
            const nuevo = await apiCreateService(body);
            setServicios(prev => [...prev, nuevo]);
            return nuevo;
        } catch (err: unknown) {
            const msg = extraerMensajeServicios(err, 'Error al crear servicio');
            setError(msg);
            toast.error(msg);
            return null;
        } finally {
            setGuardando(false);
        }
    }, []);

    const actualizar = useCallback(async (id: string, body: UpdateServiceBody): Promise<AdminService | null> => {
        setGuardando(true);
        setError(null);
        try {
            const actualizado = await apiUpdateService(id, body);
            setServicios(prev => prev.map(s => s.id === id ? actualizado : s));
            return actualizado;
        } catch (err: unknown) {
            const msg = extraerMensajeServicios(err, 'Error al actualizar servicio');
            setError(msg);
            toast.error(msg);
            return null;
        } finally {
            setGuardando(false);
        }
    }, []);

    const archivar = useCallback(async (id: string): Promise<boolean> => {
        setGuardando(true);
        setError(null);
        try {
            await apiArchiveService(id);
            setServicios(prev => prev.map(s => s.id === id ? {...s, is_active: false, status: 'archived'} : s));
            return true;
        } catch (err: unknown) {
            const msg = extraerMensajeServicios(err, 'Error al archivar servicio');
            setError(msg);
            toast.error(msg);
            return false;
        } finally {
            setGuardando(false);
        }
    }, []);

    /* [084A-10] Eliminar permanentemente un servicio (409 si tiene órdenes) */
    const eliminar = useCallback(async (id: string): Promise<boolean> => {
        setGuardando(true);
        setError(null);
        try {
            await apiDestroyService(id);
            setServicios(prev => prev.filter(s => s.id !== id));
            return true;
        } catch (err: unknown) {
            const msg = extraerMensajeServicios(err, 'Error al eliminar servicio');
            setError(msg);
            toast.error(msg);
            return false;
        } finally {
            setGuardando(false);
        }
    }, []);

    /* [124A-CMS10] Reordenar servicios con update optimista + rollback */
    const reordenar = useCallback(async (items: {id: string; sort_order: number}[]): Promise<boolean> => {
        const prev = [...servicios];
        const orderMap = new Map(items.map(i => [i.id, i.sort_order]));
        setServicios(curr =>
            [...curr].sort((a, b) => (orderMap.get(a.id) ?? a.sort_order) - (orderMap.get(b.id) ?? b.sort_order))
                .map(s => ({...s, sort_order: orderMap.get(s.id) ?? s.sort_order}))
        );
        try {
            await apiReorderServices(items);
            return true;
        } catch (err: unknown) {
            setServicios(prev);
            const msg = extraerMensajeServicios(err, 'Error al reordenar servicios');
            setError(msg);
            toast.error(msg);
            return false;
        }
    }, [servicios]);

    return {servicios, cargando, error, guardando, crear, actualizar, archivar, eliminar, reordenar, recargar: cargar};
}
