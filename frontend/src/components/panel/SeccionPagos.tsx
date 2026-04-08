/* [044A-38 Fase 3] Sección de historial de pagos en el panel.
 * [074A-61] Rediseño: lista compacta de pagos + modal de detalles al click.
 * Botón "Solicitar reembolso" movido al modal de detalle. */

import { useState } from 'react';
import { Loader2, CreditCard, AlertCircle, RotateCcw, Receipt } from 'lucide-react';
import { useOrdenes } from '../../hooks/useOrdenes';
import { usePagos } from '../../hooks/usePagos';
import { useRefundModal } from '../../hooks/useRefundModal';
import {
    PAYMENT_STATUS_LABELS,
    PAYMENT_STATUS_CLASS,
    type PaymentResponse,
} from '../../api/payments';
import { PAYMENT_MODE_LABELS, formatPrice } from '../../api/orders';
import { useAuthStore } from '../../stores/authStore';
import { Textarea } from '../ui/Textarea';
import { Button } from '../ui/Button';
import { Modal } from '../ui/Modal';
import './SeccionPagos.css';

export function SeccionPagos() {
    const { ordenes, cargando, error } = useOrdenes();
    const [ordenSeleccionada, setOrdenSeleccionada] = useState<string | null>(null);
    const { pagos, cargandoPagos, errorPagos } = usePagos(ordenSeleccionada);
    const {
        refundOrderId, refundRazon, refundEnCurso,
        setRefundRazon, abrirModal, cerrarModal, enviarSolicitud,
    } = useRefundModal();
    const effectiveRole = useAuthStore(s => s.user?.effectiveRole) || 'client';

    /* [074A-61] Modal de detalle de pago */
    const [pagoDetalle, setPagoDetalle] = useState<PaymentResponse | null>(null);

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
            {/* Selector de orden */}
            <div className="pagosOrdenesLista">
                {ordenes.map((o) => (
                    <Button
                        key={o.id}
                        variante="texto"
                        className={`pagosOrdenBtn ${ordenSeleccionada === o.id ? 'pagosOrdenBtn--activo' : ''}`}
                        onClick={() => setOrdenSeleccionada(o.id)}
                        type="button"
                    >
                        <span className="pagosOrdenNumero">#{o.order_number}</span>
                        <span className="pagosOrdenTitulo">{o.service_title}</span>
                        <span className="pagosOrdenPrecio">{formatPrice(o.final_price_cents, o.currency)}</span>
                    </Button>
                ))}
                {ordenes.length === 0 && (
                    <p className="pagosVacioTexto">No tienes órdenes</p>
                )}
            </div>

            {/* Lista compacta de pagos */}
            {ordenSeleccionada && (
                <div className="pagosDatos">
                    {cargandoPagos && <Loader2 className="pagosSpinner" size={24} />}
                    {errorPagos && (
                        <p className="pagosError"><AlertCircle size={16} /> {errorPagos}</p>
                    )}
                    {!cargandoPagos && pagos.length === 0 && (
                        <div className="pagosVacioDetalle">
                            <CreditCard size={32} />
                            <p>No hay pagos registrados para esta orden</p>
                        </div>
                    )}
                    {pagos.map((p) => (
                        <Button
                            key={p.id}
                            variante="texto"
                            className="pagoFila"
                            onClick={() => setPagoDetalle(p)}
                            type="button"
                        >
                            <Receipt size={16} className="pagoFilaIcono" />
                            <span className={`pagoFilaEstado ${PAYMENT_STATUS_CLASS[p.status]}`}>
                                {PAYMENT_STATUS_LABELS[p.status]}
                            </span>
                            <span className="pagoFilaDescripcion">
                                {p.description || (p.phase_number ? `Fase ${p.phase_number}` : 'Pago')}
                            </span>
                            <span className="pagoFilaFecha">
                                {new Date(p.created_at).toLocaleDateString('es-ES', {day: 'numeric', month: 'short'})}
                            </span>
                            <span className="pagoFilaMonto">
                                {formatPrice(p.amount_cents, p.currency)}
                            </span>
                        </Button>
                    ))}
                </div>
            )}

            {/* [074A-61] Modal de detalle de pago */}
            <Modal abierto={!!pagoDetalle} onCerrar={() => setPagoDetalle(null)}>
                {pagoDetalle && (
                    <div className="pagoModalContenido">
                        <h3 className="modalTitulo">Detalle del pago</h3>
                        <div className="pagoDetalleGrid">
                            <div className="pagoDetalleFila">
                                <span className="pagoDetalleLabel">Estado</span>
                                <span className={`pagoDetalleValor ${PAYMENT_STATUS_CLASS[pagoDetalle.status]}`}>
                                    {PAYMENT_STATUS_LABELS[pagoDetalle.status]}
                                </span>
                            </div>
                            <div className="pagoDetalleFila">
                                <span className="pagoDetalleLabel">Monto</span>
                                <span className="pagoDetalleValor pagoDetalleMontoGrande">
                                    {formatPrice(pagoDetalle.amount_cents, pagoDetalle.currency)}
                                </span>
                            </div>
                            <div className="pagoDetalleFila">
                                <span className="pagoDetalleLabel">Modo de pago</span>
                                <span className="pagoDetalleValor">
                                    {PAYMENT_MODE_LABELS[pagoDetalle.payment_mode]}
                                </span>
                            </div>
                            {pagoDetalle.phase_number != null && (
                                <div className="pagoDetalleFila">
                                    <span className="pagoDetalleLabel">Fase</span>
                                    <span className="pagoDetalleValor">Fase {pagoDetalle.phase_number}</span>
                                </div>
                            )}
                            {pagoDetalle.description && (
                                <div className="pagoDetalleFila">
                                    <span className="pagoDetalleLabel">Descripción</span>
                                    <span className="pagoDetalleValor">{pagoDetalle.description}</span>
                                </div>
                            )}
                            <div className="pagoDetalleFila">
                                <span className="pagoDetalleLabel">Fecha</span>
                                <span className="pagoDetalleValor">
                                    {new Date(pagoDetalle.created_at).toLocaleDateString('es-ES', {
                                        day: 'numeric', month: 'long', year: 'numeric',
                                        hour: '2-digit', minute: '2-digit',
                                    })}
                                </span>
                            </div>
                        </div>
                        {effectiveRole === 'client' &&
                            (pagoDetalle.status === 'held' || pagoDetalle.status === 'released') && (
                            <Button
                                variante="outline"
                                tamano="pequeno"
                                className="pagoBotonReembolso"
                                type="button"
                                onClick={() => {
                                    setPagoDetalle(null);
                                    abrirModal(ordenSeleccionada!);
                                }}
                            >
                                <RotateCcw size={14} />
                                Solicitar reembolso
                            </Button>
                        )}
                    </div>
                )}
            </Modal>

            {/* Modal de solicitud de reembolso */}
            <Modal abierto={!!refundOrderId} onCerrar={cerrarModal}>
                <div className="pagoModalContenido">
                    <h3 className="modalTitulo">Solicitar reembolso</h3>
                    <p className="pagoModalDescripcion">
                        Describe el motivo de tu solicitud de reembolso. Un administrador la revisará.
                    </p>
                    <Textarea
                        className="pagoModalTextarea"
                        placeholder="Motivo del reembolso..."
                        value={refundRazon}
                        onChange={(e) => setRefundRazon(e.target.value)}
                        rows={4}
                    />
                    <div className="pagoModalBotones">
                        <Button
                            className="pagoBotonEnviarReembolso"
                            tamano="pequeno"
                            type="button"
                            onClick={() => void enviarSolicitud()}
                            disabled={refundEnCurso || !refundRazon.trim()}
                        >
                            {refundEnCurso ? 'Enviando…' : 'Enviar solicitud'}
                        </Button>
                        <Button
                            variante="outline"
                            tamano="pequeno"
                            className="pagoBotonCancelarReembolso"
                            type="button"
                            onClick={cerrarModal}
                            disabled={refundEnCurso}
                        >
                            Cancelar
                        </Button>
                    </div>
                </div>
            </Modal>
        </div>
    );
}
