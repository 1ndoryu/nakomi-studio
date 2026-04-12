/* [044A-1] App principal con React Router.
 * Reemplaza el sistema de islands de WordPress por rutas SPA.
 * Cada island se convierte en una ruta. Las páginas de detalle
 * reciben el slug del URL param y buscan datos en data/.
 * [044A-38 Fase 1] Redirige / → /panel si el usuario está logueado.
 * [154A-6] Code splitting con React.lazy para rutas pesadas (PanelIsland, AdminEditorProvider). */

import {useEffect, lazy, Suspense} from 'react';
import {BrowserRouter, Routes, Route, useNavigate, useParams} from 'react-router-dom';
import {QueryClient, QueryClientProvider} from '@tanstack/react-query';
import {registrarNavigate} from './navegacionSPA';
import {ScrollToTop} from './components/ui/ScrollToTop';

/* Pages (ex-islands) — carga directa solo para rutas de aterrizaje principales */
import {BienvenidaIsland} from './islands/BienvenidaIsland';
import {ServiciosIsland} from './islands/ServiciosIsland';
import {ProyectosIsland} from './islands/ProyectosIsland';
import {BlogIsland} from './islands/BlogIsland';
import {NotFoundIsland} from './islands/NotFoundIsland';

/* [204A-2] Lazy: páginas de detalle y rutas secundarias.
 * Solo cargan cuando el usuario navega a ellas, reduciendo el CSS+JS inicial.
 * Las listings (Servicios, Proyectos, Blog) siguen eager porque son rutas de aterrizaje frecuentes. */
const ServicioIndividualIsland = lazy(() => import('./islands/ServicioIndividualIsland').then(m => ({default: m.ServicioIndividualIsland})));
const ProyectoIndividualIsland = lazy(() => import('./islands/ProyectoIndividualIsland').then(m => ({default: m.ProyectoIndividualIsland})));
const BlogSingleIsland = lazy(() => import('./islands/BlogSingleIsland').then(m => ({default: m.BlogSingleIsland})));
const NosotrosIsland = lazy(() => import('./islands/NosotrosIsland').then(m => ({default: m.NosotrosIsland})));
const SolucionesIsland = lazy(() => import('./islands/SolucionesIsland').then(m => ({default: m.SolucionesIsland})));
const SolucionPlaceholderIsland = lazy(() => import('./islands/SolucionPlaceholderIsland').then(m => ({default: m.SolucionPlaceholderIsland})));
const SolucionHostingIsland = lazy(() => import('./islands/SolucionHostingIsland').then(m => ({default: m.SolucionHostingIsland})));
const UsuarioPublicoIsland = lazy(() => import('./islands/UsuarioPublicoIsland').then(m => ({default: m.UsuarioPublicoIsland})));

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
    useEffect(() => {
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

/* Wrapper: resuelve slug de blog post */
function BlogDetallePage() {
    const {slug} = useParams<{slug: string}>();
    return <BlogSingleIsland slug={slug} />;
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
                    <Route path="/" element={<BienvenidaIsland />} />
                    <Route path="/servicios" element={<ServiciosIsland />} />
                    <Route path="/servicios/:slug" element={<Suspense fallback={null}><ServicioDetallePage /></Suspense>} />
                    <Route path="/proyectos" element={<ProyectosIsland />} />
                    <Route path="/proyectos/:slug" element={<Suspense fallback={null}><ProyectoDetallePage /></Suspense>} />
                    <Route path="/nosotros" element={<Suspense fallback={null}><NosotrosIsland /></Suspense>} />
                    <Route path="/blog" element={<BlogIsland />} />
                    <Route path="/blog/:slug" element={<Suspense fallback={null}><BlogDetallePage /></Suspense>} />
                    <Route path="/soluciones" element={<Suspense fallback={null}><SolucionesIsland /></Suspense>} />
                    {/* [064A-32] Hosting tiene página propia, el resto usa placeholder */}
                    <Route path="/soluciones/hosting" element={<Suspense fallback={null}><SolucionHostingIsland /></Suspense>} />
                    <Route path="/soluciones/:slug" element={<Suspense fallback={null}><SolucionPlaceholderIsland /></Suspense>} />
                    {/* [064A-5] Ruta /contacto eliminada — todos los CTAs abren el chat */}
                    <Route path="/usuario/:username" element={<Suspense fallback={null}><UsuarioPublicoIsland /></Suspense>} />
                    <Route path="/panel" element={<Suspense fallback={null}><PanelIsland /></Suspense>} />
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
