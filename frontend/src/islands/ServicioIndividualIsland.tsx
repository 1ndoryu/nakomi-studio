/**
 * Componente: ServicioIndividualIsland
 * Página de detalle de un servicio individual.
 * Estructura: Hero -> Galeria -> CTA -> Skills -> Relacionados -> Contacto -> Footer
 * Skills y datos centralizados en data/ (DRY).
 * [044A-38 Fase 1] CTA "Contratar" apunta a /panel si logueado (Fase 2 conecta con API de órdenes).
 * [084A-4] Restaurado ModalCompra — click en plan abre modal de pago (revertido 074A-36).
 * sentinel-disable-file componente-sin-hook: Lógica minimal (fetch API + wiring de estado)
 * no justifica hook separado; datos estáticos vienen de imports directos.
 */
import {useState, useCallback, useEffect} from 'react';
import {useTranslation} from 'react-i18next';
import {useAuthStore} from '../stores/authStore';
import {useChatStore} from '../stores/chatStore';
import '../styles/variables.css';
import './ServicioIndividualIsland.css';
import {LayoutPagina} from '../components/layout/LayoutPagina';
import {SEOHead} from '../components/seo/SEOHead';
import {serviceSchema} from '../components/seo/schemas';
import {SeccionHeroServicio} from '../components/servicios/SeccionHeroServicio';
import {SeccionGaleriaServicio} from '../components/servicios/SeccionGaleriaServicio';
import {SeccionSkillsServicio} from '../components/servicios/SeccionSkillsServicio';
import {SeccionServiciosRelacionados} from '../components/servicios/SeccionServiciosRelacionados';
import {SeccionCta} from '../components/ui/SeccionCta';
import {SeccionPlanesServicio} from '../components/servicios/SeccionPlanesServicio';
import {ModalCompra} from '../components/servicios/ModalCompra';
import {SeccionContacto} from '../components/home/SeccionContacto';
import {SKILLS_POR_DEFECTO} from '../data/skills';
import {obtenerPlanesServicio} from '../data/planes/index';
import type {PlanServicio} from '../data/planes/tipos';
import {SERVICIOS_DATA} from '../data/servicios';
import {apiGetServiceBySlug, type PublicService} from '../api/admin-services';

/* [064A-47] Imagen del servicio añadida al hero. */
interface ServicioIndividualIslandProps {
    titulo?: string;
    descripcion?: string;
    precio_desde?: string;
    slug?: string;
    imagen?: string;
}

export const ServicioIndividualIsland = ({titulo, descripcion, precio_desde, slug, imagen}: ServicioIndividualIslandProps): JSX.Element => {
    const {t} = useTranslation();
    const logueado = useAuthStore(s => s.logueado);
    const abrirChat = useChatStore(s => s.abrir);

    /* [084A-4] Estado del modal de compra */
    const [planSeleccionado, setPlanSeleccionado] = useState<PlanServicio | null>(null);
    const handleSeleccionarPlan = useCallback((plan: PlanServicio) => setPlanSeleccionado(plan), []);
    const handleCerrarModal = useCallback(() => setPlanSeleccionado(null), []);

    /* [064A-5] Si logueado, CTA lleva al panel. Si no, abre el chat. */
    const ctaLink = logueado ? '/panel' : undefined;

    /* ID del servicio actual para excluirlo de relacionados */
    const servicioId = slug || titulo?.toLowerCase().replace(/\s+/g, '-') || '';

    /* [074A-21] Fetch servicio de la API con fallback a datos estáticos */
    const [apiData, setApiData] = useState<PublicService | null>(null);
    useEffect(() => {
        const controller = new AbortController();
        if (slug) {
            apiGetServiceBySlug(slug)
                .then(data => { if (!controller.signal.aborted) setApiData(data); })
                .catch(() => { /* mantiene fallback estático */ });
        }
        return () => controller.abort();
    }, [slug]);

    /* [064A-64] Buscar servicio por slug para traducir titulo y descripcion */
    const servicioData = SERVICIOS_DATA.find(s => slug && s.link.includes(slug));
    const tituloFinal = apiData?.title || (servicioData ? t(`content.services.${servicioData.id}.titulo`, {defaultValue: titulo ?? ''}) : titulo);
    const descripcionFinal = apiData?.description || (servicioData ? t(`content.services.${servicioData.id}.descripcion`, {defaultValue: descripcion ?? ''}) : descripcion);
    const imagenFinal = apiData?.image_url || imagen;
    const skillsApi = apiData?.skills ? (apiData.skills as Array<{titulo: string; descripcion: string}>).map((sk, i) => ({id: i, titulo: sk.titulo, descripcion: sk.descripcion})) : null;

    /* [044A-40] Obtener precio mínimo real de los planes del servicio. */
    const datosPlanes = obtenerPlanesServicio(servicioId);
    const precioMinimo = (() => {
        if (precio_desde) return precio_desde;
        if (!datosPlanes || datosPlanes.planes.length === 0) return '';
        const precios = datosPlanes.planes
            .map(p => {
                const num = parseFloat(p.precio.replace(/[^0-9.]/g, ''));
                return isNaN(num) ? Infinity : num;
            })
            .filter(n => n !== Infinity);
        if (precios.length === 0) return '';
        return `$${Math.min(...precios)}`;
    })();

    return (
        <LayoutPagina className="servicioIndividualMain">
            <SEOHead
                title={titulo}
                description={descripcion}
                path={`/servicios/${slug || ''}`}
                jsonLd={titulo && descripcion && slug ? serviceSchema(titulo, descripcion, slug) : undefined}
            />
            <SeccionHeroServicio titulo={tituloFinal} descripcion={descripcionFinal} imagen={imagenFinal} />
            <SeccionGaleriaServicio />
            <SeccionPlanesServicio slug={servicioId} onSeleccionarPlan={handleSeleccionarPlan} />
            {/* [084A-28] Pasar contexto de servicio al abrir chat para soporte contextual */}
            <SeccionCta descripcion={[t('service_detail.cta_1'), t('service_detail.cta_2')]} textoBotonPrimario={precioMinimo ? `${t('service_detail.cta_hire')} ${precioMinimo}` : t('service_detail.cta_hire')} linkBotonPrimario={ctaLink} onBotonPrimarioClick={logueado ? undefined : () => abrirChat(`service:${slug || servicioId}`)} textoBotonSecundario={t('service_detail.cta_contact')} onBotonSecundarioClick={() => abrirChat(`service:${slug || servicioId}`)} />
            <SeccionSkillsServicio skills={skillsApi || SKILLS_POR_DEFECTO} />
            <SeccionServiciosRelacionados servicioActualId={servicioId} />
            <SeccionContacto />
            {planSeleccionado && (
                <ModalCompra
                    plan={planSeleccionado}
                    servicioSlug={servicioId}
                    abierto={true}
                    onCerrar={handleCerrarModal}
                />
            )}
        </LayoutPagina>
    );
};

export default ServicioIndividualIsland;
