/* [094A-2] Contenido de las tabs del detalle de hosting.
 * Extraído de HostingDetalle.tsx para cumplir límite de 300 líneas.
 * Tabs: General, Recursos, Eventos (aquí). Dominio & SSL, Acceso SSH → HostingDetalleAccess.tsx.
 * [094A-4] TabDominio mejorada: instrucciones DNS con registros A/CNAME.
 * [094A-5] TabAcceso mejorada: info SSH incluso sin coolify_site_name.
 * [094A-6] TabFacturacion extraída a TabFacturacion.tsx por límite de líneas.
 * [114A-5] TabDominio + TabAcceso extraídas a HostingDetalleAccess.tsx por límite. */

import {Server, Loader, ExternalLink} from 'lucide-react';
import type {useHostingDetalle} from '../../hooks/useHostingDetalle';
import {
    HOSTING_PLAN_LABELS,
    HOSTING_STATUS_LABELS,
} from '../../api/hosting';
import {Button} from '../ui/Button';
import {HostingStats} from './HostingStats';
import {EventsPanel} from './HostingSubComponents';
import {InfoRow} from './HostingDetalle';

type Subscription = NonNullable<ReturnType<typeof useHostingDetalle>['subscription']>;

/* ── Tab: General ────────────────────────── */
export function TabGeneral({sub, isAdmin, onProvision, provisionLoading}: {
    sub: Subscription;
    isAdmin: boolean;
    onProvision?: () => void;
    provisionLoading?: boolean;
}) {
    const sitioUrl = sub.domain ? `https://${sub.domain}` : null;
    /* URL real del WordPress: solo para hostings provisionados por nuestro sistema (coolify_site_name empieza con 'hosting-').
     * Los fixtures demo tienen server_ip pero no son WordPress reales. */
    const isRealProvisioned = sub.coolify_site_name?.startsWith('hosting-') && sub.server_uuid && sub.server_ip;
    const coolifyUrl = isRealProvisioned
        ? `http://wordpress-${sub.server_uuid}.${sub.server_ip}.sslip.io`
        : null;
    /* [154A-11] Provisioning disponible para admin cuando el hosting está pendiente */
    const canProvision = isAdmin && (sub.status === 'pending' || sub.status === 'provisioning') && onProvision;

    return (
        <div className="hostingDetalleSection">
            <h3 className="hostingDetalleSectionTitle">Información general</h3>
            <div className="hostingDetalleInfoGrid">
                <InfoRow label="Plan" value={HOSTING_PLAN_LABELS[sub.plan] || sub.plan} />
                <InfoRow label="Estado" value={HOSTING_STATUS_LABELS[sub.status] || sub.status} />
                <InfoRow label="Precio" value={`$${(sub.monthly_price_cents / 100).toFixed(0)}/mes`} />
                <InfoRow label="Almacenamiento" value={`${(sub.storage_limit_mb / 1024).toFixed(0)} GB`} />
                {sub.domain && (
                    <InfoRow label="Dominio" value={sub.domain} copyable link={sitioUrl ?? undefined} />
                )}
                {sub.server_ip && (
                    <InfoRow label="IP servidor" value={sub.server_ip} copyable />
                )}
                {coolifyUrl && (
                    <InfoRow label="URL del sitio" value={coolifyUrl} copyable link={coolifyUrl} />
                )}
                {sub.coolify_site_name && (
                    <InfoRow label="Servicio" value={sub.coolify_site_name} copyable />
                )}
                <InfoRow
                    label="Creado"
                    value={new Date(sub.created_at).toLocaleDateString('es', {
                        year: 'numeric', month: 'long', day: 'numeric',
                    })}
                />
                {isAdmin && (
                    <>
                        <InfoRow label="Cliente" value={sub.client_name} />
                        <InfoRow label="Email" value={sub.client_email} copyable />
                    </>
                )}
            </div>
            <div className="hostingDetalleAcciones">
                {canProvision && (
                    <Button
                        type="button"
                        variante="primario"
                        tamano="pequeno"
                        onClick={onProvision}
                        disabled={provisionLoading}
                    >
                        {provisionLoading ? (
                            <><Loader size={14} className="hostingSpinner" /> Provisionando…</>
                        ) : (
                            <><Server size={14} /> Provisionar WordPress</>
                        )}
                    </Button>
                )}
                {coolifyUrl && (
                    <a href={coolifyUrl} target="_blank" rel="noopener noreferrer" className="hostingDetalleAccionLink">
                        <Button type="button" variante="primario" tamano="pequeno">
                            <ExternalLink size={14} /> Abrir WordPress
                        </Button>
                    </a>
                )}
                {sitioUrl && !coolifyUrl && (
                    <a href={sitioUrl} target="_blank" rel="noopener noreferrer" className="hostingDetalleAccionLink">
                        <Button type="button" variante="outline" tamano="pequeno">
                            <ExternalLink size={14} /> Visitar sitio
                        </Button>
                    </a>
                )}
            </div>
        </div>
    );
}

/* ── Tab: Recursos ───────────────────────── */
export function TabRecursos({sub}: {sub: Subscription}) {
    /* Hostings provisionados en Docker (Coolify) no exponen métricas de uso todavía.
     * Solo mostramos el límite del plan; uso real requiere integración con Coolify metrics API. */
    const isDockerHosting = sub.coolify_site_name?.startsWith('hosting-');
    return (
        <div className="hostingDetalleSection">
            <h3 className="hostingDetalleSectionTitle">Uso de recursos</h3>
            {(sub.status === 'active' || sub.status === 'provisioning') ? (
                <div className="hostingDetalleRecursos">
                    {isDockerHosting ? (
                        <div className="hostingDetalleInfoGrid">
                            <InfoRow label="Almacenamiento incluido" value={`${(sub.storage_limit_mb / 1024).toFixed(0)} GB`} />
                            <InfoRow label="Monitoreo en tiempo real" value="Próximamente" />
                        </div>
                    ) : (
                        <HostingStats sub={sub} />
                    )}
                    <p className="hostingDetalleRecursosNota">
                        Los datos de uso se actualizan periódicamente. Los valores mostrados son aproximados.
                    </p>
                </div>
            ) : (
                <div className="hostingDetalleRecursosInactivo">
                    <Server size={32} strokeWidth={1.2} />
                    <p>El hosting debe estar activo para ver las estadísticas de uso.</p>
                </div>
            )}
        </div>
    );
}

/* ── Re-exports de tabs extraídas ─────── */
export {TabDominio, TabAcceso} from './HostingDetalleAccess';

/* ── Tab: Eventos ────────────────────────── */
export function TabEventos({hostingId, clientName}: {
    hostingId: string;
    clientName: string;
}) {
    return (
        <div className="hostingDetalleSection">
            <EventsPanel subscriptionId={hostingId} clientName={clientName} />
        </div>
    );
}
