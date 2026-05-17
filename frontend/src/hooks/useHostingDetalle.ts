/* [094A-2] Hook de estado para HostingDetalle.
 * Gestiona tabs internos, carga de datos del hosting seleccionado y acciones. */

import {useState, useMemo} from 'react';
import {useQuery} from '@tanstack/react-query';
import {
    apiGetHostingSubscription,
    apiListHostingEvents,
} from '../api/hosting';

export type HostingDetalleTab = 'general' | 'recursos' | 'dominio' | 'acceso' | 'facturacion' | 'eventos';

/* [094A-2] Info de acceso SSH derivada del hosting */
interface SshInfo {
    host: string;
    port: number;
    user: string;
    command: string;
}

/* [094A-4] Info de dominio derivada del hosting.
 * serverIp se usa para instrucciones de registros A en la tab de dominio. */
interface DomainInfo {
    domain: string | null;
    nameservers: string[];
    sslStatus: 'active' | 'pending' | 'none';
    sslProvider: string;
    serverIp: string | null;
    domainVerificationStatus: 'none' | 'pending_verification' | 'verified' | 'active';
    domainVerificationToken: string | null;
    domainVerifiedAt: string | null;
    txtHostname: string | null;
}

export function useHostingDetalle(hostingId: string) {
    const [tabActiva, setTabActiva] = useState<HostingDetalleTab>('general');

    const {data: subscription, isLoading} = useQuery({
        queryKey: ['hosting-subscription', hostingId],
        queryFn: () => apiGetHostingSubscription(hostingId),
        enabled: !!hostingId,
    });

    const {data: events = [], isLoading: eventsLoading} = useQuery({
        queryKey: ['hosting-events', hostingId],
        queryFn: () => apiListHostingEvents(hostingId),
        enabled: !!hostingId && tabActiva === 'eventos',
    });

    /* [094A-2] Derivar info SSH del hosting.
     * En producción vendrá del backend; por ahora se construye desde datos conocidos. */
    const sshInfo = useMemo((): SshInfo | null => {
        if (!subscription || !subscription.coolify_site_name) return null;
        const host = subscription.domain || 'vps1.nakomi.studio';
        return {
            host,
            port: 22,
            user: subscription.coolify_site_name,
            command: `ssh ${subscription.coolify_site_name}@${host}`,
        };
    }, [subscription]);

    /* [094A-4] Derivar info de dominio.
     * SSL es automático vía Let's Encrypt en Coolify para dominios configurados.
     * [104A-42] serverIp viene del backend (server_ip real del VPS), no hardcodeada. */
    const domainInfo = useMemo((): DomainInfo => {
        const domain = subscription?.domain ?? null;
        /* IP proviene del backend tras provisioning en Coolify */
        const serverIp = subscription?.server_ip ?? null;
        const domainVerificationStatus = subscription?.domain_verification_status ?? 'none';
        return {
            domain,
            nameservers: ['ns1.contabo.net', 'ns2.contabo.net', 'ns3.contabo.net'],
            sslStatus: domain ? (domainVerificationStatus === 'active' ? 'active' : 'pending') : 'none',
            sslProvider: 'Let\'s Encrypt (automático vía Coolify)',
            serverIp,
            domainVerificationStatus,
            domainVerificationToken: subscription?.domain_verification_token ?? null,
            domainVerifiedAt: subscription?.domain_verified_at ?? null,
            txtHostname: domain ? `_nakomi-verify.${domain}` : null,
        };
    }, [subscription]);

    return {
        subscription,
        isLoading,
        events,
        eventsLoading,
        tabActiva,
        setTabActiva,
        sshInfo,
        domainInfo,
    };
}
