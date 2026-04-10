/**
 * Componente: SeccionBlog
 * Muestra las últimas entradas del blog en la home.
 * [104A-35] Conectado a API pública /api/blog — misma fuente que BlogIsland.
 * Fallback a datos estáticos si la API falla.
 */
import React, {useState, useEffect} from 'react';
import {useTranslation} from 'react-i18next';
import {SeccionHeader} from '../ui/SeccionHeader';
import {obtenerImagenBlog} from '../../hooks/useImagenes';
import {POSTS_BLOG, apiPostToPostBlog} from '../../data/blog';
import {apiListPublicBlog} from '../../api/admin-blog';
import {navegar} from '../../navegacionSPA';
import OptimizedImage from '../ui/OptimizedImage';
import './SeccionBlog.css';

export const SeccionBlog: React.FC = () => {
    const {t} = useTranslation();
    const [posts, setPosts] = useState(POSTS_BLOG);

    /* [104A-35] Fetch últimos 3 posts publicados, fallback a datos estáticos */
    useEffect(() => {
        const controller = new AbortController();
        apiListPublicBlog(1, 3)
            .then(data => {
                if (!controller.signal.aborted && data.posts.length > 0) {
                    setPosts(data.posts.map(apiPostToPostBlog));
                }
            })
            .catch(() => { /* Fallback silencioso a datos estáticos */ });
        return () => controller.abort();
    }, []);

    return (
        <section className="seccionBlog" id="blog">
            <div className="blogContenedor">
                <SeccionHeader titulo={t('sections.journal')} />

                <div className="blogGrid">
                    {posts.slice(0, 3).map(post => (
                        <a key={post.id} href={post.link || '#'} onClick={(e) => { e.preventDefault(); if (post.link) navegar(post.link); }} className="blogCard">
                            <OptimizedImage src={post.imagen || obtenerImagenBlog(post.id)} alt={post.titulo} className="blogImagen" sizes="(max-width: 768px) 100vw, 33vw" />
                            <div className="blogOverlay">
                                <span className="blogCategoria">{post.categoria}</span>
                                <h3 className="blogTitulo">{post.titulo}</h3>
                            </div>
                        </a>
                    ))}
                </div>
            </div>
        </section>
    );
};
