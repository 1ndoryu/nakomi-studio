/* [304A-3] Sección Dominios del panel admin.
 * Muestra dominios registrados en Contabo y permite gestionarlos.
 * Backend: GET /api/hosting/domains (admin only, Contabo API).
 * Pendiente: formulario de compra/registro de nuevos dominios. */

import React from 'react';
import {Globe, RefreshCw} from 'lucide-react';
import {useQuery} from '@tanstack/react-query';
import {apiListDomains, type ContaboDomain} from '../../api/hosting';
import {Button} from '../ui/Button';
import './SeccionDominios.css';

function DomainCard({domain}: {domain: ContaboDomain}) {
    const nombre = [domain.sld, domain.tld].filter(Boolean).join('.');
    const estado = domain.status ?? 'unknown';
    const vence = domain.paidUntil
        ? new Date(domain.paidUntil).toLocaleDateString('es')
        : null;

    return (
        <div className="dominioCard">
            <div className="dominioCardHeader">
                <Globe size={18} strokeWidth={1.4} className="dominioCardIcono" />
                <span className="dominioCombre">{nombre || '(sin nombre)'}</span>
                <span className={`dominioEstado dominioEstado--${estado.toLowerCase()}`}>
                    {estado}
                </span>
            </div>
            {vence && (
                <p className="dominioVence">Vence: {vence}</p>
            )}
            {domain.nameservers && domain.nameservers.length > 0 && (
                <div className="dominioNs">
                    {domain.nameservers.slice(0, 2).map((ns) => (
                        <span key={ns.hostname} className="dominioNsItem">{ns.hostname}</span>
                    ))}
                    {domain.nameservers.length > 2 && (
                        <span className="dominioNsItem">+{domain.nameservers.length - 2} más</span>
                    )}
                </div>
            )}
        </div>
    );
}

export const SeccionDominios: React.FC = () => {
    const {data: dominios = [], isLoading, isError, refetch, isFetching} = useQuery({
        queryKey: ['admin-domains'],
        queryFn: apiListDomains,
        staleTime: 2 * 60 * 1000, /* 2 minutos — Contabo API es lenta */
    });

    return (
        <div className="dominiosContenedor">
            <div className="dominiosHeader seccionHeader">
                <Globe size={20} strokeWidth={1.4} />
                <h2 className="seccionTitulo">Dominios</h2>
                <Button
                    type="button"
                    variante="texto"
                    tamano="pequeno"
                    onClick={() => refetch()}
                    disabled={isFetching}
                    title="Recargar dominios desde Contabo"
                >
                    <RefreshCw size={14} className={isFetching ? 'dominiosSpinning' : ''} />
                </Button>
            </div>

            {isLoading && (
                <div className="dominiosCargando">
                    <p>Cargando dominios desde Contabo…</p>
                </div>
            )}

            {isError && (
                <div className="dominiosError">
                    <p>Error al cargar dominios. Verifica que Contabo API esté configurada.</p>
                    <Button type="button" variante="outline" tamano="pequeno" onClick={() => refetch()}>
                        Reintentar
                    </Button>
                </div>
            )}

            {!isLoading && !isError && dominios.length === 0 && (
                <div className="dominiosVacio">
                    <Globe size={40} strokeWidth={1.2} />
                    <p>No hay dominios registrados en Contabo.</p>
                </div>
            )}

            {!isLoading && dominios.length > 0 && (
                <div className="dominiosLista">
                    {dominios.map((d, i) => (
                        <DomainCard key={`${d.sld}-${d.tld}-${i}`} domain={d} />
                    ))}
                </div>
            )}
        </div>
    );
};
