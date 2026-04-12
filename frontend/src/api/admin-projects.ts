/* [074A-12] API client para CMS de proyectos.
 * Mismo patrón que admin-blog.ts. Tipos alineados con ProjectResponse del backend. */

import instance from './axios-instance';
import type {GaleriaImagen} from '../types/contenido';

export interface ProjectLink {
    tipo: string;
    url: string;
    etiqueta?: string;
}

export interface ProjectSkill {
    titulo: string;
    descripcion?: string;
}

export interface AdminProject {
    id: string;
    title: string;
    slug: string;
    client: string | null;
    description: string;
    featured_image: string | null;
    gallery: GaleriaImagen[];
    categories: string[];
    technologies: string[];
    links: ProjectLink[];
    skills: ProjectSkill[];
    status: string;
    sort_order: number;
    is_featured: boolean;
    in_carousel: boolean;
    showcase_category: string | null;
    detail_title: string | null;
    use_first_gallery_image: boolean;
    meta_title: string | null;
    meta_description: string | null;
    created_at: string;
    updated_at: string;
}

export interface CreateProjectBody {
    title: string;
    slug: string;
    client?: string;
    description?: string;
    featured_image?: string;
    gallery?: GaleriaImagen[];
    categories?: string[];
    technologies?: string[];
    links?: ProjectLink[];
    skills?: ProjectSkill[];
    status?: string;
    sort_order?: number;
    is_featured?: boolean;
    in_carousel?: boolean;
    showcase_category?: string;
    detail_title?: string;
    use_first_gallery_image?: boolean;
    meta_title?: string;
    meta_description?: string;
}

export interface UpdateProjectBody {
    title?: string;
    slug?: string;
    client?: string;
    description?: string;
    featured_image?: string;
    gallery?: GaleriaImagen[];
    categories?: string[];
    technologies?: string[];
    links?: ProjectLink[];
    skills?: ProjectSkill[];
    status?: string;
    sort_order?: number;
    is_featured?: boolean;
    in_carousel?: boolean;
    showcase_category?: string;
    detail_title?: string;
    use_first_gallery_image?: boolean;
    meta_title?: string;
    meta_description?: string;
}

export async function apiListAdminProjects(): Promise<AdminProject[]> {
    const { data } = await instance.get<AdminProject[]>('/api/admin/projects');
    return data;
}

export async function apiCreateProject(body: CreateProjectBody): Promise<AdminProject> {
    const { data } = await instance.post<AdminProject>('/api/admin/projects', body);
    return data;
}

export async function apiUpdateProject(id: string, body: UpdateProjectBody): Promise<AdminProject> {
    const { data } = await instance.put<AdminProject>(`/api/admin/projects/${id}`, body);
    return data;
}

export async function apiArchiveProject(id: string): Promise<void> {
    await instance.delete(`/api/admin/projects/${id}`);
}

/* [084A-10] Eliminación permanente de un proyecto */
export async function apiDestroyProject(id: string): Promise<void> {
    await instance.post(`/api/admin/projects/${id}/destroy`);
}

/* [124A-CMS3] Reordenar proyectos en batch */
export async function apiReorderProjects(items: {id: string; sort_order: number}[]): Promise<void> {
    await instance.put('/api/admin/projects/reorder', {items});
}

export async function apiListPublicProjects(): Promise<AdminProject[]> {
    const { data } = await instance.get<AdminProject[]>('/api/projects');
    return data;
}

export async function apiGetProjectBySlug(slug: string): Promise<AdminProject> {
    const { data } = await instance.get<AdminProject>(`/api/projects/${slug}`);
    return data;
}
