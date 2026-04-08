/* [094A-2] Contenido de las tabs del detalle de hosting.
 * Extraído de HostingDetalle.tsx para cumplir límite de 300 líneas.
 * Tabs: General, Recursos, Dominio & SSL, Acceso SSH, Eventos.
 * [094A-4] TabDominio mejorada: instrucciones DNS con registros A/CNAME.
 * [094A-5] TabAcceso mejorada: info SSH incluso sin coolify_site_name.
 * [094A-6] TabFacturacion extraída a TabFacturacion.tsx por límite de líneas. */

import {Server, Shield, Terminal, ExternalLink} from 'lucide-react';
import type {useHostingDetalle} from '../../hooks/useHostingDetalle';
import {
    HOSTING_PLAN_LABELS,
    HOSTING_STATUS_LABELS,
} from '../../api/hosting';
import {Button} from '../ui/Button';
import {HostingStats} from './HostingStats';
import {EventsPanel} from './HostingSubComponents';
import {CopyButton, InfoRow} from './HostingDetalle';

type Subscription = NonNullable<ReturnType<typeof useHostingDetalle>['subscription']>;

/* ── Tab: General ────────────────────────── */
export function TabGeneral({sub, isAdmin}: {sub: Subscription; isAdmin: boolean}) {
    const sitioUrl = sub.domain ? `https://${sub.domain}` : null;

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
                {sitioUrl && (
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

/* ── Tab: Dominio & SSL ──────────────────── */
/* [094A-4] Instrucciones DNS claras: registros A/CNAME, propagación, SSL auto. */
export function TabDominio({domainInfo}: {
    domainInfo: ReturnType<typeof useHostingDetalle>['domainInfo'];
}) {
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
/* [094A-5] Muestra info SSH del contenedor si existe, o acceso base al VPS si no.
 * PuTTY + terminal instructions para ambos casos. */
export function TabAcceso({sshInfo}: {
    sshInfo: ReturnType<typeof useHostingDetalle>['sshInfo'];
}) {
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

                    <h4 className="hostingDetalleSubTitle">Conexión con PuTTY (Windows)</h4>
                    <div className="hostingDetalleInfoGrid">
                        <InfoRow label="Host Name" value={sshInfo.host} copyable />
                        <InfoRow label="Port" value={String(sshInfo.port)} />
                        <InfoRow label="Connection type" value="SSH" />
                    </div>
                    <p className="hostingDetalleSectionDesc">
                        Ingresa estos datos en PuTTY y haz clic en "Open". Se te pedirá el usuario y contraseña.
                    </p>

                    <h4 className="hostingDetalleSubTitle">Conexión SFTP</h4>
                    <div className="hostingDetalleInfoGrid">
                        <InfoRow label="Protocolo" value="SFTP" />
                        <InfoRow label="Host" value={sshInfo.host} copyable />
                        <InfoRow label="Puerto" value={String(sshInfo.port)} />
                        <InfoRow label="Usuario" value={sshInfo.user} copyable />
                    </div>
                    <p className="hostingDetalleSectionDesc">
                        Usa FileZilla, WinSCP o cualquier cliente SFTP para transferir archivos.
                    </p>
                </>
            ) : (
                <div className="hostingDetalleAccesoInactivo">
                    <Terminal size={32} strokeWidth={1.2} />
                    <p>El acceso SSH estará disponible una vez que el hosting esté provisionado en el servidor.</p>
                    <p className="hostingDetalleSectionDesc">
                        Una vez activo, aquí verás las credenciales SSH y SFTP para acceder a tu hosting
                        desde terminal (Linux/Mac), PuTTY (Windows) o clientes SFTP como FileZilla.
                    </p>
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
