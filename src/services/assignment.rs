/* [044A-38 Fase 4] Servicio de asignación: auto-asignación (24h), tomar orden (employee),
 * delegación, solicitud de ayuda. Incluye background loop con tokio::spawn.
 * Gotcha: el auto-assign usa specialties del `employee_profiles` — si no hay perfiles
 * creados aún, ningún empleado será elegible. ensure_employee_profile crea uno por defecto. */

use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{
    CreateNotification, DelegationResponse, DelegationStatus, EmployeeListItem, OrderResponse,
    OrderStatus, UserRole,
};
use crate::repositories::{DelegationRepository, NotificationRepository, OrderRepository};

pub struct AssignmentService;

impl AssignmentService {
    /// Empleado toma una orden sin asignar — valida disponibilidad y máximo concurrente
    pub async fn take_order(
        pool: &PgPool,
        order_id: Uuid,
        employee_id: Uuid,
    ) -> Result<OrderResponse, AppError> {
        let order = OrderRepository::find_order_by_id(pool, order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Orden no encontrada".into()))?;

        if order.status != OrderStatus::AwaitingAssignment {
            return Err(AppError::BadRequest(
                "La orden no está disponible para asignación".into(),
            ));
        }

        /* Verificar capacidad del empleado */
        let profile = DelegationRepository::ensure_employee_profile(pool, employee_id)
            .await
            .map_err(|e| AppError::Internal(format!("Error verificando perfil: {e}")))?;

        if profile.availability == "away" || profile.availability == "offline" {
            return Err(AppError::BadRequest(
                "Tu disponibilidad no permite tomar órdenes".into(),
            ));
        }

        let active_count = DelegationRepository::count_active_orders(pool, employee_id)
            .await
            .map_err(|e| AppError::Internal(format!("Error contando órdenes: {e}")))?;

        if active_count >= i64::from(profile.max_concurrent_orders) {
            return Err(AppError::BadRequest(format!(
                "Ya tienes {active_count}/{} órdenes activas",
                profile.max_concurrent_orders
            )));
        }

        /* Asignar: reutiliza la lógica existente de OrderRepository */
        let assigned = OrderRepository::assign_order(pool, order_id, employee_id).await?;

        /* Construir response */
        let (svc_title, svc_slug, plan_name) =
            OrderRepository::get_order_display_info(pool, assigned.service_id, assigned.plan_id)
                .await?;
        let phases = OrderRepository::list_order_phases(pool, order_id).await?;
        let employee_name = OrderRepository::get_employee_display_name(pool, assigned.assigned_employee_id).await?;

        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let total_phases = phases.len() as i32;

        Ok(OrderResponse {
            id: assigned.id,
            order_number: assigned.order_number,
            client_id: assigned.client_id,
            client_name: None,
            service_title: svc_title,
            service_slug: svc_slug,
            plan_name,
            payment_mode: assigned.payment_mode,
            base_price_cents: assigned.base_price_cents,
            discount_percent: assigned.discount_percent,
            final_price_cents: assigned.final_price_cents,
            currency: assigned.currency,
            status: assigned.status,
            assigned_employee_id: assigned.assigned_employee_id,
            assigned_employee_name: employee_name,
            current_phase: assigned.current_phase,
            total_phases,
            project_description: assigned.project_description,
            client_notes: assigned.client_notes,
            started_at: assigned.started_at,
            created_at: assigned.created_at,
            ai_intermediary_enabled: assigned.ai_intermediary_enabled.unwrap_or(false),
            ai_summary: assigned.ai_summary,
        })
    }

    /* [154A-15b] Admin ve todas las órdenes sin asignar; empleados solo las marcadas open_to_employees */
    pub async fn list_unassigned(pool: &PgPool, is_admin: bool) -> Result<Vec<OrderResponse>, AppError> {
        let all_orders = OrderRepository::list_unassigned_orders(pool).await?;
        let orders: Vec<_> = if is_admin {
            all_orders
        } else {
            all_orders.into_iter().filter(|o| o.open_to_employees.unwrap_or(false)).collect()
        };
        let mut result = Vec::with_capacity(orders.len());

        for order in orders {
            let (svc_title, svc_slug, plan_name) =
                OrderRepository::get_order_display_info(pool, order.service_id, order.plan_id)
                    .await?;
            let phases = OrderRepository::list_order_phases(pool, order.id).await?;

            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            let total_phases = phases.len() as i32;

            result.push(OrderResponse {
                id: order.id,
                order_number: order.order_number,
                client_id: order.client_id,
                client_name: None,
                service_title: svc_title,
                service_slug: svc_slug,
                plan_name,
                payment_mode: order.payment_mode,
                base_price_cents: order.base_price_cents,
                discount_percent: order.discount_percent,
                final_price_cents: order.final_price_cents,
                currency: order.currency,
                status: order.status,
                assigned_employee_id: order.assigned_employee_id,
                assigned_employee_name: None,
                current_phase: order.current_phase,
                total_phases,
                project_description: order.project_description,
                client_notes: order.client_notes,
                started_at: order.started_at,
                created_at: order.created_at,
                ai_intermediary_enabled: order.ai_intermediary_enabled.unwrap_or(false),
                ai_summary: order.ai_summary,
            });
        }

        Ok(result)
    }

    /// Lista de empleados con estadísticas (admin)
    pub async fn list_employees(pool: &PgPool) -> Result<Vec<EmployeeListItem>, AppError> {
        let rows = DelegationRepository::list_employees_with_stats(pool)
            .await
            .map_err(|e| AppError::Internal(format!("Error listando empleados: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| EmployeeListItem {
                user_id: r.user_id,
                email: r.email,
                specialties: r.specialties,
                availability: r.availability,
                current_orders: r.current_orders,
                max_concurrent_orders: r.max_concurrent_orders,
                total_completed_orders: r.total_completed_orders,
                average_rating: r.average_rating,
            })
            .collect())
    }

    /// Crear delegación (empleado pide que otro tome su orden)
    pub async fn create_delegation(
        pool: &PgPool,
        order_id: Uuid,
        employee_id: Uuid,
        reason: &str,
        delegation_type: &str,
    ) -> Result<DelegationResponse, AppError> {
        let order = OrderRepository::find_order_by_id(pool, order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Orden no encontrada".into()))?;

        if order.assigned_employee_id != Some(employee_id) {
            return Err(AppError::Forbidden(
                "Solo el empleado asignado puede delegar".into(),
            ));
        }

        let d = DelegationRepository::create_delegation(
            pool,
            order_id,
            employee_id,
            reason,
            delegation_type,
        )
        .await
        .map_err(|e| AppError::Internal(format!("Error creando delegación: {e}")))?;

        let (svc_title, _, _) =
            OrderRepository::get_order_display_info(pool, order.service_id, order.plan_id).await?;

        Ok(DelegationResponse {
            id: d.id,
            order_id: d.order_id,
            order_number: order.order_number,
            service_title: svc_title,
            from_employee_id: d.from_employee_id,
            to_employee_id: d.to_employee_id,
            reason: d.reason,
            delegation_type: d.delegation_type,
            status: d.status,
            created_at: d.created_at,
            resolved_at: d.resolved_at,
        })
    }

    /// Responder a una delegación: aceptar (reasigna si es 'delegate') o rechazar
    pub async fn respond_to_delegation(
        pool: &PgPool,
        delegation_id: Uuid,
        employee_id: Uuid,
        accept: bool,
    ) -> Result<DelegationResponse, AppError> {
        let deleg = DelegationRepository::find_by_id(pool, delegation_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Delegación no encontrada".into()))?;

        if deleg.status != DelegationStatus::Requested {
            return Err(AppError::BadRequest(
                "La delegación ya fue resuelta".into(),
            ));
        }

        if deleg.from_employee_id == employee_id {
            return Err(AppError::BadRequest(
                "No puedes aceptar tu propia delegación".into(),
            ));
        }

        let d = if accept {
            /* Verificar capacidad */
            let profile = DelegationRepository::ensure_employee_profile(pool, employee_id)
                .await
                .map_err(|e| AppError::Internal(format!("Error verificando perfil: {e}")))?;
            let active = DelegationRepository::count_active_orders(pool, employee_id)
                .await
                .map_err(|e| AppError::Internal(format!("Error contando órdenes: {e}")))?;

            if active >= i64::from(profile.max_concurrent_orders) {
                return Err(AppError::BadRequest(
                    "No tienes capacidad para más órdenes".into(),
                ));
            }

            let accepted =
                DelegationRepository::accept_delegation(pool, delegation_id, employee_id)
                    .await
                    .map_err(|e| {
                        AppError::Internal(format!("Error aceptando delegación: {e}"))
                    })?;

            /* Si es delegación completa (no help_request), reasignar la orden */
            if accepted.delegation_type == "delegate" {
                OrderRepository::assign_order(pool, deleg.order_id, employee_id).await?;
                DelegationRepository::complete_delegation(pool, delegation_id)
                    .await
                    .map_err(|e| {
                        AppError::Internal(format!("Error completando delegación: {e}"))
                    })?
            } else {
                accepted
            }
        } else {
            DelegationRepository::reject_delegation(pool, delegation_id)
                .await
                .map_err(|e| AppError::Internal(format!("Error rechazando delegación: {e}")))?
        };

        let order = OrderRepository::find_order_by_id(pool, deleg.order_id)
            .await?
            .ok_or_else(|| AppError::Internal("Orden de delegación no encontrada".into()))?;
        let (svc_title, _, _) =
            OrderRepository::get_order_display_info(pool, order.service_id, order.plan_id).await?;

        Ok(DelegationResponse {
            id: d.id,
            order_id: d.order_id,
            order_number: order.order_number,
            service_title: svc_title,
            from_employee_id: d.from_employee_id,
            to_employee_id: d.to_employee_id,
            reason: d.reason,
            delegation_type: d.delegation_type,
            status: d.status,
            created_at: d.created_at,
            resolved_at: d.resolved_at,
        })
    }

    /// Lista delegaciones (incoming + outgoing para empleado, todas para admin)
    pub async fn list_delegations(
        pool: &PgPool,
        employee_id: Uuid,
        effective_role: UserRole,
    ) -> Result<Vec<DelegationResponse>, AppError> {
        let delegations = if effective_role == UserRole::Admin {
            DelegationRepository::list_all(pool).await?
        } else {
            let mut all = DelegationRepository::list_incoming(pool, employee_id).await?;
            let outgoing = DelegationRepository::list_outgoing(pool, employee_id).await?;
            for d in outgoing {
                if !all.iter().any(|existing| existing.id == d.id) {
                    all.push(d);
                }
            }
            all
        };

        let mut result = Vec::with_capacity(delegations.len());
        for d in delegations {
            let order = OrderRepository::find_order_by_id(pool, d.order_id).await?;
            let (order_number, svc_title) = if let Some(ref o) = order {
                let (title, _, _) =
                    OrderRepository::get_order_display_info(pool, o.service_id, o.plan_id).await?;
                (o.order_number, title)
            } else {
                (0, "Orden eliminada".into())
            };

            result.push(DelegationResponse {
                id: d.id,
                order_id: d.order_id,
                order_number,
                service_title: svc_title,
                from_employee_id: d.from_employee_id,
                to_employee_id: d.to_employee_id,
                reason: d.reason,
                delegation_type: d.delegation_type,
                status: d.status,
                created_at: d.created_at,
                resolved_at: d.resolved_at,
            });
        }

        Ok(result)
    }

    /// Background loop: cada 60s revisa órdenes vencidas y auto-asigna al mejor empleado
    pub async fn auto_assign_loop(pool: PgPool) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Err(e) = Self::check_and_auto_assign(&pool).await {
                tracing::error!("Auto-assign error: {e}");
            }
        }
    }

    /* [154A-15b] Tras 48h sin tomar, marcar orden como open_to_employees para que cualquier
     * empleado la pueda tomar desde la sección "Disponibles".
     * Anteriormente auto-asignaba a 24h; ahora se da visibilidad y se deja que los empleados
     * la tomen voluntariamente. */
    async fn check_and_auto_assign(pool: &PgPool) -> Result<(), AppError> {
        let overdue = OrderRepository::list_overdue_unassigned(pool).await?;
        if overdue.is_empty() {
            return Ok(());
        }

        tracing::info!(
            "Overdue check: {} órdenes pasaron deadline de 48h",
            overdue.len()
        );

        for order in overdue {
            let already_open = order.open_to_employees.unwrap_or(false);
            if already_open {
                continue;
            }

            let res = sqlx::query!(
                "UPDATE orders SET open_to_employees = true, updated_at = NOW() WHERE id = $1",
                order.id
            )
            .execute(pool)
            .await;

            match res {
                Ok(_) => {
                    tracing::info!(
                        "Overdue: orden #{} marcada como open_to_employees",
                        order.order_number
                    );

                    /* [164A-10] Notificar a empleados disponibles que hay una nueva orden */
                    let employees = sqlx::query_scalar!(
                        "SELECT user_id FROM employee_profiles WHERE availability = 'available'"
                    )
                    .fetch_all(pool)
                    .await
                    .unwrap_or_default();

                    for emp_id in employees {
                        let notif = CreateNotification {
                            user_id: emp_id,
                            notification_type: "order_available".to_string(),
                            title: format!("Nueva orden disponible — #{}", order.order_number),
                            body: Some(
                                "Una orden ha quedado disponible. Puedes tomarla desde Disponibles."
                                    .into(),
                            ),
                            link: Some("/panel?seccion=disponibles".into()),
                            reference_type: Some("order".into()),
                            reference_id: Some(order.id),
                        };
                        let _ = NotificationRepository::create(pool, &notif).await;
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Overdue: error marcando orden #{}: {e}",
                        order.order_number
                    );
                }
            }
        }

        Ok(())
    }
}
