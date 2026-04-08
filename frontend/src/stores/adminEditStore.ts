/* [084A-29] Store global para edición inline de contenido desde páginas públicas.
 * Cuando el admin hace hover en un contenido editable, puede abrir el editor CMS
 * directamente sin navegar al panel. El AdminEditorProvider escucha este store. */
import { create } from 'zustand';

export type AdminContentType = 'service' | 'blog' | 'project' | 'team';
export type AdminEditAction = 'edit' | 'archive' | 'delete';

interface AdminEditState {
    contentType: AdminContentType | null;
    itemId: string | null;
    action: AdminEditAction | null;
    requestEdit: (type: AdminContentType, id: string) => void;
    requestArchive: (type: AdminContentType, id: string) => void;
    requestDelete: (type: AdminContentType, id: string) => void;
    close: () => void;
}

export const useAdminEditStore = create<AdminEditState>((set) => ({
    contentType: null,
    itemId: null,
    action: null,
    requestEdit: (type, id) => set({ contentType: type, itemId: id, action: 'edit' }),
    requestArchive: (type, id) => set({ contentType: type, itemId: id, action: 'archive' }),
    requestDelete: (type, id) => set({ contentType: type, itemId: id, action: 'delete' }),
    close: () => set({ contentType: null, itemId: null, action: null }),
}));
