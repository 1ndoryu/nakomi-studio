/**
 * Componente: SeccionHeroProyecto
 * Descripcion: Hero para la página individual de proyecto.
 * Muestra meta (cliente + categorías), título, descripción, tecnologías y enlaces.
 * [114A-15] Extraído de ProyectoIndividualIsland para consistencia con SeccionHeroServicio.
 */
import {
    GitBranch, Globe, ExternalLink, Package,
    AtSign, Briefcase, PlayCircle, Camera, Share2,
    PenTool, CircleDot, Mail, FileText, Code, Play,
    Smartphone, MonitorPlay, Music, BookOpen,
    ShoppingBag, Palette, Pen, MessageCircle, Hash
} from 'lucide-react';
import type {EnlaceProyecto} from '../../types/contenido';
import './SeccionHeroProyecto.css';

/* [064A-8] Icono según tipo de enlace — ampliado en 154A-10 con IconPickerEnlace */
const ICONOS_ENLACE: Record<string, typeof GitBranch> = {
    github: GitBranch,
    git: GitBranch,
    web: Globe,
    npm: Package,
    demo: ExternalLink,
    figma: PenTool,
    dribbble: CircleDot,
    behance: Palette,
    twitter: AtSign,
    linkedin: Briefcase,
    youtube: PlayCircle,
    instagram: Camera,
    facebook: Share2,
    discord: MessageCircle,
    slack: Hash,
    email: Mail,
    docs: FileText,
    api: Code,
    playstore: Play,
    appstore: Smartphone,
    video: MonitorPlay,
    podcast: Music,
    blog: BookOpen,
    tienda: ShoppingBag,
    otro: Pen,
};

interface SeccionHeroProyectoProps {
    titulo: string;
    descripcion?: string;
    cliente?: string;
    categorias?: string;
    tecnologias?: string[];
    enlaces?: EnlaceProyecto[];
}

export const SeccionHeroProyecto = ({titulo, descripcion, cliente = '', categorias = '', tecnologias = [], enlaces = []}: SeccionHeroProyectoProps): JSX.Element => {
    return (
        <section className="proyectoHero">
            <div className="proyectoHeroContenido">
                <div className="proyectoHeroMeta">
                    <span className="proyectoHeroCliente">{cliente}</span>
                    <span className="proyectoHeroCategorias">{categorias}</span>
                </div>
                <h1 className="proyectoHeroTitulo">{titulo}</h1>
                {descripcion && <p className="proyectoHeroDescripcion">{descripcion}</p>}

                {/* [064A-8] Detalles técnicos: tecnologías y enlaces */}
                {(tecnologias.length > 0 || enlaces.length > 0) && (
                    <div className="proyectoHeroDetalles">
                        {tecnologias.length > 0 && (
                            <div className="proyectoHeroTecnologias">
                                {tecnologias.map(tech => (
                                    <span key={tech} className="proyectoHeroTechTag">{tech}</span>
                                ))}
                            </div>
                        )}
                        {enlaces.length > 0 && (
                            <div className="proyectoHeroEnlaces">
                                {enlaces.map(enlace => {
                                    const Icono = ICONOS_ENLACE[enlace.tipo] || ExternalLink;
                                    return (
                                        <a key={enlace.url} href={enlace.url} target="_blank" rel="noopener noreferrer" className="proyectoHeroEnlace">
                                            <Icono size={16} />
                                            <span>{enlace.etiqueta || enlace.tipo}</span>
                                        </a>
                                    );
                                })}
                            </div>
                        )}
                    </div>
                )}
            </div>
        </section>
    );
};
