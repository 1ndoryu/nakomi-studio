/* [054A-2] Sección Hosting del panel.
 * Dashboard de suscripciones de hosting: lista, status, acciones.
 * [064A-32] Ahora role-aware: admin ve todo + crear/cambiar status, cliente solo ve sus suscripciones.
 * [054A-17] Corregidos: <button>→<Button>, inline styles→CSS classes, overlay→MenuContextual. */

import React, {useState, useCallback} from 'react';
import {useQuery, useMutation, useQueryClient} from '@tanstack/react-query';
import {Server, Plus, History} from 'lucide-react';
import {
    apiListHostingSubscriptions,
    apiCreateHostingSubscription,
    apiUpdateHostingStatus,
    apiListHostingEvents,
    HOSTING_PLAN_LABELS,
    HOSTING_STATUS_LABELS,
    HOSTING_STATUS_CLASS,
    type HostingSubscription,
    type HostingEvent,
    type CreateHostingRequest,
} from '../../api/hosting';
import {toast} from '../../stores/toastStore';
import {useAuthStore} from '../../stores/authStore';
import {Modal} from '../ui/Modal';
import {Input} from '../ui/Input';
import {Select} from '../ui/Select';
import {Button} from '../ui/Button';
import {MenuContextual, type MenuContextualItem} from '../ui/ContextMenu';
import './SeccionHosting.css';

export const SeccionHosting: React.FC = () => {
    const queryClient = useQueryClient();
    const [showCreateModal, setShowCreateModal] = useState(false);
    const [selectedSub, setSelectedSub] = useState<HostingSubscription | null>(null);
    const [showEvents, setShowEvents] = useState(false);

    /* [064A-32] Detectar rol para ocultar features admin
     * [074A-55] effectiveRole en query key para invalidar cache al cambiar rol. */
    const effectiveRole = useAuthStore(s => s.user?.effectiveRole) ?? 'client';
    const isAdmin = effectiveRole === 'admin';

    const hostingKey = ['hosting-subscriptions', effectiveRole] as const;

    const {data: subscriptions = [], isLoading} = useQuery({
        queryKey: hostingKey,
        queryFn: apiListHostingSubscriptions,
    });

    const createMutation = useMutation({
        mutationFn: (req: CreateHostingRequest) => apiCreateHostingSubscription(req),
        onSuccess: () => {
            queryClient.invalidateQueries({queryKey: hostingKey});
            toast.success('Suscripción creada');
            setShowCreateModal(false);
        },
        onError: () => toast.error('Error al crear suscripción'),
    });

    const statusMutation = useMutation({
        mutationFn: ({id, status}: {id: string; status: string}) =>
            apiUpdateHostingStatus(id, status),
        onSuccess: () => {
            queryClient.invalidateQueries({queryKey: hostingKey});
            toast.success('Status actualizado');
        },
        onError: () => toast.error('Error al actualizar status'),
    });

    if (isLoading) {
        return (
            <div className="hostingLoading">
                <Server size={32} strokeWidth={1.2} />
                <p>Cargando suscripciones...</p>
            </div>
        );
    }

    return (
        <div className="hostingContenedor">
            <div className="hostingHeader">
                <h2>Hosting</h2>
                {isAdmin && (
                    <Button
                        variante="primario"
                        tamano="pequeno"
                        className="hostingBtnCrear"
                        onClick={() => setShowCreateModal(true)}
                        type="button"
                    >
                        <Plus size={16} /> Nueva suscripción
                    </Button>
                )}
            </div>

            {subscriptions.length === 0 ? (
                <div className="hostingVacio">
                    <Server size={48} strokeWidth={1.2} />
                    <p>Sin suscripciones de hosting</p>
                </div>
            ) : (
                /* [074A-57] Rediseño: cards en vez de tabla grid, similar a proyectosLista */
                <div className="hostingLista">
                    {subscriptions.map(sub => (
                        <HostingCard
                            key={sub.id}
                            sub={sub}
                            isAdmin={isAdmin}
                            onStatusChange={(status) =>
                                statusMutation.mutate({id: sub.id, status})
                            }
                            onViewEvents={() => {
                                setSelectedSub(sub);
                                setShowEvents(true);
                            }}
                        />
                    ))}
                </div>
            )}

            {/* Modal crear suscripción */}
            {showCreateModal && (
                <Modal abierto={showCreateModal} onCerrar={() => setShowCreateModal(false)}>
                    <CreateHostingForm
                        onSubmit={(req) => createMutation.mutate(req)}
                        submitting={createMutation.isPending}
                    />
                </Modal>
            )}

            {/* Modal eventos */}
            {showEvents && selectedSub && (
                <Modal abierto={showEvents} onCerrar={() => setShowEvents(false)}>
                    <EventsPanel subscriptionId={selectedSub.id} clientName={selectedSub.client_name} />
                </Modal>
            )}
        </div>
    );
};

/*    SUB-COMPONENTES */

/* [074A-57] Card de hosting — layout similar a ordenCard de proyectosLista */
function HostingCard({
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
                        {HOSTING_PLAN_LABELS[sub.plan] || sub.plan}
                    </h3>
                    <span className={`hostingStatus ${HOSTING_STATUS_CLASS[sub.status] || ''}`}>
                        {HOSTING_STATUS_LABELS[sub.status] || sub.status}
                    </span>
                </div>
                {sub.domain && (
                    <span className="hostingCardDominio">{sub.domain}</span>
                )}
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

function CreateHostingForm({
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

function EventsPanel({
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
