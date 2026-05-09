/**
 * Componente: Header
 * Cabecera global del sitio.
 * Enlaces centralizados en data/navegacion.ts (DRY).
 * Incluye submenú dropdown para "Soluciones".
 * [044A-13] Sesión conectada con authStore (Zustand + JWT).
 * Accesibilidad: aria-labels, aria-expanded, navegación por teclado.
 * [054A-19] Lógica extraída a useHeader (SRP). Links internos usan GloryLink.
 * [064A-61] Menú móvil rediseñado: overlay modal centrado, soporte submenús,
 * botón volver, acciones inline. accionCabecera oculto en mobile via CSS. */
import React, {useState} from 'react';
import {useTranslation} from 'react-i18next';
import {ChevronDown, ChevronRight, ArrowLeft, Menu, X, LogOut} from 'lucide-react';
import {Button} from '../ui/Button';
import {Modal} from '../ui/Modal';
import {MenuContextual} from '../ui/ContextMenu';
import {ENLACES_HEADER} from '../../data/navegacion';
import {ModalAutenticacion} from './ModalAutenticacion';
import {useChatStore} from '../../stores/chatStore';
import {useAuthStore} from '../../stores/authStore';
import {useCurrentProfile} from '../../hooks/useCurrentProfile';
import {GloryLink} from '../../core/router';
import {Logo} from '../ui/Logo';
import OptimizedImage from '../ui/OptimizedImage';
import {useHeader} from '../../hooks/useHeader';
import '../../styles/header.css';

/* [044A-2] Mapeo de labels estáticos (español) a claves i18n.
 * navegacion.ts mantiene los labels originales como keys de referencia. */
const NAV_KEYS: Record<string, string> = {
    'Inicio': 'nav.home',
    'Servicios': 'nav.services',
    'Proyectos': 'nav.projects',
    'Nosotros': 'nav.about',
    'Blog': 'nav.blog',
    'Soluciones': 'nav.solutions',
    'WordPress Hosting': 'nav.hosting',
};

export const Header: React.FC = () => {
    const {t} = useTranslation();
    const {
        dropdownAbierto, setDropdownAbierto,
        modalAbierto, setModalAbierto,
        menuMovilAbierto, setMenuMovilAbierto,
        subMenuMovil, setSubMenuMovil,
        cerrarMenuMovil,
        logueado, enPanel,
        dropdownRef,
        handleKeyDownDropdown,
        handleKeyDownSubmenu,
    } = useHeader();

    /* Texto y destino del botón de sesión / panel */
    const textoAccion = logueado ? (enPanel ? t('nav.back') : t('nav.panel')) : null;
    const hrefAccion = logueado ? (enPanel ? '/' : '/panel/') : null;

    /* [064A-5] Botón secundario: abre el chat en vez de navegar a /contacto */
    const textoCta = logueado ? t('nav.chat') : t('nav.contact');
    const abrirChat = useChatStore(s => s.abrir);

    /* [074A-22] Avatar con dropdown para cerrar sesión */
    const logout = useAuthStore(s => s.logout);
    const {avatarUrl} = useCurrentProfile();
    const [perfilAbierto, setPerfilAbierto] = useState(false);

    /* [064A-61] Enlace con submenú actualmente abierto en el menú móvil */
    const enlaceSubMenuActivo = subMenuMovil
        ? ENLACES_HEADER.find(l => l.label === subMenuMovil)
        : null;

    return (
        <>
            <header className="cabeceraPrincipal" role="banner">
                <div className="cabeceraPrincipalInterior">
                <div className="logoContenedor">
                    <GloryLink to="/" className="logoEnlace" aria-label={t('accessibility.logo_home')}>
                        <Logo className="logoSvg" />
                    </GloryLink>
                </div>

                {/* Botón hamburguesa para móvil */}
                <Button variante="texto" className="controlMenuMovil" onClick={() => menuMovilAbierto ? cerrarMenuMovil() : setMenuMovilAbierto(true)} aria-expanded={menuMovilAbierto} aria-controls="menu-movil" aria-label={menuMovilAbierto ? t('accessibility.close_menu') : t('accessibility.open_menu')}>
                    {menuMovilAbierto ? <X size={24} /> : <Menu size={24} />}
                </Button>

                {/* [064A-61] Navegación desktop: solo visible en desktop */}
                <nav className="navegacionPrincipal" aria-label={t('accessibility.main_nav')}>
                    {ENLACES_HEADER.map(link => (
                        <div key={link.label} className="enlaceNavegacionWrapper" ref={link.hasDropdown ? dropdownRef : undefined} onMouseEnter={() => (link.hasDropdown ? setDropdownAbierto(link.label) : null)} onMouseLeave={() => setDropdownAbierto(null)}>
                            <GloryLink to={link.href} className="enlaceNavegacion" aria-expanded={link.hasDropdown ? dropdownAbierto === link.label : undefined} aria-haspopup={link.hasDropdown ? 'true' : undefined} onKeyDown={link.hasDropdown ? e => handleKeyDownDropdown(e, link.label) : undefined}>
                                {t(NAV_KEYS[link.label] || link.label)}
                                {link.hasDropdown && <ChevronDown size={14} className="iconoDesplegable" aria-hidden="true" />}
                            </GloryLink>
                            {link.hasDropdown && link.subEnlaces && dropdownAbierto === link.label && (
                                <div className="subMenuDesplegable" role="menu" aria-label={`${t('accessibility.main_nav')}: ${t(NAV_KEYS[link.label] || link.label)}`}>
                                    {link.subEnlaces.map(sub => (
                                        <GloryLink key={sub.label} to={sub.href} className="subMenuEnlace" role="menuitem" tabIndex={0} onKeyDown={handleKeyDownSubmenu}>
                                            {t(NAV_KEYS[sub.label] || sub.label)}
                                        </GloryLink>
                                    ))}
                                </div>
                            )}
                        </div>
                    ))}
                </nav>

                {/* [064A-61] accionCabecera: oculto en mobile via CSS */}
                <div className="accionCabecera" role="group" aria-label={t('accessibility.user_actions')}>
                    {logueado ? (
                        <>
                            {/* [074A-22] Navegación Panel/Volver (solo navegación, sin logout) */}
                            <GloryLink to={hrefAccion!} className="enlaceAcceder">
                                {textoAccion}
                            </GloryLink>
                            {/* [074A-22] Avatar con MenuContextual para cerrar sesión */}
                            <MenuContextual
                                abierto={perfilAbierto}
                                onToggle={() => setPerfilAbierto(prev => !prev)}
                                onCerrar={() => setPerfilAbierto(false)}
                                ariaLabel={t('accessibility.user_actions')}
                                className="perfilDropdownWrapper"
                                triggerClassName="perfilAvatarBtn"
                                triggerContent={<OptimizedImage src={avatarUrl} alt="Perfil" className="perfilAvatarImg" loading="eager" />}
                                items={[{
                                    id: 'logout',
                                    label: t('nav.logout'),
                                    icon: <LogOut size={14} />,
                                    onSelect: logout,
                                }]}
                            />
                        </>
                    ) : (
                        <Button variante="texto" className="enlaceAcceder" onClick={() => setModalAbierto(true)}>
                            {t('nav.login')}
                        </Button>
                    )}
                    <Button variante="primario" tamano="pequeno" className="accionHeaderPrincipal" onClick={() => abrirChat()}>
                        {textoCta}
                        <ChevronRight size={14} strokeWidth={3} aria-hidden="true" />
                    </Button>
                </div>
                </div>
            </header>

            {/* [064A-61] Menú móvil: overlay modal centrado con glassmorphism.
                Usa <Modal> como base de overlay/focus-trap, no es un diálogo genérico. */}
            {/* sentinel-disable-next-line modal-estructura-no-canonica */}
            <Modal abierto={menuMovilAbierto} onCerrar={cerrarMenuMovil} className="menuMovilPanel">
                <nav role="navigation" aria-label={t('accessibility.main_nav')}>
                    {/* Vista principal o submenú */}
                    {!subMenuMovil ? (
                        <div className="menuMovilLista">
                            {ENLACES_HEADER.map(link => (
                                link.hasDropdown && link.subEnlaces ? (
                                    <div
                                        key={link.label}
                                        role="button"
                                        tabIndex={0}
                                        className="menuMovilEnlace menuMovilEnlaceConSub"
                                        onClick={() => setSubMenuMovil(link.label)}
                                        onKeyDown={e => { if (e.key === 'Enter' || e.key === ' ') setSubMenuMovil(link.label); }}
                                    >
                                        {t(NAV_KEYS[link.label] || link.label)}
                                        <ChevronRight size={16} aria-hidden="true" />
                                    </div>
                                ) : (
                                    <GloryLink key={link.label} to={link.href} className="menuMovilEnlace" onClick={cerrarMenuMovil}>
                                        {t(NAV_KEYS[link.label] || link.label)}
                                    </GloryLink>
                                )
                            ))}

                            {/* Acciones de usuario dentro del menú móvil */}
                            {logueado ? (
                                <>
                                    {/* [074A-25] Solo navegación en mobile, logout via avatar dropdown */}
                                    <GloryLink to={hrefAccion!} className="menuMovilEnlace" onClick={cerrarMenuMovil}>
                                        {textoAccion}
                                    </GloryLink>
                                </>
                            ) : (
                                <div
                                    role="button"
                                    tabIndex={0}
                                    className="menuMovilEnlace"
                                    onClick={() => { setModalAbierto(true); cerrarMenuMovil(); }}
                                    onKeyDown={e => { if (e.key === 'Enter') { setModalAbierto(true); cerrarMenuMovil(); } }}
                                >
                                    {t('nav.login')}
                                </div>
                            )}
                            <div
                                role="button"
                                tabIndex={0}
                                className="menuMovilEnlace menuMovilEnlaceCta"
                                onClick={() => { abrirChat(); cerrarMenuMovil(); }}
                                onKeyDown={e => { if (e.key === 'Enter') { abrirChat(); cerrarMenuMovil(); } }}
                            >
                                {textoCta}
                            </div>
                        </div>
                    ) : (
                        <div className="menuMovilLista">
                            <div
                                role="button"
                                tabIndex={0}
                                className="menuMovilVolver"
                                onClick={() => setSubMenuMovil(null)}
                                onKeyDown={e => { if (e.key === 'Enter') setSubMenuMovil(null); }}
                            >
                                <ArrowLeft size={16} /> {t('common.back', 'Volver')}
                            </div>
                            {enlaceSubMenuActivo?.subEnlaces?.map(sub => (
                                <GloryLink key={sub.label} to={sub.href} className="menuMovilEnlace" onClick={cerrarMenuMovil}>
                                    {t(NAV_KEYS[sub.label] || sub.label)}
                                </GloryLink>
                            ))}
                        </div>
                    )}
                </nav>
            </Modal>

            {!logueado && <ModalAutenticacion abierto={modalAbierto} onCerrar={() => setModalAbierto(false)} />}
        </>
    );
};
