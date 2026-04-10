/* [064A-30] Detalle de orden rediseñado: cuadro de info (freelancer, servicio,
 * número, fecha inicio, precio, 3 puntos) + historial de fases con entregas y
 * revisiones. Clientes solicitan revisiones al recibir entrega.
 * Opciones: reportar, extensión de tiempo (empleado), cancelar.
 * custom toggle (<label> con CSS switch). */
import React, {useState, useCallback} from 'react';
import {CreditCard, XCircle, ArrowLeft, AlertTriangle, User, Bot, ArrowRightLeft} from 'lucide-react';
import {
    ORDER_STATUS_LABELS,
    PAYMENT_MODE_LABELS,
    formatPrice,
    apiToggleAiIntermediary,
    type OrderResponse,
    type OrderPhaseResponse,
    type OrderStatus,
} from '../../api/orders';
import {ReviewPanel} from './ReviewPanel';
import {FaseCard} from './FaseCard';
import {Button} from '../ui/Button';
import {Input} from '../ui/Input';
import {MenuContextual, type MenuContextualItem} from '../ui/ContextMenu';
import CheckoutModal from './CheckoutModal';
import {OrderChat} from './OrderChat';
import {OrderDetailModals} from './OrderDetailModals';
import {OrderProjectDescription} from './OrderProjectDescription';
import {useOrdenDetalle} from '../../hooks/useOrdenDetalle';
import './OrdenDetalle.css';

/* [104A-29] pending_payment ya no se usa a nivel de orden. payment_held es el estado inicial. */
const CANCELABLE_STATUSES: OrderStatus[] = ['payment_held', 'awaiting_assignment'];

const STATUS_CLASS: Record<OrderStatus, string> = {
    pending_payment: 'ordenBadge--paymentHeld',
    payment_held: 'ordenBadge--paymentHeld',
    awaiting_assignment: 'ordenBadge--awaiting',
    in_progress: 'ordenBadge--inProgress',
    under_review: 'ordenBadge--underReview',
    completed: 'ordenBadge--completed',
    cancelled: 'ordenBadge--cancelled',
    disputed: 'ordenBadge--disputed',
};

/* [064A-30] Formatea fecha ISO a "DD mes YYYY" legible */
function formatFecha(iso: string): string {
    const d = new Date(iso);
    return d.toLocaleDateString('es-ES', {day: 'numeric', month: 'short', year: 'numeric'});
}

interface OrdenDetalleProps {
    order: OrderResponse;
    phases: OrderPhaseResponse[];
    effectiveRole: string;
    onVolver: () => void;
    onCancelar: (orderId: string, reason?: string) => Promise<void>;
    onAprobar: (orderId: string, phase: number) => Promise<void>;
    onRevision: (orderId: string, phase: number) => Promise<void>;
    onActualizarDescripcion: (orderId: string, projectDescription: string) => Promise<void>;
    onActualizarFase: (
        orderId: string,
        phase: number,
        req: {
            title?: string;
            description?: string;
            price_cents?: number;
            estimated_days?: number;
            max_revisions?: number;
        },
    ) => Promise<void>;
    onPagoExitoso: () => void;
    onIrADelegaciones?: () => void;
    cancelando: boolean;
    actualizandoDescripcion: boolean;
    actualizandoFase: boolean;
}

export const OrdenDetalle: React.FC<OrdenDetalleProps> = ({
    order, phases, effectiveRole, onVolver, onCancelar, onAprobar, onRevision, onPagoExitoso, cancelando,
    onActualizarDescripcion, onActualizarFase, actualizandoDescripcion, actualizandoFase, onIrADelegaciones,
}) => {
    const {
        checkout, menuAbierto, setMenuAbierto,
        modalCancelarAbierto, setModalCancelarAbierto,
        modalReportarAbierto, setModalReportarAbierto,
        confirmarCancelacion, enviarReporte, reportando, reportError, reportExito,
        cerrarReportar, abrirCheckout, cerrarCheckout,
    } = useOrdenDetalle(order, onCancelar);

    /* [T-10] Toggle IA intermediaria: solo visible para admin/employee */
    const [intermediaryEnabled, setIntermediaryEnabled] = useState(order.ai_intermediary_enabled);
    const [togglingIntermediary, setTogglingIntermediary] = useState(false);

    const handleToggleIntermediary = useCallback(async () => {
        setTogglingIntermediary(true);
        try {
            const updated = await apiToggleAiIntermediary(order.id, !intermediaryEnabled);
            setIntermediaryEnabled(updated.ai_intermediary_enabled);
        } catch {
            /* Rollback visual si falla */
            setIntermediaryEnabled(prev => prev);
        } finally {
            setTogglingIntermediary(false);
        }
    }, [order.id, intermediaryEnabled]);

    const isEmployee = effectiveRole === 'employee';
    const isClient = effectiveRole === 'client' || effectiveRole === 'admin';
    const isActiveOrder = order.status !== 'cancelled' && order.status !== 'completed';
    const canCancel = CANCELABLE_STATUSES.includes(order.status)
        || (isEmployee && order.status === 'in_progress');
    /* [104A-29] payment_held es el estado inicial — el botón de pago aparece aquí */
    const needsPayment = order.status === 'payment_held';
    /* [064A-60] Tanto phased como half_half usan pagos por fase individuales */
    const isPerPhasePayment = order.payment_mode === 'phased' || order.payment_mode === 'half_half';
    const canEditDescription = isClient && order.status !== 'completed' && order.status !== 'cancelled';
    const canDefinePhases = order.payment_mode === 'phased' && (isEmployee || effectiveRole === 'admin');

    /* [064A-31] Chat disponible cuando hay empleado asignado y la orden está activa */
    const canChat = !!order.assigned_employee_id && isActiveOrder;

    /* [104A-28] Menú contextual con acciones según rol */
    const menuItems: MenuContextualItem[] = [];

    /* Cancelar: cliente (estados iniciales) o empleado (con razón, incluyendo in_progress) */
    if (canCancel) {
        menuItems.push({
            id: 'cancel-order',
            label: cancelando ? 'Cancelando...' : 'Cancelar orden',
            onSelect: () => setModalCancelarAbierto(true),
            disabled: cancelando,
            danger: true,
            icon: <XCircle size={16} />,
        });
    }

    /* Reportar: disponible para cliente y empleado en órdenes activas con empleado asignado */
    if (order.assigned_employee_id && isActiveOrder) {
        menuItems.push({
            id: 'report-order',
            label: 'Reportar problema',
            onSelect: () => setModalReportarAbierto(true),
            icon: <AlertTriangle size={16} />,
        });
    }

    /* Delegar: solo empleados asignados en órdenes activas */
    if (isEmployee && order.assigned_employee_id && isActiveOrder && onIrADelegaciones) {
        menuItems.push({
            id: 'delegate-order',
            label: 'Delegar orden',
            onSelect: onIrADelegaciones,
            icon: <ArrowRightLeft size={16} />,
        });
    }

    return (
        <div className="ordenDetalle">
            <div className="ordenDetalleTopbar">
                <Button className="ordenDetalleVolver" onClick={onVolver} type="button" variante="texto" tamano="pequeno">
                    <ArrowLeft size={16} /> Volver
                </Button>
            </div>

            {/* [064A-30] Cuadro de información del pedido */}
            <div className="ordenInfoCard">
                <div className="ordenInfoCardHeader">
                    <span className={`ordenDetalleBadge ${STATUS_CLASS[order.status]}`}>
                        {ORDER_STATUS_LABELS[order.status]}
                    </span>
                    <div className="ordenInfoCardAcciones">
                        {/* [T-10] Toggle IA intermediaria — solo admin/employee */}
                        {!isClient && canChat && (
                            <label className={`ordenToggleIntermediary ${togglingIntermediary ? 'ordenToggleIntermediary--disabled' : ''}`}>
                                <Bot size={14} />
                                <span className="ordenToggleLabel">IA</span>
                                <Input
                                    type="checkbox"
                                    checked={intermediaryEnabled}
                                    onChange={() => void handleToggleIntermediary()}
                                    disabled={togglingIntermediary}
                                    className="ordenToggleInput"
                                />
                                <span className="ordenToggleSwitch" />
                            </label>
                        )}
                        {needsPayment && !isPerPhasePayment && (
                            <Button
                                onClick={() => abrirCheckout(order.final_price_cents)}
                                type="button"
                                variante="secundario"
                                tamano="pequeno"
                            >
                                <CreditCard size={16} /> Pagar
                            </Button>
                        )}
                        {menuItems.length > 0 && (
                            <MenuContextual
                                abierto={menuAbierto}
                                onToggle={() => setMenuAbierto(prev => !prev)}
                                onCerrar={() => setMenuAbierto(false)}
                                items={menuItems}
                                ariaLabel="Opciones de la orden"
                                triggerClassName="ordenDetalleOpcionesBoton"
                            />
                        )}
                    </div>
                </div>

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

                <OrderProjectDescription
                    orderId={order.id}
                    projectDescription={order.project_description}
                    canEdit={canEditDescription}
                    isSaving={actualizandoDescripcion}
                    onSave={onActualizarDescripcion}
                />

                {/* [T-10] Resumen IA del pedido — solo visible para admin/employee */}
                {!isClient && order.ai_summary && (
                    <div className="ordenAiSummary">
                        <span className="ordenAiSummaryLabel">
                            <Bot size={14} /> Resumen IA
                        </span>
                        <p className="ordenAiSummaryText">{order.ai_summary}</p>
                    </div>
                )}
            </div>

            <OrderDetailModals
                orderNumber={order.order_number}
                isEmployee={isEmployee}
                modalCancelarAbierto={modalCancelarAbierto}
                modalReportarAbierto={modalReportarAbierto}
                cancelando={cancelando}
                reportando={reportando}
                reportError={reportError}
                reportExito={reportExito}
                onCerrarCancelar={() => setModalCancelarAbierto(false)}
                onConfirmarCancelacion={(reason) => void confirmarCancelacion(reason)}
                onCerrarReportar={cerrarReportar}
                onEnviarReporte={(reason) => void enviarReporte(reason)}
            />

            {checkout && (
                <CheckoutModal
                    orderId={order.id}
                    orderNumber={order.order_number}
                    amountCents={checkout.amountCents}
                    currency={order.currency}
                    phaseNumber={checkout.phaseNumber}
                    onClose={cerrarCheckout}
                    onSuccess={() => {
                        cerrarCheckout();
                        onPagoExitoso();
                    }}
                />
            )}

            {/* [064A-30] Cuadro de historial: fases con progreso, entregas, revisiones */}
            {phases.length > 0 && (
                <div className="ordenHistorialCard">
                    <h3 className="ordenHistorialTitulo">Historial del proyecto</h3>
                    <div className="fasesTimeline">
                        {phases.map((phase, idx) => (
                            <FaseCard
                                key={phase.phase_number}
                                phase={phase}
                                orderId={order.id}
                                isLast={idx === phases.length - 1}
                                isClient={isClient}
                                isEmployee={isEmployee}
                                isPhased={isPerPhasePayment}
                                canDefinePhase={canDefinePhases}
                                isUpdatingDefinition={actualizandoFase}
                                onAprobar={onAprobar}
                                onRevision={onRevision}
                                onActualizarFase={onActualizarFase}
                                onPagarFase={(pn, amount) => abrirCheckout(amount, pn)}
                            />
                        ))}
                    </div>
                </div>
            )}

            {/* [084A-2] Chat siempre abierto, debajo del historial */}
            {canChat && (
                <OrderChat orderId={order.id} />
            )}

            {order.status === 'completed' && (
                <ReviewPanel orderId={order.id} effectiveRole={effectiveRole} />
            )}
        </div>
    );
};
