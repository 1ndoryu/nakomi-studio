/**
 * Componente: NosotrosIsland
 * Página "Sobre Nosotros" con misión, equipo, marcas y testimonios.
 * [074A-13] Equipo cargado desde API pública con fallback a datos estáticos.
 */
import React, {useState, useEffect} from 'react';
import {useTranslation} from 'react-i18next';
import '../styles/variables.css';
import './NosotrosIsland.css';
import {LayoutPagina} from '../components/layout/LayoutPagina';
import {SEOHead} from '../components/seo/SEOHead';
import {SeccionHeader} from '../components/ui/SeccionHeader';
import {AdminOverlay} from '../components/ui/AdminOverlay';
import OptimizedImage from '../components/ui/OptimizedImage';
import {SeccionContacto} from '../components/home/SeccionContacto';
import {MIEMBROS_DATA} from '../data/miembros';
import {Miembro} from '../types/contenido';
import {apiListPublicTeamMembers} from '../api/admin-team';
import type {AdminTeamMember} from '../api/admin-team';

interface NosotrosIslandProps {
    titulo?: string;
}

/* [064A-64] TarjetaMiembro traduce bio y cargo via i18n content translations */
const TarjetaMiembro: React.FC<{miembro: Miembro}> = ({miembro}) => {
    const {t} = useTranslation();
    return (
    <AdminOverlay contentType="team" itemId={miembro.adminId || miembro.id}>
        <article className="tarjetaMiembro">
            <div className="miembroAvatar">
                <OptimizedImage src={miembro.avatar} alt={miembro.nombre} width={150} height={150} sizes="150px" />
            </div>
            <div className="miembroInfo">
                <h3 className="miembroNombre">{miembro.nombre}</h3>
                <span className="miembroCargo">{t(`content.team.${miembro.id}.cargo`, miembro.cargo)}</span>
                <p className="miembroBio">{t(`content.team.${miembro.id}.bio`, miembro.bio)}</p>
                <div className="miembroRedes">
                    {miembro.linkedin && (
                        <a href={miembro.linkedin} target="_blank" rel="noopener noreferrer" aria-label="LinkedIn">
                            LinkedIn
                        </a>
                    )}
                    {miembro.github && (
                        <a href={miembro.github} target="_blank" rel="noopener noreferrer" aria-label="GitHub">
                            GitHub
                        </a>
                    )}
                </div>
            </div>
        </article>
    </AdminOverlay>
    );
};

/* [074A-13] Convierte AdminTeamMember (API) → Miembro (frontend) */
function convertirMiembro(m: AdminTeamMember): Miembro {
    return {
        id: m.slug,
        adminId: m.id,
        nombre: m.name,
        cargo: m.role,
        bio: m.bio,
        avatar: m.avatar || '/images/avatar-placeholder.webp',
        linkedin: m.linkedin || undefined,
        twitter: m.twitter || undefined,
        github: m.github || undefined,
    };
}

export const NosotrosIsland = ({titulo}: NosotrosIslandProps): JSX.Element => {
    const {t} = useTranslation();
    const [miembros, setMiembros] = useState<Miembro[]>(MIEMBROS_DATA);

    /* [074A-13] Cargar equipo desde API pública, fallback a datos estáticos */
    useEffect(() => {
        const ctrl = new AbortController();
        apiListPublicTeamMembers()
            .then(data => {
                if (!ctrl.signal.aborted && data.length > 0) {
                    setMiembros(data.map(convertirMiembro));
                }
            })
            .catch(() => {
                /* Fallback silencioso: se mantiene MIEMBROS_DATA */
            });
        return () => ctrl.abort();
    }, []);

    return (
        <LayoutPagina className="nosotrosMain" id="paginaNosotros">
            <SEOHead
                title="Nosotros"
                description="Conoce al equipo de Nakomi Studio, nuestra misión y valores."
                path="/nosotros"
            />
            {/* Hero */}
            <section className="nosotrosHero">
                <div className="nosotrosHeroContenido">
                    <div className="nosotrosHeroTexto">
                        <h1 className="nosotrosHeroTitulo">{titulo || t('about.title')}</h1>
                    </div>
                    <div className="nosotrosHeroDescripcion">
                        <p>{t('about.description')}</p>
                    </div>
                </div>
            </section>

            {/* Misión / Valores */}
            <section className="nosotrosMision">
                <div className="nosotrosMisionContenedor">
                    <div className="misionGrid">
                        <div className="misionItem">
                            <h3 className="misionItemTitulo">{t('about.mission_title')}</h3>
                            <p className="misionItemTexto">{t('about.mission_text')}</p>
                        </div>
                        <div className="misionItem">
                            <h3 className="misionItemTitulo">{t('about.approach_title')}</h3>
                            <p className="misionItemTexto">{t('about.approach_text')}</p>
                        </div>
                        <div className="misionItem">
                            <h3 className="misionItemTitulo">{t('about.values_title')}</h3>
                            <p className="misionItemTexto">{t('about.values_text')}</p>
                        </div>
                    </div>
                </div>
            </section>

            {/* Equipo */}
            <section className="nosotrosEquipo">
                <div className="nosotrosEquipoContenedor">
                    <SeccionHeader titulo={t('about.team_title')} />
                    <div className="equipoGrid">
                        {miembros.map(miembro => (
                            <TarjetaMiembro key={miembro.id} miembro={miembro} />
                        ))}
                    </div>
                </div>
            </section>

            {/* Contacto */}
            <SeccionContacto />
        </LayoutPagina>
    );
};

export default NosotrosIsland;
