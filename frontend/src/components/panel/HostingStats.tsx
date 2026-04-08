/* [084A-24] Estadísticas simuladas de recursos de hosting.
 * Genera valores deterministas basados en el subscription ID para consistencia.
 * Cuando tengamos integración real con Coolify, estos datos vendrán del backend. */

import type {HostingSubscription} from '../../api/hosting';
import './HostingStats.css';

/* Hash simple y determinista para generar stats consistentes por ID */
function hashId(id: string): number {
    let h = 0;
    for (let i = 0; i < id.length; i++) {
        h = ((h << 5) - h + id.charCodeAt(i)) | 0;
    }
    return Math.abs(h);
}

/* Límites por plan en MB */
const PLAN_STORAGE: Record<string, number> = {
    basico: 5120,
    pro: 20480,
    ecommerce: 51200,
};

const PLAN_BANDWIDTH: Record<string, number> = {
    basico: 50,
    pro: 200,
    ecommerce: 500,
};

interface SimulatedStats {
    storageUsedMb: number;
    storageLimitMb: number;
    bandwidthUsedGb: number;
    bandwidthLimitGb: number;
    uptimePercent: number;
}

function generateStats(sub: HostingSubscription): SimulatedStats {
    const h = hashId(sub.id);
    const storageLimitMb = sub.storage_limit_mb || PLAN_STORAGE[sub.plan] || 5120;
    const bandwidthLimitGb = PLAN_BANDWIDTH[sub.plan] || 50;

    /* Simular uso entre 10% y 65% para que se vea realista */
    const storagePercent = 10 + (h % 56);
    const bandwidthPercent = 5 + ((h >> 8) % 40);
    const uptimePercent = 99.5 + ((h % 50) / 100);

    return {
        storageUsedMb: Math.round(storageLimitMb * storagePercent / 100),
        storageLimitMb,
        bandwidthUsedGb: Math.round(bandwidthLimitGb * bandwidthPercent / 100),
        bandwidthLimitGb,
        uptimePercent: Math.min(uptimePercent, 99.99),
    };
}

function formatStorage(mb: number): string {
    return mb >= 1024 ? `${(mb / 1024).toFixed(1)} GB` : `${mb} MB`;
}

function ResourceBar({label, used, total, unit}: {
    label: string;
    used: number;
    total: number;
    unit: string;
}) {
    const percent = Math.min((used / total) * 100, 100);
    const level = percent > 80 ? 'alto' : percent > 50 ? 'medio' : 'bajo';

    return (
        <div className="hostingStatBarra">
            <div className="hostingStatBarraHeader">
                <span className="hostingStatBarraLabel">{label}</span>
                <span className="hostingStatBarraValor">
                    {unit === 'MB' ? formatStorage(used) : `${used} ${unit}`} / {unit === 'MB' ? formatStorage(total) : `${total} ${unit}`}
                </span>
            </div>
            <div className="hostingStatBarraTrack">
                <div
                    className={`hostingStatBarraFill hostingStatBarraFill--${level}`}
                    style={{width: `${percent}%`}}
                />
            </div>
        </div>
    );
}

export function HostingStats({sub}: {sub: HostingSubscription}) {
    if (sub.status !== 'active' && sub.status !== 'provisioning') {
        return null;
    }

    const stats = generateStats(sub);

    return (
        <div className="hostingStats">
            <ResourceBar
                label="Almacenamiento"
                used={stats.storageUsedMb}
                total={stats.storageLimitMb}
                unit="MB"
            />
            <ResourceBar
                label="Ancho de banda"
                used={stats.bandwidthUsedGb}
                total={stats.bandwidthLimitGb}
                unit="GB"
            />
            <div className="hostingStatUptime">
                <span className="hostingStatUptimeDot" />
                Uptime: {stats.uptimePercent.toFixed(2)}%
            </div>
        </div>
    );
}
