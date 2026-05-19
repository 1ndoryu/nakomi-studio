/* [195A-1] VpsCard — tarjeta de suscripción VPS en la tab "Mis VPS" del panel.
 * Archivo propio para cumplir límite de líneas de HostingSubComponents.
 * Reutiliza clases .hostingCard del sistema. Admin puede aprobar/rechazar pending_approval. */

import {useState} from 'react';
import {Server} from 'lucide-react';
import {VPS_STATUS_LABELS, type VpsSubscription} from '../../api/hosting';
import {Modal, ModalBody} from '../ui/Modal';
import {Input} from '../ui/Input';
import {Button} from '../ui/Button';
import './SeccionHosting.css';

export function VpsCard({
    sub,
    isAdmin,
    onApprove,
    onReject,
    approveLoading,
}: {
    sub: VpsSubscription;
    isAdmin: boolean;
    onApprove?: () => void;
    onReject?: (reason: string) => void;
    approveLoading?: boolean;
}) {
    const [showRejectModal, setShowRejectModal] = useState(false);
    const [rejectReason, setRejectReason] = useState('');

    const statusLabel = VPS_STATUS_LABELS[sub.status] || sub.status;
    const titulo = sub.requested_hostname || sub.tier_name;

    return (
        <>
            <div className="hostingCard hostingCardEstatica">
                <div className="panelCardIcono">
                    <Server size={28} strokeWidth={1.4} />
                </div>
                <div className="hostingCardBody">
                    <div className="hostingCardHeader">
                        <h3 className="hostingCardTitulo">{titulo}</h3>
                        <span className="hostingStatus">{statusLabel}</span>
                    </div>
                    <span className="hostingCardPlan">{sub.tier_name}</span>
                    {isAdmin && (
                        <span className="hostingCardCliente">
                            {sub.client_name} · {sub.client_email}
                        </span>
                    )}
                    <div className="hostingCardFooter">
                        {sub.provisioning_ip && (
                            <span className="hostingCardRecurso">{sub.provisioning_ip}</span>
                        )}
                        {sub.access_username && (
                            <span className="hostingCardRecurso">usuario: {sub.access_username}</span>
                        )}
                        {isAdmin && sub.status === 'pending_approval' && (
                            <div className="hostingCardAcciones">
                                <Button
                                    type="button"
                                    variante="primario"
                                    tamano="pequeno"
                                    onClick={onApprove}
                                    disabled={approveLoading}
                                >
                                    {approveLoading ? 'Aprobando…' : 'Aprobar'}
                                </Button>
                                <Button
                                    type="button"
                                    variante="outline"
                                    tamano="pequeno"
                                    onClick={() => setShowRejectModal(true)}
                                >
                                    Rechazar
                                </Button>
                            </div>
                        )}
                    </div>
                </div>
            </div>

            {showRejectModal && (
                <Modal abierto={showRejectModal} onCerrar={() => setShowRejectModal(false)}>
                    <ModalBody
                        as="form"
                        className="hostingCrearContenido"
                        onSubmit={(e: React.FormEvent) => {
                            e.preventDefault();
                            onReject?.(rejectReason);
                            setShowRejectModal(false);
                        }}
                    >
                        <p className="hostingStatusModalSub">
                            Rechazar solicitud VPS de: <strong>{sub.client_name}</strong>
                        </p>
                        <Input
                            type="text"
                            placeholder="Motivo de rechazo"
                            value={rejectReason}
                            onChange={e => setRejectReason(e.target.value)}
                        />
                        <div className="modalAcciones">
                            <Button type="submit" tamano="pequeno" disabled={!rejectReason.trim()}>
                                Confirmar rechazo
                            </Button>
                        </div>
                    </ModalBody>
                </Modal>
            )}
        </>
    );
}
