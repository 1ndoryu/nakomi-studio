/* [044A-38 Fase 3] Sección de historial de pagos en el panel.
 * [074A-61] Rediseño: lista compacta de pagos + modal de detalles al click.
 * [084A-21] Rediseño como tabla profesional con columnas alineadas, totales y diseño limpio.
 * Botón "Solicitar reembolso" movido al modal de detalle. */

import { useState, useMemo } from 'react';
import { Loader2, CreditCard, AlertCircle, RotateCcw } from 'lucide-react';
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

    const [pagoDetalle, setPagoDetalle] = useState<PaymentResponse | null>(null);

    /* [084A-21] Totales calculados para el resumen */
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
            {/* [084A-44] Selector de orden como tabla profesional */}
            <div className="pagosOrdenesWrapper">
                {ordenes.length === 0 ? (
                    <p className="pagosVacioTexto">No tienes órdenes</p>
                ) : (
                    <table className="pagosOrdenesTabla">
                        <thead>
                            <tr>
                                <th className="pagosOrdenesHead">Orden</th>
                                <th className="pagosOrdenesHead">Servicio</th>
                                <th className="pagosOrdenesHead pagosOrdenesHeadDerecha">Total</th>
                            </tr>
                        </thead>
                        <tbody>
                            {ordenes.map((o) => (
                                <tr
                                    key={o.id}
                                    className={`pagosOrdenesFila ${ordenSeleccionada === o.id ? 'pagosOrdenesFila--activa' : ''}`}
                                    onClick={() => setOrdenSeleccionada(o.id)}
                                >
                                    <td className="pagosOrdenesCelda pagosOrdenesNumero">#{o.order_number}</td>
                                    <td className="pagosOrdenesCelda pagosOrdenesTitulo">{o.service_title}</td>
                                    <td className="pagosOrdenesCelda pagosOrdenesCeldaDerecha">{formatPrice(o.final_price_cents, o.currency)}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                )}
            </div>

            {/* [084A-21] Tabla profesional de pagos */}
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
                    {!cargandoPagos && pagos.length > 0 && (
                        <>
                            <div className="pagosTablaWrapper">
                                <table className="pagosTabla">
                                    <thead>
                                        <tr>
                                            <th className="pagosTablaHead">Estado</th>
                                            <th className="pagosTablaHead">Descripción</th>
                                            <th className="pagosTablaHead">Modo</th>
                                            <th className="pagosTablaHead pagosTablaHeadDerecha">Fecha</th>
                                            <th className="pagosTablaHead pagosTablaHeadDerecha">Monto</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {pagos.map((p) => (
                                            <tr
                                                key={p.id}
                                                className="pagosTablaFila"
                                                onClick={() => setPagoDetalle(p)}
                                            >
                                                <td className="pagosTablaCelda">
                                                    <span className={`pagoEstadoBadge ${PAYMENT_STATUS_CLASS[p.status]}`}>
                                                        {PAYMENT_STATUS_LABELS[p.status]}
                                                    </span>
                                                </td>
                                                <td className="pagosTablaCelda pagosTablaCeldaDescripcion">
                                                    {p.description || (p.phase_number ? `Fase ${p.phase_number}` : 'Pago')}
                                                </td>
                                                <td className="pagosTablaCelda pagosTablaCeldaModo">
                                                    {PAYMENT_MODE_LABELS[p.payment_mode]}
                                                </td>
                                                <td className="pagosTablaCelda pagosTablaCeldaDerecha">
                                                    {new Date(p.created_at).toLocaleDateString('es-ES', {
                                                        day: 'numeric', month: 'short', year: '2-digit',
                                                    })}
                                                </td>
                                                <td className="pagosTablaCelda pagosTablaCeldaDerecha pagosTablaCeldaMonto">
                                                    {formatPrice(p.amount_cents, p.currency)}
                                                </td>
                                            </tr>
                                        ))}
                                    </tbody>
                                </table>
                            </div>

                            {/* [084A-21] Resumen de totales */}
                            {totales && (
                                <div className="pagosTotales">
                                    {totales.pagado > 0 && (
                                        <div className="pagosTotalItem">
                                            <span className="pagosTotalLabel">Pagado</span>
                                            <span className="pagosTotalValor pagosTotalPagado">
                                                {formatPrice(totales.pagado, totales.currency)}
                                            </span>
                                        </div>
                                    )}
                                    {totales.pendiente > 0 && (
                                        <div className="pagosTotalItem">
                                            <span className="pagosTotalLabel">Pendiente</span>
                                            <span className="pagosTotalValor pagosTotalPendiente">
                                                {formatPrice(totales.pendiente, totales.currency)}
                                            </span>
                                        </div>
                                    )}
                                    {totales.reembolsado > 0 && (
                                        <div className="pagosTotalItem">
                                            <span className="pagosTotalLabel">Reembolsado</span>
                                            <span className="pagosTotalValor pagosTotalReembolsado">
                                                {formatPrice(totales.reembolsado, totales.currency)}
                                            </span>
                                        </div>
                                    )}
                                </div>
                            )}
                        </>
                    )}
                </div>
            )}

            {/* Modal de detalle de pago */}
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
