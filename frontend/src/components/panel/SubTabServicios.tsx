/* [084A-22] Sub-tab Servicios del CMS — extraído de SeccionContenido para SRP.
 * Gestiona estado de editor, CRUD de servicios y planes.
 * sentinel-disable-file componente-sin-hook: Los callbacks son wiring trivial que
 * delega a useContenidoServicios. La lógica real vive en el hook. */
import React, {useState, useCallback} from 'react';
import {ListaServicios} from './ListaServicios';
import {EditorServicio} from './EditorServicio';
import {useContenidoServicios} from '../../hooks/useContenidoServicios';
import type {AdminService, CreateServiceBody, UpdateServiceBody, SavePlanBody} from '../../api/admin-services';
import {apiSaveServicePlans} from '../../api/admin-services';

export const SubTabServicios: React.FC = () => {
    const {servicios, cargando, error, guardando, crear, actualizar, archivar, eliminar: eliminarServicio, recargar} = useContenidoServicios();
    const [editorAbierto, setEditorAbierto] = useState(false);
    const [servicioEditando, setServicioEditando] = useState<AdminService | null>(null);

    const handleEditar = useCallback((svc: AdminService) => {
        setServicioEditando(svc);
        setEditorAbierto(true);
    }, []);

    const handleCrear = useCallback(() => {
        setServicioEditando(null);
        setEditorAbierto(true);
    }, []);

    /* [154A-2] Refetch después de guardar planes para que features se reflejen en estado */
    const handleGuardar = useCallback(async (body: CreateServiceBody | UpdateServiceBody, planes: SavePlanBody[]) => {
        if (servicioEditando) {
            const result = await actualizar(servicioEditando.id, body as UpdateServiceBody);
            if (result) {
                await apiSaveServicePlans(servicioEditando.id, planes);
                await recargar();
                setEditorAbierto(false);
            }
        } else {
            const result = await crear(body as CreateServiceBody);
            if (result) {
                if (planes.length > 0) {
                    await apiSaveServicePlans(result.id, planes);
                }
                await recargar();
                setEditorAbierto(false);
            }
        }
    }, [servicioEditando, actualizar, crear, recargar]);

    const handleArchivar = useCallback(async (id: string) => {
        await archivar(id);
    }, [archivar]);

    const handleDesarchivar = useCallback(async (id: string) => {
        await actualizar(id, {status: 'draft'} as UpdateServiceBody);
    }, [actualizar]);

    const handlePublicar = useCallback(async (id: string) => {
        await actualizar(id, {status: 'published'} as UpdateServiceBody);
    }, [actualizar]);

    const handleEliminar = useCallback(async (id: string) => {
        await eliminarServicio(id);
    }, [eliminarServicio]);

    return (
        <>
            {error && <div className="contenidoError">{error}</div>}
            <ListaServicios
                servicios={servicios}
                cargando={cargando}
                onEditar={handleEditar}
                onCrear={handleCrear}
                onArchivar={handleArchivar}
                onDesarchivar={handleDesarchivar}
                onEliminar={handleEliminar}
                onPublicar={handlePublicar}
            />
            <EditorServicio
                abierto={editorAbierto}
                onCerrar={() => setEditorAbierto(false)}
                servicio={servicioEditando}
                onGuardar={handleGuardar}
                guardando={guardando}
            />
        </>
    );
};
