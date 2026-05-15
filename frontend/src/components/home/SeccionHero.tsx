/**
 * Componente: SeccionHero
 * Descripcion: Seccion principal (Hero) de la pagina de bienvenida.
 * Muestra un titulo, descripcion, boton de llamada a la accion y una galería de imágenes.
 */

import './SeccionHero.css';
import {useTranslation} from 'react-i18next';
import {Button} from '../ui/Button';
import {GaleriaHero} from './GaleriaHero';
import {useChatStore} from '../../stores/chatStore';

export const SeccionHero = (): JSX.Element => {
    const {t} = useTranslation();
    const abrirChat = useChatStore(s => s.abrir);

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
                    <Button variante="primario" tamano="mediano" className="heroBoton" onClick={() => abrirChat('hero:start-project')}>
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
