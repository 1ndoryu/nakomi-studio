/* [035A-23] Sub-componentes de SeccionPagos extraídos para cumplir límite de 300 líneas.
 * PagoResumenCard: card clicable que abre el modal de factura.
 * FacturaLinea: fila individual de una transacción de pago. */

import OptimizedImage from '../ui/OptimizedImage';
import { Button } from '../ui/Button';
import { PAYMENT_STATUS_LABELS, PAYMENT_STATUS_CLASS, type PaymentResponse } from '../../api/payments';
import { PAYMENT_MODE_LABELS, formatPrice, type OrderResponse } from '../../api/orders';
import { getServiceImage } from '../../utils/serviceImages';

export function PagoResumenCard({ order, onOpen }: { order: OrderResponse; onOpen: () => void }) {
    return (
        <Button
            type="button"
            className="pagoResumenCard"
            onClick={onOpen}
            variante="texto"
            aria-label={`Ver pagos de la orden #${order.order_number}`}
        >
            <div className="pagoResumenCardImagenWrapper">
                <OptimizedImage
                    className="pagoResumenCardImagen"
                    src={getServiceImage(order.service_slug)}
                    alt={order.service_title}
                    loading="lazy"
                />
            </div>
            <div className="pagoResumenCardBody">
                <div className="pagoResumenCardHeader">
                    <h3 className="pagoResumenCardTitulo">{order.service_title}</h3>
                    <span className="pagoResumenCardBadge">
                        Orden #{order.order_number}
                    </span>
                </div>
                <div className="pagoResumenCardFooter">
                    <span className="pagoResumenCardPrecio">{formatPrice(order.final_price_cents, order.currency)}</span>
                    <span className="pagoResumenCardModo">{PAYMENT_MODE_LABELS[order.payment_mode]}</span>
                    <span className="pagoResumenCardFecha">
                        {new Date(order.created_at).toLocaleDateString('es-ES', {
                            day: 'numeric', month: 'short',
                        })}
                    </span>
                </div>
            </div>
        </Button>
    );
}

export function FacturaLinea({ payment }: { payment: PaymentResponse }) {
    return (
        <div className="pagosFacturaLinea">
            <div className="pagosFacturaLineaInfo">
                <span className="pagosFacturaLineaConcepto">
                    {payment.description || (payment.phase_number ? `Fase ${payment.phase_number}` : 'Pago')}
                </span>
                <span className="pagosFacturaLineaFecha">
                    {new Date(payment.created_at).toLocaleDateString('es-ES', {
                        day: 'numeric', month: 'short', year: 'numeric',
                        hour: '2-digit', minute: '2-digit',
                    })}
                </span>
            </div>
            <div className="pagosFacturaLineaEstado">
                <span className={`pagoEstadoBadge ${PAYMENT_STATUS_CLASS[payment.status]}`}>
                    {PAYMENT_STATUS_LABELS[payment.status]}
                </span>
            </div>
            <strong className="pagosFacturaLineaMonto">
                {formatPrice(payment.amount_cents, payment.currency)}
            </strong>
        </div>
    );
}
