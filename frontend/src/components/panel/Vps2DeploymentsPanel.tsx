/* [215A-14] Panel admin de despliegues reales — vista tabla minimalista.
 * Reemplaza las tarjetas por una tabla compacta con recursos por despliegue,
 * resumen del VPS (CPU, RAM, disco, conteos WP vs Normal), íconos de tipo,
 * columna de usuario dueño y menú contextual de 3 puntos para acciones. */

import React, {useState, useRef, useEffect} from 'react';
import {Globe, MoreVertical, PlusCircle, Server, Trash2} from 'lucide-react';
import {useMutation, useQueryClient} from '@tanstack/react-query';
import type {CoolifyDeployment, VpsSummary} from '../../api/hosting';
import {HOSTING_PLAN_LABELS, apiCreateHostingSubscription, apiDeleteVps2Deployment} from '../../api/hosting';
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

/* Formatea MB a display legible */
function formatMb(mb: number | null | undefined): string {
    if (mb == null) return '—';
    if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`;
    return `${Math.round(mb)} MB`;
}

function formatCpu(percent: number | null | undefined): string {
    if (percent == null) return '—';
    return `${percent.toFixed(2)}%`;
}

/* Ícono SVG inline para WordPress */
function WordPressIcon() {
    return (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="12" r="10" />
            <path d="M2 12h4l3 8 4-16 3 8h4" />
        </svg>
    );
}

/* Ícono para hosting normal (server simple) */
function HostingIcon() {
    return (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
            <rect x="2" y="3" width="20" height="8" rx="2" />
            <rect x="2" y="13" width="20" height="8" rx="2" />
            <circle cx="7" cy="7" r="1" />
            <circle cx="7" cy="17" r="1" />
        </svg>
    );
}

/* Identifica si el plan es WordPress o hosting normal */
function isWordPressDeployment(plan: string | null): boolean {
    if (!plan) return false;
    return !plan.startsWith('normal-');
}

function getDeploymentTypeLabel(plan: string | null): string {
    if (!plan) return 'Sin plan';
    return HOSTING_PLAN_LABELS[plan] || plan;
}

/* ─── Menú contextual de 3 puntos ─── */

interface ContextMenuProps {
    deployment: CoolifyDeployment;
    onCreateSubscription: () => void;
    onDeleteDeployment: () => void;
    isDeleting: boolean;
}

function ContextMenu({deployment, onCreateSubscription, onDeleteDeployment, isDeleting}: ContextMenuProps) {
    const [abierto, setAbierto] = useState(false);
    const menuRef = useRef<HTMLDivElement>(null);
    const isLinked = Boolean(deployment.linked_subscription_id);

    useEffect(() => {
        function handleClickOutside(e: MouseEvent) {
            if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
                setAbierto(false);
            }
        }

        if (abierto) {
            document.addEventListener('mousedown', handleClickOutside);
        }

        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, [abierto]);

    /* Solo despliegues huérfanos tienen acciones */
    if (isLinked) return null;

    return (
        <div className="infraMenuContenedor" ref={menuRef}>
            <button
                type="button"
                className="infraMenuBoton"
                onClick={() => setAbierto(!abierto)}
                aria-label="Acciones del despliegue"
            >
                <MoreVertical size={16} />
            </button>
            {abierto && (
                <div className="infraMenuDropdown">
                    <button
                        type="button"
                        className="infraMenuItem"
                        onClick={() => {
                            setAbierto(false);
                            onCreateSubscription();
                        }}
                    >
                        <PlusCircle size={14} />
                        Crear suscripción vinculada
                    </button>
                    <button
                        type="button"
                        className="infraMenuItem infraMenuItem--peligro"
                        onClick={() => {
                            setAbierto(false);
                            onDeleteDeployment();
                        }}
                        disabled={isDeleting}
                    >
                        <Trash2 size={14} />
                        Eliminar despliegue
                    </button>
                </div>
            )}
        </div>
    );
}

/* ─── Resumen del VPS ─── */

interface VpsSummaryBarProps {
    vpsInstances: VpsSummary[];
    deployments: CoolifyDeployment[];
}

function VpsSummaryBar({vpsInstances, deployments}: VpsSummaryBarProps) {
    /* Conteos de tipo de hosting */
    const wpCount = deployments.filter(d =>
        d.linked_subscription_plan && isWordPressDeployment(d.linked_subscription_plan),
    ).length;
    const normalCount = deployments.filter(d =>
        d.linked_subscription_plan && !isWordPressDeployment(d.linked_subscription_plan),
    ).length;
    const orphanCount = deployments.filter(d => !d.linked_subscription_id).length;

    /* Usar VPS2 si existe, sino el primero disponible */
    const vps = vpsInstances.find(v => v.name.toLowerCase().includes('vps 2'))
        || vpsInstances.find(v => v.ip === '173.249.50.44')
        || vpsInstances[0];

    return (
        <div className="infraResumen">
            {vps && (
                <>
                    <div className="infraResumenItem">
                        <span className="infraResumenLabel">CPU</span>
                        <span className="infraResumenValor">{vps.cpu_cores} vCPU</span>
                    </div>
                    <div className="infraResumenItem">
                        <span className="infraResumenLabel">RAM</span>
                        <span className="infraResumenValor">{formatMb(vps.ram_mb)}</span>
                    </div>
                    <div className="infraResumenItem">
                        <span className="infraResumenLabel">Disco</span>
                        <span className="infraResumenValor">{formatMb(vps.disk_mb)}</span>
                    </div>
                </>
            )}
            <div className="infraResumenItem">
                <span className="infraResumenLabel">Despliegues</span>
                <span className="infraResumenValor">
                    {deployments.length} total
                </span>
            </div>
            <div className="infraResumenItem">
                <span className="infraResumenLabel">Tipo</span>
                <span className="infraResumenValor">
                    {wpCount > 0 && <>{wpCount} WP</>}
                    {wpCount > 0 && normalCount > 0 && ' · '}
                    {normalCount > 0 && <>{normalCount} Hosting</>}
                    {wpCount === 0 && normalCount === 0 && '—'}
                </span>
            </div>
            {orphanCount > 0 && (
                <div className="infraResumenItem infraResumenItem--warning">
                    <span className="infraResumenLabel">Huérfanos</span>
                    <span className="infraResumenValor">{orphanCount}</span>
                </div>
            )}
        </div>
    );
}

/* ─── Fila expandible de la tabla ─── */

interface DeploymentRowProps {
    deployment: CoolifyDeployment;
}

function DeploymentRow({deployment}: DeploymentRowProps) {
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
        <>
            <tr
                className={`infraFila ${expandido ? 'infraFila--expandida' : ''} ${!isLinked ? 'infraFila--huerfana' : ''}`}
                onClick={() => setExpandido(!expandido)}
            >
                <td className="infraCelda infraCelda--tipo">
                    <span className="infraTipoIcono" title={isWp ? 'WordPress' : 'Hosting'}>
                        {deployment.linked_subscription_plan
                            ? (isWp ? <WordPressIcon /> : <HostingIcon />)
                            : <Server size={14} />
                        }
                    </span>
                </td>
                <td className="infraCelda">
                    <div className="infraCeldaNombre">
                        <span className="infraNombreTexto">{deployment.name}</span>
                        <span className="infraServerBadge">{deployment.server_label}</span>
                    </div>
                </td>
                <td className="infraCelda">
                    <span className={`vpsStatus ${getDeploymentStatusClass(deployment.status)}`}>
                        {formatStatus(deployment.status)}
                    </span>
                </td>
                <td className="infraCelda infraCelda--plan">
                    {getDeploymentTypeLabel(deployment.linked_subscription_plan)}
                </td>
                <td className="infraCelda infraCelda--usuario">
                    {deployment.linked_subscription_client || '—'}
                </td>
                <td className="infraCelda infraCelda--recurso">{formatCpu(deployment.cpu_percent)}</td>
                <td className="infraCelda infraCelda--recurso">{formatMb(deployment.ram_used_mb)}</td>
                <td className="infraCelda infraCelda--recurso">{formatMb(deployment.storage_used_mb)}</td>
                <td className="infraCelda infraCelda--acciones" onClick={e => e.stopPropagation()}>
                    <ContextMenu
                        deployment={deployment}
                        onCreateSubscription={() => setShowCreateForm(true)}
                        onDeleteDeployment={() => setShowDeleteConfirm(true)}
                        isDeleting={deleteMutation.isPending}
                    />
                </td>
            </tr>

            {/* Fila expandida con detalles */}
            {expandido && (
                <tr className="infraFilaDetalle">
                    <td colSpan={9}>
                        <div className="infraDetalleContenido">
                            <div className="infraDetalleGrid">
                                <div className="infraDetalleCampo">
                                    <span className="infraDetalleLabel">UUID</span>
                                    <span className="infraDetalleValor">{deployment.uuid}</span>
                                </div>
                                <div className="infraDetalleCampo">
                                    <span className="infraDetalleLabel">Entorno</span>
                                    <span className="infraDetalleValor">{deployment.environment_name || 'production'}</span>
                                </div>
                                <div className="infraDetalleCampo">
                                    <span className="infraDetalleLabel">Servidor</span>
                                    <span className="infraDetalleValor">{deployment.server_name || deployment.server_label}</span>
                                </div>
                                {deployment.ram_limit_mb != null && (
                                    <div className="infraDetalleCampo">
                                        <span className="infraDetalleLabel">RAM límite</span>
                                        <span className="infraDetalleValor">{formatMb(deployment.ram_limit_mb)}</span>
                                    </div>
                                )}
                                {deployment.storage_limit_mb != null && (
                                    <div className="infraDetalleCampo">
                                        <span className="infraDetalleLabel">Disco límite</span>
                                        <span className="infraDetalleValor">{formatMb(deployment.storage_limit_mb)}</span>
                                    </div>
                                )}
                                {fqdn && (
                                    <div className="infraDetalleCampo">
                                        <span className="infraDetalleLabel">FQDN</span>
                                        <a
                                            href={fqdn}
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            className="infraDetalleLink"
                                            onClick={e => e.stopPropagation()}
                                        >
                                            <Globe size={12} />
                                            {fqdn}
                                        </a>
                                    </div>
                                )}
                            </div>
                            {!isLinked && (
                                <div className="infraDetalleAlerta">
                                    Despliegue sin suscripción vinculada en el panel.
                                </div>
                            )}
                        </div>
                    </td>
                </tr>
            )}

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
        </>
    );
}

/* ─── Panel principal ─── */

export const Vps2DeploymentsPanel: React.FC = () => {
    const {deployments, vpsInstances, isLoading, error} = useVps2DeploymentsPanel();

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
            <VpsSummaryBar vpsInstances={vpsInstances} deployments={deployments} />

            <div className="infraTablaWrapper">
                <table className="infraTabla">
                    <thead>
                        <tr>
                            <th className="infraEncabezado infraEncabezado--tipo" />
                            <th className="infraEncabezado">Nombre</th>
                            <th className="infraEncabezado">Estado</th>
                            <th className="infraEncabezado">Plan</th>
                            <th className="infraEncabezado">Usuario</th>
                            <th className="infraEncabezado infraEncabezado--recurso">CPU</th>
                            <th className="infraEncabezado infraEncabezado--recurso">RAM</th>
                            <th className="infraEncabezado infraEncabezado--recurso">Disco</th>
                            <th className="infraEncabezado infraEncabezado--acciones" />
                        </tr>
                    </thead>
                    <tbody>
                        {deployments.map(deployment => (
                            <DeploymentRow key={deployment.uuid} deployment={deployment} />
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
};