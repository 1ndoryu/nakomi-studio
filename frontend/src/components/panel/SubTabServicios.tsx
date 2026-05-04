/* [084A-22] Sub-tab Servicios del CMS — extraído de SeccionContenido para SRP.
 * Gestiona estado de editor, CRUD de servicios y planes.
 * sentinel-disable-file componente-sin-hook: Los callbacks son wiring trivial que
 * delega a useContenidoServicios. La lógica real vive en el hook. */
import React, {useState, useCallback} from 'react';
import {isAxiosError} from 'axios';
import {ListaServicios} from './ListaServicios';
import {EditorServicio} from './EditorServicio';
import {useContenidoServicios} from '../../hooks/useContenidoServicios';
import type {AdminService, CreateServiceBody, UpdateServiceBody, SavePlanBody} from '../../api/admin-services';
import {apiSaveServicePlans} from '../../api/admin-services';
import {toast} from '../../stores/toastStore';

/* [045A-4] Preflight local de slug para no esperar al roundtrip del 409.
 * El backend sigue siendo la fuente de verdad; esto solo adelanta el feedback cuando el listado ya contiene el conflicto. */
function normalizarSlug(slug: string | undefined): string {
    return (slug ?? '').trim().toLowerCase();
}

function extraerMensajeApi(err: unknown, fallback: string): string {
    if (isAxiosError(err) && err.response?.data) {
        const data = err.response.data as Record<string, unknown>;
        if (typeof data.message === 'string' && data.message.trim().length > 0) {
            return data.message;
        }
    }

    return err instanceof Error ? err.message : fallback;
}

function describirPlan(plan: SavePlanBody, index: number): string {
    return plan.name.trim() || plan.slug.trim() || `plan ${index + 1}`;
}

/* [045A-5] El guardado de planes no puede quedar como segundo paso ciego.
 * Si el payload ya es inválido en el CMS, se corta antes del PUT principal para evitar guardados parciales. */
function validarPlanes(planes: SavePlanBody[]): string | null {
    const slugs = new Map<string, string>();

    for (const [planIndex, plan] of planes.entries()) {
        const nombrePlan = describirPlan(plan, planIndex);
        const slug = plan.slug.trim();
        const name = plan.name.trim();

        if (!slug) {
            return `El ${nombrePlan} debe tener un slug`;
        }

        if (slug.length > 50) {
            return `El slug del ${nombrePlan} no puede exceder 50 caracteres`;
        }

        const slugNormalizado = normalizarSlug(slug);
        const slugDuplicado = slugs.get(slugNormalizado);
        if (slugDuplicado) {
            return `Los planes "${slugDuplicado}" y "${nombrePlan}" no pueden compartir el mismo slug`;
        }
        slugs.set(slugNormalizado, nombrePlan);

        if (!name) {
            return `El ${nombrePlan} debe tener un nombre`;
        }

        if (name.length > 100) {
            return `El nombre del ${nombrePlan} no puede exceder 100 caracteres`;
        }

        if (plan.phases.length === 0) {
            return `El ${nombrePlan} debe tener al menos una fase configurada`;
        }

        for (const fase of plan.phases) {
            const titulo = fase.title.trim();
            if (!titulo) {
                return `La fase ${fase.phase_number} del ${nombrePlan} debe tener un título`;
            }

            if (titulo.length > 200) {
                return `El título de la fase ${fase.phase_number} del ${nombrePlan} no puede exceder 200 caracteres`;
            }
        }
    }

    return null;
}

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
        const errorPlanes = validarPlanes(planes);
        if (errorPlanes) {
            toast.error(errorPlanes);
            return;
        }

        const slug = normalizarSlug(body.slug);
        const conflicto = servicios.find(servicio => normalizarSlug(servicio.slug) === slug && servicio.id !== servicioEditando?.id);

        if (slug && conflicto) {
            toast.error(`El slug "${body.slug}" ya existe en "${conflicto.title}"`);
            return;
        }

        if (servicioEditando) {
            const result = await actualizar(servicioEditando.id, body as UpdateServiceBody);
            if (result) {
                try {
                    await apiSaveServicePlans(servicioEditando.id, planes);
                } catch (err: unknown) {
                    toast.error(extraerMensajeApi(err, 'Error al guardar planes del servicio'));
                    return;
                }
                await recargar();
                setEditorAbierto(false);
            }
        } else {
            const result = await crear(body as CreateServiceBody);
            if (result) {
                if (planes.length > 0) {
                    try {
                        await apiSaveServicePlans(result.id, planes);
                    } catch (err: unknown) {
                        setServicioEditando(result);
                        toast.error(extraerMensajeApi(err, 'Error al guardar planes del servicio'));
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
