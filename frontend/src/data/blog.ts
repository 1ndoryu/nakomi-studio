/*
 * Datos de blog centralizados.
 * [044A-33] Artículo de Kamples traducido a español, imagen corregida, navegación SPA funcional.
 */
import {PostBlog} from '../types/contenido';

const POSTS_FALLBACK: PostBlog[] = [
    {
        id: 1,
        titulo: 'Kamples: Reimaginando cómo los músicos descubren y comparten samples',
        resumen: 'Una plataforma open-source que combina una librería de samples, motor de recomendaciones, DAW ligero y funciones sociales — como WhoSampled con Pinterest para producción musical.',
        contenido: `
            <p>La producción musical actual depende en gran parte de samples, pero descubrirlos, organizarlos y compartirlos sigue fragmentado entre docenas de herramientas y comunidades. <strong>Kamples</strong> es nuestra respuesta a ese problema: una plataforma única donde los productores pueden explorar, previsualizar y compartir samples con la profundidad de WhoSampled y la curación visual de Pinterest.</p>

            <h2>El Problema</h2>
            <p>Los productores hacen malabarismos entre marketplaces de samples, DAWs y redes sociales para encontrar lo que necesitan. No existe un solo lugar que te permita <em>descubrir</em> samples algorítmicamente, <em>previsualizarlos</em> en contexto y <em>compartir</em> colecciones con tu comunidad — todo en un mismo flujo.</p>

            <h2>Qué hace Kamples</h2>
            <p>En su núcleo, Kamples es una librería de samples con tres capas construidas encima:</p>
            <ul>
                <li><strong>Motor de Recomendaciones</strong> — Un algoritmo que sugiere samples según género, mood, BPM, tonalidad y tu historial de uso. Cuanto más interactúas, mejor se vuelve.</li>
                <li><strong>DAW Integrado</strong> — Un workstation ligero basado en el navegador para previsualizar samples en contexto. Capas, mezcla y prueba antes de descargar. Sin necesidad de salir de la plataforma.</li>
                <li><strong>Capa Social</strong> — Organiza samples en colecciones y tableros (como Pinterest). Sigue a otros productores, mira qué están sampleando y explora cadenas de "más ideas como esta" que revelan conexiones inesperadas.</li>
            </ul>

            <h2>Open Source</h2>
            <p>Kamples es completamente open source. Creemos que las herramientas para la expresión creativa deben ser transparentes e impulsadas por la comunidad. Las contribuciones son bienvenidas — desde mejoras en el algoritmo hasta componentes de UI y nuevas funciones de procesamiento de audio.</p>

            <h2>Pruébalo</h2>
            <p>Kamples está disponible en <strong>kamples.com</strong>. Regístrate, sube tu primer sample y empieza a construir tu librería. La plataforma es gratuita para productores individuales — estamos explorando modelos de sostenibilidad para equipos y uso comercial.</p>
        `,
        fecha: '4 Abr, 2026',
        categoria: 'Producto',
        link: '/blog/kamples-reimagining-sample-discovery',
        imagen: '/assets/Proyectos portadas/Kamples portada.jpg'
    }
];

export const POSTS_BLOG: PostBlog[] = POSTS_FALLBACK;

/* [104A-35] Transformación compartida: AdminBlogPost → PostBlog.
 * Usada por SeccionBlog (home) y BlogIsland (página blog). */
export function apiPostToPostBlog(post: {
    id: string;
    title: string;
    excerpt?: string | null;
    content: string;
    published_at?: string | null;
    created_at: string;
    tags: string[];
    slug: string;
    featured_image?: string | null;
}): PostBlog {
    return {
        id: typeof post.id === 'string' ? parseInt(post.id.replace(/-/g, '').slice(0, 8), 16) : 0,
        adminId: post.id,
        titulo: post.title,
        resumen: post.excerpt ?? '',
        contenido: post.content,
        fecha: new Date(post.published_at ?? post.created_at).toLocaleDateString('es', {day: 'numeric', month: 'short', year: 'numeric'}),
        categoria: post.tags[0] ?? 'General',
        link: `/blog/${post.slug}`,
        imagen: post.featured_image ?? undefined,
    };
}
