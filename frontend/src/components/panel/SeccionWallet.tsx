/* [154A-15a] Sección de wallet/saldo en el panel.
 * Muestra balance actual + historial de transacciones paginado.
 * [T1-withdrawal] Añadido botón "Solicitar retiro", formulario inline
 * y listado de solicitudes de retiro del usuario. */

import { useState } from 'react';
import { Loader2, Wallet, ArrowUpRight, ArrowDownLeft, ChevronLeft, ChevronRight, Banknote, Clock, CheckCircle, XCircle } from 'lucide-react';
import { useWallet, useWalletTransactions, useWithdrawals } from '../../hooks/useWallet';
import { formatBalance } from '../../api/wallet';
import type { WalletTransactionResponse, WithdrawalRequestResponse } from '../../api/wallet';
import { Button } from '../ui/Button';
import { FormRetiro } from './FormRetiro';
import './SeccionWallet.css';

/* Mapa de tipos de transacción a etiquetas legibles */
const TIPO_LABELS: Record<string, string> = {
    credit: 'Crédito',
    debit: 'Débito',
    refund: 'Reembolso',
    commission: 'Comisión',
    refund_credit: 'Reembolso',
    payment: 'Pago',
    withdrawal: 'Retiro',
    bonus: 'Bonificación',
    adjustment: 'Ajuste',
};

function etiquetaTipo(tipo: string): string {
    return TIPO_LABELS[tipo] ?? tipo;
}

/* Mapa de estado de retiro a etiqueta + icono */
const STATUS_RETIRO: Record<string, { label: string; clase: string }> = {
    pending: { label: 'Pendiente', clase: 'retiroPendiente' },
    approved: { label: 'Aprobado', clase: 'retiroAprobado' },
    rejected: { label: 'Rechazado', clase: 'retiroRechazado' },
};

function FilaRetiro({ req }: { req: WithdrawalRequestResponse }) {
    const info = STATUS_RETIRO[req.status] ?? { label: req.status, clase: '' };
    const fecha = new Date(req.created_at);
    const IconoStatus = req.status === 'approved' ? CheckCircle
        : req.status === 'rejected' ? XCircle : Clock;

    return (
        <tr className="walletFila">
            <td className="walletCelda">
                {formatBalance(req.amount_cents)}
            </td>
            <td className="walletCelda walletDescripcion">
                {req.payment_method ?? '—'}
            </td>
            <td className="walletCelda">
                <span className={`retiroBadge ${info.clase}`}>
                    <IconoStatus size={12} />
                    {info.label}
                </span>
            </td>
            <td className="walletCelda walletFecha">
                {fecha.toLocaleDateString('es', { day: '2-digit', month: 'short', year: 'numeric' })}
            </td>
        </tr>
    );
}

function FilaTransaccion({ tx }: { tx: WalletTransactionResponse }) {
    const esPositivo = tx.amount_cents >= 0;
    const fecha = new Date(tx.created_at);

    return (
        <tr className="walletFila">
            <td className="walletCelda">
                <span className={`walletIconoTipo ${esPositivo ? 'walletIngreso' : 'walletEgreso'}`}>
                    {esPositivo ? <ArrowDownLeft size={14} /> : <ArrowUpRight size={14} />}
                </span>
                {etiquetaTipo(tx.transaction_type)}
            </td>
            <td className="walletCelda walletDescripcion">
                {tx.description ?? '—'}
            </td>
            <td className={`walletCelda walletMonto ${esPositivo ? 'walletIngreso' : 'walletEgreso'}`}>
                {esPositivo ? '+' : ''}{formatBalance(tx.amount_cents)}
            </td>
            <td className="walletCelda walletSaldoDespues">
                {formatBalance(tx.balance_after_cents)}
            </td>
            <td className="walletCelda walletFecha">
                {fecha.toLocaleDateString('es', { day: '2-digit', month: 'short', year: 'numeric' })}
            </td>
        </tr>
    );
}

export function SeccionWallet() {
    const { wallet, cargandoSaldo } = useWallet();
    const [pagina, setPagina] = useState(1);
    const PER_PAGE = 20;
    const { transacciones, total, cargando } = useWalletTransactions(pagina, PER_PAGE);

    /* Estado del formulario de retiro */
    const [mostrarFormRetiro, setMostrarFormRetiro] = useState(false);

    /* Solicitudes de retiro del usuario */
    const [paginaRetiros, setPaginaRetiros] = useState(1);
    const { solicitudes, total: totalRetiros, cargando: cargandoRetiros } = useWithdrawals(paginaRetiros, 10);
    const totalPaginasRetiros = Math.max(1, Math.ceil(totalRetiros / 10));

    const totalPaginas = Math.max(1, Math.ceil(total / PER_PAGE));
    const saldoCents = wallet?.balance_cents ?? 0;

    if (cargandoSaldo) {
        return (
            <div className="walletVacio">
                <Loader2 className="walletSpinner" size={32} />
            </div>
        );
    }

    return (
        <div className="walletContenedor">
            {/* Tarjeta de saldo + botón retiro */}
            <div className="walletSaldoTarjeta">
                <div className="walletSaldoIcono">
                    <Wallet size={24} />
                </div>
                <div className="walletSaldoInfo">
                    <span className="walletSaldoLabel">Saldo disponible</span>
                    <span className="walletSaldoValor">
                        {formatBalance(saldoCents, wallet?.currency)}
                    </span>
                </div>
                {saldoCents > 0 && (
                    // sentinel-disable-next-line button-clase-especifica
                    <Button
                        variante="secundario"
                        tamano="pequeno"
                        className="walletSaldoBotonRetiro"
                        onClick={() => setMostrarFormRetiro(v => !v)}
                    >
                        <Banknote size={16} />
                        Solicitar retiro
                    </Button>
                )}
            </div>

            {/* Formulario inline de retiro */}
            {mostrarFormRetiro && (
                <FormRetiro saldoCents={saldoCents} onClose={() => setMostrarFormRetiro(false)} />
            )}

            {/* Tabla de transacciones */}
            <div className="walletHistorial">
                <h2 className="walletHistorialTitulo">Historial de movimientos</h2>

                {cargando ? (
                    <div className="walletVacio">
                        <Loader2 className="walletSpinner" size={24} />
                    </div>
                ) : transacciones.length === 0 ? (
                    <p className="walletVacioTexto">Sin movimientos aún</p>
                ) : (
                    <>
                        <div className="walletTablaWrapper">
                            <table className="walletTabla">
                                <thead>
                                    <tr>
                                        <th className="walletHead">Tipo</th>
                                        <th className="walletHead">Descripción</th>
                                        <th className="walletHead">Monto</th>
                                        <th className="walletHead">Saldo</th>
                                        <th className="walletHead">Fecha</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {transacciones.map(tx => (
                                        <FilaTransaccion key={tx.id} tx={tx} />
                                    ))}
                                </tbody>
                            </table>
                        </div>

                        {/* Paginación */}
                        {totalPaginas > 1 && (
                            <div className="walletPaginacion">
                                <Button
                                    variante="outline"
                                    tamano="pequeno"
                                    disabled={pagina <= 1}
                                    onClick={() => setPagina(p => p - 1)}
                                >
                                    <ChevronLeft size={16} />
                                </Button>
                                <span className="walletPaginaInfo">
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
            </div>

            {/* Solicitudes de retiro */}
            <div className="walletHistorial">
                <h2 className="walletHistorialTitulo">Solicitudes de retiro</h2>

                {cargandoRetiros ? (
                    <div className="walletVacio">
                        <Loader2 className="walletSpinner" size={24} />
                    </div>
                ) : solicitudes.length === 0 ? (
                    <p className="walletVacioTexto">No has solicitado retiros aún</p>
                ) : (
                    <>
                        <div className="walletTablaWrapper">
                            <table className="walletTabla">
                                <thead>
                                    <tr>
                                        <th className="walletHead">Monto</th>
                                        <th className="walletHead">Método</th>
                                        <th className="walletHead">Estado</th>
                                        <th className="walletHead">Fecha</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {solicitudes.map(req => (
                                        <FilaRetiro key={req.id} req={req} />
                                    ))}
                                </tbody>
                            </table>
                        </div>

                        {totalPaginasRetiros > 1 && (
                            <div className="walletPaginacion">
                                <Button
                                    variante="outline"
                                    tamano="pequeno"
                                    disabled={paginaRetiros <= 1}
                                    onClick={() => setPaginaRetiros(p => p - 1)}
                                >
                                    <ChevronLeft size={16} />
                                </Button>
                                <span className="walletPaginaInfo">
                                    Página {paginaRetiros} de {totalPaginasRetiros}
                                </span>
                                <Button
                                    variante="outline"
                                    tamano="pequeno"
                                    disabled={paginaRetiros >= totalPaginasRetiros}
                                    onClick={() => setPaginaRetiros(p => p + 1)}
                                >
                                    <ChevronRight size={16} />
                                </Button>
                            </div>
                        )}
                    </>
                )}
            </div>
        </div>
    );
}
