/* [084A-29] Provider global de edición admin.
 * Se monta en App.tsx y escucha useAdminEditStore. Cuando el admin
 * hace click en "Editar" desde AdminOverlay en páginas públicas,
 * este provider obtiene el item completo y abre el editor CMS correspondiente.
 * También maneja archive/delete con confirmación. */

import { useState, useEffect, useCallback } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useAuthStore } from '../stores/authStore';
import { useAdminEditStore } from '../stores/adminEditStore';
import { toast } from '../stores/toastStore';

import { EditorServicio } from './panel/EditorServicio';
import { EditorBlog } from './panel/EditorBlog';
import { EditorProyecto } from './panel/EditorProyecto';
import { EditorMiembro } from './panel/EditorMiembro';

import {
    apiListAdminServices,
    apiUpdateService,
    apiArchiveService,
    apiDestroyService,
    apiSaveServicePlans,
    type CreateServiceBody,
    type UpdateServiceBody,
    type SavePlanBody,
} from '../api/admin-services';
import {
    apiListAdminBlog,
    apiUpdateBlogPost,
    apiArchiveBlogPost,
    apiDestroyBlogPost,
    type CreateBlogPostBody,
    type UpdateBlogPostBody,
} from '../api/admin-blog';
import {
    apiListAdminProjects,
    apiUpdateProject,
    apiArchiveProject,
    apiDestroyProject,
    type CreateProjectBody,
    type UpdateProjectBody,
} from '../api/admin-projects';
import {
    apiListAdminTeamMembers,
    apiUpdateTeamMember,
    apiArchiveTeamMember,
    apiDestroyTeamMember,
    type CreateTeamMemberBody,
    type UpdateTeamMemberBody,
} from '../api/admin-team';

export function AdminEditorProvider() {
    const isAdmin = useAuthStore(s => s.user?.effectiveRole === 'admin');
    const contentType = useAdminEditStore(s => s.contentType);
    const itemId = useAdminEditStore(s => s.itemId);
    const action = useAdminEditStore(s => s.action);
    const close = useAdminEditStore(s => s.close);
    const queryClient = useQueryClient();
    const [guardando, setGuardando] = useState(false);

    const isEditing = !!contentType && action === 'edit';

    /* Fetch admin lists — solo cuando hay un edit activo para ese tipo */
    const { data: services } = useQuery({
        queryKey: ['admin-services'],
        queryFn: apiListAdminServices,
        enabled: isAdmin === true && isEditing && contentType === 'service',
    });
    const { data: blogPosts } = useQuery({
        queryKey: ['admin-blog'],
        queryFn: apiListAdminBlog,
        enabled: isAdmin === true && isEditing && contentType === 'blog',
    });
    const { data: projects } = useQuery({
        queryKey: ['admin-projects'],
        queryFn: apiListAdminProjects,
        enabled: isAdmin === true && isEditing && contentType === 'project',
    });
    const { data: teamMembers } = useQuery({
        queryKey: ['admin-team'],
        queryFn: apiListAdminTeamMembers,
        enabled: isAdmin === true && isEditing && contentType === 'team',
    });

    /* Encontrar el item actual */
    const currentService = services?.find(s => s.id === itemId) ?? null;
    const currentPost = blogPosts?.find(p => p.id === itemId) ?? null;
    const currentProject = projects?.find(p => p.id === itemId) ?? null;
    const currentMember = teamMembers?.find(m => m.id === itemId) ?? null;

    /* [084A-29] Ejecutar archive/delete directamente cuando se piden */
    useEffect(() => {
        if (!isAdmin || !contentType || !itemId) return;
        if (action !== 'archive' && action !== 'delete') return;

        let active = true;
        const run = async () => {
            try {
                if (action === 'archive') {
                    if (contentType === 'service') await apiArchiveService(itemId);
                    if (contentType === 'blog') await apiArchiveBlogPost(itemId);
                    if (contentType === 'project') await apiArchiveProject(itemId);
                    if (contentType === 'team') await apiArchiveTeamMember(itemId);
                    if (active) toast.success('Contenido archivado');
                } else {
                    if (contentType === 'service') await apiDestroyService(itemId);
                    if (contentType === 'blog') await apiDestroyBlogPost(itemId);
                    if (contentType === 'project') await apiDestroyProject(itemId);
                    if (contentType === 'team') await apiDestroyTeamMember(itemId);
                    if (active) toast.success('Contenido eliminado');
                }
                if (active) {
                    queryClient.invalidateQueries({ queryKey: [`admin-${contentType === 'service' ? 'services' : contentType}`] });
                    queryClient.invalidateQueries({ queryKey: ['public'] });
                }
            } catch (err) {
                if (active) {
                    const msg = err instanceof Error ? err.message : 'Error desconocido';
                    toast.error(`Error: ${msg}`);
                }
            } finally {
                if (active) close();
            }
        };
        void run();
        return () => { active = false; };
    }, [isAdmin, contentType, itemId, action, close, queryClient]);

    /* Callbacks de guardado para cada editor */
    const handleGuardarServicio = useCallback(async (
        body: CreateServiceBody | UpdateServiceBody,
        planes: SavePlanBody[],
    ) => {
        if (!itemId) return;
        setGuardando(true);
        try {
            await apiUpdateService(itemId, body as UpdateServiceBody);
            if (planes.length > 0) await apiSaveServicePlans(itemId, planes);
            queryClient.invalidateQueries({ queryKey: ['admin-services'] });
            queryClient.invalidateQueries({ queryKey: ['public'] });
            toast.success('Servicio actualizado');
            close();
        } catch (err) {
            toast.error(err instanceof Error ? err.message : 'Error al guardar');
        } finally {
            setGuardando(false);
        }
    }, [itemId, close, queryClient]);

    const handleGuardarBlog = useCallback(async (
        body: CreateBlogPostBody | UpdateBlogPostBody,
    ) => {
        if (!itemId) return;
        setGuardando(true);
        try {
            await apiUpdateBlogPost(itemId, body as UpdateBlogPostBody);
            queryClient.invalidateQueries({ queryKey: ['admin-blog'] });
            queryClient.invalidateQueries({ queryKey: ['public'] });
            toast.success('Artículo actualizado');
            close();
        } catch (err) {
            toast.error(err instanceof Error ? err.message : 'Error al guardar');
        } finally {
            setGuardando(false);
        }
    }, [itemId, close, queryClient]);

    const handleGuardarProyecto = useCallback(async (
        body: CreateProjectBody | UpdateProjectBody,
    ) => {
        if (!itemId) return;
        setGuardando(true);
        try {
            await apiUpdateProject(itemId, body as UpdateProjectBody);
            queryClient.invalidateQueries({ queryKey: ['admin-projects'] });
            queryClient.invalidateQueries({ queryKey: ['public'] });
            toast.success('Proyecto actualizado');
            close();
        } catch (err) {
            toast.error(err instanceof Error ? err.message : 'Error al guardar');
        } finally {
            setGuardando(false);
        }
    }, [itemId, close, queryClient]);

    const handleGuardarMiembro = useCallback(async (
        _id: string | null,
        body: CreateTeamMemberBody | UpdateTeamMemberBody,
    ) => {
        if (!itemId) return;
        setGuardando(true);
        try {
            await apiUpdateTeamMember(itemId, body as UpdateTeamMemberBody);
            queryClient.invalidateQueries({ queryKey: ['admin-team'] });
            queryClient.invalidateQueries({ queryKey: ['public'] });
            toast.success('Miembro actualizado');
            close();
        } catch (err) {
            toast.error(err instanceof Error ? err.message : 'Error al guardar');
        } finally {
            setGuardando(false);
        }
    }, [itemId, close, queryClient]);

    if (!isAdmin || !isEditing) return null;

    return (
        <>
            {contentType === 'service' && (
                <EditorServicio
                    abierto={!!currentService}
                    onCerrar={close}
                    servicio={currentService}
                    onGuardar={handleGuardarServicio}
                    guardando={guardando}
                />
            )}
            {contentType === 'blog' && (
                <EditorBlog
                    abierto={!!currentPost}
                    onCerrar={close}
                    post={currentPost}
                    onGuardar={handleGuardarBlog}
                    guardando={guardando}
                />
            )}
            {contentType === 'project' && (
                <EditorProyecto
                    abierto={!!currentProject}
                    onCerrar={close}
                    proyecto={currentProject}
                    onGuardar={handleGuardarProyecto}
                    guardando={guardando}
                />
            )}
            {contentType === 'team' && (
                <EditorMiembro
                    abierto={!!currentMember}
                    onCerrar={close}
                    miembro={currentMember}
                    onGuardar={handleGuardarMiembro}
                    guardando={guardando}
                />
            )}
        </>
    );
}
