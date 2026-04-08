/* [044A-38 Fase 4] Sección "Disponibles" del panel (empleado/admin).
 * [074A-58] Rediseño: cards horizontales matching ordenCard/hostingCard pattern. */
import React, {useCallback, useState} from 'react';
import {FolderOpen, UserPlus, Package} from 'lucide-react';
import {useDisponibles} from '../../hooks/useAsignaciones';
import {PAYMENT_MODE_LABELS, formatPrice} from '../../api/orders';
import type {OrderResponse} from '../../api/orders';
import {Button} from '../ui/Button';
import './SeccionDisponibles.css';

export const SeccionDisponibles: React.FC = () => {
    const {disponibles, cargando, tomarOrden, tomando} = useDisponibles();
    const [error, setError] = useState<string | null>(null);

    const handleTomar = useCallback(async (orderId: string) => {
        try {
            setError(null);
            await tomarOrden(orderId);
        } catch (err: unknown) {
            setError(extraerError(err));
        }
    }, [tomarOrden]);

    if (cargando) {
        return (
            <div className="disponiblesLoading">
                <div className="disponiblesSpinner" />
                <p>Cargando órdenes disponibles...</p>
            </div>
        );
    }

    if (disponibles.length === 0) {
        return (
            <div className="disponiblesVacio">
                <FolderOpen size={48} strokeWidth={1.2} />
                <h3>Sin órdenes disponibles</h3>
                <p>No hay órdenes esperando asignación en este momento.</p>
            </div>
        );
    }

    return (
        <div className="disponiblesContenedor">
            {error && <div className="disponiblesError">{error}</div>}
            <div className="disponiblesLista">
                {disponibles.map(orden => (
                    <OrdenDisponibleCard
                        key={orden.id}
                        orden={orden}
                        onTomar={handleTomar}
                        tomando={tomando}
                    />
                ))}
            </div>
        </div>
    );
};

function OrdenDisponibleCard({
    orden,
    onTomar,
    tomando,
}: {
    orden: OrderResponse;
    onTomar: (id: string) => void;
    tomando: boolean;
}) {
    return (
        <div className="disponibleCard">
            <div className="disponibleCardIcono">
                <Package size={28} strokeWidth={1.4} />
            </div>
            <div className="disponibleCardBody">
                <div className="disponibleCardHeader">
                    <h3 className="disponibleCardTitulo">
                        #{orden.order_number} — {orden.service_title}
                    </h3>
                    <span className="disponibleCardBadge">Disponible</span>
                </div>
                <span className="disponibleCardPlan">{orden.plan_name}</span>
                {orden.client_notes && (
                    <p className="disponibleCardNotas">{orden.client_notes}</p>
                )}
                <div className="disponibleCardFooter">
                    <span className="disponibleCardPrecio">
                        {formatPrice(orden.final_price_cents, orden.currency)}
                    </span>
                    <span>{PAYMENT_MODE_LABELS[orden.payment_mode]}</span>
                    <span>{orden.total_phases} fases</span>
                    <Button
                        variante="primario"
                        tamano="pequeno"
                        className="disponibleBtnTomar"
                        onClick={() => onTomar(orden.id)}
                        disabled={tomando}
                        type="button"
                    >
                        <UserPlus size={14} />
                        {tomando ? 'Asignando...' : 'Tomar'}
                    </Button>
                </div>
            </div>
        </div>
    );
}

function extraerError(err: unknown): string {
    if (typeof err === 'object' && err !== null && 'response' in err) {
        const resp = (err as {response?: {data?: {message?: string; error?: string}}}).response;
        if (resp?.data?.message) return resp.data.message;
        if (resp?.data?.error) return resp.data.error;
    }
    return 'Error al tomar la orden';
}
