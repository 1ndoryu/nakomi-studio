/* [084A-22] Sub-tab Blog del CMS — extraído de SeccionContenido para SRP.
 * Gestiona estado de editor, CRUD de posts.
 * sentinel-disable-file componente-sin-hook: Los callbacks son wiring trivial que
 * delega a useContenidoBlog. La lógica real vive en el hook. */
import React, {useState, useCallback} from 'react';
import {ListaBlog} from './ListaBlog';
import {EditorBlog} from './EditorBlog';
import {useContenidoBlog} from '../../hooks/useContenidoBlog';
import type {AdminBlogPost, CreateBlogPostBody, UpdateBlogPostBody} from '../../api/admin-blog';

export const SubTabBlog: React.FC = () => {
    const {
        posts, cargando, error, guardando,
        crear, actualizar, archivar, eliminar, reordenar,
    } = useContenidoBlog();
    const [editorAbierto, setEditorAbierto] = useState(false);
    const [postEditando, setPostEditando] = useState<AdminBlogPost | null>(null);

    const handleEditar = useCallback((post: AdminBlogPost) => {
        setPostEditando(post);
        setEditorAbierto(true);
    }, []);

    const handleCrear = useCallback(() => {
        setPostEditando(null);
        setEditorAbierto(true);
    }, []);

    const handleGuardar = useCallback(async (body: CreateBlogPostBody | UpdateBlogPostBody) => {
        if (postEditando) {
            const result = await actualizar(postEditando.id, body as UpdateBlogPostBody);
            if (result) setEditorAbierto(false);
        } else {
            const result = await crear(body as CreateBlogPostBody);
            if (result) setEditorAbierto(false);
        }
    }, [postEditando, actualizar, crear]);

    const handleArchivar = useCallback(async (id: string) => {
        await archivar(id);
    }, [archivar]);

    const handleDesarchivar = useCallback(async (id: string) => {
        await actualizar(id, {status: 'draft'} as UpdateBlogPostBody);
    }, [actualizar]);

    const handlePublicar = useCallback(async (id: string) => {
        await actualizar(id, {status: 'published'} as UpdateBlogPostBody);
    }, [actualizar]);

    const handleEliminar = useCallback(async (id: string) => {
        await eliminar(id);
    }, [eliminar]);

    return (
        <>
            {error && <div className="contenidoError">{error}</div>}
            <ListaBlog
                posts={posts}
                cargando={cargando}
                onEditar={handleEditar}
                onCrear={handleCrear}
                onArchivar={handleArchivar}
                onDesarchivar={handleDesarchivar}
                onEliminar={handleEliminar}
                onPublicar={handlePublicar}
                onReordenar={reordenar}
            />
            <EditorBlog
                abierto={editorAbierto}
                onCerrar={() => setEditorAbierto(false)}
                post={postEditando}
                onGuardar={handleGuardar}
                guardando={guardando}
            />
        </>
    );
};
