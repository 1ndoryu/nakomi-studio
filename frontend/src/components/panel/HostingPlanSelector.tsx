/* [094A-3] Selector de planes de WordPress hosting para self-service.
 * [114A-5] Especialización WordPress.
 * Muestra 3 cards (Básico, Pro, E-commerce) con features y precio.
 * Al seleccionar, opcionalmente pide dominio, y redirige a Stripe Checkout. */

import React, {useState} from 'react';
import {Check} from 'lucide-react';
import {Button} from '../ui/Button';
import {Input} from '../ui/Input';
import type {HostingPlanInfo} from '../../api/hosting';
import {useHostingCatalog} from '../../hooks/useHostingCatalog';
import './HostingPlanSelector.css';

interface HostingPlanSelectorProps {
    onSelect: (plan: string, domain?: string) => void;
    loading: boolean;
}

export const HostingPlanSelector: React.FC<HostingPlanSelectorProps> = ({
    onSelect,
    loading,
}) => {
    const {plans} = useHostingCatalog();
    const [selectedPlan, setSelectedPlan] = useState<string | null>(null);
    const [domain, setDomain] = useState('');
    const [step, setStep] = useState<'plans' | 'domain'>('plans');

    const handlePlanClick = (plan: HostingPlanInfo) => {
        setSelectedPlan(plan.id);
        setStep('domain');
    };

    const handleConfirm = () => {
        if (!selectedPlan) return;
        onSelect(selectedPlan, domain.trim() || undefined);
    };

    if (step === 'domain' && selectedPlan) {
        const plan = plans.find(p => p.id === selectedPlan);
        return (
            <div className="planSelectorDomain">
                <h3>Dominio para tu WordPress hosting {plan?.label}</h3>
                <p className="planSelectorDomainHint">
                    Si ya tienes un dominio, ingrésalo abajo. Si no, puedes dejarlo vacío y configurarlo después.
                </p>
                <Input
                    type="text"
                    placeholder="ejemplo.com (opcional)"
                    value={domain}
                    onChange={e => setDomain(e.target.value)}
                />
                <div className="planSelectorDomainActions">
                    <Button
                        type="button"
                        variante="texto"
                        onClick={() => setStep('plans')}
                    >
                        Volver
                    </Button>
                    <Button
                        type="button"
                        variante="primario"
                        onClick={handleConfirm}
                        disabled={loading}
                    >
                        {loading ? 'Procesando…' : 'Continuar al pago'}
                    </Button>
                </div>
            </div>
        );
    }

    return (
        <div className="planSelectorGrid">
            <h3 className="planSelectorTitulo">Elige tu plan de WordPress hosting</h3>
            <div className="planSelectorCards">
                {plans.map(plan => (
                    <div
                        key={plan.id}
                        className={`planCard ${selectedPlan === plan.id ? 'planCard--selected' : ''} ${plan.recommended ? 'planCard--recommended' : ''}`}
                        role="button"
                        tabIndex={0}
                        onClick={() => handlePlanClick(plan)}
                        onKeyDown={e => { if (e.key === 'Enter') handlePlanClick(plan); }}
                    >
                        {plan.recommended && (
                            <span className="planCardBadge">Recomendado</span>
                        )}
                        <h4 className="planCardName">{plan.label}</h4>
                        <div className="planCardPrice">
                            <span className="planCardPriceAmount">
                                ${(plan.priceCents / 100).toFixed(0)}
                            </span>
                            <span className="planCardPricePeriod">/mes</span>
                        </div>
                        <ul className="planCardFeatures">
                            {plan.features.map(f => (
                                <li key={f}>
                                    <Check size={14} />
                                    <span>{f}</span>
                                </li>
                            ))}
                        </ul>
                        <Button
                            type="button"
                            variante={plan.id === 'pro' ? 'primario' : 'secundario'}
                            className="planCardBtn"
                        >
                            Seleccionar
                        </Button>
                    </div>
                ))}
            </div>
        </div>
    );
};
