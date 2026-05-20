/* [205A-1] Cobros pendientes separados del estado técnico del hosting.
 * Los hostings reales siguen activos; este bloque solo coordina suscripciones/pagos. */
import {AlertCircle, CalendarDays, CreditCard, ReceiptText} from 'lucide-react';

import {
    billingItemPeriodLabel,
    type BillingCheckoutMode,
    type BillingItem,
} from '../../api/billing';
import {Button} from '../ui/Button';
import {Tarjeta} from '../ui/Tarjeta';
import './PendingBillingPanel.css';

function formatMoney(amountCents: number, currency: string): string {
    return new Intl.NumberFormat('en-US', {
        style: 'currency',
        currency: currency.toUpperCase(),
    }).format(amountCents / 100);
}

function formatDate(value: string): string {
    return new Intl.DateTimeFormat('es', {
        day: '2-digit',
        month: 'short',
        year: 'numeric',
    }).format(new Date(value));
}

function itemKindLabel(item: BillingItem): string {
    if (item.resource_type === 'hosting') return 'Hosting';
    if (item.resource_type === 'domain') return 'Dominio';
    return 'Servicio';
}

interface PendingBillingPanelProps {
    items: BillingItem[];
    isLoading: boolean;
    checkoutLoading: boolean;
    onCheckout: (itemIds: string[] | undefined, mode: BillingCheckoutMode) => void;
}

export function PendingBillingPanel({
    items,
    isLoading,
    checkoutLoading,
    onCheckout,
}: PendingBillingPanelProps) {
    const pendingItems = items.filter(item => item.status === 'pending');

    if (isLoading || pendingItems.length === 0) {
        return null;
    }

    const PREPAY_DISCOUNT = 0.20;
    const annualTotal = pendingItems.reduce((total, item) => {
        if (item.billing_period === 'month') return total + item.amount_cents * 12;
        return total + item.amount_cents;
    }, 0);
    const annualDiscounted = Math.floor(annualTotal * (1 - PREPAY_DISCOUNT));
    const monthlyTotal = pendingItems
        .filter(item => item.billing_period === 'month')
        .reduce((total, item) => total + item.amount_cents, 0);
    const currency = pendingItems[0]?.currency ?? 'USD';
    const graceDate = pendingItems
        .map(item => item.grace_period_ends_at)
        .sort()[0];

    return (
        <Tarjeta className="billingPendientePanel" aria-label="Cobros pendientes">
            <div className="billingPendienteHeader">
                <div className="tarjetaIconoAlerta">
                    <AlertCircle size={22} strokeWidth={1.6} />
                </div>
                <div>
                    <span className="tarjetaEyebrow">Pagos pendientes</span>
                    <h2 className="tarjetaTitulo">Activa la facturación de tus servicios</h2>
                </div>
                <div className="billingPendienteResumen">
                    <span>{formatMoney(monthlyTotal, currency)}/mes</span>
                    <span className="billingResumenAnual">{formatMoney(annualDiscounted, currency)} año completo <em>−20%</em></span>
                </div>
            </div>

            <div className="billingPendienteMeta">
                <span><ReceiptText size={15} /> {pendingItems.length} pendientes</span>
                <span><CalendarDays size={15} /> Plazo hasta {formatDate(graceDate)}</span>
            </div>

            <div className="billingPendienteLista">
                {pendingItems.map(item => (
                    <Tarjeta key={item.id} className="billingPendienteItem">
                        <div>
                            <span className="tarjetaEyebrow">{itemKindLabel(item)}</span>
                            <h3 className="tarjetaTitulo tarjetaTituloCompacto">{item.title}</h3>
                            {item.description && <p className="tarjetaTexto">{item.description}</p>}
                        </div>
                        <div className="billingPendientePrecio">
                            <span className="billingPrecioCobro">
                                {formatMoney(item.amount_cents, item.currency)}{billingItemPeriodLabel(item.billing_period)}
                            </span>
                            <span>Disponible hasta {formatDate(item.grace_period_ends_at)}</span>
                        </div>
                        <Button
                            type="button"
                            variante="outline"
                            tamano="pequeno"
                            onClick={() => onCheckout([item.id], 'subscription')}
                            disabled={checkoutLoading}
                        >
                            <CreditCard size={14} /> Suscribirme
                        </Button>
                    </Tarjeta>
                ))}
            </div>

            <div className="billingPendienteAcciones">
                <Button
                    type="button"
                    variante="primario"
                    tamano="pequeno"
                    onClick={() => onCheckout(undefined, 'prepay_year')}
                    disabled={checkoutLoading}
                >
                    {checkoutLoading ? 'Preparando pago...' : `Pagar año completo — 20% off (${formatMoney(annualDiscounted, currency)})`}
                </Button>
            </div>
        </Tarjeta>
    );
}