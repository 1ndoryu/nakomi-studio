/* [215A-2] Hook del configurador público de hosting.
 * Reutiliza el patrón del configurador VPS: plan primero, datos técnicos después y checkout real. */
import {useState} from 'react';
import type React from 'react';
import {apiLogin, apiQuickRegister, extraerMensajeError} from '../api/auth';
import {apiSelfSubscribe, type HostingPlanInfo} from '../api/hosting';
import {PANEL_TAB_KEY} from '../data/panel';
import {useAuthStore} from '../stores/authStore';
import {useHostingCatalog} from './useHostingCatalog';

export type HostingConfiguradorKind = 'normal' | 'wordpress';
export type HostingBillingCycle = 1 | 6 | 12;

interface HostingConfigForm {
    selectedPlan: string;
    domain: string;
    wpAdminUser: string;
    wpAdminPassword: string;
    wpLanguage: string;
    sftpUser: string;
    sftpPassword: string;
    billingCycle: string;
    email: string;
    password: string;
}

interface HostingConfigStatus {
    submitting: boolean;
    error: string;
}

const DEFAULT_FORM: HostingConfigForm = {
    selectedPlan: '',
    domain: '',
    wpAdminUser: '',
    wpAdminPassword: '',
    wpLanguage: 'es_ES',
    sftpUser: '',
    sftpPassword: '',
    billingCycle: '1',
    email: '',
    password: '',
};

export const HOSTING_LANGUAGE_OPTIONS = [
    {value: 'es_ES', label: 'Español'},
    {value: 'en_US', label: 'English'},
    {value: 'ja', label: '日本語'},
];

export const HOSTING_BILLING_OPTIONS: Array<{months: HostingBillingCycle; label: string; description: string}> = [
    {months: 1, label: '1 mes', description: 'Sin descuento'},
    {months: 6, label: '6 meses', description: 'Descuento de $10'},
    {months: 12, label: '1 año', description: 'Descuento de $20'},
];

export function formatHostingMoney(cents: number): string {
    const amount = cents / 100;
    return amount % 1 === 0 ? `$${amount}` : `$${amount.toFixed(2)}`;
}

export function formatHostingStorage(storageMb: number): string {
    if (storageMb >= 1024) {
        const gb = storageMb / 1024;
        return `${gb % 1 === 0 ? gb.toFixed(0) : gb.toFixed(1)} GB`;
    }
    return `${storageMb} MB`;
}

function parseBillingCycle(value: string): HostingBillingCycle {
    if (value === '6') return 6;
    if (value === '12') return 12;
    return 1;
}

function hostingBillingDiscountCents(months: HostingBillingCycle): number {
    if (months === 12) return 2000;
    if (months === 6) return 1000;
    return 0;
}

function resolveSelectedPlan(plans: HostingPlanInfo[], selectedPlan: string, initialPlan?: string): HostingPlanInfo | undefined {
    const planId = selectedPlan || initialPlan || plans[0]?.id || '';
    return plans.find(plan => plan.id === planId) ?? plans[0];
}

export function useHostingConfiguradorIsland(kind: HostingConfiguradorKind, initialPlan?: string) {
    const {plans, isLoading} = useHostingCatalog(kind);
    const logueado = useAuthStore(s => s.logueado);
    const login = useAuthStore(s => s.login);
    const [form, setForm] = useState<HostingConfigForm>({...DEFAULT_FORM, selectedPlan: initialPlan ?? ''});
    const [emailExiste, setEmailExiste] = useState(false);
    const [status, setStatus] = useState<HostingConfigStatus>({submitting: false, error: ''});

    const selectedPlan = resolveSelectedPlan(plans, form.selectedPlan, initialPlan);
    const billingCycleMonths = parseBillingCycle(form.billingCycle);
    const discountCents = selectedPlan ? Math.min(hostingBillingDiscountCents(billingCycleMonths), selectedPlan.priceCents * billingCycleMonths) : 0;
    const dueToday = selectedPlan ? (selectedPlan.priceCents * billingCycleMonths) - discountCents : 0;

    const updateField = (field: keyof HostingConfigForm, value: string) => {
        setForm(prev => ({...prev, [field]: value}));
    };

    const validateTechnicalFields = (): boolean => {
        if (kind === 'wordpress' && form.wpAdminPassword.trim().length < 8) {
            setStatus({submitting: false, error: 'La contraseña de wp-admin debe tener al menos 8 caracteres.'});
            return false;
        }
        if (kind === 'wordpress' && !form.wpAdminUser.trim()) {
            setStatus({submitting: false, error: 'El usuario de wp-admin es obligatorio.'});
            return false;
        }
        if ((form.sftpUser.trim() && form.sftpPassword.trim().length < 12) || (!form.sftpUser.trim() && form.sftpPassword.trim())) {
            setStatus({submitting: false, error: 'Si defines SFTP manualmente, indica usuario y una contraseña de al menos 12 caracteres.'});
            return false;
        }
        return true;
    };

    const submitCheckout = async () => {
        if (!selectedPlan || status.submitting || !validateTechnicalFields()) return;
        setStatus({submitting: true, error: ''});
        try {
            const response = await apiSelfSubscribe({
                plan: selectedPlan.id,
                domain: form.domain.trim() || undefined,
                billing_cycle_months: billingCycleMonths,
                wp_admin_username: kind === 'wordpress' ? form.wpAdminUser.trim() || undefined : undefined,
                wp_admin_password: kind === 'wordpress' ? form.wpAdminPassword.trim() || undefined : undefined,
                wp_language: kind === 'wordpress' ? form.wpLanguage : undefined,
                sftp_user: form.sftpUser.trim() || undefined,
                sftp_password: form.sftpPassword.trim() || undefined,
            });
            localStorage.setItem(PANEL_TAB_KEY, 'hosting');
            window.location.href = response.checkout_url;
        } catch (error) {
            setStatus({submitting: false, error: extraerMensajeError(error)});
        }
    };

    const handleSubmit = async (event: React.FormEvent) => {
        event.preventDefault();
        if (!selectedPlan) return;

        if (logueado) {
            await submitCheckout();
            return;
        }

        setStatus({submitting: true, error: ''});
        try {
            const auth = emailExiste
                ? await apiLogin(form.email, form.password)
                : await apiQuickRegister(form.email);
            login(auth.token, auth.user_id, auth.email, auth.role, auth.effective_role, auth.needs_password);
            await submitCheckout();
        } catch (error) {
            const isConflict = typeof error === 'object' && error !== null && 'response' in error
                && (error as {response?: {status?: number}}).response?.status === 409;
            if (isConflict) {
                setEmailExiste(true);
                setStatus({submitting: false, error: 'Ya tienes cuenta. Introduce tu contraseña para continuar.'});
                return;
            }
            setStatus({submitting: false, error: extraerMensajeError(error)});
        }
    };

    return {
        plans,
        isLoading,
        selectedPlan,
        form,
        emailExiste,
        status,
        dueToday,
        discountCents,
        billingCycleMonths,
        logueado,
        updateField,
        handleSubmit,
    };
}