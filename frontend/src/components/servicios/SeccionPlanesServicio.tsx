/* [044A-40] Componente: SeccionPlanesServicio
 * Muestra las tarjetas de precios de un servicio.
 * [084A-4] CTA abre ModalCompra con el plan seleccionado (revertido 074A-36 que abría chat). */
import React from 'react';
import {useTranslation} from 'react-i18next';
import {obtenerPlanesServicio, type PlanServicio} from '../../data/planes/index';
import {Button} from '../ui/Button';
import './SeccionPlanesServicio.css';

interface SeccionPlanesServicioProps {
    slug?: string;
    planes?: PlanServicio[] | null;
    onSeleccionarPlan?: (plan: PlanServicio) => void;
}

interface TarjetaPlanProps {
    plan: PlanServicio;
    onSeleccionar?: (plan: PlanServicio) => void;
}

const TarjetaPlan: React.FC<TarjetaPlanProps> = ({plan, onSeleccionar}) => {
    const {t} = useTranslation();
    const claseDestacado = plan.destacado ? 'tarjetaPlanDestacado' : '';

    return (
        <div className={`tarjetaPlan ${claseDestacado}`}>
            {plan.destacado && <div className="tarjetaPlanBadge">{t('plans.recommended')}</div>}

            {/* [064A-64] Nombre, descripcion y CTA traducidos via content translations */}
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
                    onClick={() => onSeleccionar?.(plan)}
                >
                    {t(`content.plans.${plan.id}.cta`, plan.ctaTexto)}
                </Button>
            </div>
        </div>
    );
};

export const SeccionPlanesServicio: React.FC<SeccionPlanesServicioProps> = ({slug, planes, onSeleccionarPlan}) => {
    const {t} = useTranslation();
    const datos = planes && planes.length > 0
        ? {servicioSlug: slug || '', servicioTitulo: '', planes}
        : (slug ? obtenerPlanesServicio(slug) : null);

    if (!datos) return null;

    return (
        <section id="planesServicio" className="planesSeccion">
            <div className="planesContenedor">
                <div className="planesCabecera">
                    <h2 className="planesTitulo">{t('plans.title')}</h2>
                    <p className="planesSubtitulo">{t('plans.subtitle')}</p>
                </div>
                <div className="planesGrid">
                    {datos.planes.map(plan => (
                        <TarjetaPlan key={plan.id} plan={plan} onSeleccionar={onSeleccionarPlan} />
                    ))}
                </div>
            </div>
        </section>
    );
};
