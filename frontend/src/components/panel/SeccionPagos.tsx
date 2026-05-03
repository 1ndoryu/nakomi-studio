/* [044A-38 Fase 3] Sección de historial de pagos en el panel.
 * [035A-18] Rehecha como lista compacta de cards con imagen del servicio y modal 
 * tipo factura para concentrar los detalles, 100% alineado al sistema visual. */

import { useMemo, useState } from 'react';
import { Loader2, AlertCircle, RotateCcw, Package, CreditCard } from 'lucide-react';
import { useOrdenes } from '../../hooks/useOrdenes';
import { usePagos } from '../../hooks/usePagos';
import { useRefundModal } from '../../hooks/useRefundModal';
import {
    PAYMENT_STATUS_LABELS,
    PAYMENT_STATUS_CLASS,
    type PaymentResponse,
} from '../../api/payments';
import { PAYMENT_MODE_LABELS, formatPrice, type OrderResponse } from '../../api/orders';
import { useAuthStore } from '../../stores/authStore';
import { Textarea } from '../ui/Textarea';
import { Button } from '../ui/Button';
import { Modal } from '../ui/Modal';
import OptimizedImage from '../ui/OptimizedImage';
import { getServiceImage } from '../../utils/serviceImages';
import './SeccionPagos.css';

function PagoResumenCard({ order, onOpen }: { order: OrderResponse; onOpen: () => void }) {
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
                    <span className="pagoResumenCardBadge" style={{ backgroundColor: 'var(--bg-item-active)', color: 'var(--brand-black)' }}>
                        Orden #{order.order_number}
                    </span>
                </div>
                <div className="pagoResumenCardFooter">
                    <span className="pagoResumenCardPrecio">{formatPrice(order.final_price_cents, order.currency)}</span>
                    <span className="pagoResumenCardModo">{PAYMENT_MODE_LABELS[order.payment_mode]}</span>
                    <span className="pagoResumenCardFecha">
                        {new Date(order.created_at).toLocaleDateString('es-ES', {
                            day: 'numeric', month: 'short'
                        })}
                    </span>
                </div>
            </div>
        </Button>
    );
}

function FacturaLinea({ payment }: { payment: PaymentResponse }) {
    return (
        <div className="pagosFacturaLinea">
            <div className="pagosFacturaLineaInfo">
                <span className="pagosFacturaLineaConcepto">
                    {payment.description || (payment.phase_number ? `Fase ${payment.phase_number}` : 'Pago')}
                </span>
                <span className="pagosFacturaLineaFecha">
                    {new Date(payment.created_at).toLocaleDateString('es-ES', {
                        day: 'numeric', month: 'short', year: 'numeric',
                        hour: '2-digit', minute: '2-digit'
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

export function SeccionPagos() {
    const { ordenes, cargando, error } = useOrdenes();
    const [ordenSeleccionada, setOrdenSeleccionada] = useState<string | null>(null);
    const { pagos, cargandoPagos, errorPagos } = usePagos(ordenSeleccionada);
    const {
        refundOrderId, refundRazon, refundEnCurso,
        setRefundRazon, abrirModal, cerrarModal, enviarSolicitud,
    } = useRefundModal();
    const effectiveRole = useAuthStore(s => s.user?.effectiveRole) || 'client';
    
    const ordenActiva = useMemo(
        () => ordenes.find(orden => orden.id === ordenSeleccionada) ?? null,
        [ordenes, ordenSeleccionada],
    );

    const ordenesOrdenadas = useMemo(
        () => [...ordenes].sort(
            (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime(),
        ),
        [ordenes],
    );

    const totales = useMemo(() => {
        if (!pagos.length) return null;
        const pagado = pagos
            .filter(p => p.status === 'released' || p.status === 'held')
            .reduce((sum, p) => sum + p.amount_cents, 0);
        const reembolsado = pagos
            .filter(p => p.status === 'refunded')
            .reduce((sum, p) => sum + p.amount_cents, 0);
        const pendiente = pagos
            .filter(p => p.status === 'pending')
            .reduce((sum, p) => sum + p.amount_cents, 0);
        const currency = pagos[0]?.currency || 'USD';
        return { pagado, reembolsado, pendiente, currency };
    }, [pagos]);

    const puedeSolicitarReembolso = effectiveRole === 'client' && 
        pagos.some(p => p.status === 'held' || p.status === 'released');

    if (cargando) {
        return (
            <div className="pagosVacio">
                <Loader2 className="pagosSpinner" size={32} />
            </div>
        );
    }

    if (error) {
        return (
            <div className="pagosError">
                <AlertCircle size={20} />
                <span>{error}</span>
            </div>
        );
    }

    return (
        <div className="pagosContenedor">
            {ordenesOrdenadas.length === 0 ? (
                <div className="pagosVacioDetalle">
                    <Package size={32} />
                    <p className="pagosVacioTexto">No tienes órdenes con historial de pago</p>
                </div>
            ) : (
                <div className="pagosListaCompacta">
                    {ordenesOrdenadas.map(orden => (
                        <PagoResumenCard
                            key={orden.id}
                            order={orden}
                            onOpen={() => setOrdenSeleccionada(orden.id)}
                        />
                    ))}
                </div>
            )}

            <Modal
                abierto={!!ordenActiva}
                onCerrar={() => setOrdenSeleccionada(null)}
                className="pagosFacturaModal"
            >
                {ordenActiva && (
                    <div className="pagosFactura">
                        {/* Header Minimalista */}
                        <div className="pagosFacturaHeader">
                            <div className="pagosFacturaServicio">
                                <OptimizedImage
                                    className="pagosFacturaImagen"
                                    src={getServiceImage(ordenActiva.service_slug)}
                                    alt={ordenActiva.service_title}
                                    loading="lazy"
                                />
                                <div className="pagosFacturaServicioInfo">
                                    <h3 className="modalTitulo">{ordenActiva.service_title}</h3>
                                    <p className="modalTexto">
                                        Orden #{ordenActiva.order_number}
                                    </p>
                                </div>
                            </div>
                            <div className="pagosFacturaTotalesBloque">
                                <span className="pagosFacturaTotalEyebrow">Total Contratado</span>
                                <strong className="pagosFacturaTotalValor">
                                    {formatPrice(ordenActiva.final_price_cents, ordenActiva.currency)}
                                </strong>
                            </div>
                        </div>

                        {/* Metadatos en linea */}
                        <div className="pagosFacturaMetaList">
                            <div className="pagosFacturaMetaGroup">
                                <span className="pagosFacturaMetaLabel">Moneda</span>
                                <span className="pagosFacturaMetaValue">{ordenActiva.currency}</span>
                            </div>
                            <div className="pagosFacturaMetaGroup">
                                <span className="pagosFacturaMetaLabel">Modalidad</span>
                                <span className="pagosFacturaMetaValue">{PAYMENT_MODE_LABELS[ordenActiva.payment_mode]}</span>
                            </div>
                            <div className="pagosFacturaMetaGroup">
                                <span className="pagosFacturaMetaLabel">Fecha Contratación</span>
                                <span className="pagosFacturaMetaValue">
                                    {new Date(ordenActiva.created_at).toLocaleDateString('es-ES', {
                                        day: 'numeric', month: 'long', year: 'numeric'
                                    })}
                                </span>
                            </div>
                        </div>

                        {/* Lineas de Transacciones */}
                        <div className="pagosFacturaLineasBloque">
                            <h4 className="modalTitulo" style={{ fontSize: 'var(--text-sm)', marginBottom: 'var(--spacing-sm)' }}>
                                Movimientos de pago
                            </h4>
                            
                            {cargandoPagos && (
                                <div className="pagosVacioDetalle">
                                    <Loader2 className="pagosSpinner" size={24} />
                                    <p className="pagosVacioTexto">Cargando...</p>
                                </div>
                            )}

                            {errorPagos && (
                                <p className="pagosError"><AlertCircle size={16} /> {errorPagos}</p>
                            )}

                            {!cargandoPagos && !errorPagos && pagos.length === 0 && (
                                <div className="pagosVacioDetalle">
                                    <CreditCard size={32} />
                                    <p className="pagosVacioTexto">No hay movimientos registrados.</p>
                                </div>
                            )}

                            {!cargandoPagos && pagos.length > 0 && (
                                <>
                                    <div className="pagosFacturaLineas">
                                        {pagos.map(pago => (
                                            <FacturaLinea key={pago.id} payment={pago} />
                                        ))}
                                    </div>

                                    {totales && (
                                        <div className="pagosFacturaResumenTotales">
                                            {totales.pagado > 0 && (
                                                <div className="pagosFacturaResumenItem">
                                                    <span className="pagosFacturaMetaLabel">Pagado</span>
                                                    <span className="pagosFacturaMetaValue pagosFacturaResumenPagado">
                                                        {formatPrice(totales.pagado, totales.currency)}
                                                    </span>
                                                </div>
                                            )}
                                            {totales.pendiente > 0 && (
                                                <div className="pagosFacturaResumenItem">
                                                    <span className="pagosFacturaMetaLabel">Pendiente</span>
                                                    <span className="pagosFacturaMetaValue pagosFacturaResumenPendiente">
                                                        {formatPrice(totales.pendiente, totales.currency)}
                                                    </span>
                                                </div>
                                            )}
                                            {totales.reembolsado > 0 && (
                                                <div className="pagosFacturaResumenItem">
                                                    <span className="pagosFacturaMetaLabel">Reembolsado</span>
                                                    <span className="pagosFacturaMetaValue pagosFacturaResumenReembolsado">
                                                        {formatPrice(totales.reembolsado, totales.currency)}
                                                    </span>
                                                </div>
                                            )}
                                        </div>
                                    )}
                                </>
                            )}
                        </div>

                        {puedeSolicitarReembolso && (
                            <div className="modalAcciones" style={{ paddingTop: 'var(--spacing-md)' }}>
                                <Button
                                    variante="outline"
                                    tamano="pequeno"
                                    onClick={() => {
                                        setOrdenSeleccionada(null);
                                        abrirModal(ordenActiva.id);
                                    }}
                                >
                                    <RotateCcw size={14} style={{ marginRight: '6px' }} />
                                    Solicitar Reembolso
                                </Button>
                            </div>
                        )}
                    </div>
                )}
            </Modal>

            {/* Modal de solicitud de reembolso */}
            <Modal abierto={!!refundOrderId} onCerrar={cerrarModal}>
                <div className="pagoModalContenido">
                    <h3 className="modalTitulo">Solicitar Reembolso</h3>
                    <p className="modalTexto" style={{ marginBottom: 'var(--spacing-sm)' }}>
                        Describe el motivo de tu solicitud de reembolso. Un administrador la revisará.
                    </p>
                    <Textarea
                        className="pagoModalTextarea"
                        placeholder="Motivo del reembolso..."
                        value={refundRazon}
                        onChange={(e) => setRefundRazon(e.target.value)}
                        rows={4}
                    />
                    <div className="modalAcciones" style={{ marginTop: 'var(--spacing-md)' }}>
                        <Button
                            variante="outline"
                            onClick={cerrarModal}
                            disabled={refundEnCurso}
                        >
                            Cancelar
                        </Button>
                        <Button
                            onClick={() => void enviarSolicitud()}
                            disabled={refundEnCurso || !refundRazon.trim()}
                        >
                            {refundEnCurso ? 'Enviando...' : 'Enviar solicitud'}
                        </Button>
                    </div>
                </div>
            </Modal>
        </div>
    );
}