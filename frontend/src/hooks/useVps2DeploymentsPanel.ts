/* [164A-19] Hook del panel admin para despliegues reales de la VPS2.
 * Se separa de useVpsPanel porque Contabo y Coolify responden preguntas distintas:
 * proveedor de VPS vs servicios realmente desplegados dentro de la VPS2.
 * [215A-14] Combina despliegues Coolify con datos de Contabo para mostrar
 * resumen del VPS (CPU, RAM, disco) junto con los despliegues. */

import {useQuery} from '@tanstack/react-query';
import {apiListVps2Deployments, apiListVps, type CoolifyDeployment, type VpsSummary} from '../api/hosting';

export function useVps2DeploymentsPanel() {
    const {data: deployments = [], isLoading: isLoadingDeployments, error: deploymentsError} = useQuery<CoolifyDeployment[]>({
        queryKey: ['vps2-deployments'],
        queryFn: apiListVps2Deployments,
        staleTime: 60_000,
        retry: 1,
    });

    /* [215A-14] Datos de VPS Contabo para resumen de infraestructura */
    const {data: vpsInstances = [], isLoading: isLoadingVps} = useQuery<VpsSummary[]>({
        queryKey: ['vps-instances'],
        queryFn: apiListVps,
        staleTime: 60_000,
        retry: 1,
    });

    const isLoading = isLoadingDeployments || isLoadingVps;

    return {deployments, vpsInstances, isLoading, error: deploymentsError};
}