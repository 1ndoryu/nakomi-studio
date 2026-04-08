/* [074A-63] Sub-componentes de SeccionHosting extraidos para cumplir limite de 300 lineas.
 * HostingCard, CreateHostingForm, EventsPanel, EventItem. */

import React, {useState, useCallback} from 'react';
import {useQuery} from '@tanstack/react-query';
import {Server, History} from 'lucide-react';
import {
    apiListHostingEvents,
    HOSTING_PLAN_LABELS,
    HOSTING_STATUS_LABELS,
    HOSTING_STATUS_CLASS,
    type HostingSubscription,
    type HostingEvent,
    type CreateHostingRequest,
} from '../../api/hosting';
import {Input} from '../ui/Input';
import {Select} from '../ui/Select';
import {Button} from '../ui/Button';
import {MenuContextual, type MenuContextualItem} from '../ui/ContextMenu';

/* [074A-57] Card de hosting — layout similar a ordenCard de proyectosLista
 * [074A-63] Titulo = dominio o nombre del hosting (identidad unica), plan va debajo. */
export function HostingCard({
    sub,
    isAdmin,
    onStatusChange,
    onViewEvents,
}: {
    sub: HostingSubscription;
    isAdmin: boolean;
    onStatusChange: (status: string) => void;
    onViewEvents: () => void;
}) {
    const [menuOpen, setMenuOpen] = useState(false);

    const statusItems: MenuContextualItem[] = (['pending', 'provisioning', 'active', 'suspended', 'cancelled'] as const)
        .filter(s => s !== sub.status)
        .map(s => ({
            id: s,
            label: HOSTING_STATUS_LABELS[s] || s,
            onSelect: () => onStatusChange(s),
            danger: s === 'suspended' || s === 'cancelled',
        }));

    return (
        <div className="hostingCard">
            <div className="hostingCardIcono">
                <Server size={28} strokeWidth={1.4} />
            </div>
            <div className="hostingCardBody">
                <div className="hostingCardHeader">
                    <h3 className="hostingCardTitulo">
                        {sub.domain || sub.client_name || HOSTING_PLAN_LABELS[sub.plan] || sub.plan}
                    </h3>
                    <span className={`hostingStatus ${HOSTING_STATUS_CLASS[sub.status] || ''}`}>
                        {HOSTING_STATUS_LABELS[sub.status] || sub.status}
                    </span>
                </div>
                <span className="hostingCardPlan">
                    {HOSTING_PLAN_LABELS[sub.plan] || sub.plan}
                </span>
                {isAdmin && (
                    <span className="hostingCardCliente">
                        {sub.client_name} · {sub.client_email}
                    </span>
                )}
                <div className="hostingCardFooter">
                    <span className="hostingCardPrecio">
                        ${(sub.monthly_price_cents / 100).toFixed(0)}/mes
                    </span>
                    <div className="hostingCardAcciones">
                        <Button
                            variante="texto"
                            tamano="pequeno"
                            type="button"
                            onClick={onViewEvents}
                            title="Ver historial"
                        >
                            <History size={16} />
                        </Button>
                        {isAdmin && (
                            <MenuContextual
                                abierto={menuOpen}
                                onToggle={() => setMenuOpen(prev => !prev)}
                                onCerrar={() => setMenuOpen(false)}
                                items={statusItems}
                                ariaLabel="Cambiar status de suscripción"
                            />
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}

export function CreateHostingForm({
    onSubmit,
    submitting,
}: {
    onSubmit: (req: CreateHostingRequest) => void;
    submitting: boolean;
}) {
    const [form, setForm] = useState<CreateHostingRequest>({
        client_name: '',
        client_email: '',
        plan: 'basico',
        domain: '',
    });

    const handleSubmit = useCallback(
        (e: React.FormEvent) => {
            e.preventDefault();
            if (!form.client_name.trim() || !form.client_email.trim()) return;
            onSubmit({
                ...form,
                domain: form.domain?.trim() || undefined,
            });
        },
        [form, onSubmit],
    );

    return (
        <form className="hostingFormCrear" onSubmit={handleSubmit}>
            <h3>Nueva suscripción de hosting</h3>
            <Input
                type="text"
                placeholder="Nombre del cliente"
                value={form.client_name}
                onChange={e => setForm(prev => ({...prev, client_name: e.target.value}))}
            />
            <Input
                type="email"
                placeholder="Email del cliente"
                value={form.client_email}
                onChange={e => setForm(prev => ({...prev, client_email: e.target.value}))}
            />
            <Select
                className="hostingSelect"
                value={form.plan}
                onChange={e => setForm(prev => ({...prev, plan: e.target.value}))}
            >
                <option value="basico">Básico ($15/mes)</option>
                <option value="pro">Profesional ($35/mes)</option>
                <option value="ecommerce">E-commerce ($60/mes)</option>
                <option value="custom">Custom (cotización)</option>
            </Select>
            <Input
                type="text"
                placeholder="Dominio (opcional)"
                value={form.domain || ''}
                onChange={e => setForm(prev => ({...prev, domain: e.target.value}))}
            />
            <Button type="submit" className="hostingBtnSubmit" disabled={submitting}>
                {submitting ? 'Creando...' : 'Crear suscripción'}
            </Button>
        </form>
    );
}

function EventItem({event}: {event: HostingEvent}) {
    return (
        <div className="hostingEvento">
            <span className="hostingEventoTipo">{event.event_type}</span>
            <span className="hostingEventoFecha">
                {new Date(event.created_at).toLocaleString('es')}
            </span>
            {event.details && (
                <pre className="hostingEventoDetalles">
                    {JSON.stringify(event.details, null, 2)}
                </pre>
            )}
        </div>
    );
}

export function EventsPanel({
    subscriptionId,
    clientName,
}: {
    subscriptionId: string;
    clientName: string;
}) {
    const {data: events = [], isLoading: cargando} = useQuery({
        queryKey: ['hosting-events', subscriptionId],
        queryFn: () => apiListHostingEvents(subscriptionId),
    });

    return (
        <div className="hostingEventos">
            <h3>Historial: {clientName}</h3>
            {cargando ? (
                <p>Cargando...</p>
            ) : events.length === 0 ? (
                <p className="hostingEventosVacio">Sin eventos registrados</p>
            ) : (
                <div className="hostingEventosLista">
                    {events.map(ev => (
                        <EventItem key={ev.id} event={ev} />
                    ))}
                </div>
            )}
        </div>
    );
}
