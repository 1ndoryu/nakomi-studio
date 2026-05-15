/* [094A-2] Contenido de las tabs del detalle de hosting.
 * Extraído de HostingDetalle.tsx para cumplir límite de 300 líneas.
 * Tabs: General, Recursos, Eventos (aquí). Dominio & SSL, Acceso SSH → HostingDetalleAccess.tsx.
 * [094A-4] TabDominio mejorada: instrucciones DNS con registros A/CNAME.
 * [094A-5] TabAcceso mejorada: info SSH incluso sin coolify_site_name.
 * [094A-6] TabFacturacion extraída a TabFacturacion.tsx por límite de líneas.
 * [114A-5] TabDominio + TabAcceso extraídas a HostingDetalleAccess.tsx por límite. */

import {Server, Loader, ExternalLink, RefreshCw, Square, Play} from 'lucide-react';
import type {useHostingDetalle} from '../../hooks/useHostingDetalle';
import {
    HOSTING_PLAN_LABELS,
    HOSTING_STATUS_LABELS,
} from '../../api/hosting';
import {Button} from '../ui/Button';
import {HostingStats} from './HostingStats';
import {EventsPanel} from './HostingEventsPanel';
import {InfoRow} from './HostingDetalle';

type Subscription = NonNullable<ReturnType<typeof useHostingDetalle>['subscription']>;

/* ── Tab: General ────────────────────────── */
export function TabGeneral({sub, isAdmin, onProvision, provisionLoading, onRestart, restartLoading, onStop, stopLoading, onStart, startLoading}: {
    sub: Subscription;
    isAdmin: boolean;
    onProvision?: () => void;
    provisionLoading?: boolean;
    onRestart?: () => void;
    restartLoading?: boolean;
    onStop?: () => void;
    stopLoading?: boolean;
    onStart?: () => void;
    startLoading?: boolean;
}) {
    const sitioUrl = sub.domain ? `https://${sub.domain}` : null;
    /* [155A-13] URL real del sitio: WordPress usa servicio `wordpress`, hosting normal usa `site`. */
    const isRealProvisioned = sub.coolify_site_name?.startsWith('hosting-') && sub.server_uuid && sub.server_ip;
    const servicePrefix = sub.plan.startsWith('normal-') ? 'site' : 'wordpress';
    const coolifyUrl = isRealProvisioned
        ? `http://${servicePrefix}-${sub.server_uuid}.${sub.server_ip}.sslip.io`
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
                            <><Server size={14} /> Provisionar hosting</>
                        )}
                    </Button>
                )}
                {coolifyUrl && (
                    <a href={coolifyUrl} target="_blank" rel="noopener noreferrer" className="hostingDetalleAccionLink">
                        <Button type="button" variante="primario" tamano="pequeno">
                            <ExternalLink size={14} /> Abrir sitio
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
                {/* [154A-9] Botones de control de servicio — solo para hostings provisionados */}
                {isRealProvisioned && sub.status === 'active' && (
                    <>
                        <Button type="button" variante="outline" tamano="pequeno"
                            onClick={onRestart} disabled={restartLoading || stopLoading || startLoading}>
                            {restartLoading ? <Loader size={14} className="hostingSpinner" /> : <RefreshCw size={14} />} Reiniciar
                        </Button>
                        <Button type="button" variante="outline" tamano="pequeno"
                            onClick={onStop} disabled={stopLoading || restartLoading || startLoading}>
                            {stopLoading ? <Loader size={14} className="hostingSpinner" /> : <Square size={14} />} Detener
                        </Button>
                        <Button type="button" variante="outline" tamano="pequeno"
                            onClick={onStart} disabled={startLoading || restartLoading || stopLoading}>
                            {startLoading ? <Loader size={14} className="hostingSpinner" /> : <Play size={14} />} Iniciar
                        </Button>
                    </>
                )}
            </div>
        </div>
    );
}

/* ── Tab: Recursos ───────────────────────── */
export function TabRecursos({sub}: {sub: Subscription}) {
    /* [114A-15+] Docker hostings ahora muestran CPU/RAM reales via Docker stats SSH.
     * HostingStats maneja ambos casos (Docker y no-Docker) con los mismos ResourceBars. */
    return (
        <div className="hostingDetalleSection">
            <h3 className="hostingDetalleSectionTitle">Uso de recursos</h3>
            {(sub.status === 'active' || sub.status === 'provisioning') ? (
                <div className="hostingDetalleRecursos">
                    <HostingStats sub={sub} />
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
export function TabEventos({hostingId}: {
    hostingId: string;
}) {
    return (
        <div className="hostingDetalleSection">
            <EventsPanel subscriptionId={hostingId} />
        </div>
    );
}
