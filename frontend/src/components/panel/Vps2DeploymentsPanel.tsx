/* [164A-19] Panel admin de despliegues reales en VPS2.
 * Muestra servicios de Coolify y si cada despliegue está o no vinculado a una
 * suscripción del panel para evitar otra falsa equivalencia entre VPS y deployment. */

import React, {useState} from 'react';
import {Activity, ExternalLink, Globe, Link2, PlusCircle, Server, ShieldAlert, Trash2} from 'lucide-react';
import {useMutation, useQueryClient} from '@tanstack/react-query';
import type {CoolifyDeployment} from '../../api/hosting';
import {apiCreateHostingSubscription, apiDeleteVps2Deployment} from '../../api/hosting';
import {useVps2DeploymentsPanel} from '../../hooks/useVps2DeploymentsPanel';
import {CreateHostingForm} from './HostingCreateForm';
import {Modal} from '../ui/Modal';
import {Button} from '../ui/Button';
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

    return 'No se pudo consultar Coolify para listar los despliegues reales';
}

/* Formatea el status de Coolify para display: "Running:Unknown" → "Running", "Running:Healthy" → "Running · Healthy" */
function formatStatus(status: string): string {
    const [main, sub] = status.split(':');
    if (!sub || sub.toLowerCase() === 'unknown') return main;
    return `${main} · ${sub}`;
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
    /* [165A-4] Crea y limpia huérfanos directamente desde el card para evitar drift manual. */
    const [showCreateForm, setShowCreateForm] = useState(false);
    const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
    const queryClient = useQueryClient();
    const createMutation = useMutation({
        mutationFn: apiCreateHostingSubscription,
        onSuccess: () => {
            toast.success('Suscripción creada y vinculada al despliegue');
            void queryClient.invalidateQueries({queryKey: ['hosting-subscriptions']});
            void queryClient.invalidateQueries({queryKey: ['vps2-deployments']});
            setShowCreateForm(false);
        },
        onError: error => toast.error(getPanelErrorMessage(error)),
    });
    const deleteMutation = useMutation({
        mutationFn: (uuid: string) => apiDeleteVps2Deployment(uuid),
        onSuccess: () => {
            toast.success('Despliegue eliminado de Coolify');
            void queryClient.invalidateQueries({queryKey: ['vps2-deployments']});
            setShowDeleteConfirm(false);
        },
        onError: error => toast.error(getPanelErrorMessage(error)),
    });

    const closeCreateForm = () => {
        if (!createMutation.isPending) {
            setShowCreateForm(false);
        }
    };

    const closeDeleteConfirm = () => {
        if (!deleteMutation.isPending) {
            setShowDeleteConfirm(false);
        }
    };

    return (
        <article className="vpsCard">
            <div className="vpsCardHeader">
                <Server size={20} strokeWidth={1.4} />
                <h4 className="vpsCardNombre">{deployment.name}</h4>
                <span className="vpsServerLabel">{deployment.server_label}</span>
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
                    <span className="vpsStatValor">{deployment.server_name || deployment.server_label}</span>
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
                {!isLinked && (
                    <div className="vpsOrphanActions">
                        <Button
                            variante="secundario"
                            tamano="pequeno"
                            className="vpsOrphanLinkBtn"
                            onClick={() => setShowCreateForm(true)}
                            type="button"
                            disabled={deleteMutation.isPending}
                        >
                            <PlusCircle size={14} />
                            Crear suscripción vinculada
                        </Button>
                        <Button
                            variante="outline"
                            tamano="pequeno"
                            className="vpsOrphanDeleteBtn"
                            onClick={() => setShowDeleteConfirm(true)}
                            type="button"
                            disabled={createMutation.isPending}
                        >
                            <Trash2 size={14} />
                            Eliminar despliegue
                        </Button>
                    </div>
                )}
            </div>

            <Modal abierto={showCreateForm} onCerrar={closeCreateForm}>
                <CreateHostingForm
                    initialCoolifyName={deployment.name}
                    submitting={createMutation.isPending}
                    onSubmit={req => createMutation.mutate(req)}
                />
            </Modal>

            <Modal abierto={showDeleteConfirm} onCerrar={closeDeleteConfirm}>
                <h3 className="modalTitulo">Eliminar despliegue huérfano</h3>
                <p className="modalTexto">
                    Se eliminará {deployment.name} de Coolify junto con sus volúmenes y red del stack.
                    Úsalo solo cuando confirmes que este UUID no corresponde a ninguna suscripción del panel.
                </p>
                <p className="modalTexto vpsDeleteConfirmMeta">UUID: {deployment.uuid}</p>
                <div className="modalAcciones">
                    <Button
                        variante="secundario"
                        tamano="pequeno"
                        onClick={closeDeleteConfirm}
                        disabled={deleteMutation.isPending}
                        type="button"
                    >
                        Cancelar
                    </Button>
                    <Button
                        variante="primario"
                        tamano="pequeno"
                        onClick={() => deleteMutation.mutate(deployment.uuid)}
                        disabled={deleteMutation.isPending}
                        type="button"
                    >
                        {deleteMutation.isPending ? 'Eliminando...' : 'Eliminar de Coolify'}
                    </Button>
                </div>
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
                <p>Consultando despliegues reales de todas las VPS...</p>
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
                <p>No se encontraron despliegues reales en ninguna VPS configurada</p>
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