/* [205A-1] Helpers, constantes y tipos del configurador VPS extraídos de la island
 * para mantener el componente bajo 300 líneas (regla 8). */
import type {PublicVpsPlan} from '../api/hosting';

export function formatMoney(cents: number): string {
    const value = cents / 100;
    return `$${value.toFixed(cents % 100 === 0 ? 0 : 2)}`;
}

export function formatRam(ramMb: number): string {
    return `${ramMb / 1024} GB`;
}

export function formatPort(speedMbps: number): string {
    return speedMbps >= 1000 ? `${speedMbps / 1000} Gbit/s` : `${speedMbps} Mbit/s`;
}

export function storageLabel(plan: PublicVpsPlan): string {
    return plan.storage_options.length > 0
        ? plan.storage_options.join(' / ')
        : `${plan.disk_mb / 1024} GB ${plan.storage_type}`;
}

/* Ubicaciones reales de Contabo con continente para agrupar en <optgroup>.
 * UK y ubicaciones fuera de EU tienen costo adicional (varía por plan). */
export interface ContaboLocation {
    code: string;
    label: string;
    continent: string;
    free: boolean;
}

export const VPS_LOCATIONS: ContaboLocation[] = [
    {code: 'EU',      label: 'European Union', continent: 'Europa',  free: true},
    {code: 'UK',      label: 'United Kingdom', continent: 'Europa',  free: false},
    {code: 'US-EAST', label: 'US East',         continent: 'América', free: false},
    {code: 'US-WEST', label: 'US West',         continent: 'América', free: false},
    {code: 'SIN',     label: 'Singapore',       continent: 'Asia',    free: false},
    {code: 'AUS',     label: 'Australia',       continent: 'Oceanía', free: false},
];

export const VPS_CONTINENTS = ['Europa', 'América', 'Asia', 'Oceanía'] as const;

/* Costos adicionales de storage según lista de Contabo (informativos, sin markup aplicado) */
const STORAGE_EXTRA: Record<string, string> = {
    '150 GB SSD':  'Incluido',
    '300 GB SSD':  '+€1.55/mes',
    '75 GB NVMe':  'Incluido',
    '150 GB NVMe': '+€1.85/mes',
    '200 GB SSD':  'Incluido',
    '400 GB SSD':  '+€1.95/mes',
    '100 GB NVMe': 'Incluido',
    '200 GB NVMe': '+€2.55/mes',
};

export function storageExtraLabel(opt: string): string {
    return STORAGE_EXTRA[opt] ?? '';
}

export const OS_IMAGES: string[] = [
    'Ubuntu 24.04 LTS',
    'Ubuntu 22.04 LTS',
    'Debian 12',
    'Debian 11',
    'Rocky Linux 9',
    'AlmaLinux 9',
];
