import {FiltroCategoria} from '../types/navegacion';

const CATEGORY_ORDER = ['web', 'software', 'ai', 'branding'];

const CATEGORY_LABELS: Record<string, string> = {
    web: 'Diseño Web',
    software: 'Software',
    ai: 'Inteligencia Artificial',
    branding: 'Branding',
};

const CATEGORY_ALIASES: Record<string, string> = {
    web: 'web',
    'diseno-web': 'web',
    'diseno-websites': 'web',
    'diseno-sitios-web': 'web',
    'diseno-web-y-ux': 'web',
    'diseño-web': 'web',
    software: 'software',
    app: 'software',
    apps: 'software',
    aplicacion: 'software',
    aplicaciones: 'software',
    'desarrollo-apps': 'software',
    'desarrollo-aplicaciones': 'software',
    ia: 'ai',
    ai: 'ai',
    'inteligencia-artificial': 'ai',
    branding: 'branding',
    marca: 'branding',
    'identidad-marca': 'branding',
    'identidad-de-marca': 'branding',
};

function slugifyCategory(value: string): string {
    return value
        .normalize('NFD')
        .replace(/[\u0300-\u036f]/g, '')
        .toLowerCase()
        .trim()
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/^-+|-+$/g, '');
}

function splitCategoryValue(value: string): string[] {
    return value
        .split(',')
        .map(item => item.trim())
        .filter(Boolean);
}

function labelFromCategoryId(categoryId: string): string {
    if (CATEGORY_LABELS[categoryId]) {
        return CATEGORY_LABELS[categoryId];
    }

    return categoryId
        .split('-')
        .filter(Boolean)
        .map(part => part.charAt(0).toUpperCase() + part.slice(1))
        .join(' ');
}

export function normalizeCategoryId(value: string): string {
    const slug = slugifyCategory(value);
    return CATEGORY_ALIASES[slug] ?? slug;
}

/* [065A-3] Proyectos llegan del CMS como CSV (por ejemplo "software, web") y
 * servicios ya usan arrays reales desde el nuevo contrato del backend. Este helper
 * unifica ambos formatos para que el filtro público solo muestre categorías existentes. */
export function extractCategoryIds(categories: string | string[] | null | undefined): string[] {
    const values = Array.isArray(categories)
        ? categories.flatMap(item => splitCategoryValue(item))
        : typeof categories === 'string'
            ? splitCategoryValue(categories)
            : [];

    return Array.from(new Set(values.map(normalizeCategoryId).filter(Boolean)));
}

export function buildFilterCategories(categoryIds: Iterable<string>): FiltroCategoria[] {
    const unique = Array.from(new Set(Array.from(categoryIds).filter(Boolean)));
    const sorted = unique.sort((left, right) => {
        const leftIndex = CATEGORY_ORDER.indexOf(left);
        const rightIndex = CATEGORY_ORDER.indexOf(right);

        if (leftIndex === -1 && rightIndex === -1) {
            return left.localeCompare(right);
        }

        if (leftIndex === -1) return 1;
        if (rightIndex === -1) return -1;
        return leftIndex - rightIndex;
    });

    return [
        {id: 'todos', label: 'Todos'},
        ...sorted.map(categoryId => ({
            id: categoryId,
            label: labelFromCategoryId(categoryId),
        })),
    ];
}