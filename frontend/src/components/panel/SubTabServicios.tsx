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
import {toast} from '../../stores/toastStore';
import {
    didServicePlansChange,
    extractServiceApiMessage,
    normalizeServiceSlug,
    validateServicePlans,
} from '../../utils/servicePlanEditorUtils';

/* [045A-4] Preflight local de slug para no esperar al roundtrip del 409.
 * El backend sigue siendo la fuente de verdad; esto solo adelanta el feedback cuando el listado ya contiene el conflicto. */
export const SubTabServicios: React.FC = () => {
    const {servicios, cargando, error, guardando, crear, actualizar, archivar, eliminar: eliminarServicio, reordenar, recargar} = useContenidoServicios();
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
        const requiereGuardarPlanes = didServicePlansChange(servicioEditando, planes);

        if (requiereGuardarPlanes) {
            const errorPlanes = validateServicePlans(planes);
            if (errorPlanes) {
                toast.error(errorPlanes);
                return;
            }
        }

        const slug = normalizeServiceSlug(body.slug);
        const conflicto = servicios.find(servicio => normalizeServiceSlug(servicio.slug) === slug && servicio.id !== servicioEditando?.id);

        if (slug && conflicto) {
            toast.error(`El slug "${body.slug}" ya existe en "${conflicto.title}"`);
            return;
        }

        if (servicioEditando) {
            const result = await actualizar(servicioEditando.id, body as UpdateServiceBody);
            if (result) {
                if (requiereGuardarPlanes) {
                    try {
                        await apiSaveServicePlans(servicioEditando.id, planes);
                    } catch (err: unknown) {
                        toast.error(extractServiceApiMessage(err, 'Error al guardar planes del servicio'));
                        return;
                    }
                }
                await recargar();
                setEditorAbierto(false);
            }
        } else {
            const result = await crear(body as CreateServiceBody);
            if (result) {
                if (requiereGuardarPlanes) {
                    try {
                        await apiSaveServicePlans(result.id, planes);
                    } catch (err: unknown) {
                        setServicioEditando(result);
                        toast.error(extractServiceApiMessage(err, 'Error al guardar planes del servicio'));
                        return;
                    }
                }
                await recargar();
                setEditorAbierto(false);
            }
        }
    }, [servicios, servicioEditando, actualizar, crear, recargar]);

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

    /* [124A-CMS4] Toggle visibilidad en home (is_active) */
    const handleToggleHome = useCallback(async (id: string, visible: boolean) => {
        await actualizar(id, {is_active: visible} as UpdateServiceBody);
    }, [actualizar]);

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
                onToggleHome={handleToggleHome}
                onReordenar={reordenar}
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
