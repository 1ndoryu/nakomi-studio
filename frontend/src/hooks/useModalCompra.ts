/* [044A-40] Hook para la lógica del modal de compra.
 * Maneja: pasos del modal, auth inline, creación de orden e inicio de pago.
 * Extraído de ModalCompra.tsx para cumplir SRP (max 3 useState en componente).
 * [064A-3] Flujo simplificado: solo pide email. Si el email ya existe, pide password. */
import {useState} from 'react';
import {useAuthStore} from '../stores/authStore';
import {apiQuickRegister, apiLogin} from '../api/auth';
import {apiCreateOrder, type PaymentMode} from '../api/orders';
import {apiInitiatePayment} from '../api/payments';
import {navegar} from '../navegacionSPA';
import type {PlanServicio} from '../data/planes/tipos';

export type PasoModal = 'resumen' | 'auth' | 'procesando' | 'error';

interface UseModalCompraParams {
    plan: PlanServicio;
    servicioSlug: string;
    onClose: () => void;
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
    /* [084A-28] Meses de pago para hosting (1-12). No aplica a servicios. */
    const [months, setMonths] = useState(1);
    const [projectDescription, setProjectDescription] = useState('');
    const isHosting = !!plan.periodo;

    /* Crear orden y redirigir a pasarela de pago */
    const crearOrdenYPagar = async () => {
        setPaso('procesando');
        try {
            const orden = await apiCreateOrder({
                service_slug: servicioSlug,
                plan_slug: plan.id,
                payment_mode: isHosting ? 'full' : paymentMode,
                project_description: projectDescription.trim() || undefined,
                client_notes: isHosting && months > 1 ? `Pago anticipado: ${months} meses` : undefined
            });

            /* Intentar iniciar pago Stripe */
            try {
                await apiInitiatePayment(orden.id, {phase_number: 1});
                /* TO-DO: Integrar Stripe Elements con client_secret para confirmación de pago.
                 * Por ahora redirigir al panel donde puede ver la orden pendiente de pago. */
            } catch {
                /* Si Stripe no está configurado, ir directo al panel */
            }

            navegar('/panel');
            onClose();
        } catch (err: unknown) {
            setPaso('error');
            const msg = err instanceof Error ? err.message : 'Error al crear la orden.';
            setErrorMsg(msg);
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
            crearOrdenYPagar();
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
                login(authResp.token, authResp.user_id, email, authResp.role, authResp.effective_role);
                await crearOrdenYPagar();
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
            login(authResp.token, authResp.user_id, email, authResp.role, authResp.effective_role);
            await crearOrdenYPagar();
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
        months,
        setMonths,
        projectDescription,
        setProjectDescription,
        isHosting,
        handleContinuar,
        handleAuth,
        reintentar
    };
}
