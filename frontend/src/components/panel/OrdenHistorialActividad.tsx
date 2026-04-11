/* [154A-15d] Timeline de actividad visible de una orden.
 * Muestra eventos del activity_log: creación, asignación, cancelación,
 * aprobación de fases, revisiones, completado, etc. */
import React from 'react';
import {useQuery} from '@tanstack/react-query';
import {apiGetOrderActivity, type ActivityEntry} from '../../api/orders';
import {
    Clock, UserPlus, XCircle, CheckCircle, RotateCcw,
    FileText, AlertTriangle, ArrowRightLeft, Trophy,
} from 'lucide-react';
import './OrdenHistorialActividad.css';

const ACTION_CONFIG: Record<string, {label: string; icon: React.ReactNode; color: string}> = {
    order_created: {
        label: 'Orden creada',
        icon: <FileText size={16} />,
        color: 'var(--brand-primary)',
    },
    order_assigned: {
        label: 'Empleado asignado',
        icon: <UserPlus size={16} />,
        color: '#0077b6',
    },
    employee_assigned: {
        label: 'Empleado asignado',
        icon: <UserPlus size={16} />,
        color: '#0077b6',
    },
    order_cancelled: {
        label: 'Orden cancelada',
        icon: <XCircle size={16} />,
        color: '#d62828',
    },
    phase_approved: {
        label: 'Fase aprobada',
        icon: <CheckCircle size={16} />,
        color: '#2d6a4f',
    },
    revision_requested: {
        label: 'Revisión solicitada',
        icon: <RotateCcw size={16} />,
        color: '#c59000',
    },
    order_completed: {
        label: 'Orden completada',
        icon: <Trophy size={16} />,
        color: '#2d6a4f',
    },
    cancellation_requested: {
        label: 'Cancelación solicitada',
        icon: <AlertTriangle size={16} />,
        color: '#d62828',
    },
    cancellation_accepted: {
        label: 'Cancelación aceptada',
        icon: <XCircle size={16} />,
        color: '#d62828',
    },
    cancellation_rejected: {
        label: 'Cancelación rechazada',
        icon: <ArrowRightLeft size={16} />,
        color: '#0077b6',
    },
};

function formatTimestamp(iso: string): string {
    const d = new Date(iso);
    return d.toLocaleDateString('es-ES', {
        day: 'numeric',
        month: 'short',
        year: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
    });
}

function buildDetail(entry: ActivityEntry): string | null {
    if (!entry.details) return null;
    const d = entry.details;
    if (d.reason && typeof d.reason === 'string') return d.reason;
    if (d.phase_number) return `Fase ${d.phase_number}`;
    if (d.service && d.plan) return `${d.service} — ${d.plan}`;
    if (d.refund_cents && typeof d.refund_cents === 'number') {
        return `Reembolso: $${(d.refund_cents as number / 100).toFixed(2)}`;
    }
    return null;
}

interface Props {
    orderId: string;
}

export const OrdenHistorialActividad: React.FC<Props> = ({orderId}) => {
    const {data: entries, isLoading} = useQuery({
        queryKey: ['order-activity', orderId],
        queryFn: () => apiGetOrderActivity(orderId),
        staleTime: 30_000,
    });

    if (isLoading) return <div className="actividadCargando">Cargando actividad...</div>;
    if (!entries || entries.length === 0) return null;

    return (
        <div className="actividadCard">
            <h3 className="actividadTitulo">
                <Clock size={18} /> Actividad
            </h3>
            <div className="actividadTimeline">
                {entries.map((entry, idx) => {
                    const config = ACTION_CONFIG[entry.action] ?? {
                        label: entry.action,
                        icon: <FileText size={16} />,
                        color: 'var(--text-muted)',
                    };
                    const detail = buildDetail(entry);
                    const isLast = idx === entries.length - 1;

                    return (
                        <div key={entry.id} className="actividadItem">
                            <div className="actividadLinea">
                                <div
                                    className="actividadIcono"
                                    style={{backgroundColor: config.color}}
                                >
                                    {config.icon}
                                </div>
                                {!isLast && <div className="actividadConector" />}
                            </div>
                            <div className="actividadContenido">
                                <span className="actividadLabel">{config.label}</span>
                                {detail && <span className="actividadDetalle">{detail}</span>}
                                <span className="actividadFecha">{formatTimestamp(entry.created_at)}</span>
                            </div>
                        </div>
                    );
                })}
            </div>
        </div>
    );
};
