/* [064A-32] Página de hosting.
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
import {Button} from '../components/ui/Button';
import {Tarjeta} from '../components/ui/Tarjeta';
import {SolucionHeroImagen} from '../components/soluciones/SolucionHeroImagen';
import type {PlanServicio} from '../data/planes/tipos';
import {incluida} from '../data/planes/tipos';
import {useHostingCatalog} from '../hooks/useHostingCatalog';
import '../components/servicios/SeccionPlanesServicio.css';
import './SolucionHostingIsland.css';

type HostingSolutionKind = 'normal' | 'wordpress';

const WORDPRESS_FEATURES_FALLBACK = [
    {icono: Zap, titulo: 'WordPress Optimizado', desc: 'Servidores SSD configurados específicamente para WordPress con caché y PHP optimizado.'},
    {icono: Shield, titulo: 'Seguridad WordPress', desc: 'SSL gratuito, firewall, backups automáticos, hardening SSH y monitoreo 24/7.'},
    {icono: Clock, titulo: '99.9% Uptime', desc: 'Infraestructura redundante con failover automático para máxima disponibilidad.'},
    {icono: Globe, titulo: 'CDN Global', desc: 'Red de distribución de contenido para velocidad óptima desde cualquier ubicación.'},
    {icono: Server, titulo: 'WordPress Administrado', desc: 'Actualizaciones de WordPress, plugins, parches de seguridad y optimizaciones sin que toques un servidor.'},
    {icono: Headphones, titulo: 'WP-CLI & Soporte', desc: 'Acceso SSH con WP-CLI incluido y equipo técnico experto en WordPress.'},
];

const NORMAL_FEATURES_FALLBACK = [
    {icono: Zap, titulo: 'Nginx optimizado', desc: 'Servidor web administrado para sitios estaticos, frontends y proyectos sin CMS pesado.'},
    {icono: Shield, titulo: 'Seguridad incluida', desc: 'SSL gratuito, aislamiento por servicio, hardening SSH/SFTP y controles de acceso.'},
    {icono: Clock, titulo: '99.9% Uptime', desc: 'Infraestructura preparada para disponibilidad estable y monitoreo operativo.'},
    {icono: Globe, titulo: 'Dominios y SSL', desc: 'Dominios propios con certificados automaticos y configuracion limpia de DNS.'},
    {icono: Server, titulo: 'SFTP administrado', desc: 'Sube archivos del sitio por SFTP sin administrar servidores ni contenedores.'},
    {icono: Headphones, titulo: 'Soporte tecnico', desc: 'Ayuda para publicar, diagnosticar DNS y mantener el sitio operativo.'},
];

/* [064A-64] Features del hosting traducidos via content.solutions.hosting.
 * Se declara como funcion dentro del componente para acceder a t(). */
const FEATURE_ICONS = [Zap, Shield, Clock, Globe, Server, Headphones];
const FEATURE_KEYS = ['performance', 'security', 'uptime', 'cdn', 'managed', 'support'];

function formatMonthlyPrice(priceCents: number): string {
    return `$${(priceCents / 100).toFixed(priceCents % 100 === 0 ? 0 : 2)}`;
}

function SolucionHostingContenido({kind}: {kind: HostingSolutionKind}): JSX.Element {
    const {t} = useTranslation();
    const abrirChat = useChatStore(s => s.abrir);
    const isWordPress = kind === 'wordpress';
    const {plans} = useHostingCatalog(isWordPress ? 'wordpress' : 'normal');
    const [planSeleccionado, setPlanSeleccionado] = useState<PlanServicio | null>(null);

    const chatContext = isWordPress ? 'page:hosting-wordpress' : 'page:hosting';
    const seoTitle = isWordPress ? 'Hosting WordPress' : 'Hosting';
    const seoPath = isWordPress ? '/soluciones/hosting-wordpress' : '/soluciones/hosting';
    const heroEtiqueta = isWordPress ? t('content.solutions.hosting.titulo', 'Hosting WordPress') : 'Hosting';
    const heroTitulo = isWordPress ? t('hosting_page.hero_title', 'WordPress que escala contigo') : 'Hosting normal para sitios ligeros y frontends';
    const heroDesc = isWordPress
        ? t('hosting_page.hero_desc', 'Olvídate de la administración de servidores. WordPress pre-instalado, WP-CLI, backups automáticos y rendimiento optimizado.')
        : 'Nginx administrado, SSL, SFTP y recursos aislados para publicar sitios sin WordPress ni base de datos.';
    const plansTitle = isWordPress ? t('hosting_page.plans_title', 'Planes de Hosting WordPress') : 'Planes de hosting';
    const plansSubtitle = isWordPress ? t('hosting_page.plans_subtitle', 'Elige el plan que mejor se adapte a tu proyecto') : 'Elige hosting normal cuando quieres publicar sin WordPress.';
    const featureTitle = isWordPress ? t('hosting_page.features_title', 'Todo Incluido') : 'Todo lo necesario para publicar';
    const featureSubtitle = isWordPress
        ? t('hosting_page.features_subtitle', 'Cada plan incluye las herramientas esenciales para mantener tu sitio rápido y seguro.')
        : 'Cada plan incluye servidor web, SSL, SFTP y soporte operativo sin que administres contenedores.';

    const planCards: PlanServicio[] = plans.map(plan => ({
        id: `hosting-${plan.id}`,
        nombre: plan.label,
        precio: formatMonthlyPrice(plan.priceCents),
        periodo: '/mes',
        descripcion: plan.description ?? '',
        destacado: !!plan.recommended,
        ctaTexto: plan.recommended ? `Elegir ${plan.label}` : 'Comenzar',
        ctaLink: '#',
        stripeModo: 'subscription',
        caracteristicas: plan.features.map(feature => incluida(feature)),
    }));

    const lowestHostingPrice = planCards.reduce((min, plan) => {
        const numeric = Number(plan.precio.replace('$', ''));
        return Number.isFinite(numeric) ? Math.min(min, numeric) : min;
    }, Number.POSITIVE_INFINITY);
    const lowestHostingPriceLabel = Number.isFinite(lowestHostingPrice)
        ? `$${lowestHostingPrice.toFixed(lowestHostingPrice % 1 === 0 ? 0 : 2)}/mes`
        : isWordPress ? '$2.48/mes' : '$3.23/mes';

    const features = isWordPress
        ? FEATURE_KEYS.map((key, i) => ({
            icono: FEATURE_ICONS[i],
            titulo: t(`content.solutions.hosting.features.${key}.titulo`, WORDPRESS_FEATURES_FALLBACK[i].titulo),
            desc: t(`content.solutions.hosting.features.${key}.desc`, WORDPRESS_FEATURES_FALLBACK[i].desc),
        }))
        : NORMAL_FEATURES_FALLBACK;

    return (
        <LayoutPagina className="hostingPaginaMain">
            <SEOHead
                title={seoTitle}
                description={isWordPress
                    ? `WordPress hosting optimizado con WP-CLI, backups automáticos y soporte experto. Planes desde ${lowestHostingPriceLabel} con SSL, WordPress pre-instalado y acceso SSH.`
                    : `Hosting normal administrado con Nginx, SSL y SFTP. Planes desde ${lowestHostingPriceLabel} para sitios sin WordPress.`}
                path={seoPath}
            />

            {/* Hero */}
            <section className="hostingHero">
                <div className="hostingHeroContenido">
                    <span className="hostingHeroEtiqueta">{heroEtiqueta}</span>
                    <h1 className="hostingHeroTitulo">
                        {heroTitulo}
                    </h1>
                    <p className="hostingHeroDesc">
                        {heroDesc}
                    </p>
                    <div className="hostingHeroBotones">
                        <Button variante="primario" onClick={() => {
                            document.getElementById('planesHosting')?.scrollIntoView({behavior: 'smooth'});
                        }}>
                            {t('hosting_page.view_plans', 'Ver Planes')}
                        </Button>
                        <Button variante="outline" onClick={() => abrirChat(chatContext)}>
                            {t('plans.chat_with_us', 'Conversar')}
                        </Button>
                    </div>
                </div>
            </section>

            <SolucionHeroImagen
                src="/assets/random/85a51ba9a4233272662e744b48f97d67.jpg"
                alt={isWordPress ? 'Panel de hosting WordPress con infraestructura administrada.' : 'Panel de hosting con infraestructura administrada.'}
                storageKey={isWordPress ? 'nakomi-hosting-wordpress-hero-image' : 'nakomi-hosting-hero-image'}
            />

            {/* Features */}
            <section className="hostingFeatures">
                <h2 className="hostingFeaturesTitle">{featureTitle}</h2>
                <p className="hostingFeaturesSubtitle">
                    {featureSubtitle}
                </p>
                <div className="hostingFeaturesGrid">
                    {features.map((f) => (
                        <Tarjeta key={f.titulo} className="hostingFeatureCard" fondo="#f5f3f1">
                            <div className="hostingFeatureIcono">
                                <f.icono size={20} strokeWidth={1.5} />
                            </div>
                            <h3 className="hostingFeatureTitulo">{f.titulo}</h3>
                            <p>{f.desc}</p>
                        </Tarjeta>
                    ))}
                </div>
            </section>

            {/* Planes */}
            <section id="planesHosting" className="planesSeccion">
                <div className="planesContenedor">
                    <div className="planesCabecera">
                        <h2 className="planesTitulo">{plansTitle}</h2>
                        <p className="planesSubtitulo">{plansSubtitle}</p>
                    </div>
                    <div className="planesGrid">
                        {planCards.map(plan => (
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
                                    <Button variante="texto" className="tarjetaPlanConversar" onClick={() => abrirChat(chatContext)}>
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
}

export const SolucionHostingIsland = (): JSX.Element => <SolucionHostingContenido kind="normal" />;

export const SolucionHostingWordPressIsland = (): JSX.Element => <SolucionHostingContenido kind="wordpress" />;
