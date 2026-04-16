/* [164A-19] Hook del panel admin para despliegues reales de la VPS2.
 * Se separa de useVpsPanel porque Contabo y Coolify responden preguntas distintas:
 * proveedor de VPS vs servicios realmente desplegados dentro de la VPS2. */

import {useQuery} from '@tanstack/react-query';
import {apiListVps2Deployments, type CoolifyDeployment} from '../api/hosting';

export function useVps2DeploymentsPanel() {
    const {data: deployments = [], isLoading, error} = useQuery<CoolifyDeployment[]>({
        queryKey: ['vps2-deployments'],
        queryFn: apiListVps2Deployments,
        staleTime: 60_000,
        retry: 1,
    });

    return {deployments, isLoading, error};
}