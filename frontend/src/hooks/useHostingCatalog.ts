import {useQuery} from '@tanstack/react-query';
import {
    apiListPublicHostingPlans,
    HOSTING_PLANS_FALLBACK,
    toHostingPlanInfo,
    type HostingPlanInfo,
} from '../api/hosting';

export type HostingCatalogKind = 'all' | 'wordpress' | 'normal';

function isNormalHostingPlan(planId: string): boolean {
    return planId.startsWith('normal-');
}

function filterPlans(plans: HostingPlanInfo[], kind: HostingCatalogKind): HostingPlanInfo[] {
    if (kind === 'all') return plans;
    return plans.filter(plan => kind === 'normal' ? isNormalHostingPlan(plan.id) : !isNormalHostingPlan(plan.id));
}

export function useHostingCatalog(kind: HostingCatalogKind = 'all') {
    const query = useQuery({
        queryKey: ['hosting-public-plans'],
        queryFn: apiListPublicHostingPlans,
        staleTime: 5 * 60 * 1000,
    });

    const allPlans: HostingPlanInfo[] = query.data?.map(toHostingPlanInfo) ?? HOSTING_PLANS_FALLBACK;
    const plans = filterPlans(allPlans, kind);

    return {
        plans,
        isLoading: query.isLoading,
        error: query.error,
    };
}