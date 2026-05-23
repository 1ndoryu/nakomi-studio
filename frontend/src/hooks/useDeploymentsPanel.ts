/* [164A-19] Hook del panel admin para despliegues reales de infraestructura.
 * Se separa de useVpsPanel porque Contabo y Coolify responden preguntas distintas:
 * proveedor de VPS vs servicios realmente desplegados en servidores configurados.
 * [215A-14] Combina despliegues Coolify con datos de VPS para mostrar
 * resumen de infraestructura junto con los despliegues. */

import {useQuery} from '@tanstack/react-query';
import {apiGetDeploymentMetrics, apiListDeployments, apiListVps, type CoolifyDeployment, type DeploymentMetricsResponse, type VpsSummary} from '../api/hosting';

export const DEPLOYMENTS_QUERY_KEY = ['deployments'] as const;

export function useDeploymentsPanel() {
    const {data: deployments = [], isLoading: isLoadingDeployments, error: deploymentsError} = useQuery<CoolifyDeployment[]>({
        queryKey: DEPLOYMENTS_QUERY_KEY,
        queryFn: apiListDeployments,
        staleTime: 60_000,
        retry: 1,
    });

    /* [215A-14] Datos de VPS para resumen de infraestructura */
    const {data: vpsInstances = [], isLoading: isLoadingVps} = useQuery<VpsSummary[]>({
        queryKey: ['vps-instances'],
        queryFn: apiListVps,
        staleTime: 60_000,
        retry: 1,
    });

    const isLoading = isLoadingDeployments || isLoadingVps;

    return {deployments, vpsInstances, isLoading, error: deploymentsError};
}

export function useDeploymentMetrics(deploymentUuid: string, enabled: boolean) {
    return useQuery<DeploymentMetricsResponse>({
        queryKey: ['deployment-metrics', deploymentUuid, '24h'],
        queryFn: () => apiGetDeploymentMetrics(deploymentUuid, '24h'),
        enabled,
        staleTime: 120_000,
        retry: 1,
    });
}