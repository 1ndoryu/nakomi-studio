/**
 * Componente: SeccionGaleriaServicio
 * Descripcion: Galeria de imagenes con scroll infinito, efecto de arrastre y formato 3:4.
 * [044A-3] Acepta imagenes como prop; si no se pasan, usa useImagenes().
 */
import React from 'react';
import {useCarruselInfinito} from '../../hooks/useCarruselInfinito';
import {useImagenes} from '../../hooks/useImagenes';
import './SeccionGaleriaServicio.css';

interface SeccionGaleriaServicioProps {
    imagenes?: string[];
}

export const SeccionGaleriaServicio: React.FC<SeccionGaleriaServicioProps> = ({imagenes: imagenesProp}) => {
    const {imagenes: imagenesHook} = useImagenes();
    const imagenes = (imagenesProp && imagenesProp.length > 0) ? imagenesProp : imagenesHook;

    /* [074A-59] Hook movido antes del early return para no violar Rules of Hooks.
     * Cuando galeria pasa de [] a [urls] entre renders, el conteo de hooks debe ser estable. */
    const {indiceActual, conTransicion, dragOffset, handlers} = useCarruselInfinito({
        totalItems: imagenes.length || 1,
        tiempoEspera: 6000,
        tiempoTransicion: 800
    });

    if (imagenes.length === 0) return null;

    // Duplicamos las imagenes para efecto infinito
    const itemsTotales = [...imagenes, ...imagenes];

    return (
        <section className="seccionGaleriaServicio">
            <div className="galeriaContenedorPrincipal">
                <div
                    className="galeriaPista"
                    {...handlers}
                    style={
                        {
                            transform: `translateX(calc( -1 * (var(--galeria-item-width) + var(--galeria-item-gap)) * ${indiceActual} + ${dragOffset}px))`,
                            transition: conTransicion ? 'transform 800ms cubic-bezier(0.25, 1, 0.5, 1)' : 'none',
                            cursor: 'grab',
                            touchAction: 'pan-y'
                        } as React.CSSProperties
                    }>
                    {itemsTotales.map((src, index) => (
                        <div key={`img-${index}`} className="galeriaItem">
                            <div className="galeriaImagenWrapper">
                                <img src={src} alt={`Galeria servicio ${index + 1}`} className="galeriaImagen" draggable={false} loading="lazy" />
                            </div>
                        </div>
                    ))}
                </div>
            </div>
        </section>
    );
};
