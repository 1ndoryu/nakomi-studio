/* [154A-15a] Sección de wallet/saldo en el panel.
 * Muestra balance actual + historial de transacciones paginado.
 * Visible para todos los roles (todos los usuarios tienen wallet). */

import { useState } from 'react';
import { Loader2, Wallet, ArrowUpRight, ArrowDownLeft, ChevronLeft, ChevronRight } from 'lucide-react';
import { useWallet, useWalletTransactions } from '../../hooks/useWallet';
import { formatBalance } from '../../api/wallet';
import type { WalletTransactionResponse } from '../../api/wallet';
import { Button } from '../ui/Button';
import './SeccionWallet.css';

/* Mapa de tipos de transacción a etiquetas legibles */
const TIPO_LABELS: Record<string, string> = {
    refund_credit: 'Reembolso',
    payment: 'Pago',
    withdrawal: 'Retiro',
    bonus: 'Bonificación',
    adjustment: 'Ajuste',
};

function etiquetaTipo(tipo: string): string {
    return TIPO_LABELS[tipo] ?? tipo;
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

    const totalPaginas = Math.max(1, Math.ceil(total / PER_PAGE));

    if (cargandoSaldo) {
        return (
            <div className="walletVacio">
                <Loader2 className="walletSpinner" size={32} />
            </div>
        );
    }

    return (
        <div className="walletContenedor">
            {/* Tarjeta de saldo */}
            <div className="walletSaldoTarjeta">
                <div className="walletSaldoIcono">
                    <Wallet size={24} />
                </div>
                <div className="walletSaldoInfo">
                    <span className="walletSaldoLabel">Saldo disponible</span>
                    <span className="walletSaldoValor">
                        {formatBalance(wallet?.balance_cents ?? 0, wallet?.currency)}
                    </span>
                </div>
            </div>

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
        </div>
    );
}
