/* [044A-28] Handlers SEO: robots.txt y sitemap.xml.
 * [114A-SEO3] sitemap.xml ahora incluye rutas dinámicas (servicios, proyectos) desde BD.
 * [124A-SENT-R1] Queries directas → ServiceRepository::public_slugs y ProjectRepository::public_slugs. */
use axum::http::header;
use axum::{extract::State, response::IntoResponse, routing::get, Router};

use crate::repositories::{ProjectRepository, ServiceRepository};
use crate::AppState;

const SITE_URL: &str = "https://nakomi.studio";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/robots.txt", get(robots_txt))
        .route("/sitemap.xml", get(sitemap_xml))
}

async fn robots_txt() -> impl IntoResponse {
    let body = format!(
        "User-agent: *\n\
         Allow: /\n\
         Disallow: /panel\n\
         Disallow: /swagger-ui\n\
         Disallow: /api-docs\n\n\
         Sitemap: {SITE_URL}/sitemap.xml"
    );
    ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body)
}

async fn sitemap_xml(State(state): State<AppState>) -> impl IntoResponse {
    /* Rutas estáticas del SPA */
    let mut rutas: Vec<(String, &str, &str)> = vec![
        ("/".into(), "1.0", "weekly"),
        ("/servicios".into(), "0.9", "weekly"),
        ("/proyectos".into(), "0.9", "weekly"),
        ("/nosotros".into(), "0.7", "monthly"),
        ("/blog".into(), "0.8", "daily"),
        ("/soluciones".into(), "0.8", "monthly"),
        ("/soluciones/hosting".into(), "0.8", "monthly"),
    ];

    /* [114A-SEO3] Rutas dinámicas desde BD: servicios con slug público */
    if let Ok(slugs) = ServiceRepository::public_slugs(&state.pool).await {
        for slug in slugs {
            rutas.push((format!("/servicios/{slug}"), "0.8", "monthly"));
        }
    }

    /* [114A-SEO3] Rutas dinámicas: proyectos con slug público */
    if let Ok(slugs) = ProjectRepository::public_slugs(&state.pool).await {
        for slug in slugs {
            rutas.push((format!("/proyectos/{slug}"), "0.7", "monthly"));
        }
    }

    let urls: String = rutas
        .iter()
        .map(|(path, priority, freq)| {
            format!(
                "  <url>\n\
                 \x20   <loc>{SITE_URL}{path}</loc>\n\
                 \x20   <changefreq>{freq}</changefreq>\n\
                 \x20   <priority>{priority}</priority>\n\
                 \x20 </url>"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n\
         {urls}\n\
         </urlset>"
    );

    (
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        xml,
    )
}
