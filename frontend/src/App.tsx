/* [044A-1] App principal con React Router.
 * Reemplaza el sistema de islands de WordPress por rutas SPA.
 * Cada island se convierte en una ruta. Las páginas de detalle
 * reciben el slug del URL param y buscan datos en data/.
 * [044A-38 Fase 1] Redirige / → /panel si el usuario está logueado. */

import {useEffect} from 'react';
import {BrowserRouter, Routes, Route, useNavigate, useParams} from 'react-router-dom';
import {QueryClient, QueryClientProvider} from '@tanstack/react-query';
import {registrarNavigate} from './navegacionSPA';
import {ScrollToTop} from './components/ui/ScrollToTop';

/* Pages (ex-islands) */
import {BienvenidaIsland} from './islands/BienvenidaIsland';
import {ServiciosIsland} from './islands/ServiciosIsland';
import {ServicioIndividualIsland} from './islands/ServicioIndividualIsland';
import {ProyectosIsland} from './islands/ProyectosIsland';
import {ProyectoIndividualIsland} from './islands/ProyectoIndividualIsland';
import {NosotrosIsland} from './islands/NosotrosIsland';
import {BlogIsland} from './islands/BlogIsland';
import {BlogSingleIsland} from './islands/BlogSingleIsland';
import {SolucionesIsland} from './islands/SolucionesIsland';
import {SolucionPlaceholderIsland} from './islands/SolucionPlaceholderIsland';
import {SolucionHostingIsland} from './islands/SolucionHostingIsland';
import {PanelIsland} from './islands/PanelIsland';
import {UsuarioPublicoIsland} from './islands/UsuarioPublicoIsland';
import {NotFoundIsland} from './islands/NotFoundIsland';

/* [054A-5] Toast system */
import {ToastContainer} from './components/ui/ToastContainer';

/* [054A-3] Chat widget flotante para visitantes */
import {ChatWidget} from './components/chat/ChatWidget';

/* Data para resolver slugs */
import {SERVICIOS_DATA} from './data/servicios';
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
    const servicio = SERVICIOS_DATA.find(s => {
        const sSlug = s.link?.split('/').filter(Boolean).pop() || '';
        return sSlug === slug || String(s.id) === slug;
    });
    return (
        <ServicioIndividualIsland
            titulo={servicio?.titulo}
            descripcion={servicio?.descripcion}
            imagen={servicio?.imagen}
            slug={slug}
        />
    );
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
                    <Route path="/servicios/:slug" element={<ServicioDetallePage />} />
                    <Route path="/proyectos" element={<ProyectosIsland />} />
                    <Route path="/proyectos/:slug" element={<ProyectoDetallePage />} />
                    <Route path="/nosotros" element={<NosotrosIsland />} />
                    <Route path="/blog" element={<BlogIsland />} />
                    <Route path="/blog/:slug" element={<BlogDetallePage />} />
                    <Route path="/soluciones" element={<SolucionesIsland />} />
                    {/* [064A-32] Hosting tiene página propia, el resto usa placeholder */}
                    <Route path="/soluciones/hosting" element={<SolucionHostingIsland />} />
                    <Route path="/soluciones/:slug" element={<SolucionPlaceholderIsland />} />
                    {/* [064A-5] Ruta /contacto eliminada — todos los CTAs abren el chat */}
                    <Route path="/usuario/:username" element={<UsuarioPublicoIsland />} />
                    <Route path="/panel" element={<PanelIsland />} />
                    {/* [044A-28] Página 404 real en vez de redirigir silenciosamente al home */}
                    <Route path="*" element={<NotFoundIsland />} />
                </Routes>
                {/* [054A-3] Chat flotante para visitantes (se oculta en /panel) */}
                <ChatWidget />
            </BrowserRouter>
        </QueryClientProvider>
    );
}

export default App;
