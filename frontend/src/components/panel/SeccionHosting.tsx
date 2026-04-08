/* [054A-2] Sección Hosting del panel.
 * Dashboard de suscripciones de hosting: lista, status, acciones.
 * [064A-32] Ahora role-aware: admin ve todo + crear/cambiar status, cliente solo ve sus suscripciones.
 * [054A-17] Corregidos: <button>→<Button>, inline styles→CSS classes, overlay→MenuContextual.
 * [074A-63] Tabs Activos/Inactivos como en SeccionProyectos. Titulo de card = dominio o nombre del hosting.
 *           Logica de estado extraida a useSeccionHosting. Sub-componentes en HostingSubComponents.
 * [084A-24] Tab "Servidores" para admin: muestra VPS reales de Contabo. */

import React from 'react';
import {Server, Plus} from 'lucide-react';
import {useSeccionHosting} from '../../hooks/useSeccionHosting';
import {Modal} from '../ui/Modal';
import {Button} from '../ui/Button';
import {HostingCard, CreateHostingForm} from './HostingSubComponents';
import {HostingDetalle} from './HostingDetalle';
import {VpsPanel} from './VpsPanel';
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
        setSelectedHostingId,
        createMutation,
        statusMutation,
        updateMutation,
        deleteMutation,
        cancelMutation,
        checkoutMutation,
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
                onVolver={() => setSelectedHostingId(null)}
            />
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
                {/* [084A-24] Tab de servidores solo para admin */}
                {isAdmin && (
                    <Button
                        type="button"
                        variante="texto"
                        className={`hostingTab ${tabActiva === 'servidores' ? 'hostingTab--activa' : ''}`}
                        onClick={() => setTabActiva('servidores')}
                    >
                        Servidores
                    </Button>
                )}
            </div>

            {/* [084A-24] Contenido condicional por tab */}
            {tabActiva === 'servidores' && isAdmin ? (
                <VpsPanel />
            ) : subscriptions.length === 0 ? (
                <div className="hostingVacio">
                    <Server size={48} strokeWidth={1.2} />
                    <p>Sin suscripciones de hosting</p>
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
                            onSelect={() => setSelectedHostingId(sub.id)}
                            onStatusChange={(status) =>
                                statusMutation.mutate({id: sub.id, status})
                            }
                            onUpdate={(req) =>
                                updateMutation.mutate({id: sub.id, req})
                            }
                            onDelete={() => deleteMutation.mutate(sub.id)}
                            onCancel={() => cancelMutation.mutate(sub.id)}
                            onCheckout={() => checkoutMutation.mutate(sub.id)}
                            checkoutLoading={checkoutMutation.isPending}
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

        </div>
    );
};
