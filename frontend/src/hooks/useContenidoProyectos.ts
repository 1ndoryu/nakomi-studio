/* [074A-12] Hook CRUD para proyectos en CMS admin.
 * Mismo patrón que useContenidoBlog.ts. */

import { useState, useEffect, useCallback } from 'react';
import {
    apiListAdminProjects,
    apiCreateProject,
    apiUpdateProject,
    apiArchiveProject,
    apiDestroyProject,
    AdminProject,
    CreateProjectBody,
    UpdateProjectBody,
} from '../api/admin-projects';

export function useContenidoProyectos() {
    const [proyectos, setProyectos] = useState<AdminProject[]>([]);
    const [cargando, setCargando] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [guardando, setGuardando] = useState(false);

    const cargar = useCallback(async () => {
        setCargando(true);
        setError(null);
        try {
            const data = await apiListAdminProjects();
            setProyectos(data);
        } catch (err) {
            const msg = err instanceof Error ? err.message : 'Error cargando proyectos';
            setError(msg);
        } finally {
            setCargando(false);
        }
    }, []);

    useEffect(() => { cargar(); }, [cargar]);

    const crear = useCallback(async (body: CreateProjectBody): Promise<AdminProject | null> => {
        setGuardando(true);
        try {
            const nuevo = await apiCreateProject(body);
            setProyectos(prev => [nuevo, ...prev]);
            return nuevo;
        } catch (err) {
            const msg = err instanceof Error ? err.message : 'Error creando proyecto';
            setError(msg);
            return null;
        } finally {
            setGuardando(false);
        }
    }, []);

    const actualizar = useCallback(async (id: string, body: UpdateProjectBody): Promise<AdminProject | null> => {
        setGuardando(true);
        try {
            const actualizado = await apiUpdateProject(id, body);
            setProyectos(prev => prev.map(p => p.id === id ? actualizado : p));
            return actualizado;
        } catch (err) {
            const msg = err instanceof Error ? err.message : 'Error actualizando proyecto';
            setError(msg);
            return null;
        } finally {
            setGuardando(false);
        }
    }, []);

    const archivar = useCallback(async (id: string): Promise<boolean> => {
        setGuardando(true);
        try {
            await apiArchiveProject(id);
            setProyectos(prev => prev.filter(p => p.id !== id));
            return true;
        } catch (err) {
            const msg = err instanceof Error ? err.message : 'Error archivando proyecto';
            setError(msg);
            return false;
        } finally {
            setGuardando(false);
        }
    }, []);

    /* [084A-10] Eliminar permanentemente un proyecto */
    const eliminar = useCallback(async (id: string): Promise<boolean> => {
        setGuardando(true);
        setError(null);
        try {
            await apiDestroyProject(id);
            setProyectos(prev => prev.filter(p => p.id !== id));
            return true;
        } catch (err: unknown) {
            const msg = err instanceof Error ? err.message : 'Error al eliminar proyecto';
            setError(msg);
            return false;
        } finally {
            setGuardando(false);
        }
    }, []);

    return { proyectos, cargando, error, guardando, crear, actualizar, archivar, eliminar, recargar: cargar };
}
