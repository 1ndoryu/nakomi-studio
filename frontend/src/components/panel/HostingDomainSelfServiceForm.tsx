/* [165A-7] Formulario self-service para asociar el dominio del cliente al hosting.
 * Reutiliza el update existente de suscripción y refresca el detalle sin abrir soporte. */

import {useEffect, useState, type FormEvent} from 'react';
import {useMutation, useQueryClient} from '@tanstack/react-query';
import type {useHostingDetalle} from '../../hooks/useHostingDetalle';
import {apiUpdateHostingSubscription} from '../../api/hosting';
import {toast} from '../../stores/toastStore';
import {Button} from '../ui/Button';
import {Input} from '../ui/Input';

type Subscription = NonNullable<ReturnType<typeof useHostingDetalle>['subscription']>;

function normalizeDomainInput(value: string): string {
    return value
        .trim()
        .toLowerCase()
        .replace(/^https?:\/\//, '')
        .replace(/\/.*$/, '');
}

export function HostingDomainSelfServiceForm({
    sub,
    subscriptionId,
    currentDomain,
    serverIp,
}: {
    sub: Subscription;
    subscriptionId: string;
    currentDomain: string | null;
    serverIp: string | null;
}) {
    const [domainDraft, setDomainDraft] = useState(currentDomain ?? '');
    const queryClient = useQueryClient();

    useEffect(() => {
        setDomainDraft(currentDomain ?? '');
    }, [currentDomain]);

    const saveDomainMutation = useMutation({
        mutationFn: (domain: string) => apiUpdateHostingSubscription(subscriptionId, {
            plan: sub.plan,
            domain,
        }),
        onSuccess: async (updated) => {
            await Promise.all([
                queryClient.invalidateQueries({queryKey: ['hosting-subscription', subscriptionId]}),
                queryClient.invalidateQueries({queryKey: ['hosting-subscriptions']}),
            ]);
            setDomainDraft(updated.domain ?? '');
            toast.success(
                updated.domain_verification_status === 'pending_verification'
                    ? 'Dominio guardado. Añade el TXT de verificación antes de activar el dominio en hosting.'
                    : 'Dominio guardado.',
            );
        },
        onError: (error: unknown) => {
            const message = error instanceof Error ? error.message : 'No se pudo guardar el dominio';
            toast.error(message);
        },
    });

    const normalizedCurrentDomain = normalizeDomainInput(currentDomain ?? '');
    const normalizedDraftDomain = normalizeDomainInput(domainDraft);
    const canSaveDomain = normalizedDraftDomain.length > 0
        && normalizedDraftDomain !== normalizedCurrentDomain
        && !saveDomainMutation.isPending;

    const handleDomainSubmit = (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        if (!normalizedDraftDomain) {
            toast.error('Ingresa un dominio válido para continuar');
            return;
        }
        saveDomainMutation.mutate(normalizedDraftDomain);
    };

    return (
        <form className="dnsForm" onSubmit={handleDomainSubmit}>
            <div className="dnsFormRow">
                <label className="dnsFormField dnsFormField--value" htmlFor="hosting-domain-input">
                    <span>{currentDomain ? 'Cambiar dominio' : 'Tu dominio'}</span>
                    <Input
                        id="hosting-domain-input"
                        type="text"
                        inputMode="url"
                        autoCapitalize="none"
                        autoCorrect="off"
                        spellCheck={false}
                        value={domainDraft}
                        onChange={event => setDomainDraft(event.target.value)}
                        placeholder="tudominio.com"
                        disabled={saveDomainMutation.isPending}
                    />
                </label>
                <div className="dnsFormActions">
                    <Button
                        type="submit"
                        variante="primario"
                        tamano="pequeno"
                        disabled={!canSaveDomain}
                    >
                        {saveDomainMutation.isPending ? 'Guardando…' : currentDomain ? 'Actualizar dominio' : 'Guardar dominio'}
                    </Button>
                </div>
            </div>
            <p className="hostingDetalleSectionDesc">
                Guarda primero el dominio. Si es nuevo, verifícalo con un TXT y después crea los registros A hacia {serverIp ?? 'la IP de tu servidor'}.
            </p>
        </form>
    );
}