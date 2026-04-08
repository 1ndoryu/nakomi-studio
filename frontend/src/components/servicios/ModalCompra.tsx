/* [044A-40] Modal de compra de servicio.
 * Flujo: muestra resumen del plan → si no logueado pide email → crea cuenta → orden → pago.
 * [064A-3] Simplificado: solo pide email. Si ya existe, pide password.
 * Se abre al hacer click en CTA de un plan (SeccionPlanesServicio).
 * Usa componente <Modal> base y hook useModalCompra para la lógica. */
import React from 'react';
import {useTranslation} from 'react-i18next';
import {Modal} from '../ui/Modal';
import {Button} from '../ui/Button';
import {Input} from '../ui/Input';
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
        handleContinuar, handleAuth, reintentar
    } = useModalCompra({plan, servicioSlug, onClose: onCerrar});

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
                    {/* [064A-60] Selector de modo de pago */}
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
                    <Button variante="primario" tamano="mediano" onClick={handleContinuar}>
                        {t('purchase.continue', 'Continuar')} ({plan.precio})
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
                        {t('purchase.continue_pay', 'Continuar al pago')} ({plan.precio})
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
