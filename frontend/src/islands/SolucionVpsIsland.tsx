import {useState} from 'react';
import {useTranslation} from 'react-i18next';
import {Cpu, Shield, HardDrive, TerminalSquare, Activity, Server} from 'lucide-react';
import {LayoutPagina} from '../components/layout/LayoutPagina';
import {SEOHead} from '../components/seo/SEOHead';
import {SeccionContacto} from '../components/home/SeccionContacto';
import {ModalCompra} from '../components/servicios/ModalCompra';
import {useChatStore} from '../stores/chatStore';
import {Button} from '../components/ui/Button';
import {Tarjeta} from '../components/ui/Tarjeta';
import {SolucionHeroImagen} from '../components/soluciones/SolucionHeroImagen';
import {incluida, type PlanServicio} from '../data/planes/tipos';
import {useVpsCatalog} from '../hooks/useVpsCatalog';
import '../components/servicios/SeccionPlanesServicio.css';
import './SolucionHostingIsland.css';

const FEATURES_FALLBACK = [
    {icono: Cpu, titulo: 'Recursos dedicados', desc: 'Cada VPS entrega CPU, RAM y SSD dedicados sin compartir el nodo con otros clientes.'},
    {icono: Shield, titulo: 'Aprobación manual', desc: 'El alta no es automática: revisamos cada compra para reducir fraude y provisioning basura.'},
    {icono: HardDrive, titulo: 'Disco NVMe/SSD', desc: 'Espacio rápido para bases de datos, colas, workers y despliegues propios.'},
    {icono: TerminalSquare, titulo: 'Root + SSH', desc: 'Acceso completo al servidor para administrar procesos, paquetes y despliegues.'},
    {icono: Activity, titulo: 'Bootstrap inicial', desc: 'Entregamos hostname, MOTD, Docker y firewall básico ya configurados.'},
    {icono: Server, titulo: 'Escalado claro', desc: 'Pasas de un tier a otro cuando tus necesidades cambien, con pricing transparente y sin sorpresas.'},
];

function formatMonthlyPrice(priceCents: number): string {
    return `$${(priceCents / 100).toFixed(priceCents % 100 === 0 ? 0 : 2)}`;
}

export const SolucionVpsIsland = (): JSX.Element => {
    const {t} = useTranslation();
    const abrirChat = useChatStore(s => s.abrir);
    const {plans} = useVpsCatalog();
    const [planSeleccionado, setPlanSeleccionado] = useState<PlanServicio | null>(null);

    const planCards: PlanServicio[] = plans.map(plan => ({
        id: plan.tier_name,
        nombre: plan.display_name,
        precio: formatMonthlyPrice(plan.monthly_price_cents),
        periodo: '/mes',
        descripcion: plan.description,
        destacado: plan.recommended,
        ctaTexto: plan.recommended ? 'Elegir recomendado' : 'Solicitar VPS',
        ctaLink: '#',
        stripeModo: 'subscription',
        caracteristicas: plan.features.map(feature => incluida(feature)),
    }));

    const lowestPrice = Math.min(...plans.map(plan => plan.monthly_price_cents));
    const lowestPriceLabel = Number.isFinite(lowestPrice)
        ? formatMonthlyPrice(lowestPrice)
        : '$6.88';

    return (
        <LayoutPagina className="hostingPaginaMain">
            <SEOHead
                title="Servidores VPS"
                description={`Servidores VPS dedicados con acceso root, bootstrap inicial y aprobación manual. Planes desde ${lowestPriceLabel}/mes.`}
                path="/soluciones/vps"
            />

            <section className="hostingHero">
                <div className="hostingHeroContenido">
                    <span className="hostingHeroEtiqueta">{t('content.solutions.vps.titulo', 'Servidores VPS')}</span>
                    <h1 className="hostingHeroTitulo">Infraestructura dedicada sin markup inflado</h1>
                    <p className="hostingHeroDesc">
                        Compra un VPS dedicado con acceso root, Docker listo, firewall básico y revisión manual antes de provisionar para evitar fraude y desperdicio operativo.
                    </p>
                    <div className="hostingHeroBotones">
                        <Button variante="primario" onClick={() => {
                            document.getElementById('planesVps')?.scrollIntoView({behavior: 'smooth'});
                        }}>
                            Ver planes
                        </Button>
                        <Button variante="outline" onClick={() => abrirChat('page:vps')}>
                            Conversar
                        </Button>
                    </div>
                </div>
            </section>

            <SolucionHeroImagen
                src="/assets/Proyectos portadas/Kamples portada.jpg"
                alt="Infraestructura VPS dedicada para despliegues de aplicaciones."
                storageKey="nakomi-vps-hero-image"
            />

            <section className="hostingFeatures">
                <h2 className="hostingFeaturesTitle">Qué incluye la entrega</h2>
                <p className="hostingFeaturesSubtitle">
                    La provisión se entrega lista para operar, pero manteniendo el control completo del servidor en tus manos.
                </p>
                <div className="hostingFeaturesGrid">
                    {FEATURES_FALLBACK.map(feature => (
                        <Tarjeta key={feature.titulo} className="hostingFeatureCard" fondo="#f5f3f1">
                            <div className="hostingFeatureIcono">
                                <feature.icono size={20} strokeWidth={1.5} />
                            </div>
                            <h3>{feature.titulo}</h3>
                            <p>{feature.desc}</p>
                        </Tarjeta>
                    ))}
                </div>
            </section>

            <section id="planesVps" className="planesSeccion">
                <div className="planesContenedor">
                    <div className="planesCabecera">
                        <h2 className="planesTitulo">Planes VPS dedicados</h2>
                        <p className="planesSubtitulo">Checkout mensual con aprobación manual antes del provisioning definitivo.</p>
                    </div>
                    <div className="planesGrid">
                        {planCards.map(plan => (
                            <div key={plan.id} className={`tarjetaPlan ${plan.destacado ? 'tarjetaPlanDestacado' : ''}`}>
                                {plan.destacado && <div className="tarjetaPlanBadge">Recomendado</div>}
                                <div className="tarjetaPlanCabecera">
                                    <h3 className="tarjetaPlanNombre">{plan.nombre}</h3>
                                    <div className="tarjetaPlanPrecio">
                                        <span className="tarjetaPlanPrecioCifra">{plan.precio}</span>
                                        {plan.periodo && <span className="tarjetaPlanPrecioPeriodo">{plan.periodo}</span>}
                                    </div>
                                    <p className="tarjetaPlanDescripcion">{plan.descripcion}</p>
                                </div>
                                <ul className="tarjetaPlanCaracteristicas">
                                    {plan.caracteristicas.map(caracteristica => (
                                        <li key={caracteristica.texto} className="tarjetaPlanItem tarjetaPlanItemIncluido">
                                            <span className="tarjetaPlanItemIcono">
                                                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                                                    <polyline points="20 6 9 17 4 12" />
                                                </svg>
                                            </span>
                                            <span className="tarjetaPlanItemTexto">{caracteristica.texto}</span>
                                        </li>
                                    ))}
                                </ul>
                                <div className="tarjetaPlanAccion">
                                    <Button
                                        variante={plan.destacado ? 'primario' : 'outline'}
                                        tamano="mediano"
                                        onClick={() => setPlanSeleccionado(plan)}
                                    >
                                        {plan.ctaTexto}
                                    </Button>
                                    <Button variante="texto" className="tarjetaPlanConversar" onClick={() => abrirChat('page:vps')}>
                                        Conversar
                                    </Button>
                                </div>
                            </div>
                        ))}
                    </div>
                </div>
            </section>

            <SeccionContacto />

            {planSeleccionado && (
                <ModalCompra
                    plan={planSeleccionado}
                    servicioSlug="vps"
                    abierto={!!planSeleccionado}
                    onCerrar={() => setPlanSeleccionado(null)}
                />
            )}
        </LayoutPagina>
    );
};