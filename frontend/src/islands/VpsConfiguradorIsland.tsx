/* [195A-1] Configurador VPS estilo proveedor: plan, recursos, hostname y checkout real.
 * La compra ya no abre un modal genérico; el detalle técnico queda visible antes de Stripe.
 * [205A-1] Helpers/constantes en vpsConfiguradorHelpers.ts para mantenerse bajo 300 líneas. */
import {useParams} from 'react-router-dom';
import {Cpu, HardDrive, KeyRound, Monitor, Network, Server, ShieldCheck} from 'lucide-react';
import {LayoutPagina} from '../components/layout/LayoutPagina';
import {SEOHead} from '../components/seo/SEOHead';
import {Button} from '../components/ui/Button';
import {Input} from '../components/ui/Input';
import {Select} from '../components/ui/Select';
import {useVpsConfiguradorIsland} from '../hooks/useVpsConfiguradorIsland';
import {navegar} from '../navegacionSPA';
import {
    formatMoney, formatRam, formatPort, storageLabel,
    VPS_LOCATIONS, VPS_CONTINENTS, storageExtraLabel, OS_IMAGES,
} from './vpsConfiguradorHelpers';
import './VpsConfiguradorIsland.css';

export function VpsConfiguradorIsland(): JSX.Element {
    const {tier} = useParams<{tier?: string}>();
    const configurador = useVpsConfiguradorIsland(tier);
    const {
        plans, selectedPlan, form, emailExiste, status,
        dueToday, logueado, updateField, handleSubmit,
    } = configurador;

    return (
        <LayoutPagina className="vpsConfiguradorPagina">
            <SEOHead
                title="Configurar VPS"
                description="Configura un VPS Nakomi con precios Contabo ajustados, recursos visibles y checkout seguro."
                path="/soluciones/vps/configurar"
            />

            <div className="vpsConfiguradorNavBar">
                <Button type="button" variante="texto" tamano="pequeno" className="vpsConfiguradorVolver" onClick={() => navegar('/soluciones/vps')}>
                    ← Volver a VPS
                </Button>
            </div>
            <section className="vpsConfiguradorHero">
                <span className="vpsConfiguradorEtiqueta">Nakomi VPS</span>
                <h1 className="vpsConfiguradorTitulo">Configura tu servidor</h1>
            </section>

            <form className="vpsConfiguradorLayout" onSubmit={handleSubmit}>
                <main className="vpsConfiguradorPrincipal">
                    <section className="vpsConfiguradorBloque">
                        <div className="vpsConfiguradorBloqueHeader">
                            <Server size={18} />
                            <h2 className="vpsConfiguradorBloqueTitulo">Plan</h2>
                        </div>
                        <Select
                            className="vpsConfiguradorSelect"
                            value={form.selectedTier || selectedPlan?.tier_name || ''}
                            onChange={e => updateField('selectedTier', e.target.value)}
                        >
                            {plans.map(plan => (
                                <option key={plan.tier_name} value={plan.tier_name}>
                                    {plan.display_name} — {formatMoney(plan.monthly_price_cents)}/mes
                                </option>
                            ))}
                        </Select>
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
                                        <span>Snapshots</span>
                                        <strong>{selectedPlan.snapshot_count}</strong>
                                    </div>
                                </div>
                                {selectedPlan.storage_options.length > 1 && (
                                    <div className="vpsConfiguradorStorageSel">
                                        <span className="vpsConfiguradorStorageLabel">Storage</span>
                                        <div className="vpsConfiguradorStorageOpciones">
                                            {selectedPlan.storage_options.map(opt => {
                                                const extra = storageExtraLabel(opt);
                                                const activo = (form.selectedStorage || selectedPlan.storage_options[0]) === opt;
                                                return (
                                                    <Button
                                                        key={opt}
                                                        type="button"
                                                        variante="outline"
                                                        tamano="pequeno"
                                                        className={`vpsConfiguradorStorageOpcion${activo ? ' vpsConfiguradorStorageOpcionActivo' : ''}`}
                                                        onClick={() => updateField('selectedStorage', opt)}
                                                    >
                                                        {opt}{extra && extra !== 'Incluido' ? ` (${extra})` : ''}
                                                    </Button>
                                                );
                                            })}
                                        </div>
                                    </div>
                                )}
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
                                </div>
                                <div className="vpsConfiguradorRegionSel">
                                    <span className="vpsConfiguradorStorageLabel">Región</span>
                                    <Select
                                        className="vpsConfiguradorSelect"
                                        value={form.selectedRegion}
                                        onChange={e => updateField('selectedRegion', e.target.value)}
                                    >
                                        {VPS_CONTINENTS.map(continent => (
                                            <optgroup key={continent} label={continent}>
                                                {VPS_LOCATIONS.filter(l => l.continent === continent).map(loc => (
                                                    <option key={loc.code} value={loc.code}>
                                                        {loc.label}{loc.free ? ' — Incluido' : ' (+costo)'}
                                                    </option>
                                                ))}
                                            </optgroup>
                                        ))}
                                    </Select>
                                </div>
                            </section>
                        </>
                    )}

                    {selectedPlan && (
                        <section className="vpsConfiguradorBloque">
                            <div className="vpsConfiguradorBloqueHeader">
                                <Monitor size={18} />
                                <h2 className="vpsConfiguradorBloqueTitulo">Sistema operativo</h2>
                            </div>
                            <Select
                                className="vpsConfiguradorSelect"
                                value={form.selectedOs}
                                onChange={e => updateField('selectedOs', e.target.value)}
                            >
                                {OS_IMAGES.map(os => (
                                    <option key={os} value={os}>{os}</option>
                                ))}
                            </Select>
                        </section>
                    )}

                    {selectedPlan && (
                        <section className="vpsConfiguradorBloque">
                            <div className="vpsConfiguradorBloqueHeader">
                                <KeyRound size={18} />
                                <h2 className="vpsConfiguradorBloqueTitulo">Acceso al servidor</h2>
                            </div>
                            <label className="vpsConfiguradorCampo">
                                <span>Usuario</span>
                                <Input type="text" value="root" disabled />
                            </label>
                            <label className="vpsConfiguradorCampo">
                                <span>Contraseña de root</span>
                                <Input
                                    type="password"
                                    value={form.serverPassword}
                                    onChange={event => updateField('serverPassword', event.target.value)}
                                    placeholder="Mínimo 8 caracteres"
                                    minLength={8}
                                />
                            </label>
                            <p className="vpsConfiguradorMuted">No se enviará por email. Guárdala en un gestor de contraseñas.</p>
                        </section>
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
                                <strong>{form.selectedStorage || selectedPlan.storage_options[0] || storageLabel(selectedPlan)}</strong>
                            </div>
                            <div className="vpsConfiguradorResumenFila">
                                <span>OS</span>
                                <strong>{form.selectedOs}</strong>
                            </div>
                            <div className="vpsConfiguradorResumenFila">
                                <span>Puerto</span>
                                <strong>{formatPort(selectedPlan.port_speed_mbps)}</strong>
                            </div>
                            <div className="vpsConfiguradorResumenFila">
                                <span>Región</span>
                                <strong>{form.selectedRegion}</strong>
                            </div>
                            <div className="vpsConfiguradorResumenFila">
                                <span>Snapshots</span>
                                <strong>{selectedPlan.snapshot_count}</strong>
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
