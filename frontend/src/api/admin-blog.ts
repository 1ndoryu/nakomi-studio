/* [074A-11] API client para CRUD de blog posts desde panel admin.
 * Conecta con endpoints /api/admin/blog y /api/blog del backend.
 * Patrón: misma estructura que admin-services.ts. */
import instance from './axios-instance';

export interface AdminBlogPost {
    id: string;
    author_id: string;
    author_name: string | null;
    title: string;
    slug: string;
    excerpt: string | null;
    content: string;
    featured_image: string | null;
    status: string;
    tags: string[];
    meta_title: string | null;
    meta_description: string | null;
    sort_order: number;
    is_featured: boolean;
    published_at: string | null;
    created_at: string;
    updated_at: string;
}

export interface PaginatedBlogPosts {
    posts: AdminBlogPost[];
    total: number;
    page: number;
    per_page: number;
}

export interface CreateBlogPostBody {
    title: string;
    slug: string;
    excerpt?: string;
    content: string;
    featured_image?: string;
    status?: string;
    tags?: string[];
    meta_title?: string;
    meta_description?: string;
}

export interface UpdateBlogPostBody {
    title?: string;
    slug?: string;
    excerpt?: string;
    content?: string;
    featured_image?: string;
    status?: string;
    tags?: string[];
    is_featured?: boolean;
    meta_title?: string;
    meta_description?: string;
}

/* Admin: listar todos (incluso borradores y archivados) */
export async function apiListAdminBlog(): Promise<AdminBlogPost[]> {
    const {data} = await instance.get<AdminBlogPost[]>('/api/admin/blog');
    return data;
}

/* Admin: crear post */
export async function apiCreateBlogPost(body: CreateBlogPostBody): Promise<AdminBlogPost> {
    const {data} = await instance.post<AdminBlogPost>('/api/admin/blog', body);
    return data;
}

/* Admin: actualizar post */
export async function apiUpdateBlogPost(id: string, body: UpdateBlogPostBody): Promise<AdminBlogPost> {
    const {data} = await instance.put<AdminBlogPost>(`/api/admin/blog/${id}`, body);
    return data;
}

/* Admin: archivar post (soft delete) */
export async function apiArchiveBlogPost(id: string): Promise<void> {
    await instance.delete(`/api/admin/blog/${id}`);
}

/* [084A-10] Eliminación permanente de un blog post */
export async function apiDestroyBlogPost(id: string): Promise<void> {
    await instance.post(`/api/admin/blog/${id}/destroy`);
}

/* [124A-CMS10] Reordenar blog posts en batch */
export async function apiReorderBlog(items: {id: string; sort_order: number}[]): Promise<void> {
    await instance.put('/api/admin/blog/reorder', {items});
}

/* Público: listar posts publicados con paginación */
export async function apiListPublicBlog(page = 1, perPage = 10): Promise<PaginatedBlogPosts> {
    const {data} = await instance.get<PaginatedBlogPosts>('/api/blog', {
        params: {page, per_page: perPage},
    });
    return data;
}

/* Público: detalle de post por slug */
export async function apiGetBlogPostBySlug(slug: string): Promise<AdminBlogPost> {
    const {data} = await instance.get<AdminBlogPost>(`/api/blog/${slug}`);
    return data;
}
