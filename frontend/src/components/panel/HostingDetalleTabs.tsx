/* [094A-2] Contenido de las tabs del detalle de hosting.
 * Extraído de HostingDetalle.tsx para cumplir límite de 300 líneas.
 * Tabs: General, Recursos, Dominio & SSL, Acceso SSH, Eventos.
 * [094A-4] TabDominio mejorada: instrucciones DNS con registros A/CNAME.
 * [094A-5] TabAcceso mejorada: info SSH incluso sin coolify_site_name.
 * [094A-6] TabFacturacion extraída a TabFacturacion.tsx por límite de líneas. */

import {useState} from 'react';
import {Server, Shield, Terminal, ExternalLink, Loader, CheckCircle, XCircle, AlertCircle} from 'lucide-react';
import type {useHostingDetalle} from '../../hooks/useHostingDetalle';
import {
    HOSTING_PLAN_LABELS,
    HOSTING_STATUS_LABELS,
    apiDnsCheck,
} from '../../api/hosting';
import type {DnsCheckResult} from '../../api/hosting';
import {Button} from '../ui/Button';
import {HostingStats} from './HostingStats';
import {EventsPanel} from './HostingSubComponents';
import {CopyButton, InfoRow} from './HostingDetalle';

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
                            <><Server size={14} /> Provisionar hosting</>
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

/* ── Tab: Dominio & SSL ──────────────────── */
/* [094A-4] Instrucciones DNS claras: registros A/CNAME, propagación, SSL auto.
 * [154A-16] Verificación DNS interactiva: resuelve dominio y compara con server_ip. */
export function TabDominio({domainInfo, subscriptionId}: {
    domainInfo: ReturnType<typeof useHostingDetalle>['domainInfo'];
    subscriptionId: string;
}) {
    const [dnsResult, setDnsResult] = useState<DnsCheckResult | null>(null);
    const [checking, setChecking] = useState(false);

    const handleDnsCheck = async () => {
        setChecking(true);
        try {
            const result = await apiDnsCheck(subscriptionId);
            setDnsResult(result);
        } catch {
            setDnsResult({configured: false, message: 'Error al verificar DNS'});
        } finally {
            setChecking(false);
        }
    };
    return (
        <div className="hostingDetalleSection">
            <h3 className="hostingDetalleSectionTitle">Dominio</h3>
            {domainInfo.domain ? (
                <div className="hostingDetalleInfoGrid">
                    <InfoRow label="Dominio principal" value={domainInfo.domain} copyable />
                    <InfoRow
                        label="Sitio"
                        value={`https://${domainInfo.domain}`}
                        link={`https://${domainInfo.domain}`}
                    />
                </div>
            ) : (
                <p className="hostingDetalleNoDomain">
                    No hay dominio configurado. Contacta soporte para configurar tu dominio.
                </p>
            )}

            <h3 className="hostingDetalleSectionTitle">Configuración DNS</h3>
            <p className="hostingDetalleSectionDesc">
                Configura estos registros en tu registrador de dominios (GoDaddy, Namecheap, Cloudflare, etc.):
            </p>

            {domainInfo.serverIp && (
                <div className="hostingDetalleDnsRecords">
                    <div className="hostingDetalleDnsRecord">
                        <span className="hostingDetalleDnsType">A</span>
                        <span className="hostingDetalleDnsHost">@</span>
                        <span className="hostingDetalleDnsValue">
                            {domainInfo.serverIp}
                            <CopyButton text={domainInfo.serverIp} />
                        </span>
                    </div>
                    <div className="hostingDetalleDnsRecord">
                        <span className="hostingDetalleDnsType">A</span>
                        <span className="hostingDetalleDnsHost">www</span>
                        <span className="hostingDetalleDnsValue">
                            {domainInfo.serverIp}
                            <CopyButton text={domainInfo.serverIp} />
                        </span>
                    </div>
                </div>
            )}

            <h4 className="hostingDetalleSubTitle">Nameservers (alternativa)</h4>
            <p className="hostingDetalleSectionDesc">
                Si prefieres usar nameservers en lugar de registros A:
            </p>
            <div className="hostingDetalleNameservers">
                {domainInfo.nameservers.map(ns => (
                    <div key={ns} className="hostingDetalleNameserver">
                        <code>{ns}</code>
                        <CopyButton text={ns} />
                    </div>
                ))}
            </div>

            <p className="hostingDetalleDnsPropagation">
                Los cambios DNS pueden tardar entre 15 minutos y 48 horas en propagarse globalmente.
            </p>

            {/* [154A-16] Verificación DNS interactiva */}
            {domainInfo.domain && (
                <div className="hostingDetalleDnsCheck">
                    <h4 className="hostingDetalleSubTitle">Verificación DNS</h4>
                    <Button
                        type="button"
                        variante="outline"
                        tamano="pequeno"
                        onClick={handleDnsCheck}
                        disabled={checking}
                    >
                        {checking ? (
                            <><Loader size={14} className="hostingSpinner" /> Verificando…</>
                        ) : (
                            'Verificar DNS'
                        )}
                    </Button>
                    {dnsResult && (
                        <div className={`hostingDetalleDnsResult ${
                            dnsResult.points_to_server ? 'hostingDetalleDnsResult--ok' :
                            dnsResult.resolved ? 'hostingDetalleDnsResult--warn' :
                            'hostingDetalleDnsResult--error'
                        }`}>
                            {dnsResult.points_to_server ? (
                                <><CheckCircle size={16} /> DNS configurado correctamente. El dominio apunta a tu servidor.</>
                            ) : dnsResult.resolved ? (
                                <><AlertCircle size={16} /> El dominio resuelve a {dnsResult.resolved_ips?.join(', ')} pero se esperaba {dnsResult.expected_ip}.</>
                            ) : dnsResult.error ? (
                                <><XCircle size={16} /> {dnsResult.error}</>
                            ) : (
                                <><XCircle size={16} /> {dnsResult.message}</>
                            )}
                        </div>
                    )}
                </div>
            )}

            <h3 className="hostingDetalleSectionTitle">
                <Shield size={16} /> Certificado SSL
            </h3>
            <div className="hostingDetalleInfoGrid">
                <InfoRow
                    label="Estado"
                    value={domainInfo.sslStatus === 'active' ? 'Activo' : domainInfo.sslStatus === 'pending' ? 'Pendiente' : 'No configurado'}
                />
                <InfoRow label="Proveedor" value={domainInfo.sslProvider} />
            </div>
            {domainInfo.sslStatus === 'active' && (
                <p className="hostingDetalleSslNota">
                    Tu certificado SSL se renueva automáticamente. No necesitas hacer nada.
                </p>
            )}
        </div>
    );
}

/* ── Tab: Acceso SSH/SFTP ────────────────── */
/* [094A-5] Para hostings Docker (Coolify): muestra panel WP admin + credenciales SFTP reales.
 * [104A-18] Las credenciales SFTP (user/password/port) se generan al provisionar y se muestran aquí.
 * La contraseña se puede mostrar/ocultar con toggle para seguridad básica en pantallas compartidas. */
export function TabAcceso({sshInfo, sub}: {
    sshInfo: ReturnType<typeof useHostingDetalle>['sshInfo'];
    sub: Subscription;
}) {
    const [showPassword, setShowPassword] = useState(false);
    const isDockerHosting = sub.coolify_site_name?.startsWith('hosting-') && sub.server_uuid && sub.server_ip;
    const wpAdminUrl = isDockerHosting
        ? `http://wordpress-${sub.server_uuid}.${sub.server_ip}.sslip.io/wp-admin`
        : null;
    const hasSftp = isDockerHosting && sub.sftp_user && sub.sftp_password && sub.sftp_port;

    if (isDockerHosting) {
        return (
            <div className="hostingDetalleSection">
                <h3 className="hostingDetalleSectionTitle">Panel de administración</h3>
                <div className="hostingDetalleInfoGrid">
                    <InfoRow label="WordPress admin" value={wpAdminUrl!} copyable link={wpAdminUrl!} />
                </div>
                <div className="hostingDetalleAcciones">
                    <a href={wpAdminUrl!} target="_blank" rel="noopener noreferrer" className="hostingDetalleAccionLink">
                        <Button type="button" variante="primario" tamano="pequeno">
                            <ExternalLink size={14} /> Abrir panel WordPress
                        </Button>
                    </a>
                </div>

                <h3 className="hostingDetalleSectionTitle">Acceso SFTP</h3>
                {hasSftp ? (
                    <>
                        <p className="hostingDetalleSectionDesc">
                            Usa estas credenciales con FileZilla, Cyberduck o cualquier cliente SFTP para acceder a los archivos de tu sitio.
                        </p>
                        <div className="hostingDetalleInfoGrid">
                            <InfoRow label="Host" value={sub.server_ip!} copyable />
                            <InfoRow label="Puerto" value={String(sub.sftp_port!)} copyable />
                            <InfoRow label="Usuario" value={sub.sftp_user!} copyable />
                            <InfoRow label="Protocolo" value="SFTP (SSH File Transfer Protocol)" />
                            <InfoRow
                                label="Contraseña"
                                value={showPassword ? sub.sftp_password! : '••••••••••••••••'}
                                copyable={showPassword}
                            />
                        </div>
                        <div className="hostingDetalleAcciones">
                            <Button
                                type="button"
                                variante="outline"
                                tamano="pequeno"
                                onClick={() => setShowPassword(p => !p)}
                            >
                                {showPassword ? 'Ocultar contraseña' : 'Mostrar contraseña'}
                            </Button>
                        </div>

                        <h4 className="hostingDetalleSubTitle">Conectar por terminal</h4>
                        <div className="hostingDetalleCodeBlock">
                            <code>{`sftp -P ${sub.sftp_port} ${sub.sftp_user}@${sub.server_ip}`}</code>
                            <CopyButton text={`sftp -P ${sub.sftp_port} ${sub.sftp_user}@${sub.server_ip}`} />
                        </div>
                        <p className="hostingDetalleSectionDesc">
                            El acceso es SFTP (transferencia de archivos), no SSH interactivo. Para gestión avanzada del servidor, usa el panel WordPress o contacta soporte.
                        </p>
                    </>
                ) : (
                    <p className="hostingDetalleSectionDesc">
                        Las credenciales SFTP estarán disponibles una vez que el hosting esté provisionado.
                    </p>
                )}
            </div>
        );
    }

    return (
        <div className="hostingDetalleSection">
            <h3 className="hostingDetalleSectionTitle">Acceso SSH</h3>
            {sshInfo ? (
                <>
                    <div className="hostingDetalleInfoGrid">
                        <InfoRow label="Host" value={sshInfo.host} copyable />
                        <InfoRow label="Puerto" value={String(sshInfo.port)} />
                        <InfoRow label="Usuario" value={sshInfo.user} copyable />
                    </div>

                    <h4 className="hostingDetalleSubTitle">Conexión por terminal</h4>
                    <div className="hostingDetalleCodeBlock">
                        <code>{sshInfo.command}</code>
                        <CopyButton text={sshInfo.command} />
                    </div>

                    <h4 className="hostingDetalleSubTitle">Conexión SFTP</h4>
                    <div className="hostingDetalleInfoGrid">
                        <InfoRow label="Protocolo" value="SFTP" />
                        <InfoRow label="Host" value={sshInfo.host} copyable />
                        <InfoRow label="Puerto" value={String(sshInfo.port)} />
                        <InfoRow label="Usuario" value={sshInfo.user} copyable />
                    </div>
                </>
            ) : (
                <div className="hostingDetalleAccesoInactivo">
                    <Terminal size={32} strokeWidth={1.2} />
                    <p>El acceso SSH estará disponible una vez que el hosting esté provisionado en el servidor.</p>
                </div>
            )}
        </div>
    );
}

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
