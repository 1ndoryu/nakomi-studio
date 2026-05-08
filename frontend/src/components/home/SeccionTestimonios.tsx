/* Componente: SeccionTestimonios
 * Muestra un carrusel interactivo de testimonios.
 * Datos centralizados en data/testimonios.ts (DRY).
 * Incluye botón para escribir un testimonio (abre modal).
 * sentinel-disable-file inline-style-prohibido: CSS custom properties dinámicas (--testimonio-idx,
 * --testimonio-drag, --testimonio-transition) se asignan via style={{}}, no hay alternativa CSS pura. */
import React, {useState} from 'react';
import './SeccionTestimonios.css';
import {useCarruselInfinito} from '../../hooks/useCarruselInfinito';
import {SeccionHeader} from '../ui/SeccionHeader';
import {TESTIMONIOS} from '../../data/testimonios';
import {ModalTestimonio} from './ModalTestimonio';
import OptimizedImage from '../ui/OptimizedImage';
import {Button} from '../ui/Button';

export const SeccionTestimonios: React.FC = () => {
    const items = TESTIMONIOS;
    const itemsVisuales = [...items, ...items];
    const [modalAbierto, setModalAbierto] = useState(false);

    /* [074A-marketing] Hook siempre llamado (Rules of Hooks) con totalItems seguro en 0.
     * El guard vacío viene DESPUÉS de todos los hooks. */
    const {indiceActual, conTransicion, dragOffset, handlers} = useCarruselInfinito({
        totalItems: items.length,
        tiempoEspera: 5000,
        tiempoTransicion: 500
    });

    if (items.length === 0) return null;

    return (
        <section className="seccionTestimonios">
            <div className="testimoniosContenedor">
                <div className="testimoniosHeader">
                    <SeccionHeader titulo="Testimonials" />
                    <Button variante="outline" tamano="pequeno" className="testimoniosAccionEscribir" onClick={() => setModalAbierto(true)} type="button">
                        Escribir un comentario
                    </Button>
                </div>

                <div
                    className="testimoniosCarruselWindow"
                    {...handlers}
                >
                    <div
                        className="testimoniosPista"
                        /* eslint-disable-next-line react/forbid-dom-props -- valores dinamicos de JS, necesarios para CSS custom properties */
                        style={{
                            '--testimonio-idx': indiceActual,
                            '--testimonio-drag': `${dragOffset}px`,
                            '--testimonio-transition': conTransicion ? 'transform 0.5s cubic-bezier(0.25, 1, 0.5, 1)' : 'none',
                        } as React.CSSProperties}>
                        {itemsVisuales.map((item, index) => (
                            <article key={`${item.id}-${index}`} className="testimonioCard">
                                <p className="testimonioTexto">{item.texto}</p>
                                <div className="testimonioAutor">
                                    {item.avatar && <OptimizedImage src={item.avatar} alt={item.autor} className="autorAvatar" sizes="48px" />}
                                    <div className="autorInfo">
                                        <span className="autorNombre">{item.autor}</span>
                                        <span className="autorCargo">{item.cargo}</span>
                                    </div>
                                </div>
                            </article>
                        ))}
                    </div>
                </div>
            </div>

            <ModalTestimonio abierto={modalAbierto} onCerrar={() => setModalAbierto(false)} />
        </section>
    );
};
