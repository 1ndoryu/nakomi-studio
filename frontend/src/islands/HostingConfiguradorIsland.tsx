/* [215A-2] Configurador público reutilizable para hosting normal y WordPress.
 * Sustituye el modal de compra por una página visible, equivalente al configurador VPS. */
import {useParams} from 'react-router-dom';
import {CreditCard, Globe, HardDrive, KeyRound, Languages, Server, ShieldCheck} from 'lucide-react';
import {LayoutPagina} from '../components/layout/LayoutPagina';
import {SEOHead} from '../components/seo/SEOHead';
import {Button} from '../components/ui/Button';
import {Input} from '../components/ui/Input';
import {SelectDropdown} from '../components/ui/SelectDropdown';
import {
    formatHostingMoney, formatHostingStorage, HOSTING_BILLING_OPTIONS,
    HOSTING_LANGUAGE_OPTIONS, type HostingConfiguradorKind,
    useHostingConfiguradorIsland,
} from '../hooks/useHostingConfiguradorIsland';
import './VpsConfiguradorIsland.css';
import './HostingConfiguradorIsland.css';

interface HostingConfiguradorIslandProps {
    kind: HostingConfiguradorKind;
}

export function HostingConfiguradorIsland({kind}: HostingConfiguradorIslandProps): JSX.Element {
    const {plan} = useParams<{plan?: string}>();
    const isWordPress = kind === 'wordpress';
    const configurador = useHostingConfiguradorIsland(kind, plan);
    const {
        plans, selectedPlan, form, emailExiste, status, dueToday, discountCents,
        stripeFeeCents, billingCycleMonths, logueado, updateField, handleSubmit,
    } = configurador;

    const title = isWordPress ? 'Configurar Hosting WordPress' : 'Configurar hosting';
    const path = isWordPress ? '/soluciones/hosting-wordpress/configurar' : '/soluciones/hosting/configurar';

    return (
        <LayoutPagina className="vpsConfiguradorPagina hostingConfiguradorPagina">
            <SEOHead
                title={title}
                description="Configura tu hosting Nakomi con plan, dominio, acceso SFTP y checkout seguro."
                path={path}
            />

            <form className="vpsConfiguradorLayout" onSubmit={handleSubmit}>
                <main className="vpsConfiguradorPrincipal">
                    <section className="vpsConfiguradorBloque">
                        <div className="vpsConfiguradorBloqueHeader">
                            <Server size={18} />
                            <h2 className="vpsConfiguradorBloqueTitulo">Plan</h2>
                        </div>
                        <SelectDropdown
                            value={form.selectedPlan || selectedPlan?.id || ''}
                            opciones={plans.map(item => ({value: item.id, label: `${item.label} — ${formatHostingMoney(item.priceCents)}/mes`}))}
                            onChange={value => updateField('selectedPlan', value)}
                            ariaLabel="Seleccionar plan de hosting"
                        />
                    </section>

                    {selectedPlan && (
                        <section className="vpsConfiguradorBloque">
                            <div className="vpsConfiguradorBloqueHeader">
                                <HardDrive size={18} />
                                <h2 className="vpsConfiguradorBloqueTitulo">Recursos incluidos</h2>
                            </div>
                            <div className="vpsConfiguradorSpecsGrid">
                                <div className="vpsConfiguradorSpec">
                                    <span>Almacenamiento</span>
                                    <strong>{formatHostingStorage(selectedPlan.storageMb)}</strong>
                                </div>
                                <div className="vpsConfiguradorSpec">
                                    <span>Tráfico</span>
                                    <strong>Tráfico ilimitado</strong>
                                </div>
                            </div>
                            <ul className="hostingConfiguradorBeneficios">
                                {selectedPlan.features.map(feature => <li key={feature}>{feature}</li>)}
                            </ul>
                        </section>
                    )}

                    <section className="vpsConfiguradorBloque">
                        <div className="vpsConfiguradorBloqueHeader">
                            <CreditCard size={18} />
                            <h2 className="vpsConfiguradorBloqueTitulo">Periodo de pago</h2>
                        </div>
                        <div className="hostingConfiguradorPagoOpciones">
                            {HOSTING_BILLING_OPTIONS.map(option => {
                                const activo = billingCycleMonths === option.months;
                                return (
                                    <Button
                                        key={option.months}
                                        type="button"
                                        variante="outline"
                                        tamano="pequeno"
                                        className={`vpsConfiguradorStorageOpcion${activo ? ' vpsConfiguradorStorageOpcionActivo' : ''}`}
                                        onClick={() => updateField('billingCycle', String(option.months))}
                                    >
                                        <span className="hostingConfiguradorPagoLabel">{option.label}</span>
                                        <span className="hostingConfiguradorPagoDesc">{option.description}</span>
                                    </Button>
                                );
                            })}
                        </div>
                    </section>

                    <section className="vpsConfiguradorBloque">
                        <div className="vpsConfiguradorBloqueHeader">
                            <Globe size={18} />
                            <h2 className="vpsConfiguradorBloqueTitulo">Dominio</h2>
                        </div>
                        <label className="vpsConfiguradorCampo">
                            <span>Dominio propio</span>
                            <Input
                                type="text"
                                value={form.domain}
                                onChange={event => updateField('domain', event.target.value)}
                                placeholder="ejemplo.com (opcional)"
                            />
                        </label>
                        <p className="vpsConfiguradorMuted">Si lo dejas vacío, recibirás un Free temporary domain para publicar primero y conectar tu dominio después.</p>
                    </section>

                    {isWordPress && (
                        <section className="vpsConfiguradorBloque">
                            <div className="vpsConfiguradorBloqueHeader">
                                <Languages size={18} />
                                <h2 className="vpsConfiguradorBloqueTitulo">WordPress</h2>
                            </div>
                            <label className="vpsConfiguradorCampo">
                                <span>Usuario wp-admin</span>
                                <Input
                                    type="text"
                                    value={form.wpAdminUser}
                                    onChange={event => updateField('wpAdminUser', event.target.value)}
                                    placeholder="Opcional"
                                />
                            </label>
                            <label className="vpsConfiguradorCampo">
                                <span>Contraseña wp-admin</span>
                                <Input
                                    type="password"
                                    value={form.wpAdminPassword}
                                    onChange={event => updateField('wpAdminPassword', event.target.value)}
                                    placeholder="Opcional, mínimo 8 caracteres"
                                    minLength={8}
                                />
                            </label>
                            <div className="hostingConfiguradorSelectWrap">
                                <span className="hostingConfiguradorSelectLabel">Idioma</span>
                                <SelectDropdown
                                    value={form.wpLanguage}
                                    opciones={HOSTING_LANGUAGE_OPTIONS}
                                    onChange={value => updateField('wpLanguage', value)}
                                    ariaLabel="Seleccionar idioma de WordPress"
                                />
                            </div>
                        </section>
                    )}

                    <section className="vpsConfiguradorBloque">
                        <div className="vpsConfiguradorBloqueHeader">
                            <KeyRound size={18} />
                            <h2 className="vpsConfiguradorBloqueTitulo">Acceso SFTP</h2>
                        </div>
                        <label className="vpsConfiguradorCampo">
                            <span>Usuario SFTP</span>
                            <Input
                                type="text"
                                value={form.sftpUser}
                                onChange={event => updateField('sftpUser', event.target.value)}
                                placeholder="Opcional"
                            />
                        </label>
                        <label className="vpsConfiguradorCampo">
                            <span>Contraseña SFTP</span>
                            <Input
                                type="password"
                                value={form.sftpPassword}
                                onChange={event => updateField('sftpPassword', event.target.value)}
                                placeholder="Opcional, mínimo 12 caracteres si la defines"
                                minLength={form.sftpPassword ? 12 : undefined}
                            />
                        </label>
                        <p className="vpsConfiguradorMuted">Puedes dejarlo vacío; generaremos credenciales seguras automáticamente al provisionar el hosting.</p>
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
                                <span>{selectedPlan.label}</span>
                                <strong>{formatHostingMoney(selectedPlan.priceCents)}/mes</strong>
                            </div>
                            <div className="vpsConfiguradorResumenFila">
                                <span>Periodo</span>
                                <strong>{billingCycleMonths === 12 ? '1 año' : `${billingCycleMonths} mes${billingCycleMonths > 1 ? 'es' : ''}`}</strong>
                            </div>
                            {discountCents > 0 && (
                                <div className="vpsConfiguradorResumenFila">
                                    <span>Descuento</span>
                                    <strong>-{formatHostingMoney(discountCents)}</strong>
                                </div>
                            )}
                            <div className="vpsConfiguradorResumenFila">
                                <span>Dominio temporal</span>
                                <strong>Incluido</strong>
                            </div>
                            <div className="vpsConfiguradorResumenFila">
                                <span>SSL</span>
                                <strong>Incluido</strong>
                            </div>
                            {isWordPress && (
                                <div className="vpsConfiguradorResumenFila">
                                    <span>CDN</span>
                                    <strong>Incluido</strong>
                                </div>
                            )}
                            {stripeFeeCents > 0 && (
                                <div className="vpsConfiguradorResumenFila">
                                    <span>Procesamiento de pago</span>
                                    <strong>+{formatHostingMoney(stripeFeeCents)}</strong>
                                </div>
                            )}
                            <div className="vpsConfiguradorResumenTotal">
                                <span>Hoy</span>
                                <strong>{formatHostingMoney(dueToday)}</strong>
                            </div>
                            <p className="vpsConfiguradorResumenNota">Se renueva a {formatHostingMoney(selectedPlan.priceCents)}/mes.</p>
                        </>
                    ) : (
                        <p className="vpsConfiguradorMuted">No hay planes de hosting disponibles.</p>
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