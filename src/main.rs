/* sentinel-disable-file sqlx-query-sin-macro: main.rs usa queries dinámicas para
 * setup inicial (admin seeding, cleanup test data) con formatos generados en runtime. */
use std::net::SocketAddr;

use argon2::password_hash::rand_core::OsRng;
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
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
                tracing_subscriber::EnvFilter::new(
                    "glory_backend=debug,glory_rs=debug,tower_http=debug",
                )
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
     * Inserta, actualiza datos declarativos y borra huérfanos automáticamente.
     * [074A-23] Cleanup previo: borra pedidos y hosting legacy del seed que no
     * están rastreados por fixtures para evitar duplicados. */
    let password_hasher: glory_rs::fixtures::PasswordHasher = Box::new(|plain| {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(plain.as_bytes(), &salt)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })?
            .to_string();
        Ok(hash)
    });
    let fixture_manager =
        ContentManager::new(pool.clone(), "content").with_password_hasher(password_hasher);

    /* Limpiar datos de seed legacy no rastreados por fixtures.
     * Solo afecta a órdenes/hosting de los emails de test conocidos.
     * Es no-op si ya se ejecutó antes o no hay datos legacy. */
    cleanup_legacy_seed(&pool).await;

    /* [204A-1] Sync de fixtures controlado por FIXTURES_SYNC.
     * En producción (FIXTURES_SYNC=false o no definido) no se ejecuta,
     * evitando que datos de prueba sobreescriban datos reales.
     * En desarrollo: FIXTURES_SYNC=true en .env para sincronizar content/ TOMLs. */
    let fixtures_sync = std::env::var("FIXTURES_SYNC")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false);

    if fixtures_sync {
        match fixture_manager.sync_all().await {
            Ok(report) => {
                tracing::info!("[fixtures] {}", report.summary());
                for err in &report.errors {
                    tracing::error!("[fixtures] {err}");
                }
            }
            Err(e) => tracing::error!("[fixtures] Error syncing: {e}"),
        }
    } else {
        tracing::info!("[fixtures] Sync desactivado (FIXTURES_SYNC != true)");
    }

    /* [044A-38 Fase 4] Background task: auto-asigna órdenes sin empleado tras 24h */
    let bg_pool = pool.clone();
    tokio::spawn(async move {
        AssignmentService::auto_assign_loop(bg_pool).await;
    });

    /* [114A-13] Background task: cierra sesiones de chat inactivas (>24h sin actividad).
     * Ejecuta cada hora. Previene acumulación de sesiones zombie. */
    let chat_cleanup_pool = pool.clone();
    tokio::spawn(async move {
        session_cleanup_loop(chat_cleanup_pool).await;
    });

    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("Servidor iniciando en {addr}");
    tracing::info!("Swagger UI disponible en http://{addr}/swagger-ui/");

    let app = handlers::create_app(pool, config);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    /* [074A-41] into_make_service_with_connect_info para que GovernorLayer
     * (PeerIpKeyExtractor) y ConnectInfo<SocketAddr> en ws_visitor funcionen.
     * Sin esto, tower_governor devuelve "Unable To Extract Key!" en todas las rutas /api/ routes. */
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

/* [074A-23] Limpia datos de seed legacy que ahora son manejados por fixtures.
 * Borra órdenes (con cascade FK completo) y hosting de test emails conocidos
 * que NO están rastreados en _glory_fixtures. Es no-op si no hay datos legacy. */
async fn cleanup_legacy_seed(pool: &sqlx::PgPool) {
    let test_emails = &["cliente@test.com", "empleado@test.com"];
    let tables_exist: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = '_glory_fixtures')",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    if !tables_exist {
        return;
    }

    /* Subquery: IDs de órdenes legacy (no fixture-tracked) de test users */
    let legacy_orders_subquery = "SELECT o.id FROM orders o
         JOIN users u ON o.client_id = u.id
         WHERE u.email = ANY($1)
         AND NOT EXISTS (
             SELECT 1 FROM _glory_fixtures gf
             WHERE gf.table_name = 'orders' AND gf.db_id = o.id::text
         )";

    /* Romper FK circular orders↔chat_sessions */
    let _ = sqlx::query(&format!(
        "UPDATE orders SET chat_session_id = NULL WHERE id IN ({legacy_orders_subquery})"
    ))
    .bind(test_emails)
    .execute(pool)
    .await;

    /* Cascade completo: chat → reviews → refunds → delegations → payments → deliverables → phases → orders */
    let cascade_tables = [
        (
            "chat_messages",
            "session_id IN (SELECT id FROM chat_sessions WHERE order_id IN ({q}))",
        ),
        (
            "chat_session_notes",
            "session_id IN (SELECT id FROM chat_sessions WHERE order_id IN ({q}))",
        ),
        ("chat_sessions", "order_id IN ({q})"),
        ("order_reviews", "order_id IN ({q})"),
        ("order_refunds", "order_id IN ({q})"),
        ("order_delegations", "order_id IN ({q})"),
        ("order_payments", "order_id IN ({q})"),
        (
            "phase_deliverables",
            "phase_id IN (SELECT id FROM order_phases WHERE order_id IN ({q}))",
        ),
        ("order_phases", "order_id IN ({q})"),
    ];

    let mut total_deleted = 0u64;
    for (table, condition_tpl) in &cascade_tables {
        let condition = condition_tpl.replace("{q}", legacy_orders_subquery);
        let sql = format!("DELETE FROM {table} WHERE {condition}");
        if let Ok(r) = sqlx::query(&sql).bind(test_emails).execute(pool).await {
            total_deleted += r.rows_affected();
        }
    }

    /* Borrar órdenes legacy */
    let sql = format!("DELETE FROM orders WHERE id IN ({legacy_orders_subquery})");
    if let Ok(r) = sqlx::query(&sql).bind(test_emails).execute(pool).await {
        total_deleted += r.rows_affected();
    }

    /* Borrar hosting legacy (no fixture-tracked) */
    let legacy_hosting_subquery = "SELECT hs.id FROM hosting_subscriptions hs
         JOIN users u ON hs.user_id = u.id
         WHERE u.email = ANY($1)
         AND NOT EXISTS (
             SELECT 1 FROM _glory_fixtures gf
             WHERE gf.table_name = 'hosting_subscriptions' AND gf.db_id = hs.id::text
         )";

    let _ = sqlx::query(&format!(
        "DELETE FROM hosting_events WHERE subscription_id IN ({legacy_hosting_subquery})"
    ))
    .bind(test_emails)
    .execute(pool)
    .await
    .map(|r| total_deleted += r.rows_affected());

    let _ = sqlx::query(&format!(
        "DELETE FROM hosting_subscriptions WHERE id IN ({legacy_hosting_subquery})"
    ))
    .bind(test_emails)
    .execute(pool)
    .await
    .map(|r| total_deleted += r.rows_affected());

    if total_deleted > 0 {
        tracing::info!("[cleanup] Legacy seed: {total_deleted} records deleted");
    }
}

/* [114A-13] Background loop: cierra sesiones de chat inactivas (>24h sin actividad).
 * Ejecuta cada hora. Previene acumulación de sesiones zombie en BD y en el panel staff. */
async fn session_cleanup_loop(pool: sqlx::PgPool) {
    use glory_backend::repositories::ChatRepository;

    const INACTIVITY_HOURS: i32 = 24;
    const CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(3600);

    loop {
        tokio::time::sleep(CHECK_INTERVAL).await;
        match ChatRepository::close_inactive_sessions(&pool, INACTIVITY_HOURS).await {
            Ok(0) => {}
            Ok(n) => tracing::info!(
                "[chat-cleanup] {n} sesiones inactivas cerradas (>{INACTIVITY_HOURS}h)"
            ),
            Err(e) => tracing::error!("[chat-cleanup] Error cerrando sesiones inactivas: {e}"),
        }
    }
}
