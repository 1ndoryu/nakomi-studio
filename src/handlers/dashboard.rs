/* [044A-38 Fase 10] Handler del dashboard admin.
 * Endpoint único que ejecuta 4 queries en paralelo con tokio::join!
 * y devuelve la respuesta consolidada. Solo admin. */

use axum::{extract::State, routing::get, Json, Router};

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{
    DashboardAlerts, DashboardResponse, EmployeePerformance, OrderCounts, RevenueStats, UserRole,
};
use crate::repositories::{DashboardRepository, NotificationRepository};
use crate::AppState;

/// Obtiene las métricas consolidadas del dashboard admin
#[utoipa::path(
    get,
    path = "/api/admin/dashboard",
    responses(
        (status = 200, description = "Métricas del dashboard", body = DashboardResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_dashboard(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<DashboardResponse>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    /* Ejecutar las 4 queries en paralelo */
    let (revenue_res, orders_res, employees_res, alerts_res) = tokio::join!(
        DashboardRepository::get_revenue(&state.pool),
        DashboardRepository::get_order_counts(&state.pool),
        DashboardRepository::get_employee_performance(&state.pool),
        DashboardRepository::get_alerts(&state.pool),
    );

    let revenue_row = revenue_res?;
    let orders_row = orders_res?;
    let employee_rows = employees_res?;
    let alerts_row = alerts_res?;

    /* Conteo de notificaciones admin no leídas */
    let admin_unread = NotificationRepository::count_unread(&state.pool, auth.user_id)
        .await
        .unwrap_or(0);

    let response = DashboardResponse {
        revenue: RevenueStats {
            total_revenue: revenue_row.total_revenue.unwrap_or(0.0),
            monthly_revenue: revenue_row.monthly_revenue.unwrap_or(0.0),
            held_amount: revenue_row.held_amount.unwrap_or(0.0),
            refunded_amount: revenue_row.refunded_amount.unwrap_or(0.0),
        },
        orders: OrderCounts {
            total: orders_row.total.unwrap_or(0),
            active: orders_row.active.unwrap_or(0),
            completed: orders_row.completed.unwrap_or(0),
            cancelled: orders_row.cancelled.unwrap_or(0),
            awaiting_assignment: orders_row.awaiting_assignment.unwrap_or(0),
        },
        employees: employee_rows
            .into_iter()
            .map(|e| EmployeePerformance {
                employee_id: e.employee_id.to_string(),
                email: e.email,
                active_orders: e.active_orders.unwrap_or(0),
                completed_orders: e.completed_orders.unwrap_or(0),
                average_rating: e.average_rating,
            })
            .collect(),
        alerts: DashboardAlerts {
            unassigned_orders: alerts_row.unassigned_orders.unwrap_or(0),
            pending_refunds: alerts_row.pending_refunds.unwrap_or(0),
            overdue_orders: alerts_row.overdue_orders.unwrap_or(0),
            unread_admin_notifications: admin_unread,
        },
    };

    Ok(Json(response))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/admin/dashboard", get(get_dashboard))
}
