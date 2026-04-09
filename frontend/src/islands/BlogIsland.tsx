/**
 * Componente: BlogIsland
 * Página de listado de artículos del blog.
 * [074A-11] Conectado a API pública /api/blog con paginación real.
 * Fallback a datos estáticos si la API falla. */
import React, {useState, useMemo, useEffect} from 'react';
import {useTranslation} from 'react-i18next';
import '../styles/variables.css';
import './BlogIsland.css';
import {LayoutPagina} from '../components/layout/LayoutPagina';
import {SEOHead} from '../components/seo/SEOHead';
import {POSTS_BLOG} from '../data/blog';
import {PostBlog} from '../types/contenido';
import {obtenerImagenBlog} from '../hooks/useImagenes';
import {navegar} from '../navegacionSPA';
import {Button} from '../components/ui/Button';
import {AdminOverlay} from '../components/ui/AdminOverlay';
import OptimizedImage from '../components/ui/OptimizedImage';
import {apiListPublicBlog} from '../api/admin-blog';
import type {AdminBlogPost} from '../api/admin-blog';

interface BlogIslandProps {
    titulo?: string;
}

/*
 * Tarjeta de artículo con layout horizontal.
 * Siempre muestra imagen: backend -> fallback colors (obtenerImagenBlog).
 */
/* [044A-35] Tarjeta minimalista: imagen de fondo con título y categoría superpuestos. */
const TarjetaArticulo: React.FC<{post: PostBlog; destacado?: boolean}> = ({post, destacado = false}) => {
    const imagenFinal = post.imagen || obtenerImagenBlog(post.id);

    return (
        <AdminOverlay contentType="blog" itemId={post.adminId || String(post.id)}>
            <a href={post.link || '#'} onClick={(e) => { e.preventDefault(); if (post.link) navegar(post.link); }} className={`tarjetaArticulo ${destacado ? 'tarjetaArticuloDestacado' : ''}`}>
                <OptimizedImage src={imagenFinal} alt={post.titulo} className="articuloImagen" sizes="(max-width: 768px) 100vw, 50vw" />
                <div className="articuloOverlay">
                    <span className="articuloCategoria">{post.categoria}</span>
                    <h3 className="articuloTitulo">{post.titulo}</h3>
                </div>
            </a>
        </AdminOverlay>
    );
};

/* [074A-11] Convierte AdminBlogPost de la API a PostBlog para las tarjetas */
function apiPostToPostBlog(post: AdminBlogPost): PostBlog {
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

export const BlogIsland = ({titulo}: BlogIslandProps): JSX.Element => {
    const {t} = useTranslation();
    const [categoriaActiva, setCategoriaActiva] = useState('todos');
    const [postsApi, setPostsApi] = useState<PostBlog[] | null>(null);

    /* [074A-11] Fetch posts publicados de la API, fallback a datos estáticos */
    useEffect(() => {
        const controller = new AbortController();
        apiListPublicBlog(1, 50)
            .then(data => {
                if (!controller.signal.aborted) {
                    setPostsApi(data.posts.map(apiPostToPostBlog));
                }
            })
            .catch(() => {
                /* Fallback silencioso a datos estáticos */
            });
        return () => controller.abort();
    }, []);

    const postsReales = postsApi ?? POSTS_BLOG;

    /* Categorías únicas extraídas de los posts */
    const categorias = useMemo(() => {
        const cats = new Set(postsReales.map(p => p.categoria));
        return ['todos', ...Array.from(cats)];
    }, [postsReales]);

    const postsFiltrados = useMemo(() => {
        if (categoriaActiva === 'todos') return postsReales;
        return postsReales.filter(p => p.categoria === categoriaActiva);
    }, [categoriaActiva, postsReales]);

    return (
        <LayoutPagina className="blogMain" id="paginaBlog">
            <SEOHead
                title="Blog"
                description="Artículos sobre desarrollo web, diseño y tecnología del equipo de Nakomi Studio."
                path="/blog"
            />
            {/* Hero */}
            <section className="blogHero">
                <div className="blogHeroContenido">
                    <div>
                        <h1 className="blogHeroTitulo">{titulo || t('blog_page.title')}</h1>
                    </div>
                    <div className="blogHeroDescripcion">
                        <p>{t('blog_page.description')}</p>
                    </div>
                </div>
            </section>

            <section className="blogContenido">
                <div className="blogContenedor">
                    {/* Filtros de categoría */}
                    <div className="blogFiltros">
                        {categorias.map(cat => (
                            <Button
                                variante="texto"
                                key={cat}
                                className={`blogFiltroBtn ${categoriaActiva === cat ? 'blogFiltroBtnActivo' : ''}`}
                                onClick={() => setCategoriaActiva(cat)}
                                type="button"
                            >
                                {cat === 'todos' ? t('blog_page.all') : cat}
                            </Button>
                        ))}
                    </div>

                    {/* Todos los posts dentro de un solo contenedor */}
                    {postsFiltrados.length > 0 && (
                        <div className="blogListaArticulos">
                            {postsFiltrados.map((post, i) => (
                                <TarjetaArticulo key={post.id} post={post} destacado={i === 0} />
                            ))}
                        </div>
                    )}

                    {postsFiltrados.length === 0 && (
                        <p className="blogSinResultados">{t('blog_page.empty')}</p>
                    )}
                </div>
            </section>

        </LayoutPagina>
    );
};

export default BlogIsland;
