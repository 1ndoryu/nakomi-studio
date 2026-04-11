/* [094A-8] Estadísticas de recursos de hosting.
 * Consume el endpoint /api/hosting/subscriptions/:id/stats para datos reales.
 * Uptime se calcula desde el historial de eventos del backend.
 * Storage/bandwidth muestran límites; uso real requiere monitoreo futuro (Coolify). */

import {useQuery} from '@tanstack/react-query';
import type {CSSProperties} from 'react';
import type {HostingSubscription} from '../../api/hosting';
import {apiGetHostingStats} from '../../api/hosting';
import './HostingStats.css';

function formatStorage(mb: number): string {
    return mb >= 1024 ? `${(mb / 1024).toFixed(1)} GB` : `${mb} MB`;
}

function ResourceBar({label, used, total, unit, unavailable}: {
    label: string;
    used: number | null;
    total: number;
    unit: string;
    unavailable?: boolean;
}) {
    if (unavailable || used === null) {
        return (
            <div className="hostingStatBarra">
                <div className="hostingStatBarraHeader">
                    <span className="hostingStatBarraLabel">{label}</span>
                    <span className="hostingStatBarraValor">
                        {unit === 'MB' ? formatStorage(total) : `${total} ${unit}`} (límite)
                    </span>
                </div>
                <div className="hostingStatBarraTrack">
                    {/* [104A-11] El ancho vive en una CSS var dinámica; Sentinel lo trata
                     * como excepción válida y evitamos width inline real. */}
                    <div
                        className="hostingStatBarraFill hostingStatBarraFill--pendiente"
                        style={{'--hosting-bar-width': '0%'} as CSSProperties}
                    />
                </div>
                <span className="hostingStatBarraNota">Monitoreo pendiente</span>
            </div>
        );
    }

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
                    style={{'--hosting-bar-width': `${percent}%`} as CSSProperties}
                />
            </div>
        </div>
    );
}

export function HostingStats({sub}: {sub: HostingSubscription}) {
    const {data: stats, isLoading} = useQuery({
        queryKey: ['hosting-stats', sub.id],
        queryFn: () => apiGetHostingStats(sub.id),
        enabled: sub.status === 'active' || sub.status === 'provisioning',
        staleTime: 60_000,
        refetchInterval: 120_000,
    });

    if (sub.status !== 'active' && sub.status !== 'provisioning') {
        return null;
    }

    if (isLoading) {
        return <div className="hostingStats hostingStats--loading">Cargando estadísticas...</div>;
    }

    if (!stats) {
        return null;
    }

    return (
        <div className="hostingStats">
            {/* [114A-15+] CPU y RAM reales obtenidos via Docker stats SSH */}
            {stats.cpu_percent !== null && (
                <ResourceBar
                    label="CPU"
                    used={stats.cpu_percent}
                    total={100}
                    unit="%"
                />
            )}
            {stats.ram_used_mb !== null && stats.ram_limit_mb !== null && (
                <ResourceBar
                    label="Memoria RAM"
                    used={stats.ram_used_mb}
                    total={stats.ram_limit_mb}
                    unit="MB"
                />
            )}
            <ResourceBar
                label="Almacenamiento"
                used={stats.storage_used_mb}
                total={stats.storage_limit_mb}
                unit="MB"
                unavailable={!stats.monitoring_available}
            />
            <ResourceBar
                label="Ancho de banda"
                used={stats.bandwidth_used_gb}
                total={stats.bandwidth_limit_gb}
                unit="GB"
                unavailable={!stats.monitoring_available}
            />
            <div className="hostingStatUptime">
                <span className="hostingStatUptimeDot" />
                Uptime: {stats.uptime_percent.toFixed(2)}%
                {stats.active_since && (
                    <span className="hostingStatUptimeSince">
                        {' '}desde {new Date(stats.active_since).toLocaleDateString()}
                    </span>
                )}
            </div>
            <div className="hostingStatMeta">
                <span>{stats.total_events} eventos registrados</span>
                {stats.last_event_at && (
                    <span>Último: {new Date(stats.last_event_at).toLocaleDateString()}</span>
                )}
            </div>
        </div>
    );
}
