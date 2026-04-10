/* [044A-38 Fase 7] Sección de reembolsos para admin.
 * Lista reembolsos pendientes, permite aprobar/rechazar con respuesta. */

import { useState } from 'react';
import { Loader2, AlertCircle, CheckCircle, XCircle, Clock } from 'lucide-react';
import { useRefunds } from '../../hooks/useRefunds';
import { REFUND_STATUS_LABELS, REFUND_STATUS_CLASS } from '../../api/refunds';
import { formatPrice } from '../../api/orders';
import type { RefundResponse } from '../../api/refunds';
import { Textarea } from '../ui/Textarea';
import { Button } from '../ui/Button';
import './SeccionReembolsos.css';

export function SeccionReembolsos() {
    const { reembolsos, cargando, error, revisarReembolso } = useRefunds();
    const [refundActivo, setRefundActivo] = useState<RefundResponse | null>(null);
    const [respuestaAdmin, setRespuestaAdmin] = useState('');
    const [accionEnCurso, setAccionEnCurso] = useState(false);

    const handleRevisar = async (action: 'approve' | 'reject') => {
        if (!refundActivo) return;
        setAccionEnCurso(true);
        try {
            await revisarReembolso.mutateAsync({
                refundId: refundActivo.id,
                action,
                admin_response: respuestaAdmin || undefined,
            });
            setRefundActivo(null);
            setRespuestaAdmin('');
        } finally {
            setAccionEnCurso(false);
        }
    };

    if (cargando) {
        return (
            <div className="reembolsosVacio">
                <Loader2 className="reembolsosSpinner" size={32} />
            </div>
        );
    }

    if (error) {
        return (
            <div className="reembolsosError">
                <AlertCircle size={20} />
                <span>{error}</span>
            </div>
        );
    }

    return (
        <div className="reembolsosContenedor">
            {reembolsos.length === 0 && (
                <div className="reembolsosVacio">
                    <CheckCircle size={32} />
                    <p>No hay solicitudes de reembolso pendientes</p>
                </div>
            )}

            <div className="reembolsosLista">
                {reembolsos.map((r) => (
                    <div key={r.id} className="reembolsoCard">
                        <div className="reembolsoCardHeader">
                            <span className={`reembolsoEstado ${REFUND_STATUS_CLASS[r.status]}`}>
                                {REFUND_STATUS_LABELS[r.status]}
                            </span>
                            <span className="reembolsoMonto">
                                {formatPrice(r.amount_cents, 'usd')}
                            </span>
                        </div>

                        <p className="reembolsoRazon">{r.reason}</p>

                        <div className="reembolsoMeta">
                            <span className="reembolsoFecha">
                                <Clock size={14} />
                                {new Date(r.requested_at).toLocaleDateString('es-ES', {
                                    day: 'numeric',
                                    month: 'short',
                                    year: 'numeric',
                                })}
                            </span>
                            <span className="reembolsoOrden">Orden: {r.order_id.slice(0, 8)}…</span>
                        </div>

                        {r.admin_response && (
                            <p className="reembolsoRespuesta">
                                <span className="reembolsoRespuestaEtiqueta">Respuesta:</span> {r.admin_response}
                            </p>
                        )}

                        {(r.status === 'requested' || r.status === 'under_review') && (
                            <div className="reembolsoAcciones">
                                {refundActivo?.id === r.id ? (
                                    <div className="reembolsoFormRevision">
                                        <Textarea
                                            className="reembolsoTextarea"
                                            placeholder="Respuesta al cliente (opcional)..."
                                            value={respuestaAdmin}
                                            onChange={(e) => setRespuestaAdmin(e.target.value)}
                                            rows={3}
                                        />
                                        <div className="reembolsoBotonesRevision">
                                            <Button
                                                variante="secundario"
                                                tamano="pequeno"
                                                type="button"
                                                onClick={() => void handleRevisar('approve')}
                                                disabled={accionEnCurso}
                                            >
                                                <span className="reembolsoAccionContenido">
                                                    <CheckCircle size={16} />
                                                    {accionEnCurso ? 'Procesando…' : 'Aprobar reembolso'}
                                                </span>
                                            </Button>
                                            <Button
                                                variante="outline"
                                                tamano="pequeno"
                                                type="button"
                                                onClick={() => void handleRevisar('reject')}
                                                disabled={accionEnCurso}
                                            >
                                                <span className="reembolsoAccionContenido">
                                                    <XCircle size={16} />
                                                    Rechazar
                                                </span>
                                            </Button>
                                            <Button
                                                variante="outline"
                                                tamano="pequeno"
                                                type="button"
                                                onClick={() => {
                                                    setRefundActivo(null);
                                                    setRespuestaAdmin('');
                                                }}
                                                disabled={accionEnCurso}
                                            >
                                                Cancelar
                                            </Button>
                                        </div>
                                    </div>
                                ) : (
                                    <Button
                                        variante="primario"
                                        tamano="pequeno"
                                        type="button"
                                        onClick={() => setRefundActivo(r)}
                                    >
                                        Revisar
                                    </Button>
                                )}
                            </div>
                        )}
                    </div>
                ))}
            </div>
        </div>
    );
}
