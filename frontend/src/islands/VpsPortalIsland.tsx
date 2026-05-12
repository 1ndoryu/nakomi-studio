/* [125A-4] Landing page de vps.nakomi.studio.
 * Inspirado en relace.ai: hero bold, demo terminal dark, features grid,
 * precios live desde useVpsCatalog, FAQ accordion.
 * Standalone: nav y footer propios, sin LayoutPagina. */

import {useState, type ElementType} from 'react';
import {Cpu, Shield, HardDrive, TerminalSquare, Activity, Server, ChevronDown} from 'lucide-react';
import {SEOHead} from '../components/seo/SEOHead';
import {useVpsCatalog} from '../hooks/useVpsCatalog';
import {Button} from '../components/ui/Button';
import {spaClick} from '../navegacionSPA';
import './VpsPortalIsland.css';

interface Feature { icono: ElementType; titulo: string; desc: string; }

const FEATURES: Feature[] = [
    {icono: Cpu, titulo: 'Recursos dedicados', desc: 'CPU, RAM y NVMe SSD dedicados. Sin compartir nodo con otros clientes.'},
    {icono: Shield, titulo: 'Aprobación manual', desc: 'Revisamos cada compra antes de provisionar. Menos fraude, más calidad.'},
    {icono: HardDrive, titulo: 'Disco NVMe', desc: 'Storage rápido para bases de datos, colas, workers y despliegues.'},
    {icono: TerminalSquare, titulo: 'Root + SSH', desc: 'Acceso root completo desde el día uno. Instala lo que necesites.'},
    {icono: Activity, titulo: 'Bootstrap inicial', desc: 'Docker, firewall y hostname ya configurados. Despliega de inmediato.'},
    {icono: Server, titulo: 'Escalado claro', desc: 'Cambia de tier cuando lo necesites. Pricing transparente sin sorpresas.'},
];

const FAQ_ITEMS = [
    {q: '¿Cuándo se activa el VPS?', a: 'El alta no es automática: revisamos cada compra manualmente. El tiempo habitual es menos de 24 h en días laborables.'},
    {q: '¿Qué incluye el bootstrap inicial?', a: 'Docker instalado y activo, firewall ufw con puertos 22, 80 y 443 abiertos, hostname configurado y MOTD con tus recursos de hardware.'},
    {q: '¿Puedo cancelar en cualquier momento?', a: 'Sí. La suscripción se cancela desde tu panel y el servidor se desprovisiona al finalizar el ciclo de facturación actual.'},
    {q: '¿Qué sistema operativo incluye?', a: 'Ubuntu 22.04 LTS por defecto. Si necesitas otra distribución contáctanos antes de completar la compra.'},
    {q: '¿Tienen IPv6?', a: 'IPv4 dedicada en todos los planes. IPv6 disponible bajo consulta para VPS 3 y VPS 4.'},
];

const PLANES_FALLBACK = [
    {nombre: 'Nakomi VPS 1', precio: '$6.88', desc: 'Para proyectos ligeros y bots.', destacado: false, features: ['1 vCPU dedicado', '2 GB RAM', '40 GB NVMe SSD', 'Root + SSH', 'Docker preinstalado']},
    {nombre: 'Nakomi VPS 2', precio: '$11.99', desc: 'Ideal para apps web y APIs.', destacado: true, features: ['2 vCPU dedicados', '4 GB RAM', '60 GB NVMe SSD', 'Root + SSH', 'Docker preinstalado', 'Soporte prioritario']},
    {nombre: 'Nakomi VPS 3', precio: '$20.49', desc: 'Para cargas de trabajo mayores.', destacado: false, features: ['4 vCPU dedicados', '8 GB RAM', '100 GB NVMe SSD', 'Root + SSH', 'Docker preinstalado']},
    {nombre: 'Nakomi VPS 4', precio: '$36.99', desc: 'Infraestructura seria.', destacado: false, features: ['6 vCPU dedicados', '16 GB RAM', '200 GB NVMe SSD', 'Root + SSH', 'Docker preinstalado', 'IPv6 disponible']},
];

function formatPrice(cents: number): string {
    return `$${(cents / 100).toFixed(cents % 100 === 0 ? 0 : 2)}`;
}

function scrollTo(id: string): void {
    document.getElementById(id)?.scrollIntoView({behavior: 'smooth'});
}

export function VpsPortalIsland(): JSX.Element {
    const {plans} = useVpsCatalog();
    const [faqAbierto, setFaqAbierto] = useState<number | null>(null);

    const lowestPrice = plans.length
        ? formatPrice(Math.min(...plans.map(p => p.monthly_price_cents)))
        : '$6.88';

    const preciosActivos = plans.length > 0
        ? plans.map(p => ({
            nombre: p.display_name,
            precio: formatPrice(p.monthly_price_cents),
            desc: p.description,
            destacado: p.recommended,
            features: p.features,
        }))
        : PLANES_FALLBACK;

    return (
        <div className="vpsPortal">
            <SEOHead
                title="Nakomi VPS — Infraestructura dedicada sin intermediarios"
                description={`Servidores VPS con recursos dedicados, root SSH, Docker listo y aprobación manual. Planes desde ${lowestPrice}/mes.`}
                path="/"
            />

            <nav className="vpsNavPortal">
                <div className="vpsNavContenido">
                    <a className="vpsNavLogo" href="https://nakomi.studio">
                        <span className="vpsNavLogoMarca">Nakomi</span>
                        <span className="vpsNavLogoProducto">VPS</span>
                    </a>
                    <div className="vpsNavLinks">
                        <button className="vpsNavLink" type="button" onClick={() => scrollTo('caracteristicas')}>Características</button>
                        <button className="vpsNavLink" type="button" onClick={() => scrollTo('precios')}>Precios</button>
                        <button className="vpsNavLink" type="button" onClick={() => scrollTo('faq')}>FAQ</button>
                        <a className="vpsNavLink" href="/panel" onClick={(e) => spaClick(e, '/panel')}>Mi panel</a>
                    </div>
                    <Button variante="primario" onClick={() => scrollTo('precios')}>Ver planes</Button>
                </div>
            </nav>

            <section className="vpsHero">
                <span className="vpsHeroEtiqueta">Nakomi VPS</span>
                <h1 className="vpsHeroTitulo">
                    Infraestructura dedicada<br />
                    <span className="vpsHeroTituloAcento">sin intermediarios.</span>
                </h1>
                <p className="vpsHeroSub">
                    VPS con recursos propios, aprobación manual y bootstrap listo el día uno.
                    Precios honestos sin margen oculto.
                </p>
                <div className="vpsHeroBotones">
                    <Button variante="primario" onClick={() => scrollTo('precios')}>Ver planes y precios</Button>
                    <Button variante="outline" onClick={() => scrollTo('caracteristicas')}>Características</Button>
                </div>
                <div className="vpsHeroStats">
                    <div className="vpsHeroStat">
                        <span className="vpsHeroStatVal">100%</span>
                        <span className="vpsHeroStatLabel">Recursos dedicados</span>
                    </div>
                    <div className="vpsHeroStat">
                        <span className="vpsHeroStatVal">&lt;24h</span>
                        <span className="vpsHeroStatLabel">Alta revisada</span>
                    </div>
                    <div className="vpsHeroStat">
                        <span className="vpsHeroStatVal">desde {lowestPrice}</span>
                        <span className="vpsHeroStatLabel">por mes</span>
                    </div>
                </div>
            </section>

            <section className="vpsDemoTerminal">
                <div className="vpsDemoContenido">
                    <p className="vpsDemoEtiqueta">Bootstrap inicial</p>
                    <h2 className="vpsDemoTitulo">Tu servidor, listo para desplegar.</h2>
                    <div className="vpsDemoBloque" aria-label="Ejemplo de sesión SSH tras provisioning">
                        <div className="vpsDemoCabecera">
                            <span className="vpsDemoPunto vpsDemoPuntoRojo" />
                            <span className="vpsDemoPunto vpsDemoPuntoAmbar" />
                            <span className="vpsDemoPunto vpsDemoPuntoVerde" />
                            <span className="vpsDemoCabeceraLabel">ssh root@203.0.113.42</span>
                        </div>
                        <pre className="vpsDemoCodigo">{`$ ssh root@203.0.113.42
Ubuntu 22.04.3 LTS — Nakomi VPS 2

  Docker  : ● active
  Firewall: ● active  (22, 80, 443 open)
  CPU     : 2 vCPU dedicated
  RAM     : 4 GB
  Disk    : 60 GB NVMe SSD

root@nakomi-vps:~$ docker ps
CONTAINER ID   IMAGE   STATUS
(servidor limpio — listo para desplegar)

root@nakomi-vps:~$ █`}</pre>
                    </div>
                </div>
            </section>

            <section className="vpsFeatures" id="caracteristicas">
                <div className="vpsSectionContenido">
                    <h2 className="vpsSectionTitulo">Todo lo que necesitas para operar</h2>
                    <p className="vpsSectionSub">Sin servicios gestionados innecesarios, sin overhead de plataforma.</p>
                    <div className="vpsFeaturesGrid">
                        {FEATURES.map((f) => {
                            const Icono = f.icono;
                            return (
                                <div key={f.titulo} className="vpsFeatureCard">
                                    <span className="vpsFeatureIcono"><Icono size={18} /></span>
                                    <strong className="vpsFeatureTitulo">{f.titulo}</strong>
                                    <p className="vpsFeatureDesc">{f.desc}</p>
                                </div>
                            );
                        })}
                    </div>
                </div>
            </section>

            <section className="vpsPrecios" id="precios">
                <div className="vpsSectionContenido">
                    <h2 className="vpsSectionTitulo">Planes claros, recursos reales</h2>
                    <p className="vpsSectionSub">Suscripción mensual. Sin contratos anuales. Cancela cuando quieras.</p>
                    <div className="vpsPreciosGrid">
                        {preciosActivos.map((plan) => (
                            <div key={plan.nombre} className={`vpsPrecioCard${plan.destacado ? ' vpsPrecioCardDestacado' : ''}`}>
                                {plan.destacado && <span className="vpsPrecioRecomendado">Recomendado</span>}
                                <h3 className="vpsPrecioNombre">{plan.nombre}</h3>
                                <div className="vpsPrecioValor">
                                    <span className="vpsPrecioCantidad">{plan.precio}</span>
                                    <span className="vpsPrecioPeriodo">/mes</span>
                                </div>
                                <p className="vpsPrecioDesc">{plan.desc}</p>
                                <ul className="vpsPrecioFeatures">
                                    {plan.features.map((f) => <li key={f}>{f}</li>)}
                                </ul>
                                <Button variante={plan.destacado ? 'primario' : 'outline'} onClick={() => {}}>
                                    Solicitar {plan.nombre}
                                </Button>
                            </div>
                        ))}
                    </div>
                </div>
            </section>

            <section className="vpsFaq" id="faq">
                <div className="vpsSectionContenido vpsSectionAngosto">
                    <h2 className="vpsSectionTitulo">Preguntas frecuentes</h2>
                    <div className="vpsFaqLista">
                        {FAQ_ITEMS.map((item, i) => (
                            <div key={item.q} className="vpsFaqItem">
                                <button
                                    type="button"
                                    className="vpsFaqPregunta"
                                    onClick={() => setFaqAbierto(faqAbierto === i ? null : i)}
                                    aria-expanded={faqAbierto === i}
                                >
                                    <span>{item.q}</span>
                                    <ChevronDown className={`vpsFaqChevron${faqAbierto === i ? ' vpsFaqChevronAbierto' : ''}`} size={16} />
                                </button>
                                {faqAbierto === i && <p className="vpsFaqRespuesta">{item.a}</p>}
                            </div>
                        ))}
                    </div>
                </div>
            </section>

            <section className="vpsCtaFinal">
                <h2 className="vpsCtaFinalTitulo">Infraestructura lista en minutos.</h2>
                <p className="vpsCtaFinalSub">Solicita tu VPS hoy — revisamos tu petición y te respondemos en menos de 24 h.</p>
                <Button variante="primario" onClick={() => scrollTo('precios')}>Ver planes →</Button>
            </section>

            <footer className="vpsFooterPortal">
                <div className="vpsFooterContenido">
                    <span className="vpsFooterMarca">Nakomi VPS</span>
                    <div className="vpsFooterLinks">
                        <a href="https://nakomi.studio" className="vpsFooterLink">Nakomi Studio</a>
                        <a href="/politica-privacidad" onClick={(e) => spaClick(e, '/politica-privacidad')} className="vpsFooterLink">Privacidad</a>
                    </div>
                    <span className="vpsFooterCopy">© {new Date().getFullYear()} Nakomi Studio</span>
                </div>
            </footer>
        </div>
    );
}

export default VpsPortalIsland;
