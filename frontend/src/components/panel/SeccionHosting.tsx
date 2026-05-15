/* [054A-2] Sección Hosting del panel.
 * Dashboard de suscripciones de hosting: lista, status, acciones.
 * [064A-32] Ahora role-aware: admin ve todo + crear/cambiar status, cliente solo ve sus suscripciones.
 * [054A-17] Corregidos: <button>→<Button>, inline styles→CSS classes, overlay→MenuContextual.
 * [074A-63] Tabs Activos/Inactivos como en SeccionProyectos. Titulo de card = dominio o nombre del hosting.
 *           Logica de estado extraida a useSeccionHosting. Sub-componentes en HostingSubComponents.
 * [084A-24] Tab "Servidores" para admin: muestra VPS reales de Contabo.
 * [304A-1] Tabs 'Despliegues VPS2' y 'Contabo VPS' movidos a SeccionInfraestructura (sidebar separado). */

import React from 'react';
import {Server, Plus} from 'lucide-react';
import {useSeccionHosting} from '../../hooks/useSeccionHosting';
import {Modal} from '../ui/Modal';
import {Button} from '../ui/Button';
import {HostingCard} from './HostingSubComponents';
import {CreateHostingForm} from './HostingCreateForm';
import {HostingDetalle} from './HostingDetalle';
import {HostingPlanSelector} from './HostingPlanSelector';
import './SeccionHosting.css';

export const SeccionHosting: React.FC = () => {
    const {
        subscriptions,
        isLoading,
        isAdmin,
        tabActiva,
        setTabActiva,
        activos,
        inactivos,
        listaActual,
        showCreateModal,
        setShowCreateModal,
        selectedHostingId,
        handleSelectHosting,
        handleVolverHosting,
        createMutation,
        statusMutation,
        updateMutation,
        deleteMutation,
        cancelMutation,
        checkoutMutation,
        subscribeMutation,
        provisionMutation,
        restartMutation,
        stopMutation,
        startMutation,
        assignMutation,
        adminCheckoutMutation,
    } = useSeccionHosting();

    if (isLoading) {
        return (
            <div className="hostingLoading">
                <Server size={32} strokeWidth={1.2} />
                <p>Cargando suscripciones...</p>
            </div>
        );
    }

    /* [094A-2] Si hay un hosting seleccionado, mostrar la vista de detalle */
    if (selectedHostingId) {
        return (
            <HostingDetalle
                hostingId={selectedHostingId}
                isAdmin={isAdmin}
                onVolver={handleVolverHosting}
                onPlanChange={(plan, domain) =>
                    updateMutation.mutate({id: selectedHostingId, req: {plan, domain}})
                }
                planChangeLoading={updateMutation.isPending}
                onProvision={() => provisionMutation.mutate(selectedHostingId)}
                provisionLoading={provisionMutation.isPending}
                onRestart={() => restartMutation.mutate(selectedHostingId)}
                restartLoading={restartMutation.isPending}
                onStop={() => stopMutation.mutate(selectedHostingId)}
                stopLoading={stopMutation.isPending}
                onStart={() => startMutation.mutate(selectedHostingId)}
                startLoading={startMutation.isPending}
            />
        );
    }

    return (
        <div className="hostingContenedor">
            <div className="hostingAcciones">
                {isAdmin ? (
                    <Button
                        variante="primario"
                        tamano="pequeno"
                        className="hostingBtnCrear"
                        onClick={() => setShowCreateModal(true)}
                        type="button"
                    >
                        <Plus size={16} /> Nueva suscripción
                    </Button>
                ) : (
                    <Button
                        variante="primario"
                        tamano="pequeno"
                        className="hostingBtnCrear"
                        onClick={() => setShowCreateModal(true)}
                        type="button"
                    >
                        <Plus size={16} /> Contratar WordPress Hosting
                    </Button>
                )}
            </div>

            {/* [074A-63] Tabs como en proyectos: Activos / Inactivos
             * [074A-64] Mismo variante="texto" y type="button" que proyectosTabs */}
            <div className="hostingTabs">
                <Button
                    type="button"
                    variante="texto"
                    className={`hostingTab ${tabActiva === 'activos' ? 'hostingTab--activa' : ''}`}
                    onClick={() => setTabActiva('activos')}
                >
                    Activos ({activos.length})
                </Button>
                <Button
                    type="button"
                    variante="texto"
                    className={`hostingTab ${tabActiva === 'inactivos' ? 'hostingTab--activa' : ''}`}
                    onClick={() => setTabActiva('inactivos')}
                >
                    Inactivos ({inactivos.length})
                </Button>
            </div>

            {/* [084A-24] Contenido condicional por tab */}
            {/* [304A-1] 'deployments' y 'servidores' movidos a SeccionInfraestructura */}
            {subscriptions.length === 0 ? (
                <div className="hostingVacio">
                    <Server size={48} strokeWidth={1.2} />
                    <p>Sin suscripciones de WordPress hosting</p>
                </div>
            ) : listaActual.length === 0 ? (
                <div className="hostingVacio">
                    <Server size={32} strokeWidth={1.2} />
                    <p>Sin suscripciones {tabActiva}</p>
                </div>
            ) : (
                <div className="hostingLista">
                    {listaActual.map(sub => (
                        <HostingCard
                            key={sub.id}
                            sub={sub}
                            isAdmin={isAdmin}
                            onSelect={() => handleSelectHosting(sub.id)}
                            onStatusChange={(status) =>
                                statusMutation.mutate({id: sub.id, status})
                            }
                            onUpdate={(req) =>
                                updateMutation.mutate({id: sub.id, req})
                            }
                            onDelete={() => deleteMutation.mutate(sub.id)}
                            onCancel={() => cancelMutation.mutate(sub.id)}
                            /* [304A-3] Admin copia link de pago; cliente es redirigido */
                            onCheckout={() =>
                                isAdmin
                                    ? adminCheckoutMutation.mutate(sub.id)
                                    : checkoutMutation.mutate(sub.id)
                            }
                            checkoutLoading={
                                isAdmin ? adminCheckoutMutation.isPending : checkoutMutation.isPending
                            }
                            /* [304A-3] Admin asigna hosting a cliente por email */
                            onAssign={isAdmin ? (email) => assignMutation.mutate({id: sub.id, email}) : undefined}
                            assignLoading={assignMutation.isPending}
                        />
                    ))}
                </div>
            )}

            {/* Modal: admin ve form de crear, cliente ve selector de planes */}
            {showCreateModal && (
                <Modal abierto={showCreateModal} onCerrar={() => setShowCreateModal(false)} className={!isAdmin ? 'modalPlanSelector' : undefined}>
                    {isAdmin ? (
                        <CreateHostingForm
                            onSubmit={(req) => createMutation.mutate(req)}
                            submitting={createMutation.isPending}
                        />
                    ) : (
                        <HostingPlanSelector
                            onSelect={(plan, domain) => subscribeMutation.mutate({plan, domain})}
                            loading={subscribeMutation.isPending}
                        />
                    )}
                </Modal>
            )}

        </div>
    );
};
