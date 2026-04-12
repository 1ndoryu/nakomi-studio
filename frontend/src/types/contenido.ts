/*
 * Tipos centralizados para contenido del sitio.
 * Fuente única de verdad para interfaces de datos compartidos.
 */

export interface Testimonio {
    id: number | string;
    texto: string;
    autor: string;
    cargo: string;
    avatar?: string;
}

export interface PostBlog {
    id: number;
    titulo: string;
    resumen: string;
    contenido?: string;
    fecha: string;
    categoria: string;
    link?: string;
    imagen?: string;
    /* [084A-29] UUID original del backend para edición admin inline */
    adminId?: string;
}

/* [154A-10] Tipos de enlace ampliados con IconPickerEnlace */
export interface EnlaceProyecto {
    tipo: 'web' | 'github' | 'npm' | 'demo' | 'figma' | 'dribbble' | 'behance'
        | 'twitter' | 'linkedin' | 'youtube' | 'instagram' | 'facebook'
        | 'discord' | 'slack' | 'email' | 'docs' | 'api' | 'playstore'
        | 'appstore' | 'video' | 'podcast' | 'blog' | 'tienda' | 'git' | 'otro';
    url: string;
    etiqueta?: string;
}

export interface Proyecto {
    id: number | string;
    titulo: string;
    cliente: string;
    categorias: string | string[];
    imagen: string;
    descripcion?: string;
    contenido?: string;
    link?: string;
    skills?: Skill[];
    /* [124A-PROJ1] Galería con layout (full/half) */
    galeria?: GaleriaImagen[];
    /* [064A-8] Detalles técnicos y enlaces del proyecto */
    tecnologias?: string[];
    enlaces?: EnlaceProyecto[];
    /* [084A-29] UUID original del backend para edición admin inline */
    adminId?: string;
    /* [124A-SHOW1] Categoría showcase editable desde CMS */
    showcaseCategory?: string;
    /* [124A-DETAIL1] Título alternativo para página de detalle y opción de usar primera imagen de galería */
    detailTitle?: string;
    useFirstGalleryImage?: boolean;
}

/* [124A-PROJ1] Imagen de galería con layout */
export interface GaleriaImagen {
    url: string;
    layout: 'full' | 'half';
}

export interface CategoriaShowcase {
    titulo: string;
    proyectos: Proyecto[];
}

export interface Skill {
    id: number | string;
    titulo: string;
    descripcion?: string;
}

export interface Miembro {
    id: string;
    nombre: string;
    bio: string;
    cargo: string;
    avatar: string;
    linkedin?: string;
    twitter?: string;
    github?: string;
    /* [084A-29] UUID original del backend para edición admin inline */
    adminId?: string;
}

export interface Marca {
    id: string;
    nombre: string;
    url?: string;
    logo?: string;
}
