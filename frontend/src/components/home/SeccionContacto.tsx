/**
 * Componente: SeccionContacto (CTA)
 * Section "Have a project in mind?" reutilizable.
 * Acepta prop compacto para reducir padding cuando el contenedor padre ya tiene gap.
 * [064A-5] El botón primario abre el ChatWidget via useChatStore.
 */
import React from 'react';
import {useTranslation} from 'react-i18next';
import {SeccionHeader} from '../ui/SeccionHeader';
import {Button} from '../ui/Button';
import {useChatStore} from '../../stores/chatStore';
import {navegar} from '../../navegacionSPA';
import './SeccionContacto.css';

interface SeccionContactoProps {
    compacto?: boolean;
}

export const SeccionContacto: React.FC<SeccionContactoProps> = ({compacto = false}) => {
    const {t} = useTranslation();
    const abrirChat = useChatStore(s => s.abrir);
    const claseExtra = compacto ? 'seccionContactoCompacta' : '';

    return (
        <section className={`seccionContacto ${claseExtra}`} id="contacto">
            <div className="contactoContenedor">
                <SeccionHeader titulo={t('contact.section_title')} />
                <h2 className="contactoTitulo">{t('contact.heading')}</h2>

                <div className="contactoDescripcion">
                    <p>{t('contact.description')}</p>
                </div>

                <div className="contactoBotones">
                    <Button variante="primario" onClick={() => abrirChat()}>
                        {t('contact.btn_contact')}
                    </Button>
                    <Button variante="outline" onClick={() => navegar('/servicios/')}>
                        {t('contact.btn_hire')}
                    </Button>
                </div>
            </div>
        </section>
    );
};
