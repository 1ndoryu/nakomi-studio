/* [044A-40] Hook para la lógica del modal de compra.
 * Maneja: pasos del modal, auth inline, creación de orden e inicio de pago.
 * Extraído de ModalCompra.tsx para cumplir SRP (max 3 useState en componente).
 * [064A-3] Flujo simplificado: solo pide email. Si el email ya existe, pide password.
 * [104A-25] Guard ref contra doble invocación de iniciarCompra (previene órdenes duplicadas). */
import {useState, useRef} from 'react';
import {useAuthStore} from '../stores/authStore';
import {apiQuickRegister, apiLogin} from '../api/auth';
import {apiCreateOrder, type PaymentMode} from '../api/orders';
import {apiSelfSubscribe} from '../api/hosting';
import {apiInitiatePayment} from '../api/payments';
import {PANEL_TAB_KEY} from '../data/panel';
import {navegar} from '../navegacionSPA';
import type {PlanServicio} from '../data/planes/tipos';

export type PasoModal = 'resumen' | 'auth' | 'procesando' | 'checkout' | 'error';

interface CheckoutPendiente {
    clientSecret: string;
    orderId: string;
    orderNumber: number;
    amountCents: number;
    currency: string;
}

interface UseModalCompraParams {
    plan: PlanServicio;
    servicioSlug: string;
    onClose: () => void;
}

function normalizeHostingPlanSlug(planId: string): string {
    return planId.trim().toLowerCase().replace(/^hosting-/, '');
}

export function useModalCompra({plan, servicioSlug, onClose}: UseModalCompraParams) {
    const logueado = useAuthStore(s => s.logueado);
    const login = useAuthStore(s => s.login);

    const [paso, setPaso] = useState<PasoModal>('resumen');
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    /* [064A-3] Si el email ya existe, mostramos campo password */
    const [emailExiste, setEmailExiste] = useState(false);
    const [errorMsg, setErrorMsg] = useState('');
    /* [064A-60] Modo de pago seleccionable: full (20% desc), half_half (10%), phased (0%) */
    const [paymentMode, setPaymentMode] = useState<PaymentMode>('full');
    /* [104A-16] Hosting ya no pasa por órdenes genéricas.
     * El dominio opcional se envía al flujo self-service real de suscripciones. */
    const [hostingDomain, setHostingDomain] = useState('');
    const [projectDescription, setProjectDescription] = useState('');
    const [checkoutPendiente, setCheckoutPendiente] = useState<CheckoutPendiente | null>(null);
    const isHosting = servicioSlug === 'hosting';
    /* [104A-25] Ref guard: previene doble invocación de iniciarCompra.
     * useState es async y no previene race conditions entre renders. */
    const compraEnCurso = useRef(false);

    const navegarAlPanelPendiente = () => {
        /* [104A-15] Forzar la tab correcta evita que el usuario aterrice en la
         * ultima seccion persistida y pierda de vista el flujo que acaba de iniciar.
         * [104A-16] Hosting vuelve al panel de Hosting, no a Proyectos. */
        sessionStorage.setItem(PANEL_TAB_KEY, isHosting ? 'hosting' : 'proyectos');
        navegar('/panel');
        onClose();
    };

    const crearHostingYRedirigir = async () => {
        setPaso('procesando');
        try {
            const response = await apiSelfSubscribe({
                plan: normalizeHostingPlanSlug(plan.id),
                domain: hostingDomain.trim() || undefined,
            });
            sessionStorage.setItem(PANEL_TAB_KEY, 'hosting');
            window.location.href = response.checkout_url;
        } catch (err: unknown) {
            setPaso('error');
            const msg = err instanceof Error
                ? err.message
                : 'Error al iniciar el checkout de hosting.';
            setErrorMsg(msg);
        }
    };

    /* [104A-15] Crear la orden y abrir checkout inmediatamente con el
     * PaymentIntent ya creado, en vez de redirigir primero al panel. */
    const crearOrdenYPagar = async () => {
        setPaso('procesando');
        try {
            const orden = await apiCreateOrder({
                service_slug: servicioSlug,
                plan_slug: plan.id,
                payment_mode: paymentMode,
                project_description: projectDescription.trim() || undefined,
                client_notes: undefined,
            });

            /* Intentar iniciar pago Stripe */
            try {
                const paymentIntent = await apiInitiatePayment(orden.id, {
                    phase_number: paymentMode === 'phased' ? 1 : undefined,
                });
                sessionStorage.setItem(PANEL_TAB_KEY, 'proyectos');
                setCheckoutPendiente({
                    clientSecret: paymentIntent.client_secret,
                    orderId: orden.id,
                    orderNumber: orden.order_number,
                    amountCents: paymentIntent.amount_cents,
                    currency: paymentIntent.currency,
                });
                setPaso('checkout');
                return;
            } catch {
                /* Si Stripe no esta configurado o falla el intent, al menos dejar
                 * la orden visible en el panel como pendiente de pago. */
                navegarAlPanelPendiente();
                return;
            }
        } catch (err: unknown) {
            setPaso('error');
            const msg = err instanceof Error ? err.message : 'Error al crear la orden.';
            setErrorMsg(msg);
        }
    };

    const iniciarCompra = async () => {
        /* [104A-25] Guard contra doble invocación: si ya hay una compra en curso, ignorar */
        if (compraEnCurso.current) return;
        compraEnCurso.current = true;
        try {
            if (isHosting) {
                await crearHostingYRedirigir();
                return;
            }
            await crearOrdenYPagar();
        } finally {
            compraEnCurso.current = false;
        }
    };

    /* Paso 1: usuario confirma que quiere continuar */
    const handleContinuar = () => {
        if (!isHosting && projectDescription.trim().length < 10) {
            setErrorMsg('Describe tu proyecto con un poco más de detalle para crear la orden.');
            return;
        }

        setErrorMsg('');
        if (logueado) {
            void iniciarCompra();
        } else {
            setPaso('auth');
        }
    };

    /* [064A-3] Paso 2: solo email → quick-register. Si 409 → pedir password → login */
    const handleAuth = async (e: React.FormEvent) => {
        e.preventDefault();
        setErrorMsg('');

        /* Si ya sabemos que el email existe, hacer login con password */
        if (emailExiste) {
            setPaso('procesando');
            try {
                const authResp = await apiLogin(email, password);
                login(authResp.token, authResp.user_id, email, authResp.role, authResp.effective_role, authResp.needs_password);
                await iniciarCompra();
            } catch (err: unknown) {
                setPaso('auth');
                const msg = err instanceof Error ? err.message : 'Credenciales inválidas.';
                setErrorMsg(msg);
            }
            return;
        }

        /* Intentar registro rapido solo con email */
        setPaso('procesando');
        try {
            const authResp = await apiQuickRegister(email);
            login(authResp.token, authResp.user_id, email, authResp.role, authResp.effective_role, authResp.needs_password);
            await iniciarCompra();
        } catch (err: unknown) {
            /* 409 = email ya registrado → pedir password */
            const is409 = typeof err === 'object' && err !== null && 'response' in err
                && (err as {response?: {status?: number}}).response?.status === 409;
            if (is409) {
                setEmailExiste(true);
                setPaso('auth');
                setErrorMsg('Ya tienes cuenta. Introduce tu contraseña para continuar.');
            } else {
                setPaso('error');
                const msg = err instanceof Error ? err.message : 'Error al procesar. Intenta de nuevo.';
                setErrorMsg(msg);
            }
        }
    };

    const reintentar = () => {
        setPaso('resumen');
        setEmailExiste(false);
        setErrorMsg('');
        setCheckoutPendiente(null);
    };

    return {
        paso,
        email,
        setEmail,
        password,
        setPassword,
        emailExiste,
        errorMsg,
        paymentMode,
        setPaymentMode,
        hostingDomain,
        setHostingDomain,
        projectDescription,
        setProjectDescription,
        checkoutPendiente,
        isHosting,
        navegarAlPanelPendiente,
        handleContinuar,
        handleAuth,
        reintentar
    };
}
