/* [044A-2] Selector de idioma compacto para el Header.
 * Muestra código del idioma actual y dropdown con opciones.
 * [054A-17] Refactorizado de dropdown artesanal a MenuContextual. */
import {useState} from 'react';
import {useTranslation} from 'react-i18next';
import {MenuContextual} from './ContextMenu';

const LANGUAGES = [
    {code: 'es', label: 'ES', nombre: 'Español'},
    {code: 'en', label: 'EN', nombre: 'English'},
    {code: 'ja', label: 'JA', nombre: '日本語'}
];

export const LanguageSelector = () => {
    const {i18n} = useTranslation();
    const [open, setOpen] = useState(false);

    const currentLang = LANGUAGES.find(l => l.code === i18n.language) || LANGUAGES[0];

    const cambiarIdioma = (code: string) => {
        i18n.changeLanguage(code);
        setOpen(false);
    };

    const menuItems = LANGUAGES.map(lang => ({
        id: lang.code,
        label: lang.nombre,
        onSelect: () => cambiarIdioma(lang.code),
    }));

    return (
        <MenuContextual
            abierto={open}
            onToggle={() => setOpen(o => !o)}
            onCerrar={() => setOpen(false)}
            ariaLabel="Seleccionar idioma"
            triggerContent={currentLang.label}
            items={menuItems}
            contexto="oscuro"
        />
    );
};
