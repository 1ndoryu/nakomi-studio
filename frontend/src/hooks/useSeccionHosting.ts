/* [074A-63] Hook de estado para SeccionHosting.
 * [094A-2] Ahora incluye selectedHostingId para navegación lista→detalle.
 * [094A-3] Mutations extraídas a useHostingMutations para cumplir límite 120 líneas.
 * 3 useState: showCreateModal, tabActiva, selectedHostingId. */

import {useState, useMemo, useEffect, useCallback} from 'react';
import {useQuery} from '@tanstack/react-query';
import {apiListHostingSubscriptions} from '../api/hosting';
import {toast} from '../stores/toastStore';
import {useAuthStore} from '../stores/authStore';
import {useHostingMutations} from './useHostingMutations';

const ACTIVE_STATUSES = new Set(['pending', 'provisioning', 'active']);

export function useSeccionHosting() {
    const [showCreateModal, setShowCreateModal] = useState(false);
    const [tabActiva, setTabActiva] = useState<'activos' | 'inactivos' | 'servidores'>('activos');
    const [selectedHostingId, setSelectedHostingId] = useState<string | null>(null);

    const effectiveRole = useAuthStore(s => s.user?.effectiveRole) ?? 'client';
    const isAdmin = effectiveRole === 'admin';

    /* [084A-24] Detectar retorno de Stripe Checkout (?hosting=success/cancelled) */
    useEffect(() => {
        const params = new URLSearchParams(window.location.search);
        const hostingResult = params.get('hosting');
        if (hostingResult === 'success') {
            toast.success('¡Pago completado! Tu hosting se activará en breve.');
        } else if (hostingResult === 'cancelled') {
            toast.warning('Checkout cancelado. Puedes intentarlo de nuevo.');
        }
        if (hostingResult) {
            const url = new URL(window.location.href);
            url.searchParams.delete('hosting');
            url.searchParams.delete('session_id');
            window.history.replaceState({}, '', url.pathname + url.search);
        }
    }, []);

    const hostingKey = ['hosting-subscriptions', effectiveRole] as const;

    const {data: subscriptions = [], isLoading} = useQuery({
        queryKey: hostingKey,
        queryFn: apiListHostingSubscriptions,
    });

    const activos = useMemo(
        () => subscriptions.filter(s => ACTIVE_STATUSES.has(s.status)),
        [subscriptions],
    );
    const inactivos = useMemo(
        () => subscriptions.filter(s => !ACTIVE_STATUSES.has(s.status)),
        [subscriptions],
    );
    const listaActual = tabActiva === 'activos' ? activos : inactivos;

    const onCreateSuccess = useCallback(() => setShowCreateModal(false), []);
    const mutations = useHostingMutations(hostingKey, onCreateSuccess);

    return {
        subscriptions,
        isLoading,
        isAdmin,
        tabActiva,
        setTabActiva,
        activos,
        inactivos,
        listaActual,
        showCreateModal,
        setShowCreateModal,
        selectedHostingId,
        setSelectedHostingId,
        ...mutations,
    };
}
