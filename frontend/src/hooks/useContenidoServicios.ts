/* [074A-9] Hook para gestión de servicios en panel CMS.
 * Encapsula: listado, creación, actualización, archivado. */
import {useState, useCallback, useEffect} from 'react';
import {
    AdminService,
    CreateServiceBody,
    UpdateServiceBody,
    apiListAdminServices,
    apiCreateService,
    apiUpdateService,
    apiArchiveService,
    apiDestroyService,
} from '../api/admin-services';

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
            const msg = err instanceof Error ? err.message : 'Error al cargar servicios';
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
            const msg = err instanceof Error ? err.message : 'Error al crear servicio';
            setError(msg);
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
            const msg = err instanceof Error ? err.message : 'Error al actualizar servicio';
            setError(msg);
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
            const msg = err instanceof Error ? err.message : 'Error al archivar servicio';
            setError(msg);
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
            const msg = err instanceof Error ? err.message : 'Error al eliminar servicio';
            setError(msg);
            return false;
        } finally {
            setGuardando(false);
        }
    }, []);

    return {servicios, cargando, error, guardando, crear, actualizar, archivar, eliminar, recargar: cargar};
}
