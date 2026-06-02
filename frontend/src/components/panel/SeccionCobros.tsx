/* [026B-1] Sección admin: gestión de cobros pendientes (billing_items).
 * Lista todos los billing_items de todos los usuarios con su email.
 * Permite marcar un item como pagado o pendiente manualmente. */

import { useState } from 'react';
import { Loader2, AlertCircle, ReceiptText, CheckCircle, Circle, Search } from 'lucide-react';
import { useAdminBilling } from '../../hooks/useAdminBilling';
import { Button } from '../ui/Button';
import { Input } from '../ui/Input';
import { Tarjeta } from '../ui/Tarjeta';
import './SeccionCobros.css';

function formatMoney(cents: number, currency: string): string {
    return new Intl.NumberFormat('en-US', {
        style: 'currency',
        currency: currency.toUpperCase(),
    }).format(cents / 100);
}

function formatDate(value: string): string {
    return new Intl.DateTimeFormat('es', {
        day: '2-digit', month: 'short', year: 'numeric',
    }).format(new Date(value));
}

export function SeccionCobros() {
    const [statusFilter, setStatusFilter] = useState<string>('');
    const [search, setSearch] = useState('');
    const { items, isLoading, error, toggleStatusMut } = useAdminBilling(statusFilter);

    if (isLoading) {
        return (
            <div className="cobrosVacio">
                <Loader2 className="cobrosSpinner" size={32} />
            </div>
        );
    }

    if (error) {
        return (
            <div className="cobrosError">
                <AlertCircle size={20} />
                <span>Error al cargar cobros</span>
            </div>
        );
    }

    const filteredItems = search
        ? items.filter(item =>
            item.title.toLowerCase().includes(search.toLowerCase()) ||
            item.user_email.toLowerCase().includes(search.toLowerCase())
        )
        : items;

    const pendingCount = items.filter(i => i.status === 'pending').length;
    const paidCount = items.filter(i => i.status === 'paid').length;

    return (
        <div className="cobrosContenedor">
            <div className="cobrosHeaderFiltros">
                <div className="cobrosBuscador">
                    <Search size={16} />
                    <Input
                        type="text"
                        placeholder="Buscar por título o email..."
                        value={search}
                        onChange={e => setSearch(e.target.value)}
                        variante="outline"
                        className="cobrosBuscadorInput"
                    />
                </div>
                <div className="cobrosFiltros">
                    <Button
                        variante="texto"
                        tamano="pequeno"
                        className={`cobrosFiltro ${statusFilter === '' ? 'cobrosFiltroActivo' : ''}`}
                        onClick={() => setStatusFilter('')}
                    >
                        Todos ({items.length})
                    </Button>
                    <Button
                        variante="texto"
                        tamano="pequeno"
                        className={`cobrosFiltro ${statusFilter === 'pending' ? 'cobrosFiltroActivo' : ''}`}
                        onClick={() => setStatusFilter('pending')}
                    >
                        Pendientes ({pendingCount})
                    </Button>
                    <Button
                        variante="texto"
                        tamano="pequeno"
                        className={`cobrosFiltro ${statusFilter === 'paid' ? 'cobrosFiltroActivo' : ''}`}
                        onClick={() => setStatusFilter('paid')}
                    >
                        Pagados ({paidCount})
                    </Button>
                </div>
            </div>

            {filteredItems.length === 0 ? (
                <div className="cobrosVacioDetalle">
                    <ReceiptText size={32} />
                    <p>No hay cobros con este filtro</p>
                </div>
            ) : (
                <div className="cobrosLista">
                    {filteredItems.map(item => {
                        const isPending = item.status === 'pending';
                        const isPaid = item.status === 'paid';
                        const isToggling = toggleStatusMut.isPending && toggleStatusMut.variables?.itemId === item.id;

                        return (
                            <Tarjeta
                                key={item.id}
                                className={`cobrosItem ${isPending ? 'cobrosItemPendiente' : 'cobrosItemPagado'}`}
                            >
                                <div className="cobrosItemInfo">
                                    <div className="cobrosItemTipo">
                                        {item.resource_type === 'hosting' ? 'Hosting' : item.resource_type === 'domain' ? 'Dominio' : 'Servicio'}
                                    </div>
                                    <div className="cobrosItemTitulo">{item.title}</div>
                                    {item.description && (
                                        <div className="cobrosItemDescripcion">{item.description}</div>
                                    )}
                                    <div className="cobrosItemMeta">
                                        <span className="cobrosItemEmail">{item.user_email}</span>
                                        <span className="cobrosItemFecha">
                                            Vence: {formatDate(item.grace_period_ends_at)}
                                        </span>
                                        {isPaid && item.paid_at && (
                                            <span className="cobrosItemPagadoEn">
                                                Pagado: {formatDate(item.paid_at)}
                                            </span>
                                        )}
                                    </div>
                                </div>
                                <div className="cobrosItemPrecio">
                                    {formatMoney(item.amount_cents, item.currency)}
                                    <span className="cobrosItemPeriodo">
                                        {item.billing_period === 'month' ? '/mes' : item.billing_period === 'year' ? '/año' : ''}
                                    </span>
                                </div>
                                <div className="cobrosItemEstado">
                                    {isPaid ? (
                                        <span className="cobrosBadge cobrosBadgePagado">
                                            <CheckCircle size={14} /> Pagado
                                        </span>
                                    ) : (
                                        <span className="cobrosBadge cobrosBadgePendiente">
                                            <Circle size={14} /> Pendiente
                                        </span>
                                    )}
                                </div>
                                <Button
                                    type="button"
                                    variante={isPending ? 'primario' : 'outline'}
                                    tamano="pequeno"
                                    disabled={isToggling}
                                    onClick={() => toggleStatusMut.mutate({
                                        itemId: item.id,
                                        newStatus: isPending ? 'paid' : 'pending',
                                    })}
                                >
                                    {isToggling ? (
                                        <Loader2 size={14} className="cobrosSpinnerInline" />
                                    ) : isPending ? (
                                        'Marcar pagado'
                                    ) : (
                                        'Marcar pendiente'
                                    )}
                                </Button>
                            </Tarjeta>
                        );
                    })}
                </div>
            )}
        </div>
    );
}
