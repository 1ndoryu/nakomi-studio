/* [084A-24] Panel VPS: muestra instancias Contabo con stats reales.
 * Solo visible para admin. Consume GET /api/hosting/vps (proxy Contabo API). */

import React from 'react';
import {Server, Cpu, HardDrive, MemoryStick, Globe, Activity} from 'lucide-react';
import {useVpsPanel} from '../../hooks/useVpsPanel';
import type {VpsSummary} from '../../api/hosting';
import './VpsPanel.css';

function formatMb(mb: number): string {
    if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`;
    return `${mb} MB`;
}

function getVpsErrorMessage(error: unknown): string {
    const apiMessage = (error as {
        response?: {data?: {message?: string}};
    })?.response?.data?.message;

    if (typeof apiMessage === 'string' && apiMessage.trim()) {
        return apiMessage;
    }

    if (error instanceof Error && error.message) {
        return error.message;
    }

    return 'No se pudo conectar con la API de Contabo';
}

function VpsCard({instance}: {instance: VpsSummary}) {
    const statusClass = instance.status === 'running'
        ? 'vpsStatus--running'
        : instance.status === 'stopped'
            ? 'vpsStatus--stopped'
            : 'vpsStatus--other';

    return (
        <div className="vpsCard">
            <div className="vpsCardHeader">
                <Server size={20} strokeWidth={1.4} />
                <h4 className="vpsCardNombre">{instance.name || `VPS #${instance.instance_id}`}</h4>
                <span className={`vpsStatus ${statusClass}`}>
                    {instance.status}
                </span>
            </div>

            <div className="vpsCardStats">
                <div className="vpsStat">
                    <Globe size={14} />
                    <span className="vpsStatLabel">IP</span>
                    <span className="vpsStatValor">{instance.ip}</span>
                </div>
                <div className="vpsStat">
                    <Activity size={14} />
                    <span className="vpsStatLabel">Región</span>
                    <span className="vpsStatValor">{instance.region}</span>
                </div>
                <div className="vpsStat">
                    <Cpu size={14} />
                    <span className="vpsStatLabel">CPU</span>
                    <span className="vpsStatValor">{instance.cpu_cores} vCPU</span>
                </div>
                <div className="vpsStat">
                    <MemoryStick size={14} />
                    <span className="vpsStatLabel">RAM</span>
                    <span className="vpsStatValor">{formatMb(instance.ram_mb)}</span>
                </div>
                <div className="vpsStat">
                    <HardDrive size={14} />
                    <span className="vpsStatLabel">Disco</span>
                    <span className="vpsStatValor">{formatMb(instance.disk_mb)}</span>
                </div>
            </div>
        </div>
    );
}

export const VpsPanel: React.FC = () => {
    const {instances, isLoading, error} = useVpsPanel();

    if (isLoading) {
        return (
            <div className="vpsLoading">
                <Server size={28} strokeWidth={1.2} />
                <p>Consultando VPS...</p>
            </div>
        );
    }

    if (error) {
        return (
            <div className="vpsError">
                <p>{getVpsErrorMessage(error)}</p>
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
                    <VpsCard key={inst.instance_id} instance={inst} />
                ))}
            </div>
        </div>
    );
};
