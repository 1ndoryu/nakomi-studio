/* [084A-24] Panel VPS: muestra instancias Contabo con stats reales.
 * Solo visible para admin. Consume GET /api/hosting/vps (proxy Contabo API). */

import React from 'react';
import {useTranslation} from 'react-i18next';
import {Server, Cpu, HardDrive, MemoryStick, Globe, Activity} from 'lucide-react';
import {useVpsPanel} from '../../hooks/useVpsPanel';
import type {VpsSummary} from '../../api/hosting';
import './VpsPanel.css';

function formatMb(mb: number | null | undefined): string {
    if (mb == null) return '—';
    if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`;
    return `${mb} MB`;
}

function formatCpu(cpu: number | null | undefined): string {
    return cpu == null ? '—' : `${cpu} vCPU`;
}

function formatPercent(percent: number | null | undefined): string {
    return percent == null ? '—' : `${percent.toFixed(1)}%`;
}

function formatUsage(used: number | null | undefined, limit: number | null | undefined): string {
    if (used == null) return '—';
    if (limit == null || limit <= 0) return formatMb(used);
    return `${formatMb(used)} / ${formatMb(limit)}`;
}

function getVpsErrorMessage(error: unknown, t?: (key: string, fb: string) => string): string {
    const apiMessage = (error as {
        response?: {data?: {message?: string}};
    })?.response?.data?.message;

    if (typeof apiMessage === 'string' && apiMessage.trim()) {
        return apiMessage;
    }

    if (error instanceof Error && error.message) {
        return error.message;
    }

    const fn = t || ((_: string, fb: string) => fb);
    return fn('panel.vps_stats.error', 'No se pudo conectar con la API de Contabo');
}

function VpsCard({instance, t}: {instance: VpsSummary; t: (key: string, fb: string) => string}) {
    const statusClass = instance.status === 'running' || instance.status === 'configured'
        ? 'vpsStatus--running'
        : instance.status === 'stopped'
            ? 'vpsStatus--stopped'
            : 'vpsStatus--other';

    return (
        <div className="vpsCard">
            <div className="vpsCardHeader">
                <Server size={20} strokeWidth={1.4} />
                <h4 className="vpsCardNombre">{instance.label || instance.name || `VPS #${instance.instance_id ?? 'configurada'}`}</h4>
                <span className={`vpsStatus ${statusClass}`}>
                    {instance.status}
                </span>
            </div>

            <div className="vpsCardStats">
                <div className="vpsStat">
                    <Globe size={14} />
                    <span className="vpsStatLabel">{t('panel.vps_stats.ip', 'IP')}</span>
                    <span className="vpsStatValor">{instance.ip}</span>
                </div>
                <div className="vpsStat">
                    <Activity size={14} />
                    <span className="vpsStatLabel">{t('panel.vps_stats.origin', 'Origen')}</span>
                    <span className="vpsStatValor">{instance.region}</span>
                </div>
                <div className="vpsStat">
                    <Cpu size={14} />
                    <span className="vpsStatLabel">{t('panel.vps_stats.cpu', 'CPU')}</span>
                    <span className="vpsStatValor">{formatCpu(instance.cpu_cores)}</span>
                </div>
                <div className="vpsStat">
                    <MemoryStick size={14} />
                    <span className="vpsStatLabel">{t('panel.vps_stats.ram', 'RAM')}</span>
                    <span className="vpsStatValor">{formatMb(instance.ram_mb)}</span>
                </div>
                <div className="vpsStat">
                    <HardDrive size={14} />
                    <span className="vpsStatLabel">{t('panel.vps_stats.disk', 'Disco')}</span>
                    <span className="vpsStatValor">{formatMb(instance.disk_mb)}</span>
                </div>
                <div className="vpsStat">
                    <Activity size={14} />
                    <span className="vpsStatLabel">{t('panel.vps_stats.cpu_avg', 'CPU prom.')}</span>
                    <span className="vpsStatValor">{formatPercent(instance.cpu_avg_1h)}</span>
                </div>
                <div className="vpsStat">
                    <MemoryStick size={14} />
                    <span className="vpsStatLabel">{t('panel.vps_stats.ram_avg', 'RAM prom.')}</span>
                    <span className="vpsStatValor">{formatUsage(instance.ram_used_mb, instance.ram_limit_mb)}</span>
                </div>
                <div className="vpsStat">
                    <HardDrive size={14} />
                    <span className="vpsStatLabel">{t('panel.vps_stats.disk_used', 'Disco usado')}</span>
                    <span className="vpsStatValor">{formatUsage(instance.disk_used_mb, instance.disk_limit_mb)}</span>
                </div>
            </div>
        </div>
    );
}

export const VpsPanel: React.FC = () => {
    const {t} = useTranslation();
    const {instances, isLoading, error} = useVpsPanel();

    if (isLoading) {
        return (
            <div className="vpsLoading">
                <Server size={28} strokeWidth={1.2} />
                <p>{t('panel.vps_stats.loading', 'Consultando VPS...')}</p>
            </div>
        );
    }

    if (error) {
        return (
            <div className="vpsError">
                <p>{getVpsErrorMessage(error, t)}</p>
            </div>
        );
    }

    if (instances.length === 0) {
        return (
            <div className="vpsVacio">
                <Server size={36} strokeWidth={1.2} />
                <p>No se encontraron instancias VPS</p>
            </div>
        );
    }

    return (
        <div className="vpsContenedor">
            <div className="vpsLista">
                {instances.map(inst => (
                    <VpsCard key={inst.inventory_id} instance={inst} t={t} />
                ))}
            </div>
        </div>
    );
};
