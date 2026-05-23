/* [084A-24] Hook para el panel VPS (admin only).
 * Consulta Contabo API via backend proxy. */

import {useQuery} from '@tanstack/react-query';
import {apiListVps, type VpsSummary} from '../api/hosting';

const VPS_REFETCH_INTERVAL_MS = 30_000;

export function useVpsPanel() {
    const {data: instances = [], isLoading, error} = useQuery<VpsSummary[]>({
        queryKey: ['vps-instances'],
        queryFn: apiListVps,
        staleTime: 60_000,
        refetchInterval: VPS_REFETCH_INTERVAL_MS,
        refetchOnMount: 'always',
        refetchOnWindowFocus: true,
        retry: 1,
    });

    return {instances, isLoading, error};
}
