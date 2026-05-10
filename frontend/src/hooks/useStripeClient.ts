import {useEffect, useMemo, useState} from 'react';
import {loadStripe, type Stripe} from '@stripe/stripe-js';

import {apiGetPublicConfig} from '../api/publicConfig';

type StripePromise = Promise<Stripe | null>;

interface StripeClientState {
    stripePromise: StripePromise | null;
    cargando: boolean;
    configurado: boolean;
}

let cachedStripePromise: StripePromise | null = null;

function getBuildTimeStripeKey(): string | null {
    return (import.meta.env.VITE_STRIPE_PUBLISHABLE_KEY as string | undefined)?.trim() || null;
}

async function resolveStripeKey(): Promise<string | null> {
    const buildTimeKey = getBuildTimeStripeKey();
    if (buildTimeKey) {
        return buildTimeKey;
    }

    const config = await apiGetPublicConfig();
    return config.stripe_publishable_key?.trim() || null;
}

export function getStripePromise(): StripePromise {
    cachedStripePromise ??= resolveStripeKey().then((key) => (key ? loadStripe(key) : null));
    return cachedStripePromise;
}

export function useStripeClient(): StripeClientState {
    const stripePromise = useMemo(() => getStripePromise(), []);
    const [configurado, setConfigurado] = useState(false);
    const [cargando, setCargando] = useState(true);

    useEffect(() => {
        let activo = true;
        stripePromise
            .then((stripe) => {
                if (activo) {
                    setConfigurado(Boolean(stripe));
                }
            })
            .catch(() => {
                if (activo) {
                    setConfigurado(false);
                }
            })
            .finally(() => {
                if (activo) {
                    setCargando(false);
                }
            });

        return () => {
            activo = false;
        };
    }, [stripePromise]);

    return {stripePromise, cargando, configurado};
}
