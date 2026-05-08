/**
 * Componente: SeccionHero
 * Descripcion: Seccion principal (Hero) de la pagina de bienvenida.
 * Muestra un titulo, descripcion, boton de llamada a la accion y una galería de imágenes.
 * [074A-marketing] CTA conectado a /contacto (antes no tenía onClick ni href).
 * [074A-marketing] Badges diferenciadores: Rust, editorial, multilingüe.
 */

import './SeccionHero.css';
import {useTranslation} from 'react-i18next';
import {Button} from '../ui/Button';
import {GaleriaHero} from './GaleriaHero';
import {navegar} from '../../navegacionSPA';

export const SeccionHero = (): JSX.Element => {
    const {t} = useTranslation();

    return (
        <section className="seccionHero">
            <div className="heroContenido">
                <div>
                    <h1 className="heroTitulo">
                        {t('hero.title_start')}<span>{t('hero.title_web')}</span> y <span>{t('hero.title_software')}</span>.
                    </h1>
                </div>

                <div className="heroDescripcion">
                    <p>{t('hero.description')}</p>
                    <div className="heroBadges" aria-label="Diferenciadores">
                        <span className="heroBadge">{t('hero.badge_rust')}</span>
                        <span className="heroBadge">{t('hero.badge_editorial')}</span>
                        <span className="heroBadge">{t('hero.badge_langs')}</span>
                    </div>
                    <Button variante="primario" tamano="mediano" className="heroBoton" onClick={() => navegar('/contacto/')}>
                        {t('hero.cta')}
                    </Button>
                </div>
            </div>

            <div className="heroImagenFondo">
                <GaleriaHero />
            </div>
        </section>
    );
};
