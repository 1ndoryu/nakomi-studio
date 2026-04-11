/* [154A-15d] Timeline de actividad visible de una orden.
 * Muestra eventos del activity_log: creación, asignación, cancelación,
 * aprobación de fases, revisiones, completado, etc.
 * [164A-5] Simplificado: dots minimalistas tipo faseTimelineDot en vez de iconos coloreados. */
import React from 'react';
import {useQuery} from '@tanstack/react-query';
import {apiGetOrderActivity, type ActivityEntry} from '../../api/orders';
import {Clock} from 'lucide-react';
import './OrdenHistorialActividad.css';

const ACTION_LABELS: Record<string, string> = {
    order_created: 'Orden creada',
    order_assigned: 'Empleado asignado',
    employee_assigned: 'Empleado asignado',
    order_cancelled: 'Orden cancelada',
    phase_approved: 'Fase aprobada',
    revision_requested: 'Revisión solicitada',
    order_completed: 'Orden completada',
    cancellation_requested: 'Cancelación solicitada',
    cancellation_accepted: 'Cancelación aceptada',
    cancellation_rejected: 'Cancelación rechazada',
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
                    const label = ACTION_LABELS[entry.action] ?? entry.action;
                    const detail = buildDetail(entry);
                    const isLast = idx === entries.length - 1;

                    return (
                        <div key={entry.id} className="actividadItem">
                            <div className="actividadLinea">
                                <div className="actividadDot" />
                                {!isLast && <div className="actividadConector" />}
                            </div>
                            <div className="actividadContenido">
                                <span className="actividadLabel">{label}</span>
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
