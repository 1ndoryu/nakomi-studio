/* [044A-38 Fase 4] Sección "Delegaciones" del panel (empleado/admin).
 * [074A-58] Rediseño: cards horizontales matching disponibleCard/hostingCard pattern. */
import React, {useState, useCallback} from 'react';
import {ArrowRightLeft, HelpCircle, Check, X} from 'lucide-react';
import {useDelegaciones} from '../../hooks/useAsignaciones';
import {
    DELEGATION_STATUS_LABELS,
    DELEGATION_STATUS_CLASS,
    type DelegationResponse,
} from '../../api/assignment';
import {useAuthStore} from '../../stores/authStore';
import {Button} from '../ui/Button';
import './SeccionDelegaciones.css';

export const SeccionDelegaciones: React.FC = () => {
    const userId = useAuthStore(s => s.user?.userId);
    const {delegaciones, cargando, responder, respondiendo} = useDelegaciones();
    const [error, setError] = useState<string | null>(null);

    const handleResponder = useCallback(async (delegationId: string, accept: boolean) => {
        try {
            setError(null);
            await responder({delegationId, accept});
        } catch (err: unknown) {
            setError(extraerError(err));
        }
    }, [responder]);

    if (cargando) {
        return (
            <div className="delegLoading">
                <div className="delegSpinner" />
                <p>Cargando delegaciones...</p>
            </div>
        );
    }

    if (delegaciones.length === 0) {
        return (
            <div className="delegVacio">
                <ArrowRightLeft size={48} strokeWidth={1.2} />
                <h3>Sin delegaciones</h3>
                <p>No hay solicitudes de delegación o ayuda activas.</p>
            </div>
        );
    }

    return (
        <div className="delegContenedor">
            {error && <div className="delegError">{error}</div>}
            <div className="delegLista">
                {delegaciones.map(d => (
                    <DelegacionCard
                        key={d.id}
                        delegacion={d}
                        userId={userId}
                        onResponder={handleResponder}
                        respondiendo={respondiendo}
                    />
                ))}
            </div>
        </div>
    );
};

function DelegacionCard({
    delegacion,
    userId,
    onResponder,
    respondiendo,
}: {
    delegacion: DelegationResponse;
    userId: string | undefined;
    onResponder: (id: string, accept: boolean) => void;
    respondiendo: boolean;
}) {
    const esPropia = delegacion.from_employee_id === userId;
    const esDelegacion = delegacion.delegation_type === 'delegate';
    const pendiente = delegacion.status === 'requested';
    const puedeResponder = pendiente && !esPropia;

    return (
        <div className="delegCard">
            <div className="panelCardIcono">
                {esDelegacion
                    ? <ArrowRightLeft size={28} strokeWidth={1.4} />
                    : <HelpCircle size={28} strokeWidth={1.4} />
                }
            </div>
            <div className="delegCardBody">
                <span className="delegCardTipo">
                    {esDelegacion ? 'Delegación' : 'Solicitud de ayuda'}
                </span>
                <div className="delegCardHeader">
                    <h3 className="delegCardTitulo">
                        #{delegacion.order_number} — {delegacion.service_title}
                    </h3>
                    <span className={`delegCardBadge ${DELEGATION_STATUS_CLASS[delegacion.status]}`}>
                        {DELEGATION_STATUS_LABELS[delegacion.status]}
                    </span>
                </div>
                {delegacion.reason && (
                    <p className="delegCardRazon">{delegacion.reason}</p>
                )}
                <div className="delegCardFooter">
                    <span>{esPropia ? 'Enviada por ti' : 'De otro empleado'}</span>
                    <span>{new Date(delegacion.created_at).toLocaleDateString('es')}</span>
                    {delegacion.resolved_at && (
                        <span>Resuelta: {new Date(delegacion.resolved_at).toLocaleDateString('es')}</span>
                    )}
                    {puedeResponder && (
                        <div className="delegCardAcciones">
                            <Button
                                variante="primario"
                                tamano="pequeno"
                                className="delegBtnAceptar"
                                onClick={() => onResponder(delegacion.id, true)}
                                disabled={respondiendo}
                                type="button"
                            >
                                <Check size={14} />
                                {esDelegacion ? 'Tomar orden' : 'Aceptar'}
                            </Button>
                            <Button
                                variante="outline"
                                tamano="pequeno"
                                className="delegBtnRechazar"
                                onClick={() => onResponder(delegacion.id, false)}
                                disabled={respondiendo}
                                type="button"
                            >
                                <X size={14} />
                                Rechazar
                            </Button>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}

function extraerError(err: unknown): string {
    if (typeof err === 'object' && err !== null && 'response' in err) {
        const resp = (err as {response?: {data?: {message?: string; error?: string}}}).response;
        if (resp?.data?.message) return resp.data.message;
        if (resp?.data?.error) return resp.data.error;
    }
    return 'Error en delegación';
}
