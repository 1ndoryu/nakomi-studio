import {isAxiosError} from 'axios';
import type {AdminService, AdminServicePlan, SavePlanBody} from '../api/admin-services';

/* [045A-7] El editor inline y el panel CMS deben decidir igual cuándo persistir planes.
 * Este helper evita drift entre ambos flujos y corta PUTs redundantes a /plans. */
export function normalizeServiceSlug(slug: string | undefined): string {
    return (slug ?? '').trim().toLowerCase();
}

export function extractServiceApiMessage(err: unknown, fallback: string): string {
    if (isAxiosError(err) && err.response?.data) {
        const data = err.response.data as Record<string, unknown>;
        if (typeof data.message === 'string' && data.message.trim().length > 0) {
            return data.message;
        }
    }

    return err instanceof Error ? err.message : fallback;
}

function describePlan(plan: SavePlanBody, index: number): string {
    return plan.name.trim() || plan.slug.trim() || `plan ${index + 1}`;
}

function normalizeFeatures(features: unknown): string[] {
    if (!Array.isArray(features)) {
        return [];
    }

    return features.map(feature => {
        if (typeof feature === 'object' && feature !== null && 'texto' in feature) {
            return String((feature as Record<string, unknown>).texto);
        }

        return String(feature);
    });
}

function serializeSavedPlans(plans: SavePlanBody[]): string {
    return JSON.stringify(
        plans.map((plan, index) => ({
            id: plan.id ?? null,
            slug: plan.slug,
            name: plan.name,
            price_cents: plan.price_cents,
            description: plan.description ?? null,
            features: [...plan.features],
            is_highlighted: plan.is_highlighted,
            is_custom: plan.is_custom,
            sort_order: plan.sort_order ?? index,
            phases: plan.phases.map(phase => ({
                phase_number: phase.phase_number,
                title: phase.title,
                description: phase.description ?? null,
                percentage_of_total: phase.percentage_of_total,
                estimated_days: phase.estimated_days,
                max_revisions: phase.max_revisions,
            })),
        })),
    );
}

function convertAdminPlansToSaveBody(planes: AdminServicePlan[]): SavePlanBody[] {
    return planes.map((plan, index) => ({
        id: plan.id,
        slug: plan.slug,
        name: plan.name,
        price_cents: plan.price_cents,
        description: plan.description ?? null,
        features: normalizeFeatures(plan.features),
        is_highlighted: plan.is_highlighted,
        is_custom: plan.is_custom,
        sort_order: index,
        phases: plan.phases.map(phase => ({
            phase_number: phase.phase_number,
            title: phase.title,
            description: phase.description ?? null,
            percentage_of_total: phase.percentage_of_total,
            estimated_days: phase.estimated_days,
            max_revisions: phase.max_revisions,
        })),
    }));
}

export function didServicePlansChange(service: AdminService | null, currentPlans: SavePlanBody[]): boolean {
    if (!service) {
        return currentPlans.length > 0;
    }

    const originalPlans = convertAdminPlansToSaveBody(service.plans);
    return serializeSavedPlans(originalPlans) !== serializeSavedPlans(currentPlans);
}

export function validateServicePlans(plans: SavePlanBody[]): string | null {
    const slugs = new Map<string, string>();

    for (const [planIndex, plan] of plans.entries()) {
        const planName = describePlan(plan, planIndex);
        const slug = plan.slug.trim();
        const name = plan.name.trim();

        if (!slug) {
            return `El ${planName} debe tener un slug`;
        }

        if (slug.length > 50) {
            return `El slug del ${planName} no puede exceder 50 caracteres`;
        }

        const normalizedSlug = normalizeServiceSlug(slug);
        const duplicateSlug = slugs.get(normalizedSlug);
        if (duplicateSlug) {
            return `Los planes "${duplicateSlug}" y "${planName}" no pueden compartir el mismo slug`;
        }
        slugs.set(normalizedSlug, planName);

        if (!name) {
            return `El ${planName} debe tener un nombre`;
        }

        if (name.length > 100) {
            return `El nombre del ${planName} no puede exceder 100 caracteres`;
        }

        if (plan.phases.length === 0) {
            return `El ${planName} debe tener al menos una fase configurada`;
        }

        for (const phase of plan.phases) {
            const title = phase.title.trim();
            if (!title) {
                return `La fase ${phase.phase_number} del ${planName} debe tener un título`;
            }

            if (title.length > 200) {
                return `El título de la fase ${phase.phase_number} del ${planName} no puede exceder 200 caracteres`;
            }
        }
    }

    return null;
}