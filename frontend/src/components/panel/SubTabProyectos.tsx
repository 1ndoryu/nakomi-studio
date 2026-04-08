/* [084A-22] Sub-tab Proyectos del CMS — extraído de SeccionContenido para SRP.
 * Gestiona estado de editor, CRUD de proyectos.
 * sentinel-disable-file componente-sin-hook: Los callbacks son wiring trivial que
 * delega a useContenidoProyectos. La lógica real vive en el hook. */
import React, {useState, useCallback} from 'react';
import {ListaProyectos} from './ListaProyectos';
import {EditorProyecto} from './EditorProyecto';
import {useContenidoProyectos} from '../../hooks/useContenidoProyectos';
import type {AdminProject, CreateProjectBody, UpdateProjectBody} from '../../api/admin-projects';

export const SubTabProyectos: React.FC = () => {
    const {
        proyectos, cargando, error, guardando,
        crear, actualizar, archivar, eliminar,
    } = useContenidoProyectos();
    const [editorAbierto, setEditorAbierto] = useState(false);
    const [proyectoEditando, setProyectoEditando] = useState<AdminProject | null>(null);

    const handleEditar = useCallback((proy: AdminProject) => {
        setProyectoEditando(proy);
        setEditorAbierto(true);
    }, []);

    const handleCrear = useCallback(() => {
        setProyectoEditando(null);
        setEditorAbierto(true);
    }, []);

    const handleGuardar = useCallback(async (body: CreateProjectBody | UpdateProjectBody) => {
        if (proyectoEditando) {
            const result = await actualizar(proyectoEditando.id, body as UpdateProjectBody);
            if (result) setEditorAbierto(false);
        } else {
            const result = await crear(body as CreateProjectBody);
            if (result) setEditorAbierto(false);
        }
    }, [proyectoEditando, actualizar, crear]);

    const handleArchivar = useCallback(async (id: string) => {
        await archivar(id);
    }, [archivar]);

    const handleDesarchivar = useCallback(async (id: string) => {
        await actualizar(id, {status: 'draft'} as UpdateProjectBody);
    }, [actualizar]);

    const handlePublicar = useCallback(async (id: string) => {
        await actualizar(id, {status: 'published'} as UpdateProjectBody);
    }, [actualizar]);

    const handleEliminar = useCallback(async (id: string) => {
        await eliminar(id);
    }, [eliminar]);

    return (
        <>
            {error && <div className="contenidoError">{error}</div>}
            <ListaProyectos
                proyectos={proyectos}
                cargando={cargando}
                onEditar={handleEditar}
                onCrear={handleCrear}
                onArchivar={handleArchivar}
                onDesarchivar={handleDesarchivar}
                onEliminar={handleEliminar}
                onPublicar={handlePublicar}
            />
            <EditorProyecto
                abierto={editorAbierto}
                onCerrar={() => setEditorAbierto(false)}
                proyecto={proyectoEditando}
                onGuardar={handleGuardar}
                guardando={guardando}
            />
        </>
    );
};
