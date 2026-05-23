import {useState} from 'react';
import {Globe, MoreVertical, PlusCircle, Server, Trash2} from 'lucide-react';
import {useMutation, useQueryClient} from '@tanstack/react-query';
import type {CoolifyDeployment} from '../../api/hosting';
import {HOSTING_PLAN_LABELS, apiCreateHostingSubscription, apiDeleteDeployment} from '../../api/hosting';
import {DEPLOYMENTS_QUERY_KEY} from '../../hooks/useDeploymentsPanel';
import {toast} from '../../stores/toastStore';
import {Button} from '../ui/Button';
import {MenuContextual, type MenuContextualItem} from '../ui/ContextMenu';
import {Modal} from '../ui/Modal';
import {CreateHostingForm} from './HostingCreateForm';

export function getDeploymentPanelErrorMessage(error: unknown): string {
    const apiMessage = (error as {response?: {data?: {message?: string}}})?.response?.data?.message;
    if (typeof apiMessage === 'string' && apiMessage.trim()) return apiMessage;
    if (error instanceof Error && error.message) return error.message;
    return 'No se pudo consultar Coolify para listar los despliegues reales';
}

function formatStatus(status: string): string {
    const [main, sub] = status.split(':');
    if (!sub || sub.toLowerCase() === 'unknown') return main;
    return `${main} · ${sub}`;
}

function getDeploymentStatusClass(status: string): string {
    const normalizedStatus = status.toLowerCase();
    if (['running', 'healthy', 'active', 'ready', 'success'].some(token => normalizedStatus.includes(token))) return 'vpsStatus--running';
    if (['stopped', 'exited', 'failed', 'error', 'crashed', 'unhealthy'].some(token => normalizedStatus.includes(token))) return 'vpsStatus--stopped';
    return 'vpsStatus--other';
}

function formatMb(mb: number | null | undefined): string {
    if (mb == null) return '—';
    if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`;
    return `${Math.round(mb)} MB`;
}

function formatCpu(percent: number | null | undefined): string {
    return percent == null ? '—' : `${percent.toFixed(2)}%`;
}

function WordPressIcon() {
    return <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10" /><path d="M2 12h4l3 8 4-16 3 8h4" /></svg>;
}

function HostingIcon() {
    return <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"><rect x="2" y="3" width="20" height="8" rx="2" /><rect x="2" y="13" width="20" height="8" rx="2" /><circle cx="7" cy="7" r="1" /><circle cx="7" cy="17" r="1" /></svg>;
}

function isWordPressDeployment(plan: string | null): boolean {
    return Boolean(plan && !plan.startsWith('normal-'));
}

function getDeploymentTypeLabel(plan: string | null): string {
    if (!plan) return 'Sin plan';
    return HOSTING_PLAN_LABELS[plan] || plan;
}

interface DeploymentContextMenuProps {
    deployment: CoolifyDeployment;
    onCreateSubscription: () => void;
    onDeleteDeployment: () => void;
    isDeleting: boolean;
}

function DeploymentContextMenu({deployment, onCreateSubscription, onDeleteDeployment, isDeleting}: DeploymentContextMenuProps) {
    const [abierto, setAbierto] = useState(false);
    if (deployment.linked_subscription_id) return null;

    const items: MenuContextualItem[] = [
        {id: 'create-subscription', label: 'Crear suscripción vinculada', icon: <PlusCircle size={14} />, onSelect: onCreateSubscription},
        {id: 'delete-deployment', label: 'Eliminar despliegue', icon: <Trash2 size={14} />, danger: true, disabled: isDeleting, onSelect: onDeleteDeployment},
    ];

    return (
        <MenuContextual
            abierto={abierto}
            onToggle={() => setAbierto(value => !value)}
            onCerrar={() => setAbierto(false)}
            items={items}
            ariaLabel="Acciones del despliegue"
            triggerContent={<MoreVertical size={16} />}
        />
    );
}

export function DeploymentRow({deployment}: {deployment: CoolifyDeployment}) {
    const [expandido, setExpandido] = useState(false);
    const [showCreateForm, setShowCreateForm] = useState(false);
    const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
    const queryClient = useQueryClient();
    const isLinked = Boolean(deployment.linked_subscription_id);
    const fqdn = deployment.fqdn?.trim() || null;
    const isWp = isWordPressDeployment(deployment.linked_subscription_plan);

    const createMutation = useMutation({
        mutationFn: apiCreateHostingSubscription,
        onSuccess: () => {
            toast.success('Suscripción creada y vinculada al despliegue');
            void queryClient.invalidateQueries({queryKey: ['hosting-subscriptions']});
            void queryClient.invalidateQueries({queryKey: DEPLOYMENTS_QUERY_KEY});
            setShowCreateForm(false);
        },
        onError: error => toast.error(getDeploymentPanelErrorMessage(error)),
    });

    const deleteMutation = useMutation({
        mutationFn: (uuid: string) => apiDeleteDeployment(uuid),
        onSuccess: () => {
            toast.success('Despliegue eliminado de Coolify');
            void queryClient.invalidateQueries({queryKey: DEPLOYMENTS_QUERY_KEY});
            setShowDeleteConfirm(false);
        },
        onError: error => toast.error(getDeploymentPanelErrorMessage(error)),
    });

    const closeCreateForm = () => {
        if (!createMutation.isPending) setShowCreateForm(false);
    };

    const closeDeleteConfirm = () => {
        if (!deleteMutation.isPending) setShowDeleteConfirm(false);
    };

    return (
        <>
            <tr className={`infraFila ${expandido ? 'infraFila--expandida' : ''} ${!isLinked ? 'infraFila--huerfana' : ''}`} onClick={() => setExpandido(!expandido)}>
                <td className="infraCelda infraCelda--tipo"><span className="infraTipoIcono" title={isWp ? 'WordPress' : 'Hosting'}>{deployment.linked_subscription_plan ? (isWp ? <WordPressIcon /> : <HostingIcon />) : <Server size={14} />}</span></td>
                <td className="infraCelda"><div className="infraCeldaNombre"><span className="infraNombreTexto">{deployment.name}</span><span className="infraServerBadge">{deployment.server_label}</span></div></td>
                <td className="infraCelda"><span className={`vpsStatus ${getDeploymentStatusClass(deployment.status)}`}>{formatStatus(deployment.status)}</span></td>
                <td className="infraCelda infraCelda--plan">{getDeploymentTypeLabel(deployment.linked_subscription_plan)}</td>
                <td className="infraCelda infraCelda--usuario">{deployment.linked_subscription_client || '—'}</td>
                <td className="infraCelda infraCelda--recurso">{formatCpu(deployment.cpu_percent)}</td>
                <td className="infraCelda infraCelda--recurso">{formatMb(deployment.ram_used_mb)}</td>
                <td className="infraCelda infraCelda--recurso">{formatMb(deployment.storage_used_mb)}</td>
                <td className="infraCelda infraCelda--acciones" onClick={event => event.stopPropagation()}><DeploymentContextMenu deployment={deployment} onCreateSubscription={() => setShowCreateForm(true)} onDeleteDeployment={() => setShowDeleteConfirm(true)} isDeleting={deleteMutation.isPending} /></td>
            </tr>

            {expandido && <DeploymentDetails deployment={deployment} fqdn={fqdn} isLinked={isLinked} />}

            <Modal abierto={showCreateForm} onCerrar={closeCreateForm}>
                <CreateHostingForm initialCoolifyName={deployment.name} submitting={createMutation.isPending} onSubmit={req => createMutation.mutate(req)} />
            </Modal>

            <Modal abierto={showDeleteConfirm} onCerrar={closeDeleteConfirm}>
                <p className="modalTexto">Eliminar despliegue huérfano: se eliminará {deployment.name} de Coolify junto con sus volúmenes y red del stack.</p>
                <p className="modalTexto">Úsalo solo cuando confirmes que este UUID no corresponde a ninguna suscripción del panel.</p>
                <p className="modalTexto vpsDeleteConfirmMeta">UUID: {deployment.uuid}</p>
                <div className="modalAcciones">
                    <Button variante="secundario" tamano="pequeno" onClick={closeDeleteConfirm} disabled={deleteMutation.isPending} type="button">Cancelar</Button>
                    <Button variante="primario" tamano="pequeno" onClick={() => deleteMutation.mutate(deployment.uuid)} disabled={deleteMutation.isPending} type="button">{deleteMutation.isPending ? 'Eliminando...' : 'Eliminar de Coolify'}</Button>
                </div>
            </Modal>
        </>
    );
}

function DeploymentDetails({deployment, fqdn, isLinked}: {deployment: CoolifyDeployment; fqdn: string | null; isLinked: boolean}) {
    return (
        <tr className="infraFilaDetalle">
            <td colSpan={9}>
                <div className="infraDetalleContenido">
                    <div className="infraDetalleGrid">
                        <div className="infraDetalleCampo"><span className="infraDetalleLabel">UUID</span><span className="infraDetalleValor">{deployment.uuid}</span></div>
                        <div className="infraDetalleCampo"><span className="infraDetalleLabel">Entorno</span><span className="infraDetalleValor">{deployment.environment_name || 'production'}</span></div>
                        <div className="infraDetalleCampo"><span className="infraDetalleLabel">Servidor</span><span className="infraDetalleValor">{deployment.server_name || deployment.server_label}</span></div>
                        {deployment.ram_limit_mb != null && <div className="infraDetalleCampo"><span className="infraDetalleLabel">RAM límite</span><span className="infraDetalleValor">{formatMb(deployment.ram_limit_mb)}</span></div>}
                        {deployment.storage_limit_mb != null && <div className="infraDetalleCampo"><span className="infraDetalleLabel">Disco límite</span><span className="infraDetalleValor">{formatMb(deployment.storage_limit_mb)}</span></div>}
                        {fqdn && <div className="infraDetalleCampo"><span className="infraDetalleLabel">FQDN</span><a href={fqdn} target="_blank" rel="noopener noreferrer" className="infraDetalleLink" onClick={event => event.stopPropagation()}><Globe size={12} />{fqdn}</a></div>}
                    </div>
                    {!isLinked && <div className="infraDetalleAlerta">Despliegue sin suscripción vinculada en el panel.</div>}
                </div>
            </td>
        </tr>
    );
}