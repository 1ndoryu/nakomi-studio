/* [T1-withdrawal] Sección admin para gestionar solicitudes de retiro.
 * Lista solicitudes pendientes con opción de aprobar o rechazar.
 * Aprobar descuenta del wallet del usuario; rechazar solo cambia estado. */

import { useState } from 'react';
import { Loader2, CheckCircle, XCircle, ChevronLeft, ChevronRight } from 'lucide-react';
import { useAdminWithdrawals, useResolveWithdrawal } from '../../hooks/useWallet';
import { formatBalance } from '../../api/wallet';
import type { WithdrawalRequestResponse } from '../../api/wallet';
import { Button } from '../ui/Button';
import { Textarea } from '../ui/Textarea';
import './SeccionRetiros.css';

function FilaRetiroAdmin({ req, onResolver }: {
    req: WithdrawalRequestResponse;
    onResolver: (id: string, aprobar: boolean, notas: string) => void;
}) {
    const [notas, setNotas] = useState('');
    const fecha = new Date(req.created_at);

    return (
        <tr className="retiroAdminFila">
            <td className="retiroAdminCelda retiroAdminMonto">
                {formatBalance(req.amount_cents)}
            </td>
            <td className="retiroAdminCelda">
                {req.payment_method ?? '—'}
            </td>
            <td className="retiroAdminCelda retiroAdminDetalles">
                {req.payment_details ?? '—'}
            </td>
            <td className="retiroAdminCelda retiroAdminFecha">
                {fecha.toLocaleDateString('es', { day: '2-digit', month: 'short', year: 'numeric' })}
            </td>
            <td className="retiroAdminCelda retiroAdminAcciones">
                <Textarea
                    className="retiroAdminNotas"
                    value={notas}
                    onChange={e => setNotas(e.target.value)}
                    placeholder="Notas (opcional)"
                    rows={1}
                />
                <div className="retiroAdminBotones">
                    <Button
                        variante="primario"
                        tamano="pequeno"
                        onClick={() => onResolver(req.id, true, notas)}
                        title="Aprobar retiro"
                    >
                        <CheckCircle size={14} /> Aprobar
                    </Button>
                    <Button
                        variante="outline"
                        tamano="pequeno"
                        onClick={() => onResolver(req.id, false, notas)}
                        title="Rechazar retiro"
                    >
                        <XCircle size={14} /> Rechazar
                    </Button>
                </div>
            </td>
        </tr>
    );
}

export function SeccionRetiros() {
    const [pagina, setPagina] = useState(1);
    const { solicitudes, total, cargando } = useAdminWithdrawals(pagina, 20);
    const resolver = useResolveWithdrawal();
    const totalPaginas = Math.max(1, Math.ceil(total / 20));

    function handleResolver(id: string, aprobar: boolean, notas: string) {
        resolver.mutate({
            id,
            body: { approve: aprobar, admin_notes: notas || undefined },
        });
    }

    if (cargando) {
        return (
            <div className="retiroAdminVacio">
                <Loader2 className="retiroAdminSpinner" size={32} />
            </div>
        );
    }

    return (
        <div className="retiroAdminContenedor">
            <h2 className="retiroAdminTitulo">Solicitudes de retiro pendientes</h2>

            {solicitudes.length === 0 ? (
                <p className="retiroAdminVacioTexto">No hay solicitudes pendientes</p>
            ) : (
                <>
                    <div className="retiroAdminTablaWrapper">
                        <table className="retiroAdminTabla">
                            <thead>
                                <tr>
                                    <th className="retiroAdminHead">Monto</th>
                                    <th className="retiroAdminHead">Método</th>
                                    <th className="retiroAdminHead">Detalles</th>
                                    <th className="retiroAdminHead">Fecha</th>
                                    <th className="retiroAdminHead">Acciones</th>
                                </tr>
                            </thead>
                            <tbody>
                                {solicitudes.map(req => (
                                    <FilaRetiroAdmin
                                        key={req.id}
                                        req={req}
                                        onResolver={handleResolver}
                                    />
                                ))}
                            </tbody>
                        </table>
                    </div>

                    {totalPaginas > 1 && (
                        <div className="retiroAdminPaginacion">
                            <Button
                                variante="outline"
                                tamano="pequeno"
                                disabled={pagina <= 1}
                                onClick={() => setPagina(p => p - 1)}
                            >
                                <ChevronLeft size={16} />
                            </Button>
                            <span className="retiroAdminPaginaInfo">
                                Página {pagina} de {totalPaginas}
                            </span>
                            <Button
                                variante="outline"
                                tamano="pequeno"
                                disabled={pagina >= totalPaginas}
                                onClick={() => setPagina(p => p + 1)}
                            >
                                <ChevronRight size={16} />
                            </Button>
                        </div>
                    )}
                </>
            )}

            {resolver.isError && (
                <p className="retiroAdminError">Error al procesar solicitud</p>
            )}
        </div>
    );
}
