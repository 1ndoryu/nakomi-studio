#![allow(clippy::needless_for_each)] // Generado por utoipa OpenApi derive

mod admin_seed;
mod admin_services;
mod admin_users;
mod assignment;
mod auth;
mod blog;
mod chat;
mod dashboard;
mod deliverables;
mod health;
mod hosting;
mod profile;
mod notes;
mod notifications;
mod orders;
mod payments;
mod projects;
mod public_users;
mod refunds;
mod reviews;
mod seo;
mod services;
mod team_members;
mod uploads;

use axum::Router;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use axum::http::{HeaderName, HeaderValue, Method};
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::AppState;

/// Define el esquema de seguridad Bearer para Swagger UI
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        /* components existe porque el derive ya registra schemas */
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
        auth::register,
        auth::quick_register,
        auth::login,
        notes::create_note,
        notes::get_note,
        notes::list_notes,
        notes::update_note,
        notes::delete_note,
        services::list_services,
        services::get_service,
        orders::create_order,
        orders::list_orders,
        orders::get_order,
        orders::assign_order,
        orders::switch_role,
        orders::cancel_order_handler,
        orders::approve_phase,
        orders::request_revision,
        payments::initiate_payment,
        payments::stripe_webhook,
        payments::list_payments,
        assignment::take_order,
        assignment::list_unassigned,
        assignment::list_employees,
        assignment::create_delegation,
        assignment::create_help_request,
        assignment::respond_delegation,
        assignment::list_delegations,
        chat::list_sessions,
        chat::get_messages,
        chat::create_session,
        chat::send_message,
        deliverables::deliver_phase_with_files,
        deliverables::list_deliverables,
        deliverables::download_deliverable,
        refunds::request_refund,
        refunds::review_refund,
        refunds::list_refunds,
        refunds::get_order_refund,
        reviews::create_review,
        reviews::respond_review,
        reviews::get_order_review,
        reviews::list_reviews,
        notifications::list_notifications,
        notifications::get_unread_count,
        notifications::mark_read,
        notifications::mark_all_read,
        dashboard::get_dashboard,
        profile::get_profile,
        profile::upload_avatar,
        admin_users::list_users,
        admin_users::change_role,
        admin_users::change_status,
        admin_services::list_all,
        admin_services::create,
        admin_services::update,
        admin_services::archive,
        admin_services::destroy,
        blog::list_published,
        blog::get_by_slug,
        blog::list_all,
        blog::create,
        blog::update,
        blog::archive,
        blog::destroy,
        uploads::upload_image,
        projects::list_published,
        projects::get_by_slug,
        projects::list_all,
        projects::create,
        projects::update,
        projects::archive,
        projects::destroy,
        team_members::list_published,
        team_members::list_all,
        team_members::create,
        team_members::update,
        team_members::archive,
        team_members::destroy,
        public_users::get_profile,
        public_users::get_reviews_received,
        public_users::get_reviews_given,
        public_users::get_rating_distribution,
    ),
    components(schemas(
        health::HealthResponse,
        crate::models::RegisterRequest,
        crate::models::QuickRegisterRequest,
        crate::models::LoginRequest,
        crate::models::AuthResponse,
        crate::models::Note,
        crate::models::CreateNoteRequest,
        crate::models::UpdateNoteRequest,
        crate::models::PaginatedNotes,
        crate::models::UserRole,
        crate::models::ServiceDetailResponse,
        crate::models::ServicePlanResponse,
        crate::models::ServicePlanPhaseResponse,
        crate::models::CreateOrderRequest,
        crate::models::OrderResponse,
        crate::models::OrderPhaseResponse,
        crate::models::SwitchRoleRequest,
        crate::models::PaymentMode,
        crate::models::OrderStatus,
        crate::models::PhaseStatus,
        crate::models::PaymentStatus,
        crate::models::InitiatePaymentRequest,
        crate::models::PaymentIntentResponse,
        crate::models::PaymentResponse,
        crate::models::DelegationStatus,
        crate::models::DelegationResponse,
        crate::models::EmployeeListItem,
        crate::models::CreateDelegationRequest,
        crate::models::RespondDelegationRequest,
        crate::models::ChatSession,
        crate::models::ChatMessage,
        crate::models::ChatSessionResponse,
        crate::models::CreateChatSessionRequest,
        crate::models::SendMessageRequest,
        crate::models::PhaseDeliverable,
        crate::models::DeliverPhaseResponse,
        crate::models::PhaseDeliverablesResponse,
        crate::models::RefundStatus,
        crate::models::RefundResponse,
        crate::models::RequestRefundBody,
        crate::models::ReviewRefundBody,
        crate::models::ReviewAction,
        crate::models::CreateReviewBody,
        crate::models::RespondReviewBody,
        crate::models::ReviewResponse,
        crate::models::NotificationResponse,
        crate::models::UnreadCountResponse,
        crate::models::MarkReadBody,
        crate::models::WsNotification,
        crate::models::DashboardResponse,
        crate::models::RevenueStats,
        crate::models::OrderCounts,
        crate::models::EmployeePerformance,
        crate::models::DashboardAlerts,
        crate::models::AdminUserItem,
        crate::models::PaginatedUsers,
        crate::models::ChangeRoleRequest,
        crate::models::ChangeStatusRequest,
        crate::models::AdminServiceResponse,
        crate::models::CreateServiceRequest,
        crate::models::UpdateServiceRequest,
        crate::models::BlogPostResponse,
        crate::models::PaginatedBlogPosts,
        crate::models::CreateBlogPostRequest,
        crate::models::UpdateBlogPostRequest,
        crate::models::ProjectResponse,
        crate::models::ProjectLink,
        crate::models::ProjectSkill,
        crate::models::CreateProjectRequest,
        crate::models::UpdateProjectRequest,
        crate::models::TeamMemberResponse,
        crate::models::CreateTeamMemberRequest,
        crate::models::UpdateTeamMemberRequest,
        crate::models::PublicUserProfile,
        crate::models::PublicReviewItem,
        crate::models::PaginatedPublicReviews,
        crate::models::RatingDistribution,
        profile::AvatarResponse,
        uploads::UploadResponse,
        crate::errors::ErrorResponse,
    )),
    modifiers(&SecurityAddon),
    info(
        title = "Glory RS API",
        version = "0.1.0",
        description = "Template API — Rust + Axum + OpenAPI"
    )
)]
#[allow(clippy::needless_for_each)]
pub struct ApiDoc;

/// Crea el router principal con CORS, tracing, Swagger UI y todas las rutas
pub fn create_router(pool: sqlx::PgPool, config: crate::config::AppConfig) -> Router {
    let chat_hub = crate::services::ChatHub::new(pool.clone());
    let notification_hub = crate::services::NotificationHub::new(pool.clone());
    let ai_config = crate::services::AiChatConfig::from_env();

    /* [084A-24] Contabo VPS: servicio opcional, solo se activa si las 4 credenciales existen */
    let contabo_service = crate::services::ContaboConfig::from_env().map(|cfg| {
        tracing::info!("Contabo API configurado para {}", cfg.api_user);
        crate::services::ContaboService::new(cfg, reqwest::Client::new())
    });

    /* [084A-24] Stripe Hosting: config de prices para checkout de suscripciones */
    let hosting_stripe_config = crate::services::HostingStripeConfig::from_env();
    if hosting_stripe_config.is_some() {
        tracing::info!("Stripe Hosting configurado (3 planes)");
    }

    let state = AppState {
        pool,
        jwt_secret: config.jwt_secret,
        http_client: reqwest::Client::new(),
        stripe_secret_key: config.stripe_secret_key,
        stripe_webhook_secret: config.stripe_webhook_secret,
        chat_hub,
        ai_config,
        notification_hub,
        contabo_service,
        hosting_stripe_config,
    };

    /* [064A-73] CORS: restringir orígenes en producción. Si GLORY_ALLOWED_ORIGINS vacío, allow all (dev). */
    let cors = if config.allowed_origins.is_empty() {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let origins: Vec<HeaderValue> = config
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([
                HeaderName::from_static("content-type"),
                HeaderName::from_static("authorization"),
            ])
            .allow_credentials(true)
    };

    /* [064A-73] Security headers OWASP */
    let hsts = SetResponseHeaderLayer::if_not_present(
        HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    let nosniff = SetResponseHeaderLayer::if_not_present(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    let frame_deny = SetResponseHeaderLayer::if_not_present(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(seo::routes())
        /* [044A-38 Fase 5] WebSocket routes at root level (not under /api) */
        .merge(chat::ws_routes())
        /* [044A-38 Fase 9] WebSocket de notificaciones en tiempo real */
        .merge(notifications::ws_routes())
        /* [044A-43] Servir archivos estáticos de uploads/ (avatares, etc.) */
        .nest_service("/uploads", ServeDir::new("uploads"))
        .nest("/api", api_routes())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(hsts)
        .layer(nosniff)
        .layer(frame_deny)
        .with_state(state)
}

/* [044A-9] Monta el frontend React como SPA: archivos estaticos con fallback a index.html.
 * Solo se activa si STATIC_DIR esta configurado (en produccion). En desarrollo, Vite sirve el frontend. */
pub fn create_app(pool: sqlx::PgPool, config: crate::config::AppConfig) -> Router {
    let static_dir = config.static_dir.clone();
    let router = create_router(pool, config);

    if let Some(dir) = static_dir {
        let index_path = format!("{dir}/index.html");
        let serve = ServeDir::new(&dir).not_found_service(ServeFile::new(&index_path));
        router.fallback_service(serve)
    } else {
        router
    }
}

fn api_routes() -> Router<AppState> {
    /* [064A-73] Rate limiting: auth estricto (5 req/min por IP), API general (120 req/min) */
    let auth_governor = GovernorConfigBuilder::default()
        .per_second(12)
        .burst_size(5)
        .finish()
        .expect("rate limit config válida");

    let api_governor = GovernorConfigBuilder::default()
        .per_second(1)
        .burst_size(120)
        .finish()
        .expect("rate limit config válida");

    let auth_routes = auth::routes().layer(GovernorLayer {
        config: std::sync::Arc::new(auth_governor),
    });

    Router::new()
        .merge(health::routes())
        .merge(auth_routes)
        .merge(notes::routes())
        .merge(services::routes())
        /* assignment routes ANTES de orders: /orders/unassigned (literal) debe
         * registrarse antes de /orders/:order_id (parámetro) para evitar conflicto */
        .merge(assignment::routes())
        .merge(orders::routes())
        .merge(payments::routes())
        .merge(chat::rest_routes())
        .merge(deliverables::routes())
        .merge(refunds::routes())
        .merge(reviews::routes())
        .merge(notifications::routes())
        .merge(dashboard::routes())
        .merge(profile::routes())
        .merge(admin_users::routes())
        .merge(admin_services::routes())
        .merge(blog::public_routes())
        .merge(blog::admin_routes())
        .merge(projects::public_routes())
        .merge(projects::admin_routes())
        .merge(team_members::public_routes())
        .merge(team_members::admin_routes())
        .merge(public_users::public_routes())
        .merge(admin_seed::seed_routes())
        .merge(hosting::hosting_routes())
        .merge(uploads::routes())
        .layer(GovernorLayer {
            config: std::sync::Arc::new(api_governor),
        })
}
