import type {ReactNode} from 'react';
import {LayoutPagina} from './LayoutPagina';
import {SEOHead} from '../seo/SEOHead';
import './CatalogPageShell.css';

interface CatalogPageShellProps {
    id: string;
    title: string;
    description: string;
    seoTitle: string;
    seoDescription: string;
    path: string;
    children: ReactNode;
}

/* [045A-3] Servicios y proyectos comparten la misma estructura publica.
 * La shell centraliza hero, contenedor y espaciados para evitar CSS divergente. */
export const CatalogPageShell = ({
    id,
    title,
    description,
    seoTitle,
    seoDescription,
    path,
    children,
}: CatalogPageShellProps): JSX.Element => {
    return (
        <LayoutPagina className="catalogPage" id={id}>
            <SEOHead title={seoTitle} description={seoDescription} path={path} />

            <section className="catalogPageHero">
                <div className="catalogPageHeroContent">
                    <div>
                        <h1 className="catalogPageTitle">{title}</h1>
                    </div>

                    <div className="catalogPageDescription">
                        <p>{description}</p>
                    </div>
                </div>
            </section>

            <section className="catalogPageContent">
                <div className="catalogPageContainer">{children}</div>
            </section>
        </LayoutPagina>
    );
};