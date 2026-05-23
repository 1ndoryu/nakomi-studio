use axum::routing::get;
use axum::Router;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::GovernorLayer;

use super::checkout::{admin_test_subscribe, create_checkout, subscribe_self};
use super::control::{restart_hosting, start_hosting, stop_hosting};
use super::deployments::{delete_deployment, list_deployments};
use super::domain::{dns_check, verify_domain};
use super::infrastructure::{
    deployment_metrics, list_infrastructure_servers, refresh_infrastructure_metrics,
    resource_usage_report,
};
use super::plans::{list_plan_configs, list_public_plans, update_plan_config};
use super::provisioning::{provision_subscription, refresh_hosting, rotate_credentials};
use super::stats::get_hosting_stats;
use super::subscriptions::{
    assign_hosting, create_subscription, delete_subscription, get_subscription, list_events,
    list_subscriptions, request_cancel, update_status, update_subscription,
};
use super::vps::{get_vps, list_vps};
use crate::AppState;

#[allow(clippy::too_many_lines)]
pub fn hosting_routes() -> Router<AppState> {
    /* [174A-17] Rate limits específicos para endpoints de pago de hosting.
     * subscribe: máx 3 por hora por IP (evita abuso de checkouts).
     * checkout: máx 5 por hora por IP. */
    let subscribe_gov = GovernorConfigBuilder::default()
        .per_second(1200)
        .burst_size(3)
        .finish()
        .expect("subscribe rate limit config");
    let checkout_gov = GovernorConfigBuilder::default()
        .per_second(720)
        .burst_size(5)
        .finish()
        .expect("checkout rate limit config");

    Router::new()
        .route(
            "/hosting/subscriptions",
            get(list_subscriptions).post(create_subscription),
        )
        .route(
            "/hosting/subscriptions/:id",
            get(get_subscription)
                .put(update_subscription)
                .delete(delete_subscription),
        )
        .route(
            "/hosting/subscriptions/:id/status",
            axum::routing::patch(update_status),
        )
        /* [304A-3] Admin asigna hosting a cliente registrado por email */
        .route(
            "/hosting/subscriptions/:id/assign",
            axum::routing::patch(assign_hosting),
        )
        .route("/hosting/subscriptions/:id/events", get(list_events))
        /* [094A-8] Stats reales de una suscripción */
        .route("/hosting/subscriptions/:id/stats", get(get_hosting_stats))
        .route(
            "/hosting/subscriptions/:id/cancel",
            axum::routing::post(request_cancel),
        )
        /* [084A-24] Stripe checkout para hosting — rate limited */
        .route(
            "/hosting/subscriptions/:id/checkout",
            axum::routing::post(create_checkout).layer(GovernorLayer {
                config: std::sync::Arc::new(checkout_gov),
            }),
        )
        /* [094A-3] Self-service: cliente contrata + paga — rate limited */
        .route(
            "/hosting/subscribe",
            axum::routing::post(subscribe_self).layer(GovernorLayer {
                config: std::sync::Arc::new(subscribe_gov),
            }),
        )
        /* [164A-19] Despliegues reales de Coolify en servidores configurados */
        .route("/hosting/deployments", get(list_deployments))
        .route("/infrastructure/deployments", get(list_deployments))
        .route(
            "/hosting/deployments/:deployment_uuid/metrics",
            get(deployment_metrics),
        )
        .route(
            "/infrastructure/deployments/:deployment_uuid/metrics",
            get(deployment_metrics),
        )
        .route("/infrastructure/servers", get(list_infrastructure_servers))
        .route(
            "/infrastructure/metrics/refresh",
            axum::routing::post(refresh_infrastructure_metrics),
        )
        .route(
            "/infrastructure/resource-report",
            get(resource_usage_report),
        )
        .route(
            "/hosting/deployments/:deployment_uuid",
            axum::routing::delete(delete_deployment),
        )
        /* [084A-24] VPS stats: proxy a Contabo API */
        .route("/hosting/vps", get(list_vps))
        .route("/hosting/vps/:instance_id", get(get_vps))
        /* [154A-11] Provisioning real: crea servicio Nginx en Coolify */
        .route(
            "/hosting/subscriptions/:id/provision",
            axum::routing::post(provision_subscription),
        )
        /* [114A-1] Rotación de credenciales SFTP */
        .route(
            "/hosting/subscriptions/:id/rotate-credentials",
            axum::routing::post(rotate_credentials),
        )
        .route(
            "/hosting/subscriptions/:id/verify-domain",
            axum::routing::post(verify_domain),
        )
        /* [154A-16] Verificación DNS de un dominio */
        .route("/hosting/subscriptions/:id/dns-check", get(dns_check))
        /* [114A-3] Plan configs: admin gestiona precios y límites de recursos */
        .route("/hosting/plan-configs", get(list_plan_configs))
        .route("/hosting/public-plans", get(list_public_plans))
        .route(
            "/hosting/plan-configs/:plan",
            axum::routing::put(update_plan_config),
        )
        /* [114A-4] Refresh: regenera compose con plan config actual */
        .route(
            "/hosting/subscriptions/:id/refresh",
            axum::routing::post(refresh_hosting),
        )
        /* [154A-9] Control de servicio: restart / stop / start */
        .route(
            "/hosting/subscriptions/:id/restart",
            axum::routing::post(restart_hosting),
        )
        .route(
            "/hosting/subscriptions/:id/stop",
            axum::routing::post(stop_hosting),
        )
        .route(
            "/hosting/subscriptions/:id/start",
            axum::routing::post(start_hosting),
        )
        /* [154A-14] Admin test subscribe: crea hosting sin Stripe */
        .route(
            "/hosting/admin-test-subscribe",
            axum::routing::post(admin_test_subscribe),
        )
}
