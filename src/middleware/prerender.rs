/* [214A-2] Middleware SEO dinámico para crawlers.
 * Reemplaza el enfoque estático de 114A-SEO3 (HTML pre-renderizado con Puppeteer)
 * que no soportaba contenido CMS editable.
 *
 * Nuevo enfoque: lee index.html del SPA y le inyecta meta tags dinámicos
 * (title, description, OG, canonical, twitter) usando datos de la BD.
 * Rutas estáticas (/servicios, /nosotros, etc.) usan meta fijos.
 * Rutas dinámicas (/servicios/:slug, /proyectos/:slug) consultan la BD.
 * Usuarios normales reciben el SPA sin cambios. */

use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;
use std::fmt::Write;

/// Estado necesario para inyectar meta SEO dinámico
#[derive(Clone)]
pub struct PrerenderState {
    pub pool: PgPool,
    pub static_dir: String,
    pub app_url: String,
}

const CRAWLER_AGENTS: &[&str] = &[
    "googlebot",
    "bingbot",
    "yandexbot",
    "baiduspider",
    "duckduckbot",
    "slurp",
    "facebot",
    "facebookexternalhit",
    "twitterbot",
    "linkedinbot",
    "applebot",
    "semrushbot",
    "ahrefsbot",
    "quora link preview",
    "outbrain",
    "pinterestbot",
];

fn is_crawler(user_agent: &str) -> bool {
    let ua = user_agent.to_ascii_lowercase();
    CRAWLER_AGENTS.iter().any(|bot| ua.contains(bot))
}

/* Metadatos SEO por ruta */
struct SeoMeta {
    title: String,
    description: String,
    og_image: Option<String>,
    canonical: String,
    og_type: &'static str,
}

impl SeoMeta {
    /* Genera tags OG + Twitter que se inyectan antes de </head> */
    fn og_tags(&self, app_url: &str) -> String {
        let title = html_escape(&self.title);
        let desc = html_escape(&self.description);
        let mut tags = format!(
            "<link rel=\"canonical\" href=\"{canonical}\">\n\
             <meta property=\"og:title\" content=\"{title}\">\n\
             <meta property=\"og:description\" content=\"{desc}\">\n\
             <meta property=\"og:url\" content=\"{canonical}\">\n\
             <meta property=\"og:type\" content=\"{og_type}\">\n\
             <meta property=\"og:site_name\" content=\"Nakomi Studio\">\n\
             <meta name=\"twitter:card\" content=\"summary_large_image\">\n\
             <meta name=\"twitter:title\" content=\"{title}\">\n\
             <meta name=\"twitter:description\" content=\"{desc}\">\n",
            canonical = self.canonical,
            og_type = self.og_type,
        );
        if let Some(ref img) = self.og_image {
            /* Usar el proxy de imágenes para servir OG image optimizada */
            let img_url = format!("{app_url}/api/img/{img}?w=1200&q=80&fmt=webp");
            let _ = write!(
                tags,
                "<meta property=\"og:image\" content=\"{img_url}\">\n\
                 <meta name=\"twitter:image\" content=\"{img_url}\">\n",
            );
        }
        tags
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/* Construye SeoMeta a partir del path + consulta a BD para rutas dinámicas */
async fn resolve_seo_meta(path: &str, pool: &PgPool, app_url: &str) -> Option<SeoMeta> {
    let canonical = format!("{app_url}{path}");

    match path {
        "/" => Some(SeoMeta {
            title: "Nakomi Studio — Agencia Creativa Digital".into(),
            description: "Diseño web, desarrollo de software y soluciones digitales para tu negocio.".into(),
            og_image: None,
            canonical,
            og_type: "website",
        }),
        "/servicios" => Some(SeoMeta {
            title: "Nuestros Servicios — Nakomi Studio".into(),
            description: "Servicios de desarrollo web, diseño UI/UX, branding y soluciones digitales.".into(),
            og_image: None,
            canonical,
            og_type: "website",
        }),
        "/proyectos" => Some(SeoMeta {
            title: "Portfolio — Nakomi Studio".into(),
            description: "Explora nuestros proyectos y casos de éxito en desarrollo web y diseño digital.".into(),
            og_image: None,
            canonical,
            og_type: "website",
        }),
        "/nosotros" => Some(SeoMeta {
            title: "Sobre Nosotros — Nakomi Studio".into(),
            description: "Conoce al equipo detrás de Nakomi Studio y nuestra misión.".into(),
            og_image: None,
            canonical,
            og_type: "website",
        }),
        "/soluciones/hosting" => Some(SeoMeta {
            title: "Hosting Administrado — Nakomi Studio".into(),
            description: "Hosting web administrado con SSL, backups automáticos y soporte técnico.".into(),
            og_image: None,
            canonical,
            og_type: "website",
        }),
        _ => resolve_dynamic_meta(path, pool, &canonical).await,
    }
}

/* Rutas dinámicas: /servicios/:slug y /proyectos/:slug consultan la BD */
async fn resolve_dynamic_meta(
    path: &str,
    pool: &PgPool,
    canonical: &str,
) -> Option<SeoMeta> {
    if let Some(slug) = path.strip_prefix("/servicios/") {
        let slug = slug.trim_end_matches('/');
        if slug.is_empty() {
            return None;
        }
        let row: (String, Option<String>) = sqlx::query_as(
            "SELECT title, description FROM services WHERE slug = $1 AND is_active = true",
        )
        .bind(slug)
        .fetch_optional(pool)
        .await
        .ok()??;

        Some(SeoMeta {
            title: format!("{} — Nakomi Studio", row.0),
            description: row.1.unwrap_or_default(),
            og_image: None,
            canonical: canonical.to_string(),
            og_type: "website",
        })
    } else if let Some(slug) = path.strip_prefix("/proyectos/") {
        let slug = slug.trim_end_matches('/');
        if slug.is_empty() {
            return None;
        }
        let row: (String, String, Option<String>) = sqlx::query_as(
            "SELECT COALESCE(meta_title, title), \
                    COALESCE(meta_description, description), \
                    featured_image \
             FROM projects WHERE slug = $1 AND status = 'published'",
        )
        .bind(slug)
        .fetch_optional(pool)
        .await
        .ok()??;

        Some(SeoMeta {
            title: format!("{} — Nakomi Studio", row.0),
            description: row.1,
            og_image: row.2,
            canonical: canonical.to_string(),
            og_type: "article",
        })
    } else {
        None
    }
}

/* Inyecta meta SEO en el HTML del SPA: reemplaza <title> y description
 * existentes, y agrega OG/canonical antes de </head>. */
fn inject_seo_into_html(html: &str, meta: &SeoMeta, app_url: &str) -> String {
    let mut result = html.to_string();

    /* Reemplazar contenido de <title>...</title> */
    if let Some(start) = result.find("<title>") {
        let title_start = start + "<title>".len();
        if let Some(end_rel) = result[title_start..].find("</title>") {
            result.replace_range(
                title_start..title_start + end_rel,
                &html_escape(&meta.title),
            );
        }
    }

    /* Reemplazar contenido de <meta name="description" content="..."> */
    let desc_prefix = "<meta name=\"description\" content=\"";
    if let Some(start) = result.find(desc_prefix) {
        let content_start = start + desc_prefix.len();
        if let Some(end_rel) = result[content_start..].find('"') {
            result.replace_range(
                content_start..content_start + end_rel,
                &html_escape(&meta.description),
            );
        }
    }

    /* Agregar OG tags + canonical antes de </head> */
    let og = meta.og_tags(app_url);
    result = result.replace("</head>", &format!("{og}</head>"));

    result
}

/// Middleware SEO: para crawlers, inyecta meta tags dinámicos en index.html.
/// Rutas API, assets, uploads, WS, swagger y panel se ignoran.
pub async fn prerender(
    State(state): State<PrerenderState>,
    request: Request,
    next: Next,
) -> Response {
    if request.method() != axum::http::Method::GET {
        return next.run(request).await;
    }

    let path = request.uri().path();

    /* Rutas que nunca reciben meta SEO */
    if path.starts_with("/api/")
        || path.starts_with("/assets/")
        || path.starts_with("/uploads/")
        || path.starts_with("/ws/")
        || path.starts_with("/swagger-ui")
        || path == "/robots.txt"
        || path == "/sitemap.xml"
        || path.starts_with("/panel")
    {
        return next.run(request).await;
    }

    let is_bot = request
        .headers()
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(is_crawler);

    if !is_bot {
        return next.run(request).await;
    }

    /* Leer index.html del SPA como template */
    let index_path = format!("{}/index.html", state.static_dir);
    let Ok(template) = tokio::fs::read_to_string(&index_path).await else {
        return next.run(request).await;
    };

    /* Resolver meta SEO para esta ruta */
    let Some(meta) = resolve_seo_meta(path, &state.pool, &state.app_url).await else {
        /* Ruta desconocida: el SPA se encarga */
        return next.run(request).await;
    };

    let html = inject_seo_into_html(&template, &meta, &state.app_url);

    tracing::info!(path, "SEO meta dinámico inyectado para crawler");

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .body(Body::from(html))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .expect("fallback response")
        })
}
