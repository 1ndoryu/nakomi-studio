/**
 * Componente: SolucionesIsland
 * Página landing de soluciones con cards que enlazan a sub-páginas.
 */
import React from 'react';
import {useTranslation} from 'react-i18next';
import {spaClick} from '../navegacionSPA';
import '../styles/variables.css';
import './SolucionesIsland.css';
import {LayoutPagina} from '../components/layout/LayoutPagina';
import {SEOHead} from '../components/seo/SEOHead';
import {SeccionContacto} from '../components/home/SeccionContacto';

interface SolucionesIslandProps {
    titulo?: string;
}

interface Solucion {
    titulo: string;
    descripcion: string;
    enlace: string;
    etiqueta: string;
}

const SOLUCIONES: Solucion[] = [
    {
        titulo: 'WordPress Hosting',
        descripcion: 'Infraestructura de alto rendimiento con WordPress pre-instalado, WP-CLI vía SSH, backups automáticos y soporte técnico.',
        enlace: '/soluciones/hosting',
        etiqueta: 'Desde $2.48/mes'
    },
    {
        titulo: 'Servidores VPS',
        descripcion: 'Control total sobre tu entorno de servidor. Recursos dedicados, root access y aprobación manual antes del provisioning para proyectos que necesitan potencia real.',
        enlace: 'https://vps.nakomi.studio',
        etiqueta: 'Desde $6.88/mes'
    },
    {
        titulo: 'Agentes de IA',
        descripcion: 'Automatiza procesos de negocio con agentes inteligentes personalizados. Chatbots, asistentes y pipelines de ML integrados directamente en tu producto.',
        enlace: '/soluciones/agentes-ia',
        etiqueta: 'Consultar'
    }
];

/* Tarjeta de solución individual */
const TarjetaSolucion: React.FC<{solucion: Solucion}> = ({solucion}) => {
    const {t} = useTranslation();
    return (
        <a href={solucion.enlace} className="tarjetaSolucion" onClick={e => spaClick(e, solucion.enlace)}>
            <div className="solucionContenido">
                <span className="solucionEtiqueta">{solucion.etiqueta}</span>
                <h3 className="solucionTitulo">{solucion.titulo}</h3>
                <p className="solucionDescripcion">{solucion.descripcion}</p>
            </div>
            <span className="solucionEnlace">{t('solutions_page.explore')}</span>
        </a>
    );
};

export const SolucionesIsland = ({titulo}: SolucionesIslandProps): JSX.Element => {
    const {t} = useTranslation();

    return (
        <LayoutPagina className="solucionesMain" id="paginaSoluciones">
            <SEOHead
                title="Soluciones"
                description="Soluciones digitales: hosting administrado, VPS y agentes IA para tu negocio, con pricing transparente y despliegue real."
                path="/soluciones"
            />
            {/* Hero */}
            <section className="solucionesHero">
                <div className="solucionesHeroContenido">
                    <div>
                        <h1 className="solucionesHeroTitulo">{titulo || t('solutions_page.title')}</h1>
                    </div>
                    <div className="solucionesHeroDescripcion">
                        <p>{t('solutions_page.description')}</p>
                    </div>
                </div>
            </section>

            {/* Grid de soluciones */}
            <section className="solucionesContenido">
                <div className="solucionesContenedor">
                    <div className="solucionesGrid">
                        {SOLUCIONES.map(sol => (
                            <TarjetaSolucion key={sol.titulo} solucion={sol} />
                        ))}
                    </div>
                </div>
            </section>

            <SeccionContacto />
        </LayoutPagina>
    );
};

export default SolucionesIsland;
