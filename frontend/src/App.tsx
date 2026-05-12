/* [044A-1] App principal con React Router.
 * Reemplaza el sistema de islands de WordPress por rutas SPA.
 * Cada island se convierte en una ruta. Las páginas de detalle
 * reciben el slug del URL param y buscan datos en data/.
 * [044A-38 Fase 1] Redirige / → /panel si el usuario está logueado.
 * [154A-6] Code splitting con React.lazy para rutas pesadas (PanelIsland, AdminEditorProvider). */

import {useLayoutEffect, lazy, Suspense} from 'react';
import {BrowserRouter, Routes, Route, useNavigate, useParams} from 'react-router-dom';
import {QueryClient, QueryClientProvider} from '@tanstack/react-query';
import {registrarNavigate} from './navegacionSPA';
import {ScrollToTop} from './components/ui/ScrollToTop';

/* Pages (ex-islands) — carga directa solo para rutas de aterrizaje principales */
import {BienvenidaIsland} from './islands/BienvenidaIsland';
import {ServiciosIsland} from './islands/ServiciosIsland';
import {ProyectosIsland} from './islands/ProyectosIsland';
import {NotFoundIsland} from './islands/NotFoundIsland';

/* [204A-2] Lazy: páginas de detalle y rutas secundarias.
 * Solo cargan cuando el usuario navega a ellas, reduciendo el CSS+JS inicial.
 * Las listings (Servicios, Proyectos, Blog) siguen eager porque son rutas de aterrizaje frecuentes. */
const ServicioIndividualIsland = lazy(() => import('./islands/ServicioIndividualIsland').then(m => ({default: m.ServicioIndividualIsland})));
const ProyectoIndividualIsland = lazy(() => import('./islands/ProyectoIndividualIsland').then(m => ({default: m.ProyectoIndividualIsland})));
const NosotrosIsland = lazy(() => import('./islands/NosotrosIsland').then(m => ({default: m.NosotrosIsland})));
const SolucionesIsland = lazy(() => import('./islands/SolucionesIsland').then(m => ({default: m.SolucionesIsland})));
const SolucionPlaceholderIsland = lazy(() => import('./islands/SolucionPlaceholderIsland').then(m => ({default: m.SolucionPlaceholderIsland})));
const SolucionHostingIsland = lazy(() => import('./islands/SolucionHostingIsland').then(m => ({default: m.SolucionHostingIsland})));
const SolucionVpsIsland = lazy(() => import('./islands/SolucionVpsIsland').then(m => ({default: m.SolucionVpsIsland})));
/* [125A-4] Portal VPS: landing page para vps.nakomi.studio */
const VpsPortalIsland = lazy(() => import('./islands/VpsPortalIsland').then(m => ({default: m.VpsPortalIsland})));

const UsuarioPublicoIsland = lazy(() => import('./islands/UsuarioPublicoIsland').then(m => ({default: m.UsuarioPublicoIsland})));
/* [095A-5] Página legal requerida por el footer */
const PrivacidadIsland = lazy(() => import('./islands/PrivacidadIsland').then(m => ({default: m.PrivacidadIsland})));

/* [054A-5] Toast system */
import {ToastContainer} from './components/ui/ToastContainer';

/* [154A-6] Lazy load: PanelIsland importa @tiptap (~350KB), Stripe, y componentes admin pesados.
 * AdminEditorProvider importa editores inline que solo usan admins.
 * ChatWidget es moderado pero no es crítico para el first paint. */
const PanelIsland = lazy(() => import('./islands/PanelIsland').then(m => ({default: m.PanelIsland})));
const AdminEditorProvider = lazy(() => import('./components/AdminEditorProvider').then(m => ({default: m.AdminEditorProvider})));
const ChatWidget = lazy(() => import('./components/chat/ChatWidget').then(m => ({default: m.ChatWidget})));

/* Data para resolver slugs */
import {PROYECTOS_DATA} from './data/showcase';

import './App.css';

const queryClient = new QueryClient({
    defaultOptions: {
        queries: {
            staleTime: 5 * 60 * 1000,
            retry: 1,
        },
    },
});

/* Registra navigate de React Router en el módulo navegacionSPA para compatibilidad */
function NavigateRegistrar() {
    const navigate = useNavigate();
    useLayoutEffect(() => {
        /* [065A-4] Registrar navigate antes del primer paint evita que CTAs tempranos
         * caigan al fallback window.location.href y recarguen el documento completo. */
        registrarNavigate((to: string) => navigate(to));
    }, [navigate]);
    return null;
}

/* Wrapper: resuelve slug de servicio a props */
function ServicioDetallePage() {
    const {slug} = useParams<{slug: string}>();
    return <ServicioIndividualIsland slug={slug} />;
}

/* Wrapper: resuelve slug de proyecto a props */
function ProyectoDetallePage() {
    const {slug} = useParams<{slug: string}>();
    const proyecto = PROYECTOS_DATA.find(p => {
        const pSlug = p.link?.split('/').filter(Boolean).pop() || '';
        return pSlug === slug || String(p.id) === slug;
    });
    return (
        <ProyectoIndividualIsland
            titulo={proyecto?.titulo}
            descripcion={proyecto?.descripcion}
            cliente={proyecto?.cliente}
            categorias={Array.isArray(proyecto?.categorias) ? proyecto.categorias.join(', ') : proyecto?.categorias}
            imagen={proyecto?.imagen}
            slug={slug}
        />
    );
}

function HomePage() {
    /* [125A-4] Detección de hostname: vps.nakomi.studio → portal VPS, resto → home principal.
     * Para pruebas locales usar la ruta /portal-vps. */
    if (window.location.hostname === 'vps.nakomi.studio') {
        return <Suspense fallback={null}><VpsPortalIsland /></Suspense>;
    }
    return <BienvenidaIsland />;
}

/* [074A-1] Home siempre muestra BienvenidaIsland, logueado o no.
 * Los usuarios logueados pueden navegar libremente por el sitio.
 * El panel se accede desde el header (botón "Panel"). */
function App() {
    return (
        <QueryClientProvider client={queryClient}>
            <ToastContainer />
            <BrowserRouter>
                <ScrollToTop />
                <NavigateRegistrar />
                <Routes>
                    <Route path="/" element={<HomePage />} />
                    <Route path="/servicios" element={<ServiciosIsland />} />
                    <Route path="/servicios/:slug" element={<Suspense fallback={null}><ServicioDetallePage /></Suspense>} />
                    <Route path="/proyectos" element={<ProyectosIsland />} />
                    <Route path="/proyectos/:slug" element={<Suspense fallback={null}><ProyectoDetallePage /></Suspense>} />
                    <Route path="/nosotros" element={<Suspense fallback={null}><NosotrosIsland /></Suspense>} />
                    <Route path="/soluciones" element={<Suspense fallback={null}><SolucionesIsland /></Suspense>} />
                    {/* [064A-32] Hosting tiene página propia, el resto usa placeholder */}
                    <Route path="/soluciones/hosting" element={<Suspense fallback={null}><SolucionHostingIsland /></Suspense>} />
                    <Route path="/soluciones/vps" element={<Suspense fallback={null}><SolucionVpsIsland /></Suspense>} />
                    {/* [125A-4] Ruta de desarrollo para previsualizar portal VPS localmente */}
                    <Route path="/portal-vps" element={<Suspense fallback={null}><VpsPortalIsland /></Suspense>} />
                    <Route path="/soluciones/:slug" element={<Suspense fallback={null}><SolucionPlaceholderIsland /></Suspense>} />
                    {/* [064A-5] Ruta /contacto eliminada — todos los CTAs abren el chat */}
                    {/* [095A-5] Política de privacidad accesible desde el footer */}
                    <Route path="/politica-privacidad" element={<Suspense fallback={null}><PrivacidadIsland /></Suspense>} />
                    <Route path="/usuario/:username" element={<Suspense fallback={null}><UsuarioPublicoIsland /></Suspense>} />
                    <Route path="/panel" element={<Suspense fallback={<div className="panelCargando" />}><PanelIsland /></Suspense>} />
                    <Route path="/panel/chat" element={<Suspense fallback={<div className="panelCargando" />}><PanelIsland /></Suspense>} />
                    {/* [044A-28] Página 404 real en vez de redirigir silenciosamente al home */}
                    <Route path="*" element={<NotFoundIsland />} />
                </Routes>
                {/* [054A-3] Chat flotante para visitantes (se oculta en /panel) */}
                <Suspense fallback={null}><ChatWidget /></Suspense>
                {/* [084A-29] Editores admin accesibles desde páginas públicas */}
                <Suspense fallback={null}><AdminEditorProvider /></Suspense>
            </BrowserRouter>
        </QueryClientProvider>
    );
}

export default App;
