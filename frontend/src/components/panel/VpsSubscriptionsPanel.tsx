import {useMemo, useState} from 'react';
import {Server, ShieldAlert, TerminalSquare} from 'lucide-react';
import {VPS_STATUS_LABELS, type VpsSubscription} from '../../api/hosting';
import {navegar} from '../../navegacionSPA';
import {Button} from '../ui/Button';
import {Textarea} from '../ui/Textarea';
import './VpsSubscriptionsPanel.css';

function formatPrice(amountCents: number): string {
    return `$${(amountCents / 100).toFixed(amountCents % 100 === 0 ? 0 : 2)}/mes`;
}

function humanizeTier(tierName: string): string {
    switch (tierName) {
    case 'vps1':
        return 'Cloud VPS 1';
    case 'vps2':
        return 'Cloud VPS 2';
    case 'vps3':
        return 'Cloud VPS 3';
    case 'vps4':
        return 'Cloud VPS 4';
    default:
        return tierName;
    }
}

interface VpsSubscriptionsPanelProps {
    subscriptions: VpsSubscription[];
    isAdmin: boolean;
    onApprove: (id: string) => void;
    onReject: (id: string, reason: string) => void;
    approveLoading: boolean;
    rejectLoading: boolean;
}

export function VpsSubscriptionsPanel({
    subscriptions,
    isAdmin,
    onApprove,
    onReject,
    approveLoading,
    rejectLoading,
}: VpsSubscriptionsPanelProps) {
    const [rejectReasons, setRejectReasons] = useState<Record<string, string>>({});

    const orderedSubscriptions = useMemo(
        () => [...subscriptions].sort((left, right) => right.created_at.localeCompare(left.created_at)),
        [subscriptions],
    );

    if (orderedSubscriptions.length === 0) {
        return (
            <div className="vpsSubscriptionsEmpty">
                <Server size={36} strokeWidth={1.2} />
                <p>{isAdmin ? 'Todavía no hay VPS revendidos.' : 'Todavía no tienes VPS activos o pendientes.'}</p>
                {!isAdmin && (
                    <Button variante="primario" onClick={() => navegar('/soluciones/vps')}>
                        Ver planes VPS
                    </Button>
                )}
            </div>
        );
    }

    return (
        <div className="vpsSubscriptionsList">
            {orderedSubscriptions.map(subscription => {
                const rejectReason = rejectReasons[subscription.id] ?? '';
                const canReview = isAdmin && subscription.status === 'pending_approval';
                return (
                    <article key={subscription.id} className="hostingCard">
                        <div className="panelCardIcono">
                            <Server size={28} strokeWidth={1.4} />
                        </div>
                        <div className="hostingCardBody">
                            <div className="hostingCardHeader">
                                <div>
                                    <span className="hostingCardPlan">{humanizeTier(subscription.tier_name)}</span>
                                    <h3 className="hostingCardTitulo">{subscription.requested_hostname || subscription.client_name}</h3>
                                </div>
                                <span className={`vpsSubscriptionStatus vpsSubscriptionStatus--${subscription.status}`}>
                                    {VPS_STATUS_LABELS[subscription.status] ?? subscription.status}
                                </span>
                            </div>

                            <div className="vpsSubscriptionMeta">
                                <div>
                                    <span>Precio</span>
                                    <strong>{formatPrice(subscription.monthly_price_cents)}</strong>
                                </div>
                                <div>
                                    <span>Cliente</span>
                                    <strong>{subscription.client_email}</strong>
                                </div>
                                <div>
                                    <span>IP</span>
                                    <strong>{subscription.provisioning_ip || 'Pendiente'}</strong>
                                </div>
                                <div>
                                    <span>Acceso</span>
                                    <strong>{subscription.access_username || 'Se entrega al aprobar'}</strong>
                                </div>
                            </div>

                            {subscription.client_notes && (
                                <div className="vpsSubscriptionNotes">
                                    <ShieldAlert size={16} />
                                    <p>{subscription.client_notes}</p>
                                </div>
                            )}

                            {subscription.contabo_instance_id && (
                                <div className="vpsSubscriptionInstance">
                                    <TerminalSquare size={16} />
                                    <span>Instancia Contabo #{subscription.contabo_instance_id}</span>
                                </div>
                            )}

                            {subscription.rejected_reason && (
                                <div className="vpsSubscriptionRejected">
                                    <strong>Motivo del rechazo:</strong> {subscription.rejected_reason}
                                </div>
                            )}

                            {canReview && (
                                <div className="vpsSubscriptionActions">
                                    <Textarea
                                        className="vpsSubscriptionTextarea"
                                        value={rejectReason}
                                        onChange={event => setRejectReasons(current => ({
                                            ...current,
                                            [subscription.id]: event.target.value,
                                        }))}
                                        placeholder="Motivo si decides rechazar la provisión"
                                        rows={3}
                                    />
                                    <div className="vpsSubscriptionButtons">
                                        <Button
                                            variante="primario"
                                            onClick={() => onApprove(subscription.id)}
                                            disabled={approveLoading || rejectLoading}
                                        >
                                            {approveLoading ? 'Aprobando…' : 'Aprobar'}
                                        </Button>
                                        <Button
                                            variante="outline"
                                            onClick={() => onReject(subscription.id, rejectReason.trim())}
                                            disabled={approveLoading || rejectLoading || rejectReason.trim().length < 3}
                                        >
                                            {rejectLoading ? 'Rechazando…' : 'Rechazar'}
                                        </Button>
                                    </div>
                                </div>
                            )}
                        </div>
                    </article>
                );
            })}
        </div>
    );
}