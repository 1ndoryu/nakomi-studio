/* [164A-19] Panel admin de despliegues reales en VPS2.
 * Muestra servicios de Coolify y si cada despliegue está o no vinculado a una
 * suscripción del panel para evitar otra falsa equivalencia entre VPS y deployment. */

import React, {useState} from 'react';
import {Activity, ExternalLink, Globe, Link2, PlusCircle, Server, ShieldAlert} from 'lucide-react';
import {useMutation, useQueryClient} from '@tanstack/react-query';
import type {CoolifyDeployment} from '../../api/hosting';
import {apiCreateHostingSubscription} from '../../api/hosting';
import {useVps2DeploymentsPanel} from '../../hooks/useVps2DeploymentsPanel';
import {CreateHostingForm} from './HostingSubComponents';
import {Modal} from '../ui/Modal';
import {toast} from '../../stores/toastStore';
import './VpsPanel.css';

function getPanelErrorMessage(error: unknown): string {
    const apiMessage = (error as {
        response?: {data?: {message?: string}};
    })?.response?.data?.message;

    if (typeof apiMessage === 'string' && apiMessage.trim()) {
        return apiMessage;
    }

    if (error instanceof Error && error.message) {
        return error.message;
    }

    return 'No se pudo consultar Coolify para listar los despliegues reales de la VPS2';
}

/* Formatea el status de Coolify para display: "Running:Unknown" → "Running · Unknown" */
function formatStatus(status: string): string {
    return status.replace(':', ' · ');
}

function getDeploymentStatusClass(status: string): string {
    const normalizedStatus = status.toLowerCase();

    if (
        ['running', 'healthy', 'active', 'ready', 'success'].some(token => normalizedStatus.includes(token))
    ) {
        return 'vpsStatus--running';
    }

    if (
        ['stopped', 'exited', 'failed', 'error', 'crashed', 'unhealthy'].some(token => normalizedStatus.includes(token))
    ) {
        return 'vpsStatus--stopped';
    }

    return 'vpsStatus--other';
}

function resolveSubscriptionLabel(deployment: CoolifyDeployment): string {
    return deployment.linked_subscription_domain
        || deployment.linked_subscription_plan
        || 'Vinculada sin dominio visible';
}

function DeploymentCard({deployment}: {deployment: CoolifyDeployment}) {
    const isLinked = Boolean(deployment.linked_subscription_id);
    const fqdn = deployment.fqdn?.trim() || null;
    /* [304A-3] Crea suscripción directamente desde el card sin depender del hook complejo */
    const [showCreateForm, setShowCreateForm] = useState(false);
    const queryClient = useQueryClient();
    const createMutation = useMutation({
        mutationFn: apiCreateHostingSubscription,
        onSuccess: () => {
            toast.success('Suscripción creada y vinculada al despliegue');
            void queryClient.invalidateQueries({queryKey: ['hosting-subscriptions']});
            void queryClient.invalidateQueries({queryKey: ['vps2-deployments']});
            setShowCreateForm(false);
        },
        onError: () => toast.error('Error al crear la suscripción'),
    });

    return (
        <article className="vpsCard">
            <div className="vpsCardHeader">
                <Server size={20} strokeWidth={1.4} />
                <h4 className="vpsCardNombre">{deployment.name}</h4>
                <span className={`vpsStatus ${getDeploymentStatusClass(deployment.status)}`}>
                    {formatStatus(deployment.status)}
                </span>
            </div>

            <div className="vpsCardStats">
                <div className="vpsStat">
                    <Link2 size={14} />
                    <span className="vpsStatLabel">UUID</span>
                    <span className="vpsStatValor">{deployment.uuid}</span>
                </div>

                <div className="vpsStat">
                    <Activity size={14} />
                    <span className="vpsStatLabel">Entorno</span>
                    <span className="vpsStatValor">{deployment.environment_name || 'production'}</span>
                </div>

                <div className="vpsStat">
                    <Server size={14} />
                    <span className="vpsStatLabel">Servidor</span>
                    <span className="vpsStatValor">{deployment.server_name || deployment.server_uuid || 'VPS2'}</span>
                </div>

                <div className="vpsStat">
                    <ShieldAlert size={14} />
                    <span className="vpsStatLabel">Panel</span>
                    <span className="vpsStatValor">
                        {isLinked ? resolveSubscriptionLabel(deployment) : 'Sin suscripción vinculada'}
                    </span>
                </div>

                <div className="vpsStat">
                    <Activity size={14} />
                    <span className="vpsStatLabel">Estado</span>
                    <span className="vpsStatValor">
                        {deployment.linked_subscription_status || 'Solo visible en Coolify'}
                    </span>
                </div>

                {fqdn && (
                    <div className="vpsStat">
                        <Globe size={14} />
                        <span className="vpsStatLabel">FQDN</span>
                        <a
                            href={fqdn}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="vpsStatLink"
                        >
                            {fqdn}
                            <ExternalLink size={12} strokeWidth={1.8} />
                        </a>
                    </div>
                )}
            </div>

            <div className={`vpsDeploymentAudit ${isLinked ? 'vpsDeploymentAudit--linked' : 'vpsDeploymentAudit--orphan'}`}>
                <ShieldAlert size={16} />
                <p>
                    {isLinked
                        ? 'Despliegue real detectado y vinculado a una suscripción del panel.'
                        : 'Despliegue real detectado en Coolify sin vínculo con una suscripción del panel.'}
                </p>
                {/* [304A-3] Botón para vincular despliegue huérfano creando una suscripción */}
                {!isLinked && (
                    <button
                        className="vpsOrphanLinkBtn"
                        onClick={() => setShowCreateForm(true)}
                        type="button"
                    >
                        <PlusCircle size={14} />
                        Crear suscripción vinculada
                    </button>
                )}
            </div>

            {/* [304A-3] Modal para crear suscripción desde despliegue huérfano */}
            <Modal abierto={showCreateForm} onCerrar={() => setShowCreateForm(false)}>
                <CreateHostingForm
                    initialCoolifyName={deployment.name}
                    submitting={createMutation.isPending}
                    onSubmit={req => createMutation.mutate(req)}
                />
            </Modal>
        </article>
    );
}

export const Vps2DeploymentsPanel: React.FC = () => {
    const {deployments, isLoading, error} = useVps2DeploymentsPanel();
    const linkedDeployments = deployments.filter(deployment => Boolean(deployment.linked_subscription_id));
    const orphanDeployments = deployments.length - linkedDeployments.length;

    if (isLoading) {
        return (
            <div className="vpsLoading">
                <Server size={28} strokeWidth={1.2} />
                <p>Consultando despliegues reales de la VPS2...</p>
            </div>
        );
    }

    if (error) {
        return (
            <div className="vpsError">
                <p>{getPanelErrorMessage(error)}</p>
            </div>
        );
    }

    if (deployments.length === 0) {
        return (
            <div className="vpsVacio">
                <Server size={36} strokeWidth={1.2} />
                <p>No se encontraron despliegues reales en la VPS2</p>
            </div>
        );
    }

    return (
        <div className="vpsContenedor">
            <div className="vpsDeploymentsSummary">
                <div className="vpsDeploymentsSummaryCard">
                    <span className="vpsDeploymentsSummaryLabel">Coolify</span>
                    <span className="vpsDeploymentsSummaryValue">{deployments.length} despliegues reales</span>
                </div>
                <div className="vpsDeploymentsSummaryCard">
                    <span className="vpsDeploymentsSummaryLabel">Vinculados</span>
                    <span className="vpsDeploymentsSummaryValue">{linkedDeployments.length} en panel</span>
                </div>
                <div className="vpsDeploymentsSummaryCard">
                    <span className="vpsDeploymentsSummaryLabel">Huérfanos</span>
                    <span className="vpsDeploymentsSummaryValue">{orphanDeployments} sin vínculo</span>
                </div>
            </div>

            <div className="vpsLista">
                {deployments.map(deployment => (
                    <DeploymentCard key={deployment.uuid} deployment={deployment} />
                ))}
            </div>
        </div>
    );
};