/* [124A-PROJ1] Galería de proyecto con layout alternante full/half.
 * Imágenes "full" (1/1) ocupan 100% ancho, "half" (1/2) se muestran en pares.
 * Grid de 2 columnas: full = span 2, half = span 1. */
import React from 'react';
import OptimizedImage from '../ui/OptimizedImage';
import type {GaleriaImagen} from '../../types/contenido';
import './SeccionGaleriaProyecto.css';

interface SeccionGaleriaProyectoProps {
    imagenes: GaleriaImagen[];
}

export const SeccionGaleriaProyecto: React.FC<SeccionGaleriaProyectoProps> = ({imagenes}) => {
    if (imagenes.length === 0) return null;

    return (
        <section className="proyectoGaleria">
            <div className="proyectoGaleriaGrid">
                {imagenes.map((img, idx) => (
                    <div
                        key={`gal-${idx}`}
                        className={`proyectoGaleriaItem ${img.layout === 'full' ? 'proyectoGaleriaItem--full' : 'proyectoGaleriaItem--half'}`}
                    >
                        <OptimizedImage
                            src={img.url}
                            alt={`Galería ${idx + 1}`}
                            className="proyectoGaleriaImagen"
                            sizes={img.layout === 'full' ? '100vw' : '50vw'}
                        />
                    </div>
                ))}
            </div>
        </section>
    );
};
