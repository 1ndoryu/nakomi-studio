import {useQuery} from '@tanstack/react-query';
import {
    apiListPublicHostingPlans,
    HOSTING_PLANS_FALLBACK,
    toHostingPlanInfo,
    type HostingPlanInfo,
} from '../api/hosting';

export function useHostingCatalog() {
    const query = useQuery({
        queryKey: ['hosting-public-plans'],
        queryFn: apiListPublicHostingPlans,
        staleTime: 5 * 60 * 1000,
    });

    const plans: HostingPlanInfo[] = query.data?.map(toHostingPlanInfo) ?? HOSTING_PLANS_FALLBACK;

    return {
        plans,
        isLoading: query.isLoading,
        error: query.error,
    };
}