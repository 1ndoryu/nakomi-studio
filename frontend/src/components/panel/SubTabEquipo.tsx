/* [084A-22] Sub-tab Equipo del CMS — extraído de SeccionContenido para SRP.
 * Gestiona estado de editor, CRUD de miembros del equipo.
 * sentinel-disable-file componente-sin-hook: Los callbacks son wiring trivial que
 * delega a useContenidoEquipo. La lógica real vive en el hook. */
import React, {useState, useCallback} from 'react';
import {ListaEquipo} from './ListaEquipo';
import {EditorMiembro} from './EditorMiembro';
import {useContenidoEquipo} from '../../hooks/useContenidoEquipo';
import type {AdminTeamMember, CreateTeamMemberBody, UpdateTeamMemberBody} from '../../api/admin-team';

export const SubTabEquipo: React.FC = () => {
    const {
        miembros, cargando, error, guardando,
        crear, actualizar, archivar, eliminar,
    } = useContenidoEquipo();
    const [editorAbierto, setEditorAbierto] = useState(false);
    const [miembroEditando, setMiembroEditando] = useState<AdminTeamMember | null>(null);

    const handleEditar = useCallback((m: AdminTeamMember) => {
        setMiembroEditando(m);
        setEditorAbierto(true);
    }, []);

    const handleCrear = useCallback(() => {
        setMiembroEditando(null);
        setEditorAbierto(true);
    }, []);

    const handleGuardar = useCallback(async (id: string | null, body: CreateTeamMemberBody | UpdateTeamMemberBody) => {
        if (id) {
            await actualizar(id, body as UpdateTeamMemberBody);
        } else {
            await crear(body as CreateTeamMemberBody);
        }
        setEditorAbierto(false);
    }, [actualizar, crear]);

    const handleArchivar = useCallback(async (id: string) => {
        await archivar(id);
    }, [archivar]);

    const handleDesarchivar = useCallback(async (id: string) => {
        await actualizar(id, {status: 'draft'} as UpdateTeamMemberBody);
    }, [actualizar]);

    const handlePublicar = useCallback(async (id: string) => {
        await actualizar(id, {status: 'published'} as UpdateTeamMemberBody);
    }, [actualizar]);

    const handleEliminar = useCallback(async (id: string) => {
        await eliminar(id);
    }, [eliminar]);

    return (
        <>
            {error && <div className="contenidoError">{error}</div>}
            <ListaEquipo
                miembros={miembros}
                cargando={cargando}
                onEditar={handleEditar}
                onCrear={handleCrear}
                onArchivar={handleArchivar}
                onDesarchivar={handleDesarchivar}
                onEliminar={handleEliminar}
                onPublicar={handlePublicar}
            />
            <EditorMiembro
                abierto={editorAbierto}
                onCerrar={() => setEditorAbierto(false)}
                miembro={miembroEditando}
                onGuardar={handleGuardar}
                guardando={guardando}
            />
        </>
    );
};
