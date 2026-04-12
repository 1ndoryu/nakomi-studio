/* [074A-12] Hook interno para el estado del formulario de EditorProyecto.
 * Extrae los useState del componente para cumplir regla max 3 useState.
 * Patrón: misma estructura que useEditorBlog.ts, ampliado con campos de proyecto. */
import { useState, useCallback, useEffect } from 'react';
import type {
    AdminProject, CreateProjectBody, UpdateProjectBody,
    ProjectLink, ProjectSkill,
} from '../api/admin-projects';
import type {GaleriaImagen} from '../types/contenido';

export function useEditorProyecto(proyecto: AdminProject | null, abierto: boolean) {
    const [titulo, setTitulo] = useState('');
    const [slug, setSlug] = useState('');
    const [cliente, setCliente] = useState('');
    const [descripcion, setDescripcion] = useState('');
    const [contenido, setContenido] = useState('');
    const [imagenUrl, setImagenUrl] = useState('');
    /* [124A-PROJ1] Galería con layout (full/half) en vez de string[] */
    const [galeria, setGaleria] = useState<GaleriaImagen[]>([]);
    const [categorias, setCategorias] = useState<string[]>([]);
    const [tecnologias, setTecnologias] = useState<string[]>([]);
    const [enlaces, setEnlaces] = useState<ProjectLink[]>([]);
    const [skills, setSkills] = useState<ProjectSkill[]>([]);
    const [status, setStatus] = useState('published');
    const [sortOrder, setSortOrder] = useState(0);
    const [metaTitle, setMetaTitle] = useState('');
    const [metaDescription, setMetaDescription] = useState('');
    /* [124A-SHOW1] Categoría showcase editable desde CMS */
    const [showcaseCategory, setShowcaseCategory] = useState('');

    const resetear = useCallback(() => {
        setTitulo(''); setSlug(''); setCliente(''); setDescripcion('');
        setContenido(''); setImagenUrl(''); setGaleria([]); setCategorias([]);
        setTecnologias([]); setEnlaces([]); setSkills([]);
        setStatus('published'); setSortOrder(0);
        setMetaTitle(''); setMetaDescription(''); setShowcaseCategory('');
    }, []);

    useEffect(() => {
        if (proyecto) {
            setTitulo(proyecto.title);
            setSlug(proyecto.slug);
            setCliente(proyecto.client ?? '');
            setDescripcion(proyecto.description);
            setContenido('');
            setImagenUrl(proyecto.featured_image ?? '');
            setGaleria(proyecto.gallery);
            setCategorias(proyecto.categories);
            setTecnologias(proyecto.technologies);
            setEnlaces(proyecto.links);
            setSkills(proyecto.skills);
            setStatus(proyecto.status);
            setSortOrder(proyecto.sort_order);
            setMetaTitle(proyecto.meta_title ?? '');
            setMetaDescription(proyecto.meta_description ?? '');
            setShowcaseCategory(proyecto.showcase_category ?? '');
        } else {
            resetear();
        }
    }, [proyecto, abierto, resetear]);

    const buildBody = useCallback((): CreateProjectBody | UpdateProjectBody => ({
        title: titulo,
        slug,
        client: cliente || undefined,
        description: descripcion || undefined,
        featured_image: imagenUrl || undefined,
        gallery: galeria.length > 0 ? galeria : undefined,
        categories: categorias.length > 0 ? categorias : undefined,
        technologies: tecnologias.length > 0 ? tecnologias : undefined,
        links: enlaces.length > 0 ? enlaces : undefined,
        skills: skills.length > 0 ? skills : undefined,
        status,
        sort_order: sortOrder,
        meta_title: metaTitle || undefined,
        meta_description: metaDescription || undefined,
        showcase_category: showcaseCategory || undefined,
    }), [titulo, slug, cliente, descripcion, imagenUrl, galeria, categorias,
        tecnologias, enlaces, skills, status, sortOrder, metaTitle, metaDescription, showcaseCategory]);

    return {
        titulo, slug, cliente, descripcion, contenido, imagenUrl,
        galeria, categorias, tecnologias, enlaces, skills,
        status, sortOrder, metaTitle, metaDescription, showcaseCategory,
        setTitulo, setSlug, setCliente, setDescripcion, setContenido,
        setImagenUrl, setGaleria, setCategorias, setTecnologias,
        setEnlaces, setSkills, setStatus, setSortOrder,
        setMetaTitle, setMetaDescription, setShowcaseCategory,
        buildBody, resetear,
    };
}
