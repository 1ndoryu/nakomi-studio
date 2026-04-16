/* [094A-6] Tab de Facturación del detalle de hosting.
 * Extraído de HostingDetalleTabs.tsx para cumplir límite 300 líneas.
 * Incluye plan actual con features, cambio de plan (upgrade/downgrade). */

import {useState} from 'react';
import {ArrowUpCircle} from 'lucide-react';
import type {useHostingDetalle} from '../../hooks/useHostingDetalle';
import {HOSTING_PLAN_LABELS} from '../../api/hosting';
import {useHostingCatalog} from '../../hooks/useHostingCatalog';
import {Button} from '../ui/Button';

type Subscription = NonNullable<ReturnType<typeof useHostingDetalle>['subscription']>;

export function TabFacturacion({sub, onPlanChange, planChangeLoading}: {
    sub: Subscription;
    onPlanChange?: (plan: string, domain?: string) => void;
    planChangeLoading?: boolean;
}) {
    const {plans} = useHostingCatalog();
    const [showPlanChange, setShowPlanChange] = useState(false);
    const currentPlanInfo = plans.find(p => p.id === sub.plan);
    const otherPlans = plans.filter(p => p.id !== sub.plan);

    const handleSelectPlan = (planId: string) => {
        if (onPlanChange) {
            onPlanChange(planId, sub.domain ?? undefined);
            setShowPlanChange(false);
        }
    };

    return (
        <div className="hostingDetalleSection">
            <h3 className="hostingDetalleSectionTitle">Plan actual</h3>
            <div className="hostingDetallePlanCard">
                <div className="hostingDetallePlanInfo">
                    <span className="hostingDetallePlanNombre">
                        {HOSTING_PLAN_LABELS[sub.plan] || sub.plan}
                    </span>
                    <span className="hostingDetallePlanPrecio">
                        ${(sub.monthly_price_cents / 100).toFixed(0)}/mes
                    </span>
                </div>
                <div className="hostingDetallePlanFeatures">
                    {currentPlanInfo?.features.map(f => (
                        <span key={f}>{f}</span>
                    )) ?? (
                        <>
                            <span>Almacenamiento: {(sub.storage_limit_mb / 1024).toFixed(0)} GB</span>
                            <span>SSL incluido</span>
                            <span>Backups automáticos</span>
                        </>
                    )}
                </div>
                {onPlanChange && sub.status !== 'cancelled' && (
                    <Button
                        type="button"
                        variante="outline"
                        tamano="pequeno"
                        onClick={() => setShowPlanChange(!showPlanChange)}
                        className="hostingDetallePlanChangeBtn"
                    >
                        <ArrowUpCircle size={14} />
                        {showPlanChange ? 'Cancelar cambio' : 'Cambiar plan'}
                    </Button>
                )}
            </div>

            {showPlanChange && (
                <div className="hostingDetallePlanOptions">
                    <h4 className="hostingDetalleSubTitle">Opciones disponibles</h4>
                    <p className="hostingDetalleSectionDesc">
                        El cambio se aplica inmediatamente. Se ajustará el cobro proporcionalmente.
                    </p>
                    <div className="hostingDetallePlanGrid">
                        {otherPlans.map(plan => {
                            const isUpgrade = plan.priceCents > (currentPlanInfo?.priceCents ?? 0);
                            return (
                                <div key={plan.id} className="hostingDetallePlanOption">
                                    <div className="hostingDetallePlanOptionHeader">
                                        <span className="hostingDetallePlanNombre">{plan.label}</span>
                                        <span className="hostingDetallePlanBadge">
                                            {isUpgrade ? 'Upgrade' : 'Downgrade'}
                                        </span>
                                    </div>
                                    <span className="hostingDetallePlanPrecio">
                                        ${(plan.priceCents / 100).toFixed(0)}/mes
                                    </span>
                                    {/* [084A-45] Beneficios como lista */}
                                    <ul className="hostingDetallePlanFeaturesList">
                                        {plan.features.map(f => <li key={f}>{f}</li>)}
                                    </ul>
                                    <Button
                                        type="button"
                                        variante={isUpgrade ? 'primario' : 'secundario'}
                                        tamano="pequeno"
                                        onClick={() => handleSelectPlan(plan.id)}
                                        disabled={planChangeLoading}
                                    >
                                        {planChangeLoading ? 'Cambiando…' : `Cambiar a ${plan.label}`}
                                    </Button>
                                </div>
                            );
                        })}
                    </div>
                </div>
            )}

            <h3 className="hostingDetalleSectionTitle">Historial de pagos</h3>
            <p className="hostingDetalleSectionDesc">
                Los pagos se gestionan a través de Stripe. Consulta tu historial de facturación
                en tu correo electrónico registrado.
            </p>
        </div>
    );
}
