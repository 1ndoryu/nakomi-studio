use std::net::SocketAddr;

use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use glory_backend::config::AppConfig;
use glory_backend::handlers;
use glory_backend::services::AssignmentService;
use glory_rs::fixtures::ContentManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new("glory_backend=debug,tower_http=debug")
            }),
        )
        .init();

    let config = AppConfig::from_env()?;

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    /* [074A-22] Glory Fixtures: sincroniza archivos TOML de content/ con la BD.
     * Inserta, actualiza datos declarativos y borra huérfanos automáticamente. */
    let password_hasher: glory_rs::fixtures::PasswordHasher = Box::new(|plain| {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(plain.as_bytes(), &salt)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                e.to_string().into()
            })?
            .to_string();
        Ok(hash)
    });
    let fixture_manager = ContentManager::new(pool.clone(), "content")
        .with_password_hasher(password_hasher);
    match fixture_manager.sync_all().await {
        Ok(report) => tracing::info!("[fixtures] {}", report.summary()),
        Err(e) => tracing::error!("[fixtures] Error syncing: {e}"),
    }

    /* [044A-38 Fase 4] Background task: auto-asigna órdenes sin empleado tras 24h */
    let bg_pool = pool.clone();
    tokio::spawn(async move {
        AssignmentService::auto_assign_loop(bg_pool).await;
    });

    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("Servidor iniciando en {addr}");
    tracing::info!("Swagger UI disponible en http://{addr}/swagger-ui/");

    let app = handlers::create_app(pool, config);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    /* [074A-41] into_make_service_with_connect_info para que GovernorLayer
     * (PeerIpKeyExtractor) y ConnectInfo<SocketAddr> en ws_visitor funcionen.
     * Sin esto, tower_governor devuelve "Unable To Extract Key!" en todas las rutas /api/ routes. */
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    Ok(())
}
