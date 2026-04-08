/* [044A-38 Fase 3] Modal de checkout con Stripe Elements.
 * Inicia PaymentIntent en el backend, muestra formulario de tarjeta,
 * y confirma el pago. El webhook procesa el resultado asíncronamente.
 * [064A-55] Migrado a <Modal> del sistema (focus trap, Escape, scroll lock). */

import { useCallback, useState } from 'react';
import { loadStripe } from '@stripe/stripe-js';
import {
    Elements,
    PaymentElement,
    useStripe,
    useElements,
} from '@stripe/react-stripe-js';
import { apiInitiatePayment } from '../../api/payments';
import { formatPrice } from '../../api/orders';
import { Button } from '../ui/Button';
import { Modal } from '../ui/Modal';
import './CheckoutModal.css';

const stripePromise = import.meta.env.VITE_STRIPE_PUBLISHABLE_KEY
    ? loadStripe(import.meta.env.VITE_STRIPE_PUBLISHABLE_KEY as string)
    : null;

interface CheckoutModalProps {
    orderId: string;
    orderNumber: number;
    amountCents: number;
    currency: string;
    phaseNumber?: number;
    onClose: () => void;
    onSuccess: () => void;
}

export default function CheckoutModal(props: CheckoutModalProps) {
    const { orderId, phaseNumber, onClose, onSuccess } = props;
    const [clientSecret, setClientSecret] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);

    const iniciar = useCallback(async () => {
        setLoading(true);
        setError(null);
        try {
            const resp = await apiInitiatePayment(orderId, {
                phase_number: phaseNumber,
            });
            setClientSecret(resp.client_secret);
        } catch (err: unknown) {
            /* [074A-24] Extraer mensaje real del response body de Axios.
             * El backend devuelve { error: "...", message: "..." } en errores. */
            const axiosData = (err as { response?: { data?: { message?: string } } })?.response?.data;
            const msg = axiosData?.message
                ?? (err instanceof Error ? err.message : 'Error iniciando pago');
            setError(msg);
        } finally {
            setLoading(false);
        }
    }, [orderId, phaseNumber]);

    if (!stripePromise) {
        return (
            <Modal abierto onCerrar={onClose} className="checkoutModal">
                <p className="checkoutError">
                    Stripe no está configurado. Agrega
                    VITE_STRIPE_PUBLISHABLE_KEY al .env
                </p>
            </Modal>
        );
    }

    return (
        <Modal abierto onCerrar={onClose} className="checkoutModal">
            <h3 className="modalTitulo">
                Pagar Orden #{props.orderNumber}
            </h3>
            <p className="checkoutMonto">
                {formatPrice(props.amountCents, props.currency)}
            </p>

            {error && <p className="checkoutError">{error}</p>}

            {!clientSecret && (
                <Button
                    className="checkoutBoton"
                    type="button"
                    variante="primario"
                    tamano="mediano"
                    onClick={iniciar}
                    disabled={loading}
                >
                    {loading ? 'Preparando...' : 'Continuar al pago'}
                </Button>
            )}

            {clientSecret && (
                <Elements
                    stripe={stripePromise}
                    options={{ clientSecret, appearance: { theme: 'stripe' } }}
                >
                    <FormularioPago
                        onSuccess={onSuccess}
                        onError={setError}
                    />
                </Elements>
            )}
        </Modal>
    );
}

function FormularioPago({
    onSuccess,
    onError,
}: {
    onSuccess: () => void;
    onError: (msg: string) => void;
}) {
    const stripe = useStripe();
    const elements = useElements();
    const [procesando, setProcesando] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!stripe || !elements) return;

        setProcesando(true);
        const { error } = await stripe.confirmPayment({
            elements,
            confirmParams: {
                return_url: `${window.location.origin}/panel`,
            },
            redirect: 'if_required',
        });

        if (error) {
            onError(error.message ?? 'Error al procesar el pago');
            setProcesando(false);
        } else {
            onSuccess();
        }
    };

    return (
        <form onSubmit={handleSubmit} className="checkoutForm">
            <PaymentElement />
            <Button
                type="submit"
                className="checkoutBoton"
                disabled={!stripe || procesando}
            >
                {procesando ? 'Procesando...' : 'Pagar ahora'}
            </Button>
        </form>
    );
}
