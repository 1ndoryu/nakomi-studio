/* [044A-40] Modal de compra de servicio.
 * Flujo: muestra resumen del plan → si no logueado pide email → crea cuenta → orden → pago.
 * [064A-3] Simplificado: solo pide email. Si ya existe, pide password.
 * Se abre al hacer click en CTA de un plan (SeccionPlanesServicio).
 * Usa componente <Modal> base y hook useModalCompra para la lógica. */
import React from 'react';
import {useTranslation} from 'react-i18next';
import CheckoutModal from '../panel/CheckoutModal';
import {Modal} from '../ui/Modal';
import {Button} from '../ui/Button';
import {Input} from '../ui/Input';
import {Textarea} from '../ui/Textarea';
import {useModalCompra} from '../../hooks/useModalCompra';
import {PAYMENT_MODE_LABELS, type PaymentMode} from '../../api/orders';
import type {PlanServicio} from '../../data/planes/tipos';
import './ModalCompra.css';

/* [064A-60] Info de descuento por modo de pago. Alineado con backend discount_for_mode */
const PAYMENT_MODE_DISCOUNT: Record<PaymentMode, number> = {
    full: 20,
    half_half: 10,
    phased: 0,
};

/* [064A-60] Descripciones breves de cada modo */
const PAYMENT_MODE_DESC: Record<PaymentMode, string> = {
    full: 'Un solo pago al confirmar',
    half_half: '50% al inicio, 50% al entregar',
    phased: 'Paga cada fase por separado',
};

/* [084A-12] Parsea precio string ("$100", "$350") a cents.
 * Retorna null si no es numérico ("A medida", "Contactar"). */
function parsePrecioCents(precio: string): number | null {
    const match = precio.replace(/,/g, '').match(/\$?\s*([\d.]+)/);
    return match ? Math.round(parseFloat(match[1]) * 100) : null;
}

/* [084A-12] Formatea cents a precio visible */
function formatPrecioDescontado(cents: number): string {
    const val = cents / 100;
    return val % 1 === 0 ? `$${val}` : `$${val.toFixed(2)}`;
}

interface ModalCompraProps {
    plan: PlanServicio;
    servicioSlug: string;
    abierto: boolean;
    onCerrar: () => void;
}

export const ModalCompra: React.FC<ModalCompraProps> = ({plan, servicioSlug, abierto, onCerrar}) => {
    const {t} = useTranslation();
    const {
        paso, email, setEmail, password, setPassword,
        emailExiste, errorMsg, paymentMode, setPaymentMode,
        months, setMonths, projectDescription, setProjectDescription, checkoutPendiente, isHosting,
        navegarAlPanelPendiente, handleContinuar, handleAuth, reintentar
    } = useModalCompra({plan, servicioSlug, onClose: onCerrar});

    /* [084A-28] Descuento progresivo por meses para hosting: 0% (1m) → 33% (12m) */
    const hostingDescuento = isHosting ? (months - 1) * 3 : 0;

    /* [084A-12] Precio dinámico según modo de pago o meses seleccionados */
    const baseCents = parsePrecioCents(plan.precio);
    const descuento = isHosting ? hostingDescuento : PAYMENT_MODE_DISCOUNT[paymentMode];
    const totalCents = baseCents != null
        ? (isHosting
            ? Math.round(baseCents * months * (1 - descuento / 100))
            : Math.round(baseCents * (1 - descuento / 100)))
        : null;
    const precioFinal = totalCents != null ? formatPrecioDescontado(totalCents) : plan.precio;
    const tieneDescuento = baseCents != null && descuento > 0;

    if (paso === 'checkout' && checkoutPendiente) {
        return (
            <CheckoutModal
                orderId={checkoutPendiente.orderId}
                orderNumber={checkoutPendiente.orderNumber}
                amountCents={checkoutPendiente.amountCents}
                currency={checkoutPendiente.currency}
                clientSecret={checkoutPendiente.clientSecret}
                onClose={navegarAlPanelPendiente}
                onSuccess={navegarAlPanelPendiente}
            />
        );
    }

    return (
        <Modal abierto={abierto} onCerrar={onCerrar} className="modalCompraContenido">
            {/* [064A-46] Resumen: precio solo en botón, no en texto. */}
            {(paso === 'resumen' || paso === 'auth') && (
                <div className="modalCompraResumen">
                    <h2 className="modalTitulo">{plan.nombre}</h2>
                    <p className="modalCompraDescripcion">{plan.descripcion}</p>
                </div>
            )}

            {/* Paso resumen: selector de modo de pago + botón continuar */}
            {paso === 'resumen' && (
                <div className="modalCompraAcciones">
                    {!isHosting && (
                        <label className="modalCompraBrief">
                            <span className="modalCompraBriefLabel">
                                Describe tu proyecto
                            </span>
                            <Textarea
                                className="modalCompraBriefInput"
                                value={projectDescription}
                                onChange={e => setProjectDescription(e.target.value)}
                                placeholder="Cuéntanos el objetivo, lo que necesitas y cualquier contexto importante para que el pedido nazca con información útil."
                                rows={4}
                                required
                            />
                        </label>
                    )}

                    {/* [084A-28] Hosting: selector de meses con descuento progresivo */}
                    {isHosting ? (
                        <div className="modalCompraMeses">
                            <p className="modalCompraMesesLabel">¿Cuántos meses quieres pagar?</p>
                            <div className="modalCompraMesesStepper">
                                <Button
                                    variante="outline"
                                    tamano="pequeno"
                                    onClick={() => setMonths(Math.max(1, months - 1))}
                                    disabled={months <= 1}
                                    type="button"
                                >−</Button>
                                <span className="modalCompraMesesValor">{months}</span>
                                <Button
                                    variante="outline"
                                    tamano="pequeno"
                                    onClick={() => setMonths(Math.min(12, months + 1))}
                                    disabled={months >= 12}
                                    type="button"
                                >+</Button>
                            </div>
                            {baseCents != null && (
                                <div className="modalCompraMesesDetalle">
                                    <span className="modalCompraMesesCalculo">
                                        {plan.precio}/mes × {months} {months === 1 ? 'mes' : 'meses'}
                                    </span>
                                    {tieneDescuento && (
                                        <span className="modalCompraMesesDescuento">
                                            {descuento}% descuento
                                        </span>
                                    )}
                                </div>
                            )}
                        </div>
                    ) : (
                        /* [064A-60] Servicios: selector de modo de pago */
                        <div className="modalCompraModos" role="radiogroup" aria-label="Modo de pago">
                            {(['full', 'half_half', 'phased'] as PaymentMode[]).map(mode => (
                                <div
                                    key={mode}
                                    role="radio"
                                    aria-checked={paymentMode === mode}
                                    tabIndex={0}
                                    className={`modalCompraModo ${paymentMode === mode ? 'modalCompraModoActivo' : ''}`}
                                    onClick={() => setPaymentMode(mode)}
                                    onKeyDown={e => { if (e.key === 'Enter' || e.key === ' ') setPaymentMode(mode); }}
                                >
                                    <span className="modalCompraModoNombre">{PAYMENT_MODE_LABELS[mode]}</span>
                                    <span className="modalCompraModoDesc">{PAYMENT_MODE_DESC[mode]}</span>
                                    {PAYMENT_MODE_DISCOUNT[mode] > 0 && (
                                        <span className="modalCompraModoDescuento">
                                            {PAYMENT_MODE_DISCOUNT[mode]}% descuento
                                        </span>
                                    )}
                                </div>
                            ))}
                        </div>
                    )}
                    {/* [084A-12] Resumen de precio con descuento aplicado */}
                    {baseCents != null && (
                        <div className="modalCompraPrecioResumen">
                            {tieneDescuento ? (
                                <>
                                    <span className="modalCompraPrecioOriginal">{plan.precio}</span>
                                    <span className="modalCompraPrecioFinal">{precioFinal}</span>
                                    <span className="modalCompraPrecioAhorro">
                                        Ahorras {descuento}%
                                    </span>
                                </>
                            ) : (
                                <span className="modalCompraPrecioFinal">{plan.precio}</span>
                            )}
                        </div>
                    )}
                    {errorMsg && <p className="modalCompraErrorTexto">{errorMsg}</p>}
                    <Button variante="primario" tamano="mediano" onClick={handleContinuar}>
                        {t('purchase.continue', 'Continuar')} ({precioFinal})
                    </Button>
                    <p className="modalCompraAviso">{t('purchase.no_charge_yet', 'Aún no se te cobrará')}</p>
                </div>
            )}

            {/* [064A-3] Paso auth: solo email. Si email ya existe, muestra password */}
            {paso === 'auth' && (
                <form className="modalCompraAuth" onSubmit={handleAuth}>
                    <p className="modalCompraAuthTexto">
                        {emailExiste
                            ? t('purchase.existing_account', 'Ya tienes cuenta. Introduce tu contraseña para continuar.')
                            : t('purchase.enter_email', 'Introduce tu email para continuar')}
                    </p>
                    {errorMsg && <p className="modalCompraErrorTexto">{errorMsg}</p>}
                    <Input
                        type="email"
                        placeholder={t('auth.email_placeholder', 'tu@email.com')}
                        value={email}
                        onChange={e => setEmail(e.target.value)}
                        required
                        disabled={emailExiste}
                    />
                    {emailExiste && (
                        <Input
                            type="password"
                            placeholder={t('auth.password_placeholder', 'Contraseña')}
                            value={password}
                            onChange={e => setPassword(e.target.value)}
                            required
                            minLength={8}
                        />
                    )}
                    <Button variante="primario" tamano="mediano" type="submit">
                        {t('purchase.continue_pay', 'Continuar al pago')} ({precioFinal})
                    </Button>
                    <p className="modalCompraAviso">{t('purchase.no_charge_yet', 'Aún no se te cobrará')}</p>
                </form>
            )}

            {/* Procesando */}
            {paso === 'procesando' && (
                <div className="modalCompraProcesando">
                    <p>{t('purchase.processing', 'Procesando tu orden...')}</p>
                </div>
            )}

            {/* Error */}
            {paso === 'error' && (
                <div className="modalCompraError">
                    <p className="modalCompraErrorTexto">{errorMsg}</p>
                    <Button variante="outline" tamano="pequeno" onClick={reintentar}>
                        {t('common.retry', 'Intentar de nuevo')}
                    </Button>
                </div>
            )}
        </Modal>
    );
};
