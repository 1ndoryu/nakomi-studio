/* [114A-SEO3] Middleware de pre-rendering para crawlers.
 * Detecta bots por User-Agent y sirve HTML estático pre-renderizado si existe.
 * Los usuarios normales reciben el SPA como siempre.
 * El directorio de archivos pre-renderizados se configura con PRERENDER_DIR (default: "prerendered").
 * Si no existe el archivo para una ruta, se deja pasar al SPA fallback normal. */

use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::path::PathBuf;

/* Subcadenas de User-Agent de crawlers conocidos.
 * Incluye buscadores principales + redes sociales + SEO tools. */
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

/// Middleware Axum que sirve HTML pre-renderizado a crawlers.
/// Rutas API, assets, uploads, WS y swagger se ignoran.
pub async fn prerender(request: Request, next: Next) -> Response {
    /* Solo interceptar GET */
    if request.method() != axum::http::Method::GET {
        return next.run(request).await;
    }

    let path = request.uri().path();

    /* Rutas que nunca se pre-renderizan */
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

    /* ¿Es crawler? */
    let is_bot = request
        .headers()
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(is_crawler);

    if !is_bot {
        return next.run(request).await;
    }

    /* Buscar archivo pre-renderizado */
    let prerender_dir =
        std::env::var("PRERENDER_DIR").unwrap_or_else(|_| "prerendered".into());

    let clean = path.trim_start_matches('/');
    let file_path = if clean.is_empty() {
        PathBuf::from(&prerender_dir).join("index.html")
    } else {
        PathBuf::from(&prerender_dir).join(format!("{clean}.html"))
    };

    match tokio::fs::read_to_string(&file_path).await {
        Ok(html) => {
            tracing::info!(
                path,
                file = %file_path.display(),
                "Sirviendo HTML pre-renderizado a crawler"
            );
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                /* Cache corto: el contenido puede cambiar tras deploy */
                .header(header::CACHE_CONTROL, "public, max-age=3600")
                .body(Body::from(html))
                .unwrap_or_else(|_| {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::empty())
                        .expect("fallback response")
                })
        }
        /* Si no existe archivo pre-renderizado, el SPA fallback se encarga */
        Err(_) => next.run(request).await,
    }
}
