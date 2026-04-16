import {useQuery} from '@tanstack/react-query';
import {apiListPublicVpsPlans, type PublicVpsPlan} from '../api/hosting';

const VPS_PLANS_FALLBACK: PublicVpsPlan[] = [
    {
        tier_name: 'vps1',
        display_name: 'Cloud VPS 1',
        description: 'Instancia dedicada para automatizaciones, sitios privados y stacks pequeños con acceso root.',
        monthly_price_cents: 688,
        cpu_cores: 4,
        ram_mb: 8192,
        disk_mb: 204800,
        region: 'EU',
        features: ['4 vCPU dedicados', '8 GB RAM', '200 GB SSD', 'Acceso root y SSH', 'Docker + firewall inicial'],
        approval_required: true,
        recommended: false,
    },
    {
        tier_name: 'vps2',
        display_name: 'Cloud VPS 2',
        description: 'Nodo balanceado para SaaS liviano, APIs privadas y cargas sostenidas con más memoria.',
        monthly_price_cents: 1238,
        cpu_cores: 6,
        ram_mb: 16384,
        disk_mb: 409600,
        region: 'EU',
        features: ['6 vCPU dedicados', '16 GB RAM', '400 GB SSD', 'Acceso root y SSH', 'Docker + firewall inicial'],
        approval_required: true,
        recommended: true,
    },
    {
        tier_name: 'vps3',
        display_name: 'Cloud VPS 3',
        description: 'Servidor para workloads medianos, procesos concurrentes y apps con tráfico constante.',
        monthly_price_cents: 2063,
        cpu_cores: 8,
        ram_mb: 30720,
        disk_mb: 819200,
        region: 'EU',
        features: ['8 vCPU dedicados', '30 GB RAM', '800 GB SSD', 'Acceso root y SSH', 'Docker + firewall inicial'],
        approval_required: true,
        recommended: false,
    },
    {
        tier_name: 'vps4',
        display_name: 'Cloud VPS 4',
        description: 'Capacidad dedicada para pipelines pesados, servicios con mucha memoria y colas intensivas.',
        monthly_price_cents: 3713,
        cpu_cores: 12,
        ram_mb: 49152,
        disk_mb: 1638400,
        region: 'EU',
        features: ['12 vCPU dedicados', '48 GB RAM', '1.6 TB SSD', 'Acceso root y SSH', 'Docker + firewall inicial'],
        approval_required: true,
        recommended: false,
    },
];

export function useVpsCatalog() {
    const query = useQuery({
        queryKey: ['vps-public-plans'],
        queryFn: apiListPublicVpsPlans,
        staleTime: 5 * 60 * 1000,
    });

    return {
        plans: query.data ?? VPS_PLANS_FALLBACK,
        isLoading: query.isLoading,
        error: query.error,
    };
}