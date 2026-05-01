/* [074A-63] Sub-componentes de SeccionHosting extraidos para cumplir limite de 300 lineas.
 * HostingCard, CreateHostingForm, EventsPanel, EventItem.
 * [074A-65] HostingCard ahora incluye editar/eliminar en context menu.
 * [304A-3] Admin puede asignar hosting a cliente por email + generar link de pago. */

import React, {useState, useCallback} from 'react';
import {useQuery} from '@tanstack/react-query';
import {Server, ExternalLink, UserCheck, Link} from 'lucide-react';
import {
    apiListHostingEvents,
    HOSTING_PLAN_LABELS,
    HOSTING_STATUS_LABELS,
    HOSTING_STATUS_CLASS,
    type HostingSubscription,
    type HostingEvent,
    type CreateHostingRequest,
    type UpdateHostingRequest,
} from '../../api/hosting';
import {Modal} from '../ui/Modal';
import {Input} from '../ui/Input';
import {Select} from '../ui/Select';
import {Button} from '../ui/Button';
import {MenuContextual, type MenuContextualItem} from '../ui/ContextMenu';
import {HOSTING_PLAN_OPTIONS} from './hostingPlanOptions';

/* [074A-57] Card de hosting — layout similar a ordenCard de proyectosLista
 * [074A-63] Titulo = dominio o nombre del hosting (identidad unica), plan va debajo.
 * [074A-65] Context menu con editar/eliminar + edit modal inline.
 * [084A-4] Clientes ahora tienen context menu con Editar/Cancelar.
 * [304A-3] Admin puede asignar hosting a cliente y generar link de pago. */
export function HostingCard({
    sub,
    isAdmin,
    onStatusChange,
    onUpdate,
    onDelete,
    onCancel,
    onCheckout,
    onAssign,
    onSelect,
    checkoutLoading,
    assignLoading,
}: {
    sub: HostingSubscription;
    isAdmin: boolean;
    onStatusChange: (status: string) => void;
    onUpdate: (req: UpdateHostingRequest) => void;
    onDelete: () => void;
    onCancel: () => void;
    onCheckout: () => void;
    onAssign?: (email: string) => void;
    onSelect: () => void;
    checkoutLoading?: boolean;
    assignLoading?: boolean;
}) {
    const [menuOpen, setMenuOpen] = useState(false);
    const [editing, setEditing] = useState(false);
    const [showStatusModal, setShowStatusModal] = useState(false);
    const [showAssignModal, setShowAssignModal] = useState(false);
    const [assignEmail, setAssignEmail] = useState('');
    const [editPlan, setEditPlan] = useState(sub.plan);
    const [editDomain, setEditDomain] = useState(sub.domain || '');

    /* [084A-13] Context menu simplificado: "Cambiar estado" abre modal,
     * ya no lista cada status individualmente en el menú. */
    const menuItems: MenuContextualItem[] = [];

    if (isAdmin) {
        menuItems.push({
            id: 'change-status',
            label: 'Cambiar estado',
            onSelect: () => setShowStatusModal(true),
        });
        /* [304A-3] Asignar hosting a cliente registrado */
        if (onAssign) {
            menuItems.push({
                id: 'assign',
                label: sub.user_id ? 'Reasignar cliente' : 'Asignar a cliente',
                onSelect: () => {
                    setAssignEmail('');
                    setShowAssignModal(true);
                },
            });
        }
    }

    /* Editar: admin y cliente */
    menuItems.push({id: 'edit', label: 'Editar', onSelect: () => setEditing(true)});

    if (isAdmin) {
        menuItems.push({id: 'delete', label: 'Eliminar', onSelect: onDelete, danger: true});
    }

    /* Cancelar: disponible para cliente (y admin) si no está ya cancelada */
    if (sub.status !== 'cancelled') {
        menuItems.push({id: 'cancel', label: 'Cancelar suscripción', onSelect: onCancel, danger: true});
    }

    const handleEditSubmit = useCallback(() => {
        onUpdate({plan: editPlan, domain: editDomain.trim() || undefined});
        setEditing(false);
    }, [editPlan, editDomain, onUpdate]);

    const handleAssignSubmit = useCallback((e: React.FormEvent) => {
        e.preventDefault();
        if (!assignEmail.trim() || !onAssign) return;
        onAssign(assignEmail.trim().toLowerCase());
        setShowAssignModal(false);
    }, [assignEmail, onAssign]);

    return (
        <>
            <div className="hostingCard" role="button" tabIndex={0} onClick={onSelect} onKeyDown={e => { if (e.key === 'Enter') onSelect(); }}>
                <div className="panelCardIcono">
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
                            {/* [304A-3] Indicador visual de asignación */}
                            {sub.user_id ? (
                                <span className="hostingAsignadoBadge" title="Asignado a cuenta">
                                    <UserCheck size={11} /> vinculado
                                </span>
                            ) : (
                                <span className="hostingNoAsignadoBadge" title="Sin cuenta asignada">
                                    sin cuenta
                                </span>
                            )}
                        </span>
                    )}
                    {/* [104A-24] Recursos movidos al footer, precio eliminado */}
                    <div className="hostingCardFooter">
                        <span className="hostingCardRecurso">
                            {(sub.storage_limit_mb / 1024).toFixed(0)} GB
                        </span>
                        {sub.domain && (
                            <span className="hostingCardRecurso">
                                {sub.domain}
                            </span>
                        )}
                        {/* Enlace rápido al WordPress real — solo para hostings provisionados por nuestro sistema */}
                        {(sub.coolify_site_name?.startsWith('hosting-') && sub.server_uuid && sub.server_ip && sub.status === 'active') && (
                            <a
                                href={`http://wordpress-${sub.server_uuid}.${sub.server_ip}.sslip.io`}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="hostingCardSiteLink"
                                onClick={e => e.stopPropagation()}
                                title="Abrir WordPress"
                            >
                                <ExternalLink size={13} /> Ver sitio
                            </a>
                        )}
                        {/* [094A-2] stopPropagation evita navegar al detalle al usar acciones */}
                        <div className="hostingCardAcciones" onClick={e => e.stopPropagation()}>
                            {/* [084A-24] Botón de checkout Stripe para suscripciones pendientes (cliente) */}
                            {sub.status === 'pending' && !isAdmin && (
                                <Button
                                    type="button"
                                    variante="primario"
                                    tamano="pequeno"
                                    onClick={onCheckout}
                                    disabled={checkoutLoading}
                                >
                                    {checkoutLoading ? 'Redirigiendo…' : 'Pagar'}
                                </Button>
                            )}
                            {/* [304A-3] Admin genera link de pago para suscripciones pendientes sin cuenta */}
                            {sub.status === 'pending' && isAdmin && (
                                <Button
                                    type="button"
                                    variante="secundario"
                                    tamano="pequeno"
                                    onClick={onCheckout}
                                    disabled={checkoutLoading}
                                    title="Genera URL de Stripe para compartir con el cliente"
                                >
                                    <Link size={12} />
                                    {checkoutLoading ? 'Generando…' : 'Link de pago'}
                                </Button>
                            )}
                            {/* [084A-4] Menú visible para todos los roles, items varían por rol */}
                            <MenuContextual
                                abierto={menuOpen}
                                onToggle={() => setMenuOpen(prev => !prev)}
                                onCerrar={() => setMenuOpen(false)}
                                items={menuItems}
                                ariaLabel="Gestionar suscripción"
                            />
                        </div>
                    </div>
                </div>
            </div>

            {/* [074A-65] Modal de edición */}
            {editing && (
                <Modal abierto={editing} onCerrar={() => setEditing(false)}>
                    <div className="hostingFormCrear">
                        <h3>Editar suscripción</h3>
                        <Select
                            className="hostingSelect"
                            value={editPlan}
                            onChange={e => setEditPlan(e.target.value)}
                        >
                            {HOSTING_PLAN_OPTIONS.map(option => (
                                <option key={option.value} value={option.value}>{option.label}</option>
                            ))}
                        </Select>
                        <Input
                            type="text"
                            placeholder="Dominio (opcional)"
                            value={editDomain}
                            onChange={e => setEditDomain(e.target.value)}
                        />
                        <Button type="button" onClick={handleEditSubmit}>
                            Guardar cambios
                        </Button>
                    </div>
                </Modal>
            )}

            {/* [084A-13] Modal de cambio de estado — reemplaza los status sueltos del menú */}
            {showStatusModal && isAdmin && (
                <Modal abierto={showStatusModal} onCerrar={() => setShowStatusModal(false)}>
                    <div className="hostingFormCrear">
                        <h3 className="modalTitulo">Cambiar estado</h3>
                        <p className="hostingStatusModalSub">
                            Estado actual: <strong>{HOSTING_STATUS_LABELS[sub.status] || sub.status}</strong>
                        </p>
                        <div className="hostingStatusModalOpciones">
                            {(['pending', 'provisioning', 'active', 'suspended', 'cancelled'] as const)
                                .filter(s => s !== sub.status)
                                .map(s => (
                                    <Button
                                        key={s}
                                        variante={s === 'suspended' || s === 'cancelled' ? 'outline' : 'secundario'}
                                        type="button"
                                        onClick={() => {
                                            onStatusChange(s);
                                            setShowStatusModal(false);
                                        }}
                                    >
                                        {HOSTING_STATUS_LABELS[s] || s}
                                    </Button>
                                ))}
                        </div>
                    </div>
                </Modal>
            )}

            {/* [304A-3] Modal de asignación de hosting a cliente */}
            {showAssignModal && isAdmin && onAssign && (
                <Modal abierto={showAssignModal} onCerrar={() => setShowAssignModal(false)}>
                    <form className="hostingFormCrear" onSubmit={handleAssignSubmit}>
                        <h3 className="modalTitulo">
                            {sub.user_id ? 'Reasignar hosting' : 'Asignar hosting a cliente'}
                        </h3>
                        <p className="hostingStatusModalSub">
                            Hosting: <strong>{sub.domain || sub.client_name}</strong>
                        </p>
                        {sub.user_id && (
                            <p className="hostingStatusModalSub">
                                Actualmente asignado a: <strong>{sub.client_email}</strong>
                            </p>
                        )}
                        <Input
                            type="email"
                            placeholder="Email del cliente registrado"
                            value={assignEmail}
                            onChange={e => setAssignEmail(e.target.value)}
                            required
                        />
                        <p className="hostingFormNota">
                            El cliente debe tener cuenta registrada en el sistema.
                        </p>
                        <Button type="submit" disabled={assignLoading || !assignEmail.trim()}>
                            {assignLoading ? 'Asignando…' : 'Asignar'}
                        </Button>
                    </form>
                </Modal>
            )}
        </>
    );
}

export function CreateHostingForm({
    onSubmit,
    submitting,
    initialCoolifyName,
}: {
    onSubmit: (req: CreateHostingRequest) => void;
    submitting: boolean;
    /* [304A-3] Pre-llena coolify_site_name cuando se crea desde un despliegue huérfano */
    initialCoolifyName?: string;
}) {
    const [form, setForm] = useState<CreateHostingRequest>({
        client_name: '',
        client_email: '',
        plan: 'basico',
        domain: '',
        coolify_site_name: initialCoolifyName || '',
    });

    const handleSubmit = useCallback(
        (e: React.FormEvent) => {
            e.preventDefault();
            if (!form.client_name.trim() || !form.client_email.trim()) return;
            onSubmit({
                ...form,
                domain: form.domain?.trim() || undefined,
                coolify_site_name: form.coolify_site_name?.trim() || undefined,
            });
        },
        [form, onSubmit],
    );

    return (
        <form className="hostingFormCrear" onSubmit={handleSubmit}>
            <h3>Nueva suscripción de WordPress hosting</h3>
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
                {HOSTING_PLAN_OPTIONS.map(option => (
                    <option key={option.value} value={option.value}>{option.label}</option>
                ))}
            </Select>
            <Input
                type="text"
                placeholder="Dominio (opcional)"
                value={form.domain || ''}
                onChange={e => setForm(prev => ({...prev, domain: e.target.value}))}
            />
            {/* [304A-3] Campo visible solo cuando se vincula a un despliegue Coolify */}
            {initialCoolifyName !== undefined && (
                <Input
                    type="text"
                    placeholder="Nombre en Coolify"
                    value={form.coolify_site_name || ''}
                    onChange={e => setForm(prev => ({...prev, coolify_site_name: e.target.value}))}
                />
            )}
            <Button type="submit" className="hostingBtnSubmit" disabled={submitting}>
                {submitting ? 'Creando...' : 'Crear suscripción WordPress'}
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
