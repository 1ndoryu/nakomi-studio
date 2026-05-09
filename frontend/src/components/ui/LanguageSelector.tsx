/* [044A-2] Selector de idioma compacto para el Header.
 * Muestra código del idioma actual y dropdown con opciones.
 * [054A-17] Refactorizado de dropdown artesanal a MenuContextual.
 * [095A] Sin className overrides: MenuContextual detecta posicion automaticamente. */
import {useState} from 'react';
import {useTranslation} from 'react-i18next';
import {MenuContextual} from './ContextMenu';
import {Button} from './Button';

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

    return (
        <MenuContextual
            abierto={open}
            onToggle={() => setOpen(o => !o)}
            onCerrar={() => setOpen(false)}
            ariaLabel="Seleccionar idioma"
            triggerContent={currentLang.label}
        >
            {LANGUAGES.map(lang => (
                <Button
                    key={lang.code}
                    variante="texto"
                    className={lang.code === i18n.language ? 'activo' : ''}
                    onClick={() => cambiarIdioma(lang.code)}
                    type="button"
                >
                    {lang.nombre}
                </Button>
            ))}
        </MenuContextual>
    );
};
