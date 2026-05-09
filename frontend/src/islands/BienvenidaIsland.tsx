/**
 * Componente: BienvenidaIsland
 * Página principal (Home) del sitio.
 */

import '../styles/variables.css';
import '../styles/bienvenida.css';
import {LayoutPagina} from '../components/layout/LayoutPagina';
import {SEOHead} from '../components/seo/SEOHead';
import {organizationSchema, websiteSchema} from '../components/seo/schemas';
import {SeccionHero} from '../components/home/SeccionHero';
import {SeccionServicios} from '../components/home/SeccionServicios';
import {SeccionContacto} from '../components/home/SeccionContacto';
import {SeccionShowcase} from '../components/home/SeccionShowcase';

/* [044A-6] Se eliminaron SeccionClientes y SeccionTestimonios del home.
 * Los componentes siguen existiendo en components/home/ por si se reactivan. */
export const BienvenidaIsland = (): JSX.Element => {
    return (
        <LayoutPagina className="mainContainer">
            <SEOHead
                description="Estudio creativo basado en Copenhague. Diseño web, apps e IA construidos con Rust para rendimiento real. Operamos en español, inglés y japonés."
                path="/"
                jsonLd={{...organizationSchema, ...websiteSchema}}
            />
            <SeccionHero />
            <SeccionShowcase />
            <SeccionServicios />
            <SeccionContacto compacto />
        </LayoutPagina>
    );
};

export default BienvenidaIsland;
