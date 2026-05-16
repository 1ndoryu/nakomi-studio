/* [114A-5] Tabs de acceso extraídas de HostingDetalleTabs.tsx por límite de 300 líneas.
 * Contiene: TabDominio (DNS, SSL, verificación) y TabAcceso (SSH/SFTP, WP admin). */

import {useState} from 'react';
import {Shield, Terminal, ExternalLink, Loader, CheckCircle, XCircle, AlertCircle} from 'lucide-react';
import type {useHostingDetalle} from '../../hooks/useHostingDetalle';
import {
    apiDnsCheck,
    getProvisionedHostingAdminUrl,
    getProvisionedHostingSiteUrl,
} from '../../api/hosting';
import type {DnsCheckResult} from '../../api/hosting';
import {Button} from '../ui/Button';
import {CopyButton, InfoRow} from './HostingDetalle';
import {HostingDomainSelfServiceForm} from './HostingDomainSelfServiceForm';
import {DnsManager} from './DnsManager';

type Subscription = NonNullable<ReturnType<typeof useHostingDetalle>['subscription']>;

/* ── Tab: Dominio & SSL ──────────────────── */
/* [094A-4] Instrucciones DNS claras: registros A/CNAME, propagación, SSL auto.
 * [154A-16] Verificación DNS interactiva: resuelve dominio y compara con server_ip. */
export function TabDominio({sub, domainInfo, subscriptionId}: {
    sub: Subscription;
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
                    Aún no hay un dominio conectado. Puedes configurarlo tú mismo desde aquí.
                </p>
            )}

            <HostingDomainSelfServiceForm
                sub={sub}
                subscriptionId={subscriptionId}
                currentDomain={domainInfo.domain}
                serverIp={domainInfo.serverIp}
            />

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

            {/* [154A-6] Gestión de registros DNS del cliente */}
            {domainInfo.domain && <DnsManager subscriptionId={subscriptionId} />}
        </div>
    );
}

/* ── Tab: Acceso SSH/SFTP ────────────────── */
/* [094A-5][155A-13] Para hostings Docker (Coolify): muestra panel WP admin solo en WordPress
 * y URL del sitio + credenciales SFTP en hosting normal.
 * [104A-18] Las credenciales SFTP (user/password/port) se generan al provisionar y se muestran aquí.
 * La contraseña se puede mostrar/ocultar con toggle para seguridad básica en pantallas compartidas. */
export function TabAcceso({sshInfo, sub}: {
    sshInfo: ReturnType<typeof useHostingDetalle>['sshInfo'];
    sub: Subscription;
}) {
    const [showPassword, setShowPassword] = useState(false);
    const isNormalHosting = sub.plan.startsWith('normal-');
    const siteUrl = getProvisionedHostingSiteUrl(sub);
    const wpAdminUrl = getProvisionedHostingAdminUrl(sub);
    const isDockerHosting = siteUrl !== null;
    const hasSftp = isDockerHosting && sub.sftp_user && sub.sftp_password && sub.sftp_port;

    if (isDockerHosting) {
        return (
            <div className="hostingDetalleSection">
                <h3 className="hostingDetalleSectionTitle">{isNormalHosting ? 'Sitio publicado' : 'Panel de administración'}</h3>
                <div className="hostingDetalleInfoGrid">
                    {isNormalHosting ? (
                        <InfoRow label="URL del sitio" value={siteUrl!} copyable link={siteUrl!} />
                    ) : (
                        <InfoRow label="WordPress admin" value={wpAdminUrl!} copyable link={wpAdminUrl!} />
                    )}
                </div>
                <div className="hostingDetalleAcciones">
                    <a href={(isNormalHosting ? siteUrl : wpAdminUrl)!} target="_blank" rel="noopener noreferrer" className="hostingDetalleAccionLink">
                        <Button type="button" variante="primario" tamano="pequeno">
                            <ExternalLink size={14} /> {isNormalHosting ? 'Abrir sitio' : 'Abrir panel WordPress'}
                        </Button>
                    </a>
                </div>

                {!isNormalHosting && hasSftp && (
                    <>
                        <h3 className="hostingDetalleSectionTitle">Credenciales iniciales de WordPress</h3>
                        <p className="hostingDetalleSectionDesc">
                            El hosting WordPress se instala automáticamente usando el mismo usuario y contraseña iniciales del acceso SFTP. Después del primer acceso puedes cambiar la contraseña desde WordPress.
                        </p>
                        <div className="hostingDetalleInfoGrid">
                            <InfoRow label="Usuario WordPress" value={sub.sftp_user!} copyable />
                            <InfoRow
                                label="Contraseña inicial"
                                value={showPassword ? sub.sftp_password! : '••••••••••••••••'}
                                copyable={showPassword}
                            />
                        </div>
                    </>
                )}

                <h3 className="hostingDetalleSectionTitle">Acceso SFTP</h3>
                {hasSftp ? (
                    <>
                        <p className="hostingDetalleSectionDesc">
                            Usa estas credenciales con cualquier cliente SFTP (FileZilla, Cyberduck, terminal) para acceder a los archivos de tu sitio.
                        </p>
                        <div className="hostingDetalleInfoGrid">
                            <InfoRow label="Host" value={sub.server_ip!} copyable />
                            <InfoRow label="Puerto" value={String(sub.sftp_port!)} copyable />
                            <InfoRow label="Usuario" value={sub.sftp_user!} copyable />
                            <InfoRow label="Protocolo" value="SSH + SFTP" />
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
                            <code>{`ssh -p ${sub.sftp_port} ${sub.sftp_user}@${sub.server_ip}`}</code>
                            <CopyButton text={`ssh -p ${sub.sftp_port} ${sub.sftp_user}@${sub.server_ip}`} />
                        </div>
                        <h4 className="hostingDetalleSubTitle">Transferir archivos (SFTP)</h4>
                        <div className="hostingDetalleCodeBlock">
                            <code>{`sftp -P ${sub.sftp_port} ${sub.sftp_user}@${sub.server_ip}`}</code>
                            <CopyButton text={`sftp -P ${sub.sftp_port} ${sub.sftp_user}@${sub.server_ip}`} />
                        </div>
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
