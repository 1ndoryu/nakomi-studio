/* [195A-1] Configurador VPS estilo proveedor: plan, recursos, hostname y checkout real.
 * La compra ya no abre un modal genérico; el detalle técnico queda visible antes de Stripe. */
import {useParams} from 'react-router-dom';
import {Cpu, HardDrive, Network, Server, ShieldCheck} from 'lucide-react';
import {LayoutPagina} from '../components/layout/LayoutPagina';
import {SEOHead} from '../components/seo/SEOHead';
import {Button} from '../components/ui/Button';
import {Input} from '../components/ui/Input';
import type {PublicVpsPlan} from '../api/hosting';
import {useVpsConfiguradorIsland} from '../hooks/useVpsConfiguradorIsland';
import {navegar} from '../navegacionSPA';
import './VpsConfiguradorIsland.css';

function formatMoney(cents: number): string {
    const value = cents / 100;
    return `$${value.toFixed(cents % 100 === 0 ? 0 : 2)}`;
}

function formatRam(ramMb: number): string {
    return `${ramMb / 1024} GB`;
}

function formatPort(speedMbps: number): string {
    return speedMbps >= 1000 ? `${speedMbps / 1000} Gbit/s` : `${speedMbps} Mbit/s`;
}

function storageLabel(plan: PublicVpsPlan): string {
    return plan.storage_options.length > 0
        ? plan.storage_options.join(' / ')
        : `${plan.disk_mb / 1024} GB ${plan.storage_type}`;
}

export function VpsConfiguradorIsland(): JSX.Element {
    const {tier} = useParams<{tier?: string}>();
    const configurador = useVpsConfiguradorIsland(tier);
    const {
        plans, isLoading, selectedPlan, form, emailExiste, status,
        dueToday, logueado, updateField, handleSubmit,
    } = configurador;

    return (
        <LayoutPagina className="vpsConfiguradorPagina">
            <SEOHead
                title="Configurar VPS"
                description="Configura un VPS Nakomi con precios Contabo ajustados, recursos visibles y checkout seguro."
                path="/soluciones/vps/configurar"
            />

            <section className="vpsConfiguradorHero">
                <Button type="button" variante="texto" tamano="pequeno" className="vpsConfiguradorVolver" onClick={() => navegar('/soluciones/vps')}>
                    Volver a VPS
                </Button>
                <span className="vpsConfiguradorEtiqueta">Nakomi VPS</span>
                <h1 className="vpsConfiguradorTitulo">Configura tu servidor</h1>
                <p className="vpsConfiguradorSubtitulo">
                    Recursos de Contabo con identidad Nakomi, margen operativo de 5% y entrega técnica verificada.
                </p>
            </section>

            <form className="vpsConfiguradorLayout" onSubmit={handleSubmit}>
                <main className="vpsConfiguradorPrincipal">
                    <section className="vpsConfiguradorBloque">
                        <div className="vpsConfiguradorBloqueHeader">
                            <Server size={18} />
                            <h2 className="vpsConfiguradorBloqueTitulo">Plan</h2>
                        </div>
                        {isLoading && <p className="vpsConfiguradorMuted">Cargando catálogo VPS...</p>}
                        <div className="vpsConfiguradorPlanes">
                            {plans.map(plan => (
                                <Button
                                    key={plan.tier_name}
                                    type="button"
                                    variante="texto"
                                    className={`vpsConfiguradorPlan ${selectedPlan?.tier_name === plan.tier_name ? 'vpsConfiguradorPlanActivo' : ''}`}
                                    onClick={() => updateField('selectedTier', plan.tier_name)}
                                >
                                    <span className="vpsConfiguradorPlanNombre">{plan.display_name}</span>
                                    <span className="vpsConfiguradorPlanPrecio">{formatMoney(plan.monthly_price_cents)}/mes</span>
                                    <span className="vpsConfiguradorPlanSpecs">
                                        {plan.cpu_cores} vCPU · {formatRam(plan.ram_mb)} RAM · {storageLabel(plan)}
                                    </span>
                                </Button>
                            ))}
                        </div>
                    </section>

                    {selectedPlan && (
                        <>
                            <section className="vpsConfiguradorBloque">
                                <div className="vpsConfiguradorBloqueHeader">
                                    <Cpu size={18} />
                                    <h2 className="vpsConfiguradorBloqueTitulo">Recursos</h2>
                                </div>
                                <div className="vpsConfiguradorSpecsGrid">
                                    <div className="vpsConfiguradorSpec">
                                        <span>CPU</span>
                                        <strong>{selectedPlan.cpu_cores} vCPU</strong>
                                    </div>
                                    <div className="vpsConfiguradorSpec">
                                        <span>RAM</span>
                                        <strong>{formatRam(selectedPlan.ram_mb)}</strong>
                                    </div>
                                    <div className="vpsConfiguradorSpec">
                                        <span>Storage</span>
                                        <strong>{storageLabel(selectedPlan)}</strong>
                                    </div>
                                    <div className="vpsConfiguradorSpec">
                                        <span>Snapshots</span>
                                        <strong>{selectedPlan.snapshot_count}</strong>
                                    </div>
                                </div>
                            </section>

                            <section className="vpsConfiguradorBloque">
                                <div className="vpsConfiguradorBloqueHeader">
                                    <Network size={18} />
                                    <h2 className="vpsConfiguradorBloqueTitulo">Red</h2>
                                </div>
                                <div className="vpsConfiguradorSpecsGrid">
                                    <div className="vpsConfiguradorSpec">
                                        <span>Puerto</span>
                                        <strong>{formatPort(selectedPlan.port_speed_mbps)}</strong>
                                    </div>
                                    <div className="vpsConfiguradorSpec">
                                        <span>Tráfico</span>
                                        <strong>{selectedPlan.bandwidth_label}</strong>
                                    </div>
                                    <div className="vpsConfiguradorSpec">
                                        <span>Región</span>
                                        <strong>{selectedPlan.region}</strong>
                                    </div>
                                </div>
                            </section>
                        </>
                    )}

                    <section className="vpsConfiguradorBloque">
                        <div className="vpsConfiguradorBloqueHeader">
                            <HardDrive size={18} />
                            <h2 className="vpsConfiguradorBloqueTitulo">Identidad del servidor</h2>
                        </div>
                        <label className="vpsConfiguradorCampo">
                            <span>Hostname</span>
                            <Input
                                type="text"
                                value={form.hostname}
                                onChange={event => updateField('hostname', event.target.value)}
                                placeholder="cliente-vps-01 (opcional)"
                            />
                        </label>
                    </section>

                    {!logueado && (
                        <section className="vpsConfiguradorBloque">
                            <div className="vpsConfiguradorBloqueHeader">
                                <ShieldCheck size={18} />
                                <h2 className="vpsConfiguradorBloqueTitulo">Cuenta</h2>
                            </div>
                            <label className="vpsConfiguradorCampo">
                                <span>Email</span>
                                <Input
                                    type="email"
                                    value={form.email}
                                    onChange={event => updateField('email', event.target.value)}
                                    placeholder="tu@email.com"
                                    required
                                    disabled={emailExiste}
                                />
                            </label>
                            {emailExiste && (
                                <label className="vpsConfiguradorCampo">
                                    <span>Contraseña</span>
                                    <Input
                                        type="password"
                                        value={form.password}
                                        onChange={event => updateField('password', event.target.value)}
                                        placeholder="Contraseña"
                                        required
                                        minLength={8}
                                    />
                                </label>
                            )}
                        </section>
                    )}
                </main>

                <aside className="vpsConfiguradorResumen">
                    <h2 className="vpsConfiguradorResumenTitulo">Resumen</h2>
                    {selectedPlan ? (
                        <>
                            <div className="vpsConfiguradorResumenPlan">
                                <span>{selectedPlan.display_name}</span>
                                <strong>{formatMoney(selectedPlan.monthly_price_cents)}/mes</strong>
                            </div>
                            <div className="vpsConfiguradorResumenFila">
                                <span>CPU</span>
                                <strong>{selectedPlan.cpu_cores} vCPU</strong>
                            </div>
                            <div className="vpsConfiguradorResumenFila">
                                <span>RAM</span>
                                <strong>{formatRam(selectedPlan.ram_mb)}</strong>
                            </div>
                            <div className="vpsConfiguradorResumenFila">
                                <span>Storage</span>
                                <strong>{storageLabel(selectedPlan)}</strong>
                            </div>
                            <div className="vpsConfiguradorResumenFila">
                                <span>Puerto</span>
                                <strong>{formatPort(selectedPlan.port_speed_mbps)}</strong>
                            </div>
                            {selectedPlan.setup_fee_cents > 0 && (
                                <div className="vpsConfiguradorResumenFila">
                                    <span>Puesta en marcha</span>
                                    <strong>{formatMoney(selectedPlan.setup_fee_cents)}</strong>
                                </div>
                            )}
                            <div className="vpsConfiguradorResumenTotal">
                                <span>Hoy</span>
                                <strong>{formatMoney(dueToday)}</strong>
                            </div>
                            <p className="vpsConfiguradorResumenNota">
                                Luego queda la suscripción mensual de {formatMoney(selectedPlan.monthly_price_cents)}.
                            </p>
                        </>
                    ) : (
                        <p className="vpsConfiguradorMuted">No hay planes VPS disponibles.</p>
                    )}
                    {status.error && <p className="vpsConfiguradorError">{status.error}</p>}
                    <Button
                        type="submit"
                        variante="primario"
                        tamano="mediano"
                        className="vpsConfiguradorCheckout"
                        disabled={!selectedPlan || status.submitting}
                    >
                        {status.submitting ? 'Preparando checkout...' : 'Continuar a Stripe'}
                    </Button>
                </aside>
            </form>
        </LayoutPagina>
    );
}
