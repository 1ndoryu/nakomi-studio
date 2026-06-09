/* sentinel-disable-file sqlx-query-sin-macro: main.rs usa queries dinámicas para
 * setup inicial (admin seeding, cleanup test data) con formatos generados en runtime. */
use std::net::SocketAddr;
use std::time::Duration;

use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo, TokioTimer};
use hyper_util::server::conn::auto;
use tower::{Service, ServiceExt};

use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use glory_backend::config::AppConfig;
use glory_backend::handlers;
use glory_backend::services::bandwidth_enforcement::bandwidth_throttle_loop;
use glory_backend::services::cpu_burst::cpu_burst_loop;
use glory_backend::services::infrastructure_metrics::infrastructure_metrics_loop;
use glory_backend::services::storage_enforcement::storage_enforcement_loop;
use glory_backend::services::vps_monitor::vps_monitor_loop;
use glory_backend::services::{AssignmentService, ContaboConfig, ContaboService, CoolifyConfig};
use glory_rs::fixtures::ContentManager;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
#[allow(clippy::too_many_lines)]
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

    /* [096A-1] Pool con max_lifetime + idle_timeout para evitar conexiones
     * zombie que provocan CLOSE_WAIT y deadlock del event loop. */
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .max_lifetime(Duration::from_secs(1800))
        .idle_timeout(Duration::from_secs(300))
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    /* [250A-1] Fixtures + background tasks extraídos a helpers para mantener
     * main() por debajo de 100 líneas (regla funcion-larga-rs). */
    setup_and_run_fixtures(&pool).await?;
    spawn_background_services(&pool, &config);

    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("Servidor iniciando en {addr}");
    tracing::info!("Swagger UI disponible en http://{addr}/swagger-ui/");

    spawn_http_watchdog(config.port);

    let app = handlers::create_app(pool, config);

    /* [096A-1] TCP keepalive para detectar conexiones muertas del lado del
     * servidor y evitar acumulación de CLOSE_WAIT que causa deadlock. Se crea
     * el socket con socket2 para configurar SO_KEEPALIVE antes de pasarlo a tokio. */
    let sock_addr: std::net::SocketAddr = addr.parse()?;
    let socket = socket2::Socket::new(
        if sock_addr.is_ipv4() {
            socket2::Domain::IPV4
        } else {
            socket2::Domain::IPV6
        },
        socket2::Type::STREAM,
        None,
    )?;
    socket.set_keepalive(true)?;
    socket.set_nonblocking(true)?;
    socket.set_reuse_address(true)?;
    socket.bind(&sock_addr.into())?;
    socket.listen(1024)?;
    let listener = tokio::net::TcpListener::from_std(socket.into())?;

    /* [096A-2] Migrado de axum::serve() a hyper_util::auto::Builder para
     * configurar header_read_timeout a nivel HTTP. axum::serve() es
     * intencionalmente simple y NO expone configuración de conexiones
     * (ver tokio-rs/axum#2939). Sin este timeout, conexiones keep-alive
     * de clientes que desaparecen quedan en CLOSE_WAIT indefinidamente
     * hasta saturar el event loop.
     *
     * header_read_timeout se rearma después de cada respuesta (confirmado
     * por test hyper: header_read_timeout_as_idle_timeout), actuando como
     * idle timeout entre requests keep-alive. */
    let mut make_service =
        app.into_make_service_with_connect_info::<SocketAddr>();

    let shutdown = shutdown_signal();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            result = listener.accept() => {
                let (tcp_stream, remote_addr) = result?;
let tower_service = match make_service.call(remote_addr).await {
                        Ok(svc) => svc,
                        Err(err) => match err {},
                    };

                tokio::spawn(async move {
                    let io = TokioIo::new(tcp_stream);

                    let hyper_service =
                        hyper::service::service_fn(move |request: hyper::Request<Incoming>| {
                            tower_service.clone().oneshot(request)
                        });

                    let mut builder = auto::Builder::new(TokioExecutor::new());
                    let mut http1 = builder.http1();
                    let conn = http1
                        .timer(TokioTimer::new())
                        .header_read_timeout(Duration::from_secs(30))
                        .keep_alive(true)
                        .serve_connection_with_upgrades(io, hyper_service);

                    tokio::select! {
                        result = conn => {
                            if let Err(err) = result {
                                tracing::debug!("Connection error from {remote_addr}: {err:#}");
                            }
                        }
                        _ = shutdown_signal() => {
                            /* graceful shutdown: dejar que hyper cierre limpiamente */
                        }
                    }
                });
            }
            _ = &mut shutdown => {
                tracing::info!("Graceful shutdown: dejando de aceptar conexiones");
                break;
            }
        }
    }

    Ok(())
}

/* [096A-1] Señal de graceful shutdown: espera SIGTERM/SIGINT y permite
 * que las conexiones activas se drenen antes de cerrar el proceso. */
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { tracing::info!("Recibido SIGINT, iniciando graceful shutdown..."); }
        _ = terminate => { tracing::info!("Recibido SIGTERM, iniciando graceful shutdown..."); }
    }
}

/* [250A-1] Extraído de main() para cumplir límite de 100 líneas.
 * Configura password hasher, content manager, limpia seed legacy y sincroniza
 * content/ TOMLs si FIXTURES_SYNC=true. */
#[allow(clippy::too_many_lines)]
async fn setup_and_run_fixtures(pool: &sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
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

    cleanup_legacy_seed(pool).await;

    let fixtures_sync =
        std::env::var("FIXTURES_SYNC").is_ok_and(|v| v.eq_ignore_ascii_case("true") || v == "1");

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

    Ok(())
}

/* [250A-1] Extraído de main() para cumplir límite de 100 líneas.
 * Inicia todas las tareas de background: asignación, cleanup chat, storage
 * enforcement, métricas, bandwidth throttle y monitor VPS. */
#[allow(clippy::too_many_lines)]
fn spawn_background_services(pool: &sqlx::PgPool, _config: &AppConfig) {
    let bg_pool = pool.clone();
    tokio::spawn(async move {
        AssignmentService::auto_assign_loop(bg_pool).await;
    });

    let chat_cleanup_pool = pool.clone();
    tokio::spawn(async move {
        session_cleanup_loop(chat_cleanup_pool).await;
    });

    let coolify_config = CoolifyConfig::from_env();
    let coolify_config_vps1 = CoolifyConfig::from_env_with_prefix("COOLIFY_VPS1_");

    if let Some(coolify_config) = coolify_config.clone() {
        let enforcement_pool = pool.clone();
        tokio::spawn(async move {
            storage_enforcement_loop(enforcement_pool, coolify_config).await;
        });
    } else {
        tracing::warn!(
            "[storage-enforcement] Coolify no configurado — enforcement de storage desactivado"
        );
    }

    if coolify_config.is_some() || coolify_config_vps1.is_some() {
        let metrics_pool = pool.clone();
        let metrics_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("metrics HTTP client");
        let metrics_vps1 = coolify_config_vps1.clone();
        let metrics_default = coolify_config.clone();
        tokio::spawn(async move {
            infrastructure_metrics_loop(
                metrics_pool,
                metrics_client,
                metrics_vps1,
                metrics_default,
            )
            .await;
        });

        let throttle_pool = pool.clone();
        let throttle_vps1 = coolify_config_vps1.clone();
        let throttle_default = coolify_config.clone();
        tokio::spawn(async move {
            bandwidth_throttle_loop(throttle_pool, throttle_vps1, throttle_default).await;
        });

        let cpu_burst_pool = pool.clone();
        let cpu_burst_vps1 = coolify_config_vps1.clone();
        let cpu_burst_default = coolify_config.clone();
        tokio::spawn(async move {
            cpu_burst_loop(cpu_burst_pool, cpu_burst_vps1, cpu_burst_default).await;
        });
    } else {
        tracing::warn!("[infra-metrics] Coolify no configurado — sampler desactivado");
    }

    if let Some(contabo_config) = ContaboConfig::from_env() {
        let monitor_pool = pool.clone();
        let monitor_service = ContaboService::new(contabo_config, reqwest::Client::new());
        tokio::spawn(async move {
            vps_monitor_loop(monitor_pool, monitor_service).await;
        });
    } else {
        tracing::debug!("[vps-monitor] Contabo no configurado — monitor proveedor desactivado");
    }
}

fn spawn_http_watchdog(port: u16) {
    if std::env::var("GLORY_HTTP_WATCHDOG")
        .is_ok_and(|value| value.eq_ignore_ascii_case("false") || value == "0")
    {
        tracing::warn!("[watchdog] HTTP self-check desactivado por GLORY_HTTP_WATCHDOG");
        return;
    }

    /* [135A-1] Si Hyper queda aceptando TCP pero sin responder HTTP, Docker
     * puede dejar el contenedor unhealthy indefinidamente. Este watchdog fuerza
     * un reinicio limpio tras fallos consecutivos de /healthz. */
    tokio::spawn(async move {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .expect("watchdog HTTP client");
        let url = format!("http://127.0.0.1:{port}/healthz");
        let mut failures = 0_u8;

        tokio::time::sleep(Duration::from_mins(1)).await;
        loop {
            let ok = match client.get(&url).send().await {
                Ok(response) => response.status().is_success(),
                Err(error) => {
                    tracing::warn!("[watchdog] Health probe failed: {error}");
                    false
                }
            };

            if ok {
                failures = 0;
            } else {
                failures = failures.saturating_add(1);
                tracing::error!(failures, "[watchdog] HTTP health probe failed");
                if failures >= 3 {
                    tracing::error!("[watchdog] Reiniciando proceso por health HTTP congelado");
                    std::process::exit(1);
                }
            }

            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
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
    const CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_hours(1);

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
