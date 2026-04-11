/* [064A-32] Página de WordPress Hosting.
 * [084A-20] CTA de planes ahora abre ModalCompra en vez de chat.
 * [114A-5] Especialización WordPress: branding, features WP-CLI, WooCommerce.
 * "Conversar" sigue abriendo el chat. */

import {useState} from 'react';
import {useTranslation} from 'react-i18next';
import {Server, Shield, Zap, Clock, Globe, Headphones} from 'lucide-react';
import {LayoutPagina} from '../components/layout/LayoutPagina';
import {SEOHead} from '../components/seo/SEOHead';
import {SeccionContacto} from '../components/home/SeccionContacto';
import {ModalCompra} from '../components/servicios/ModalCompra';
import {useChatStore} from '../stores/chatStore';
import {PLANES_HOSTING} from '../data/planes/planes-hosting';
import {Button} from '../components/ui/Button';
import {Tarjeta} from '../components/ui/Tarjeta';
import type {PlanServicio} from '../data/planes/tipos';
import '../components/servicios/SeccionPlanesServicio.css';
import './SolucionHostingIsland.css';

const FEATURES_FALLBACK = [
    {icono: Zap, titulo: 'WordPress Optimizado', desc: 'Servidores SSD configurados específicamente para WordPress con caché y PHP optimizado.'},
    {icono: Shield, titulo: 'Seguridad WordPress', desc: 'SSL gratuito, firewall, backups automáticos, hardening SSH y monitoreo 24/7.'},
    {icono: Clock, titulo: '99.9% Uptime', desc: 'Infraestructura redundante con failover automático para máxima disponibilidad.'},
    {icono: Globe, titulo: 'CDN Global', desc: 'Red de distribución de contenido para velocidad óptima desde cualquier ubicación.'},
    {icono: Server, titulo: 'WordPress Administrado', desc: 'Actualizaciones de WordPress, plugins, parches de seguridad y optimizaciones sin que toques un servidor.'},
    {icono: Headphones, titulo: 'WP-CLI & Soporte', desc: 'Acceso SSH con WP-CLI incluido y equipo técnico experto en WordPress.'},
];

/* [064A-64] Features del hosting traducidos via content.solutions.hosting.
 * Se declara como funcion dentro del componente para acceder a t(). */
const FEATURE_ICONS = [Zap, Shield, Clock, Globe, Server, Headphones];
const FEATURE_KEYS = ['performance', 'security', 'uptime', 'cdn', 'managed', 'support'];

export const SolucionHostingIsland = (): JSX.Element => {
    const {t} = useTranslation();
    const abrirChat = useChatStore(s => s.abrir);
    const [planSeleccionado, setPlanSeleccionado] = useState<PlanServicio | null>(null);

    const features = FEATURE_KEYS.map((key, i) => ({
        icono: FEATURE_ICONS[i],
        titulo: t(`content.solutions.hosting.features.${key}.titulo`, FEATURES_FALLBACK[i].titulo),
        desc: t(`content.solutions.hosting.features.${key}.desc`, FEATURES_FALLBACK[i].desc),
    }));

    return (
        <LayoutPagina className="hostingPaginaMain">
            <SEOHead
                title="WordPress Hosting"
                description="WordPress hosting optimizado con WP-CLI, backups automáticos y soporte experto. Planes desde $5/mes con SSL, WordPress pre-instalado y acceso SSH."
                path="/soluciones/hosting"
            />

            {/* Hero */}
            <section className="hostingHero">
                <div className="hostingHeroContenido">
                    <span className="hostingHeroEtiqueta">{t('content.solutions.hosting.titulo', 'WordPress Hosting')}</span>
                    <h1 className="hostingHeroTitulo">
                        {t('hosting_page.hero_title', 'WordPress que escala contigo')}
                    </h1>
                    <p className="hostingHeroDesc">
                        {t('hosting_page.hero_desc', 'Olvídate de la administración de servidores. WordPress pre-instalado, WP-CLI, backups automáticos y rendimiento optimizado.')}
                    </p>
                    <div className="hostingHeroBotones">
                        <Button variante="primario" onClick={() => {
                            document.getElementById('planesHosting')?.scrollIntoView({behavior: 'smooth'});
                        }}>
                            {t('hosting_page.view_plans', 'Ver Planes')}
                        </Button>
                        <Button variante="outline" onClick={() => abrirChat('page:hosting')}>
                            {t('plans.chat_with_us', 'Conversar')}
                        </Button>
                    </div>
                </div>
            </section>

            {/* Features */}
            <section className="hostingFeatures">
                <h2 className="hostingFeaturesTitle">{t('hosting_page.features_title', 'Todo Incluido')}</h2>
                <p className="hostingFeaturesSubtitle">
                    {t('hosting_page.features_subtitle', 'Cada plan incluye las herramientas esenciales para mantener tu sitio rápido y seguro.')}
                </p>
                <div className="hostingFeaturesGrid">
                    {features.map((f) => (
                        <Tarjeta key={f.titulo} className="hostingFeatureCard" fondo="#f5f3f1">
                            <div className="hostingFeatureIcono">
                                <f.icono size={20} strokeWidth={1.5} />
                            </div>
                            <h3>{f.titulo}</h3>
                            <p>{f.desc}</p>
                        </Tarjeta>
                    ))}
                </div>
            </section>

            {/* Planes */}
            <section id="planesHosting" className="planesSeccion">
                <div className="planesContenedor">
                    <div className="planesCabecera">
                        <h2 className="planesTitulo">{t('hosting_page.plans_title', 'Planes de WordPress Hosting')}</h2>
                        <p className="planesSubtitulo">{t('hosting_page.plans_subtitle', 'Elige el plan que mejor se adapte a tu proyecto')}</p>
                    </div>
                    <div className="planesGrid">
                        {PLANES_HOSTING.map(plan => (
                            <div key={plan.id} className={`tarjetaPlan ${plan.destacado ? 'tarjetaPlanDestacado' : ''}`}>
                                {plan.destacado && <div className="tarjetaPlanBadge">{t('plans.recommended')}</div>}
                                <div className="tarjetaPlanCabecera">
                                    <h3 className="tarjetaPlanNombre">{t(`content.plans.${plan.id}.nombre`, plan.nombre)}</h3>
                                    <div className="tarjetaPlanPrecio">
                                        <span className="tarjetaPlanPrecioCifra">{plan.precio}</span>
                                        {plan.periodo && <span className="tarjetaPlanPrecioPeriodo">{plan.periodo}</span>}
                                    </div>
                                    <p className="tarjetaPlanDescripcion">{t(`content.plans.${plan.id}.descripcion`, plan.descripcion)}</p>
                                </div>
                                <ul className="tarjetaPlanCaracteristicas">
                                    {plan.caracteristicas.map((car, idx) => (
                                        <li key={car.texto} className={`tarjetaPlanItem ${car.incluido ? 'tarjetaPlanItemIncluido' : 'tarjetaPlanItemNoIncluido'}`}>
                                            <span className="tarjetaPlanItemIcono">
                                                {car.incluido ? (
                                                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                                                        <polyline points="20 6 9 17 4 12" />
                                                    </svg>
                                                ) : (
                                                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                                                        <line x1="18" y1="6" x2="6" y2="18" />
                                                        <line x1="6" y1="6" x2="18" y2="18" />
                                                    </svg>
                                                )}
                                            </span>
                                            <span className="tarjetaPlanItemTexto">{t(`content.plans.${plan.id}.features.${idx}`, car.texto)}</span>
                                        </li>
                                    ))}
                                </ul>
                                <div className="tarjetaPlanAccion">
                                    <Button
                                        variante={plan.destacado ? 'primario' : 'outline'}
                                        tamano="mediano"
                                        onClick={() => setPlanSeleccionado(plan)}
                                    >
                                        {t(`content.plans.${plan.id}.cta`, plan.ctaTexto)}
                                    </Button>
                                    <Button variante="texto" className="tarjetaPlanConversar" onClick={() => abrirChat('page:hosting')}>
                                        {t('plans.chat_with_us', 'Conversar')}
                                    </Button>
                                </div>
                            </div>
                        ))}
                    </div>
                </div>
            </section>

            <SeccionContacto />

            {/* [084A-20] Modal de compra para planes de hosting */}
            {planSeleccionado && (
                <ModalCompra
                    plan={planSeleccionado}
                    servicioSlug="hosting"
                    abierto={!!planSeleccionado}
                    onCerrar={() => setPlanSeleccionado(null)}
                />
            )}
        </LayoutPagina>
    );
};
