/**
 * Componente: SeccionHeroServicio
 * Descripcion: Hero para la pagina individual de servicio.
 * [064A-47] Muestra imagen grande del servicio con proporción laptop (16:10).
 */
import OptimizedImage from '../ui/OptimizedImage';

import './SeccionHeroServicio.css';

interface SeccionHeroServicioProps {
    titulo?: string;
    descripcion?: string;
    imagen?: string;
}

export const SeccionHeroServicio = ({titulo = 'Nombre del Servicio', descripcion = 'Descripcion detallada del servicio y como aportamos valor a traves de nuestra experiencia y metodologia.', imagen}: SeccionHeroServicioProps): JSX.Element => {
    return (
        <section className="heroServicio">
            <div className="heroServicioContenedor">
                <div>
                    <h1 className="heroServicioTitulo">{titulo}</h1>
                </div>
                <div className="heroServicioDescripcionWrapper">
                    <p className="heroServicioDescripcion">{descripcion}</p>
                </div>
                {imagen && (
                    <div className="heroServicioImagenWrapper">
                        <OptimizedImage
                            className="heroServicioImagen"
                            src={imagen}
                            alt={titulo}
                            loading="eager"
                        />
                    </div>
                )}
            </div>
        </section>
    );
};
