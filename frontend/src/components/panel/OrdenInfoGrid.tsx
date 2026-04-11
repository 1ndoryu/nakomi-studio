/* [164A-9] Grid de información de orden — extraído de OrdenDetalle para cumplir límite de líneas. */
import React from 'react';
import { User } from 'lucide-react';
import { formatPrice, PAYMENT_MODE_LABELS, type OrderResponse } from '../../api/orders';

function formatFecha(iso: string): string {
    const d = new Date(iso);
    return d.toLocaleDateString('es-ES', { day: 'numeric', month: 'short', year: 'numeric' });
}

interface OrdenInfoGridProps {
    order: OrderResponse;
}

export const OrdenInfoGrid: React.FC<OrdenInfoGridProps> = ({ order }) => (
    <div className="ordenInfoGrid">
        <div className="ordenInfoItem">
            <span className="ordenInfoLabel">Servicio</span>
            <span className="ordenInfoValue">{order.service_title}</span>
        </div>
        <div className="ordenInfoItem">
            <span className="ordenInfoLabel">Plan</span>
            <span className="ordenInfoValue">{order.plan_name}</span>
        </div>
        <div className="ordenInfoItem">
            <span className="ordenInfoLabel">Número de orden</span>
            <span className="ordenInfoValue">#{order.order_number}</span>
        </div>
        <div className="ordenInfoItem">
            <span className="ordenInfoLabel">Freelancer</span>
            <span className="ordenInfoValue ordenInfoFreelancer">
                <User size={14} />
                {order.assigned_employee_name ?? 'Sin asignar'}
            </span>
        </div>
        <div className="ordenInfoItem">
            <span className="ordenInfoLabel">
                {order.started_at ? 'Fecha de inicio' : 'Fecha de creación'}
            </span>
            <span className="ordenInfoValue">
                {formatFecha(order.started_at ?? order.created_at)}
            </span>
        </div>
        <div className="ordenInfoItem">
            <span className="ordenInfoLabel">Precio</span>
            <span className="ordenInfoValue ordenInfoPrecio">
                {formatPrice(order.final_price_cents, order.currency)}
            </span>
        </div>
        <div className="ordenInfoItem">
            <span className="ordenInfoLabel">Modo de pago</span>
            <span className="ordenInfoValue">{PAYMENT_MODE_LABELS[order.payment_mode]}</span>
        </div>
    </div>
);
