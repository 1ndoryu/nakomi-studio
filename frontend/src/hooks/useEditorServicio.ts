/* [074A-9] Hook interno para el estado del formulario de EditorServicio.
 * Extrae los 10+ useState del componente para cumplir regla max 3 useState.
 * [074A-66] Agrega estado de planes (array mutable) para la tab Planes. */
import {useState, useCallback, useEffect} from 'react';
import type {AdminService, CreateServiceBody, UpdateServiceBody, SavePlanBody} from '../api/admin-services';

/* Estructura local editable para un plan */
export interface PlanEditable {
    id?: string;
    key: string;
    slug: string;
    name: string;
    priceCents: number;
    description: string;
    features: string[];
    isHighlighted: boolean;
    isCustom: boolean;
    sortOrder: number;
    phases: PhaseEditable[];
}

export interface PhaseEditable {
    phaseNumber: number;
    title: string;
    description: string;
    percentageOfTotal: number;
    estimatedDays: number;
    maxRevisions: number;
}

export interface EditorServicioState {
    titulo: string;
    slug: string;
    descripcion: string;
    contenido: string;
    precioCents: number;
    imagenUrl: string;
    metaTitle: string;
    metaDescription: string;
    status: string;
    sortOrder: number;
    planes: PlanEditable[];
    setTitulo: (v: string) => void;
    setSlug: (v: string) => void;
    setDescripcion: (v: string) => void;
    setContenido: (v: string) => void;
    setPrecioCents: (v: number) => void;
    setImagenUrl: (v: string) => void;
    setMetaTitle: (v: string) => void;
    setMetaDescription: (v: string) => void;
    setStatus: (v: string) => void;
    setSortOrder: (v: number) => void;
    setPlanes: (v: PlanEditable[]) => void;
    buildBody: () => CreateServiceBody | UpdateServiceBody;
    buildPlansBody: () => SavePlanBody[];
    resetear: () => void;
}

export function useEditorServicio(servicio: AdminService | null, abierto: boolean): EditorServicioState {
    const [titulo, setTitulo] = useState('');
    const [slug, setSlug] = useState('');
    const [descripcion, setDescripcion] = useState('');
    const [contenido, setContenido] = useState('');
    const [precioCents, setPrecioCents] = useState(0);
    const [imagenUrl, setImagenUrl] = useState('');
    const [metaTitle, setMetaTitle] = useState('');
    const [metaDescription, setMetaDescription] = useState('');
    const [status, setStatus] = useState('draft');
    const [sortOrder, setSortOrder] = useState(0);
    const [planes, setPlanes] = useState<PlanEditable[]>([]);

    /* [084A-3] Convierte AdminServicePlan[] del servidor a PlanEditable[] editables.
     * features en BD puede ser [{texto,incluido}] (fixtures) o ["string"] (guardado desde editor).
     * Siempre normalizamos a string[] extrayendo el campo texto si es objeto. */
    const parsePlans = useCallback((svc: AdminService): PlanEditable[] => {
        return svc.plans.map((p, i) => ({
            id: p.id,
            key: `${p.id}-${i}`,
            slug: p.slug,
            name: p.name,
            priceCents: p.price_cents,
            description: p.description ?? '',
            features: Array.isArray(p.features)
                ? (p.features as unknown[]).map(f =>
                    typeof f === 'object' && f !== null && 'texto' in f
                        ? String((f as Record<string, unknown>).texto)
                        : String(f)
                )
                : [],
            isHighlighted: p.is_highlighted,
            isCustom: p.is_custom,
            sortOrder: i,
            phases: p.phases.map(ph => ({
                phaseNumber: ph.phase_number,
                title: ph.title,
                description: ph.description ?? '',
                percentageOfTotal: ph.percentage_of_total,
                estimatedDays: ph.estimated_days,
                maxRevisions: ph.max_revisions,
            })),
        }));
    }, []);

    const resetear = useCallback(() => {
        setTitulo('');
        setSlug('');
        setDescripcion('');
        setContenido('');
        setPrecioCents(0);
        setImagenUrl('');
        setMetaTitle('');
        setMetaDescription('');
        setStatus('draft');
        setSortOrder(0);
        setPlanes([]);
    }, []);

    /* Sincronizar con servicio seleccionado */
    useEffect(() => {
        if (servicio) {
            setTitulo(servicio.title);
            setSlug(servicio.slug);
            setDescripcion(servicio.description ?? '');
            setContenido(servicio.content ?? '');
            setPrecioCents(servicio.base_price_cents);
            setImagenUrl(servicio.image_url ?? '');
            setMetaTitle(servicio.meta_title ?? '');
            setMetaDescription(servicio.meta_description ?? '');
            setStatus(servicio.status);
            setSortOrder(servicio.sort_order);
            setPlanes(parsePlans(servicio));
        } else {
            resetear();
        }
    }, [servicio, abierto, resetear, parsePlans]);

    const buildBody = useCallback(() => ({
        title: titulo,
        slug,
        description: descripcion || undefined,
        base_price_cents: precioCents,
        image_url: imagenUrl || undefined,
        content: contenido || undefined,
        meta_title: metaTitle || undefined,
        meta_description: metaDescription || undefined,
        status,
        sort_order: sortOrder,
    }), [titulo, slug, descripcion, precioCents, imagenUrl, contenido, metaTitle, metaDescription, status, sortOrder]);

    /* [074A-66] Convierte PlanEditable[] a SavePlanBody[] para el API */
    const buildPlansBody = useCallback((): SavePlanBody[] => {
        return planes.map((p, i) => ({
            id: p.id,
            slug: p.slug,
            name: p.name,
            price_cents: p.priceCents,
            description: p.description || null,
            features: p.features,
            is_highlighted: p.isHighlighted,
            is_custom: p.isCustom,
            sort_order: i,
            phases: p.phases.map(ph => ({
                phase_number: ph.phaseNumber,
                title: ph.title,
                description: ph.description || null,
                percentage_of_total: ph.percentageOfTotal,
                estimated_days: ph.estimatedDays,
                max_revisions: ph.maxRevisions,
            })),
        }));
    }, [planes]);

    return {
        titulo, slug, descripcion, contenido, precioCents, imagenUrl,
        metaTitle, metaDescription, status, sortOrder, planes,
        setTitulo, setSlug, setDescripcion, setContenido, setPrecioCents,
        setImagenUrl, setMetaTitle, setMetaDescription, setStatus, setSortOrder,
        setPlanes, buildBody, buildPlansBody, resetear,
    };
}
