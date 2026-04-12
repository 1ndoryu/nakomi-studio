/**
 * Componente: SeccionHeroProyecto
 * [124A-PROJ1] Rediseño estilo kontrapunkt: hero + portada full-width + case introduction.
 * Hero: izquierda descripción breve, derecha título H1.
 * Portada: featured_image a 100% ancho.
 * Case intro: izquierda Client/Industry/Deliveries/Links, derecha descripción completa.
 */
import type {EnlaceProyecto} from '../../types/contenido';
import OptimizedImage from '../ui/OptimizedImage';
import './SeccionHeroProyecto.css';

interface SeccionHeroProyectoProps {
    titulo: string;
    descripcion?: string;
    cliente?: string;
    categorias?: string;
    tecnologias?: string[];
    enlaces?: EnlaceProyecto[];
    imagenPortada?: string;
}

export const SeccionHeroProyecto = ({titulo, descripcion, cliente = '', categorias = '', tecnologias = [], enlaces = [], imagenPortada}: SeccionHeroProyectoProps): JSX.Element => {
    return (
        <>
            {/* Hero: texto breve izq + título der */}
            <section className="proyectoHero">
                <div className="proyectoHeroContenido">
                    <div className="proyectoHeroIzquierda">
                        {cliente && <span className="proyectoHeroCliente">{cliente}</span>}
                    </div>
                    <div className="proyectoHeroDerecha">
                        <h1 className="proyectoHeroTitulo">{titulo}</h1>
                    </div>
                </div>
            </section>

            {/* Portada full-width */}
            {imagenPortada && (
                <section className="proyectoPortada">
                    <OptimizedImage
                        src={imagenPortada}
                        alt={titulo}
                        className="proyectoPortadaImagen"
                        sizes="100vw"
                    />
                </section>
            )}

            {/* Case introduction: meta izq + descripción der */}
            <section className="proyectoCaseIntro">
                <div className="proyectoCaseIntroContenido">
                    <div className="proyectoCaseIntroMeta">
                        {cliente && (
                            <div className="proyectoCaseIntroBloque">
                                <span className="proyectoCaseIntroLabel">Client</span>
                                <span className="proyectoCaseIntroValor">{cliente}</span>
                            </div>
                        )}
                        {categorias && (
                            <div className="proyectoCaseIntroBloque">
                                <span className="proyectoCaseIntroLabel">Industry</span>
                                <span className="proyectoCaseIntroValor">{categorias}</span>
                            </div>
                        )}
                        {tecnologias.length > 0 && (
                            <div className="proyectoCaseIntroBloque">
                                <span className="proyectoCaseIntroLabel">Deliveries</span>
                                <div className="proyectoCaseIntroLista">
                                    {tecnologias.map(tech => (
                                        <span key={tech}>{tech}</span>
                                    ))}
                                </div>
                            </div>
                        )}
                        {enlaces.length > 0 && (
                            <div className="proyectoCaseIntroBloque">
                                <span className="proyectoCaseIntroLabel">Links</span>
                                <div className="proyectoCaseIntroLista">
                                    {enlaces.map(enlace => (
                                        <a key={enlace.url} href={enlace.url} target="_blank" rel="noopener noreferrer" className="proyectoCaseIntroEnlace">
                                            {enlace.etiqueta || enlace.tipo}
                                        </a>
                                    ))}
                                </div>
                            </div>
                        )}
                    </div>
                    {descripcion && (
                        <div className="proyectoCaseIntroDescripcion">
                            <p>{descripcion}</p>
                        </div>
                    )}
                </div>
            </section>
        </>
    );
};
