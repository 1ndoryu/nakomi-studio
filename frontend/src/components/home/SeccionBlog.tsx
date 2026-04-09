/**
 * Componente: SeccionBlog
 * Muestra las últimas entradas del blog.
 * Datos centralizados en data/blog.ts (DRY).
 * Imágenes centralizadas en hooks/useImagenes.ts (DRY).
 */
import React from 'react';
import {useTranslation} from 'react-i18next';
import {SeccionHeader} from '../ui/SeccionHeader';
import {obtenerImagenBlog} from '../../hooks/useImagenes';
import {POSTS_BLOG} from '../../data/blog';
import {navegar} from '../../navegacionSPA';
import OptimizedImage from '../ui/OptimizedImage';
import './SeccionBlog.css';

export const SeccionBlog: React.FC = () => {
    const {t} = useTranslation();

    return (
        <section className="seccionBlog" id="blog">
            <div className="blogContenedor">
                <SeccionHeader titulo={t('sections.journal')} />

                <div className="blogGrid">
                    {/* [044A-35] Card minimalista: imagen + overlay con categoría y título */}
                    {POSTS_BLOG.slice(0, 3).map(post => (
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
