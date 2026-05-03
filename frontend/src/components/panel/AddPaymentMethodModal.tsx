import {useState} from 'react';
import {loadStripe} from '@stripe/stripe-js';
import {CardElement, Elements, useElements, useStripe} from '@stripe/react-stripe-js';

import {
    apiSavePaymentMethod,
    type SavedPaymentMethod,
} from '../../api/payments';
import {useAddPaymentMethodModal} from '../../hooks/useAddPaymentMethodModal';
import {Button} from '../ui/Button';
import {Modal, ModalBody, ModalField, ModalLabel} from '../ui/Modal';

import './AddPaymentMethodModal.css';

const stripePromise = import.meta.env.VITE_STRIPE_PUBLISHABLE_KEY
    ? loadStripe(import.meta.env.VITE_STRIPE_PUBLISHABLE_KEY as string)
    : null;

interface AddPaymentMethodModalProps {
    open: boolean;
    onClose: () => void;
    onSaved: (paymentMethod: SavedPaymentMethod) => void;
}

export function AddPaymentMethodModal({open, onClose, onSaved}: AddPaymentMethodModalProps) {
    const {clientSecret, isLoadingIntent, loadError} = useAddPaymentMethodModal(open);

    if (!open) {
        return null;
    }

    if (!stripePromise) {
        return (
            <Modal abierto onCerrar={onClose} className="agregarTarjetaModal">
                <ModalBody>
                    <p className="agregarTarjetaError">Stripe no esta configurado en este entorno.</p>
                    <div className="modalAcciones">
                        <Button type="button" variante="outline" tamano="pequeno" onClick={onClose}>Cerrar</Button>
                    </div>
                </ModalBody>
            </Modal>
        );
    }

    return (
        <Modal abierto onCerrar={onClose} className="agregarTarjetaModal">
            <ModalBody>
                <p className="modalTexto">
                    La tarjeta se guardara en Stripe y quedara disponible para pagos futuros.
                </p>

                {loadError && <p className="agregarTarjetaError">{loadError}</p>}
                {isLoadingIntent && <p className="modalTexto">Preparando formulario seguro...</p>}

                {clientSecret && (
                    <Elements stripe={stripePromise}>
                        <AddPaymentMethodForm
                            clientSecret={clientSecret}
                            onClose={onClose}
                            onSaved={onSaved}
                        />
                    </Elements>
                )}
            </ModalBody>
        </Modal>
    );
}

function AddPaymentMethodForm({
    clientSecret,
    onClose,
    onSaved,
}: {
    clientSecret: string;
    onClose: () => void;
    onSaved: (paymentMethod: SavedPaymentMethod) => void;
}) {
    const stripe = useStripe();
    const elements = useElements();
    const [isSubmitting, setIsSubmitting] = useState(false);
    const [submitError, setSubmitError] = useState<string | null>(null);

    const handleSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        if (!stripe || !elements) {
            return;
        }

        const cardElement = elements.getElement(CardElement);
        if (!cardElement) {
            setSubmitError('El campo de tarjeta no esta listo todavia.');
            return;
        }

        setIsSubmitting(true);
        setSubmitError(null);

        const {error, setupIntent} = await stripe.confirmCardSetup(clientSecret, {
            payment_method: {
                card: cardElement,
            },
        });

        if (error || !setupIntent?.id) {
            setSubmitError(error?.message ?? 'No se pudo confirmar la tarjeta');
            setIsSubmitting(false);
            return;
        }

        try {
            const paymentMethod = await apiSavePaymentMethod({
                setup_intent_id: setupIntent.id,
            });
            onSaved(paymentMethod);
        } catch (saveError) {
            const message = (saveError as {response?: {data?: {message?: string}}})?.response?.data?.message
                ?? (saveError instanceof Error ? saveError.message : 'No se pudo guardar la tarjeta');
            setSubmitError(message);
        } finally {
            setIsSubmitting(false);
        }
    };

    return (
        <ModalBody as="form" onSubmit={handleSubmit}>
            <ModalField>
                <ModalLabel htmlFor="agregarTarjetaCardElement">
                    Datos de la tarjeta
                </ModalLabel>
                <div className="agregarTarjetaStripeField" id="agregarTarjetaCardElement">
                    <CardElement
                        options={{
                            hidePostalCode: true,
                            style: {
                                base: {
                                    color: '#111111',
                                    fontFamily: 'var(--font-primary)',
                                    fontSize: '16px',
                                    '::placeholder': {
                                        color: '#7b7b7b',
                                    },
                                },
                                invalid: {
                                    color: '#b42318',
                                },
                            },
                        }}
                    />
                </div>
            </ModalField>

            {submitError && <p className="agregarTarjetaError">{submitError}</p>}

            <div className="modalAcciones">
                <Button type="button" variante="outline" tamano="pequeno" onClick={onClose} disabled={isSubmitting}>
                    Cancelar
                </Button>
                <Button type="submit" variante="primario" tamano="pequeno" disabled={!stripe || isSubmitting}>
                    {isSubmitting ? 'Guardando...' : 'Guardar tarjeta'}
                </Button>
            </div>
        </ModalBody>
    );
}