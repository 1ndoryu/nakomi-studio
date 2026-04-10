/**
 * Componente: SeccionMetodosPago
 * Gestion de tarjetas de credito y direccion de facturacion.
 * [094A-15] Se elimino la intro redundante para dejar la seccion mas directa.
 * [104A-14] Tarjetas guardadas ya conectadas a Stripe SetupIntent + backend.
 * La direccion sigue informativa hasta que exista almacenamiento persistente en el panel.
 */
import React from 'react';

import {usePaymentMethods} from '../../hooks/usePaymentMethods';
import {Button} from '../ui/Button';

import {AddPaymentMethodModal} from './AddPaymentMethodModal';

import './SeccionMetodosPago.css';

function formatCardBrand(brand: string): string {
    if (!brand) {
        return 'Tarjeta';
    }

    return brand.charAt(0).toUpperCase() + brand.slice(1);
}

export const SeccionMetodosPago: React.FC = () => {
    const {
        paymentMethods,
        isLoading,
        loadError,
        deletingId,
        isAddModalOpen,
        reloadPaymentMethods,
        openAddModal,
        closeAddModal,
        handlePaymentMethodCreated,
        handleDeletePaymentMethod,
    } = usePaymentMethods();

    return (
        <div className="metodosPagoSeccion">
            <div className="metodosPagoHeader">
                <div className="metodosPagoTituloGrupo">
                    <h3 className="metodosPagoTitulo">Tarjetas guardadas</h3>
                    <p className="metodosPagoDescripcion">
                        Agrega una tarjeta una vez y mantenla lista para futuros pagos en el panel.
                    </p>
                </div>
                <Button variante="outline" tamano="pequeno" onClick={openAddModal}>
                    Agregar tarjeta
                </Button>
            </div>

            <div className="tarjetasLista">
                {isLoading && <div className="metodosPagoEstado">Cargando tus tarjetas...</div>}

                {!isLoading && loadError && (
                    <div className="metodosPagoEstado metodosPagoEstado--error">
                        <p className="metodosPagoEstadoTexto">{loadError}</p>
                        <Button variante="outline" tamano="pequeno" onClick={() => void reloadPaymentMethods()}>
                            Reintentar
                        </Button>
                    </div>
                )}

                {!isLoading && !loadError && paymentMethods.length === 0 && (
                    <div className="tarjetaVacia">
                        <div className="tarjetaVaciaIcono">
                            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                                <rect x="1" y="4" width="22" height="16" rx="2" ry="2" />
                                <line x1="1" y1="10" x2="23" y2="10" />
                            </svg>
                        </div>
                        <p className="tarjetaVaciaTexto">No tienes tarjetas registradas todavia.</p>
                        <Button variante="outline" tamano="pequeno" onClick={openAddModal}>Agregar tarjeta</Button>
                    </div>
                )}

                {!isLoading && !loadError && paymentMethods.map((paymentMethod) => (
                    <article key={paymentMethod.id} className="metodoPagoCard">
                        <div className="metodoPagoResumen">
                            <div className="metodoPagoMarca">{formatCardBrand(paymentMethod.brand)}</div>
                            <div className="metodoPagoNumero">•••• {paymentMethod.last_four}</div>
                            <div className="metodoPagoExpiracion">
                                Expira {String(paymentMethod.exp_month).padStart(2, '0')}/{paymentMethod.exp_year}
                            </div>
                        </div>
                        <div className="metodoPagoMeta">
                            {paymentMethod.is_default && (
                                <span className="metodoPagoBadge">Predeterminada</span>
                            )}
                            <Button
                                variante="texto"
                                tamano="pequeno"
                                onClick={() => void handleDeletePaymentMethod(paymentMethod.id)}
                                disabled={deletingId === paymentMethod.id}
                            >
                                {deletingId === paymentMethod.id ? 'Eliminando...' : 'Eliminar'}
                            </Button>
                        </div>
                    </article>
                ))}
            </div>

            <section className="metodosPagoInfo">
                <h3 className="metodosPagoSubtitulo">Direccion de facturacion</h3>
                <p className="metodosPagoInfoTexto">
                    El panel todavia no guarda una direccion de facturacion propia. Cuando pagues, Stripe te pedira los datos necesarios dentro del checkout seguro.
                </p>
            </section>

            <AddPaymentMethodModal
                open={isAddModalOpen}
                onClose={closeAddModal}
                onSaved={handlePaymentMethodCreated}
            />
        </div>
    );
};
