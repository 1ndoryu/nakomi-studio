import {useCallback, useEffect, useState} from 'react';
import type {OrderPhaseResponse} from '../api/orders';
import {toast} from '../stores/toastStore';

interface UseFaseCardParams {
    phase: OrderPhaseResponse;
    orderId: string;
    canDefinePhase: boolean;
    onActualizarFase: (
        orderId: string,
        phase: number,
        req: {
            title?: string;
            description?: string;
            price_cents?: number;
            estimated_days?: number;
            max_revisions?: number;
        },
    ) => Promise<void>;
}

function buildDraft(phase: OrderPhaseResponse) {
    return {
        title: phase.title,
        description: phase.description ?? '',
        priceUsd: (phase.price_cents / 100).toFixed(2),
        estimatedDays: String(phase.estimated_days),
        maxRevisions: String(phase.max_revisions),
    };
}

export function useFaseCard({phase, orderId, canDefinePhase, onActualizarFase}: UseFaseCardParams) {
    const [editando, setEditando] = useState(false);
    const [draft, setDraft] = useState(() => buildDraft(phase));

    useEffect(() => {
        setDraft(buildDraft(phase));
        setEditando(false);
    }, [phase]);

    const canEditDefinition = canDefinePhase && (phase.status === 'locked' || phase.status === 'pending_payment');

    const cancelarEdicion = useCallback(() => {
        setDraft(buildDraft(phase));
        setEditando(false);
    }, [phase]);

    const guardarDefinicion = useCallback(async () => {
        const title = draft.title.trim();
        const description = draft.description.trim();
        const priceUsd = Number.parseFloat(draft.priceUsd);
        const estimatedDays = Number.parseInt(draft.estimatedDays, 10);
        const maxRevisions = Number.parseInt(draft.maxRevisions, 10);

        if (!title) {
            toast.error('El título de la fase no puede estar vacío.');
            return;
        }

        if (!Number.isFinite(priceUsd) || priceUsd <= 0) {
            toast.error('El precio de la fase debe ser mayor que cero.');
            return;
        }

        if (!Number.isInteger(estimatedDays) || estimatedDays <= 0) {
            toast.error('Los días estimados deben ser un entero mayor que cero.');
            return;
        }

        if (!Number.isInteger(maxRevisions) || maxRevisions < 0) {
            toast.error('Las revisiones máximas deben ser cero o más.');
            return;
        }

        try {
            await onActualizarFase(orderId, phase.phase_number, {
                title,
                description: description || undefined,
                price_cents: Math.round(priceUsd * 100),
                estimated_days: estimatedDays,
                max_revisions: maxRevisions,
            });
            setEditando(false);
            toast.success('Fase actualizada.');
        } catch (err) {
            const message = err instanceof Error ? err.message : 'No se pudo guardar la fase.';
            toast.error(message);
        }
    }, [draft, onActualizarFase, orderId, phase.phase_number]);

    return {
        editando,
        setEditando,
        draft,
        setDraft,
        canEditDefinition,
        cancelarEdicion,
        guardarDefinicion,
    };
}