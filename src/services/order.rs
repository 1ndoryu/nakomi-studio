/* [044A-38] Servicio de órdenes: lógica de negocio para crear órdenes, calcular
 * descuentos según payment_mode, y generar fases automáticamente desde plantillas.
 * Descuentos: Full = 20%, HalfHalf = 10%, Phased = 0%. */

use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{
    CreateOrderRequest, Order, OrderPhaseResponse, OrderResponse,
    OrderStatus, PaymentMode, PhaseStatus, ServiceDetailResponse, ServicePlanResponse,
    ServicePlanPhaseResponse, UpdateOrderPhaseDefinitionRequest,
};
use crate::repositories::{OrderRepository, CreateOrderParams, CreatePhaseParams};

pub struct OrderService;

impl OrderService {
    /// Lista todos los servicios activos con planes y fases
    pub async fn list_services(pool: &PgPool) -> Result<Vec<ServiceDetailResponse>, AppError> {
        let services = OrderRepository::list_services(pool).await?;
        let mut result = Vec::with_capacity(services.len());

        for svc in services {
            let plans = OrderRepository::list_plans_for_service(pool, svc.id).await?;
            let mut plan_responses = Vec::with_capacity(plans.len());

            for plan in plans {
                let phases = OrderRepository::list_plan_phases(pool, plan.id).await?;
                plan_responses.push(ServicePlanResponse {
                    id: plan.id,
                    slug: plan.slug,
                    name: plan.name,
                    price_cents: plan.price_cents,
                    description: plan.description,
                    features: plan.features,
                    is_highlighted: plan.is_highlighted,
                    is_custom: plan.is_custom,
                    phases: phases.into_iter().map(ServicePlanPhaseResponse::from).collect(),
                });
            }

            result.push(ServiceDetailResponse {
                id: svc.id,
                slug: svc.slug,
                title: svc.title,
                description: svc.description,
                image_url: svc.image_url,
                base_price_cents: svc.base_price_cents,
                skills: svc.skills,
                content: svc.content,
                gallery: svc.gallery,
                meta_title: svc.meta_title,
                meta_description: svc.meta_description,
                plans: plan_responses,
            });
        }

        Ok(result)
    }

    /// Detalle completo de un servicio por slug
    pub async fn get_service(
        pool: &PgPool,
        slug: &str,
    ) -> Result<ServiceDetailResponse, AppError> {
        let svc = OrderRepository::find_service_by_slug(pool, slug)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Servicio '{slug}' no encontrado")))?;

        let plans = OrderRepository::list_plans_for_service(pool, svc.id).await?;
        let mut plan_responses = Vec::with_capacity(plans.len());

        for plan in plans {
            let phases = OrderRepository::list_plan_phases(pool, plan.id).await?;
            plan_responses.push(ServicePlanResponse {
                id: plan.id,
                slug: plan.slug,
                name: plan.name,
                price_cents: plan.price_cents,
                description: plan.description,
                features: plan.features,
                is_highlighted: plan.is_highlighted,
                is_custom: plan.is_custom,
                phases: phases.into_iter().map(ServicePlanPhaseResponse::from).collect(),
            });
        }

        Ok(ServiceDetailResponse {
            id: svc.id,
            slug: svc.slug,
            title: svc.title,
            description: svc.description,
            image_url: svc.image_url,
            base_price_cents: svc.base_price_cents,
            skills: svc.skills,
            content: svc.content,
            gallery: svc.gallery,
            meta_title: svc.meta_title,
            meta_description: svc.meta_description,
            plans: plan_responses,
        })
    }

    /// Crea una orden: resuelve servicio/plan, calcula descuento, genera fases
    pub async fn create_order(
        pool: &PgPool,
        client_id: Uuid,
        req: CreateOrderRequest,
    ) -> Result<OrderResponse, AppError> {
        let CreateOrderRequest {
            service_slug,
            plan_slug,
            payment_mode,
            project_description,
            client_notes,
        } = req;

        let svc = OrderRepository::find_service_by_slug(pool, &service_slug)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Servicio '{service_slug}' no encontrado"))
            })?;

        let plan = OrderRepository::find_plan_by_slug(pool, svc.id, &plan_slug)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Plan '{plan_slug}' no encontrado"))
            })?;

        let base_price = plan.price_cents;
        let discount = Self::discount_for_mode(payment_mode);
        let final_price = base_price - (base_price * discount / 100);

        let plan_phases = OrderRepository::list_plan_phases(pool, plan.id).await?;

        let order = OrderRepository::create_order(
            pool,
            CreateOrderParams {
                client_id,
                service_id: svc.id,
                plan_id: plan.id,
                payment_mode,
                base_price_cents: base_price,
                discount_percent: discount,
                final_price_cents: final_price,
                project_description: project_description.as_deref(),
                client_notes: client_notes.as_deref(),
            },
        )
        .await?;

        /* Generar fases de la orden a partir de las plantillas del plan */
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let total_phases = plan_phases.len() as i32;
        for tmpl in &plan_phases {
            let phase_price = final_price * tmpl.percentage_of_total / 100;
            let status = if tmpl.phase_number == 1 {
                Self::initial_phase_status(payment_mode)
            } else {
                PhaseStatus::Locked
            };

            let (title, description) = Self::resolve_phase_definition(
                payment_mode,
                tmpl.phase_number,
                &tmpl.title,
                tmpl.description.as_deref(),
                project_description.as_deref(),
            );

            OrderRepository::create_order_phase(
                pool,
                CreatePhaseParams {
                    order_id: order.id,
                    phase_number: tmpl.phase_number,
                    title: &title,
                    description: description.as_deref(),
                    price_cents: phase_price,
                    status,
                    max_revisions: tmpl.max_revisions,
                    estimated_days: tmpl.estimated_days,
                },
            )
            .await?;
        }

        Ok(OrderResponse {
            id: order.id,
            order_number: order.order_number,
            client_id: order.client_id,
            client_name: None,
            service_title: svc.title,
            service_slug: svc.slug,
            plan_name: plan.name,
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
        })
    }

    /// Obtiene el detalle de una orden con sus fases.
    /// Retorna (`client_id`, response, phases) para que el handler verifique acceso.
    pub async fn get_order(
        pool: &PgPool,
        order_id: Uuid,
    ) -> Result<(Uuid, OrderResponse, Vec<OrderPhaseResponse>), AppError> {
        let order = OrderRepository::find_order_by_id(pool, order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Orden no encontrada".into()))?;

        let phases = OrderRepository::list_order_phases(pool, order_id).await?;
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let total_phases = phases.len() as i32;

        let (svc_title, svc_slug, plan_name) =
            OrderRepository::get_order_display_info(pool, order.service_id, order.plan_id).await?;

        let employee_name = OrderRepository::get_employee_display_name(pool, order.assigned_employee_id).await?;
        let client_name = OrderRepository::get_client_display_name(pool, order.client_id).await?;

        let response = OrderResponse {
            id: order.id,
            order_number: order.order_number,
            client_id: order.client_id,
            client_name,
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
            assigned_employee_name: employee_name,
            current_phase: order.current_phase,
            total_phases,
            project_description: order.project_description,
            client_notes: order.client_notes,
            started_at: order.started_at,
            created_at: order.created_at,
            ai_intermediary_enabled: order.ai_intermediary_enabled.unwrap_or(false),
            ai_summary: order.ai_summary.clone(),
        };

        let phase_responses: Vec<OrderPhaseResponse> =
            phases.into_iter().map(OrderPhaseResponse::from).collect();

        Ok((order.client_id, response, phase_responses))
    }

    /// Lista órdenes según rol: cliente ve las suyas, employee las asignadas, admin todas
    pub async fn list_orders_for_user(
        pool: &PgPool,
        user_id: Uuid,
        effective_role: crate::models::UserRole,
    ) -> Result<Vec<OrderResponse>, AppError> {
        use crate::models::UserRole;

        let orders = match effective_role {
            UserRole::Client => OrderRepository::list_orders_for_client(pool, user_id).await?,
            UserRole::Employee => OrderRepository::list_orders_for_employee(pool, user_id).await?,
            UserRole::Admin => OrderRepository::list_all_orders(pool).await?,
        };

        let mut result = Vec::with_capacity(orders.len());
        for order in orders {
            let (svc_title, svc_slug, plan_name) =
                OrderRepository::get_order_display_info(pool, order.service_id, order.plan_id)
                    .await?;
            let phases = OrderRepository::list_order_phases(pool, order.id).await?;
            let employee_name = OrderRepository::get_employee_display_name(pool, order.assigned_employee_id).await?;
            let client_name = OrderRepository::get_client_display_name(pool, order.client_id).await?;

            result.push(OrderResponse {
                id: order.id,
                order_number: order.order_number,
                client_id: order.client_id,
                client_name,
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
                assigned_employee_name: employee_name,
                current_phase: order.current_phase,
                total_phases: {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                    { phases.len() as i32 }
                },
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

    /// Asigna un empleado a una orden (admin o auto-asignación)
    pub async fn assign_order(
        pool: &PgPool,
        order_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Order, AppError> {
        let order = OrderRepository::find_order_by_id(pool, order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Orden no encontrada".into()))?;

        if order.status != OrderStatus::AwaitingAssignment
            && order.status != OrderStatus::PaymentHeld
        {
            return Err(AppError::BadRequest(
                "La orden no está en estado de asignación".into(),
            ));
        }

        let assigned = OrderRepository::assign_order(pool, order_id, employee_id).await?;
        Ok(assigned)
    }

    /* [104A-28] Cancela una orden. Empleados pueden cancelar con razón obligatoria.
     * Máquina de estados: PendingPayment | PaymentHeld | AwaitingAssignment | InProgress → Cancelled */
    pub async fn cancel_order(
        pool: &PgPool,
        order_id: Uuid,
        user_id: Uuid,
        effective_role: crate::models::UserRole,
        reason: Option<&str>,
    ) -> Result<Order, AppError> {
        use crate::models::UserRole;

        let order = OrderRepository::find_order_by_id(pool, order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Orden no encontrada".into()))?;

        /* Verificar permisos: dueño, empleado asignado (con razón), o admin */
        match effective_role {
            UserRole::Admin => {}
            UserRole::Client => {
                if order.client_id != user_id {
                    return Err(AppError::Forbidden("No tienes permiso para cancelar esta orden".into()));
                }
            }
            UserRole::Employee => {
                if order.assigned_employee_id != Some(user_id) {
                    return Err(AppError::Forbidden("No estás asignado a esta orden".into()));
                }
                if reason.is_none_or(|r| r.trim().is_empty()) {
                    return Err(AppError::Validation(
                        "Los empleados deben proporcionar una razón para cancelar".into(),
                    ));
                }
            }
        }

        /* [104A-29] Solo se puede cancelar en estados iniciales + in_progress para empleados.
         * pending_payment ya no se usa a nivel de orden — solo en fases. */
        let can_cancel = matches!(
            order.status,
            OrderStatus::PaymentHeld
                | OrderStatus::AwaitingAssignment
        ) || (effective_role == UserRole::Employee
            && order.status == OrderStatus::InProgress);

        if !can_cancel {
            return Err(AppError::BadRequest(
                format!("No se puede cancelar una orden en estado {:?}", order.status),
            ));
        }

        /* Guardar razón si se proporcionó */
        if let Some(r) = reason {
            sqlx::query!(
                "UPDATE orders SET cancel_reason = $1 WHERE id = $2",
                r,
                order_id,
            )
            .execute(pool)
            .await
            .map_err(|e| AppError::Internal(format!("Error guardando razón: {e}")))?;
        }

        let cancelled = OrderRepository::cancel_order(pool, order_id).await?;
        Ok(cancelled)
    }

    pub async fn update_project_description(
        pool: &PgPool,
        order_id: Uuid,
        user_id: Uuid,
        effective_role: crate::models::UserRole,
        project_description: String,
    ) -> Result<OrderResponse, AppError> {
        use crate::models::UserRole;

        let order = OrderRepository::find_order_by_id(pool, order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Orden no encontrada".into()))?;

        if effective_role != UserRole::Admin && order.client_id != user_id {
            return Err(AppError::Forbidden(
                "Solo el cliente dueño puede actualizar la descripción del proyecto".into(),
            ));
        }

        if matches!(order.status, OrderStatus::Completed | OrderStatus::Cancelled) {
            return Err(AppError::BadRequest(
                "La descripción no se puede editar en una orden cerrada".into(),
            ));
        }

        let updated =
            OrderRepository::update_project_description(pool, order_id, &project_description)
                .await?;
        let phases = OrderRepository::list_order_phases(pool, order_id).await?;
        let (svc_title, svc_slug, plan_name) =
            OrderRepository::get_order_display_info(pool, updated.service_id, updated.plan_id)
                .await?;
        let employee_name =
            OrderRepository::get_employee_display_name(pool, updated.assigned_employee_id).await?;
        let client_name = OrderRepository::get_client_display_name(pool, updated.client_id).await?;

        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let total_phases = phases.len() as i32;

        Ok(OrderResponse {
            id: updated.id,
            order_number: updated.order_number,
            client_id: updated.client_id,
            client_name,
            service_title: svc_title,
            service_slug: svc_slug,
            plan_name,
            payment_mode: updated.payment_mode,
            base_price_cents: updated.base_price_cents,
            discount_percent: updated.discount_percent,
            final_price_cents: updated.final_price_cents,
            currency: updated.currency,
            status: updated.status,
            assigned_employee_id: updated.assigned_employee_id,
            assigned_employee_name: employee_name,
            current_phase: updated.current_phase,
            total_phases,
            project_description: updated.project_description,
            client_notes: updated.client_notes,
            started_at: updated.started_at,
            created_at: updated.created_at,
            ai_intermediary_enabled: updated.ai_intermediary_enabled.unwrap_or(false),
            ai_summary: updated.ai_summary,
        })
    }

    pub async fn update_phase_definition(
        pool: &PgPool,
        order_id: Uuid,
        phase_number: i32,
        user_id: Uuid,
        effective_role: crate::models::UserRole,
        req: UpdateOrderPhaseDefinitionRequest,
    ) -> Result<crate::models::OrderPhaseResponse, AppError> {
        use crate::models::UserRole;

        let order = OrderRepository::find_order_by_id(pool, order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Orden no encontrada".into()))?;

        if order.payment_mode != PaymentMode::Phased {
            return Err(AppError::BadRequest(
                "Solo las órdenes por fases admiten definición manual de fases".into(),
            ));
        }

        if effective_role != UserRole::Admin && order.assigned_employee_id != Some(user_id) {
            return Err(AppError::Forbidden(
                "Solo el empleado asignado puede definir las fases".into(),
            ));
        }

        let phase = OrderRepository::find_phase_by_number(pool, order_id, phase_number)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Fase {phase_number} no encontrada")))?;

        if !matches!(phase.status, PhaseStatus::Locked | PhaseStatus::PendingPayment) {
            return Err(AppError::BadRequest(
                "Solo se pueden definir fases que aún no comenzaron".into(),
            ));
        }

        if req.title.is_none()
            && req.description.is_none()
            && req.price_cents.is_none()
            && req.estimated_days.is_none()
            && req.max_revisions.is_none()
        {
            return Err(AppError::BadRequest(
                "Debes enviar al menos un cambio para actualizar la fase".into(),
            ));
        }

        let title = req.title.as_deref().map(str::trim);
        if matches!(title, Some("")) {
            return Err(AppError::Validation(
                "El título de la fase no puede estar vacío".into(),
            ));
        }

        let description = req.description.as_deref().map(str::trim);
        if let Some(price_cents) = req.price_cents {
            if price_cents <= 0 {
                return Err(AppError::Validation(
                    "El precio de la fase debe ser mayor que cero".into(),
                ));
            }
        }

        if let Some(estimated_days) = req.estimated_days {
            if estimated_days <= 0 {
                return Err(AppError::Validation(
                    "Los días estimados deben ser mayores que cero".into(),
                ));
            }
        }

        if let Some(max_revisions) = req.max_revisions {
            if max_revisions < 0 {
                return Err(AppError::Validation(
                    "Las revisiones máximas no pueden ser negativas".into(),
                ));
            }
        }

        let updated = OrderRepository::update_order_phase_definition(
            pool,
            phase.id,
            title,
            description,
            req.price_cents,
            req.estimated_days,
            req.max_revisions,
        )
        .await?;

        Ok(crate::models::OrderPhaseResponse::from(updated))
    }

    /* [044A-38 Fase 2] Empleado entrega una fase.
     * Máquina de estados: InProgress | Paid | RevisionRequested → Delivered.
     * Solo el empleado asignado puede entregar. */
    pub async fn deliver_phase(
        pool: &PgPool,
        order_id: Uuid,
        phase_number: i32,
        employee_id: Uuid,
    ) -> Result<crate::models::OrderPhase, AppError> {
        let order = OrderRepository::find_order_by_id(pool, order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Orden no encontrada".into()))?;

        if order.assigned_employee_id != Some(employee_id) {
            return Err(AppError::Forbidden("Solo el empleado asignado puede entregar".into()));
        }

        let phase = OrderRepository::find_phase_by_number(pool, order_id, phase_number)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Fase {phase_number} no encontrada")))?;

        match phase.status {
            PhaseStatus::InProgress | PhaseStatus::Paid | PhaseStatus::RevisionRequested => {}
            _ => {
                return Err(AppError::BadRequest(
                    format!("No se puede entregar una fase en estado {:?}", phase.status),
                ));
            }
        }

        let delivered = OrderRepository::deliver_phase(pool, phase.id).await?;
        Ok(delivered)
    }

    /* [044A-38 Fase 2] Cliente aprueba una fase.
     * Máquina de estados: Delivered → Approved.
     * Si es la última fase, marca la orden como Completed.
     * Si hay siguiente fase, la desbloquea (Locked → PendingPayment o Paid según modo). */
    pub async fn approve_phase(
        pool: &PgPool,
        order_id: Uuid,
        phase_number: i32,
        client_id: Uuid,
    ) -> Result<crate::models::OrderPhase, AppError> {
        let order = OrderRepository::find_order_by_id(pool, order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Orden no encontrada".into()))?;

        if order.client_id != client_id {
            return Err(AppError::Forbidden("Solo el cliente puede aprobar fases".into()));
        }

        let phase = OrderRepository::find_phase_by_number(pool, order_id, phase_number)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Fase {phase_number} no encontrada")))?;

        if phase.status != PhaseStatus::Delivered {
            return Err(AppError::BadRequest(
                format!("Solo se pueden aprobar fases entregadas (estado actual: {:?})", phase.status),
            ));
        }

        let approved = OrderRepository::approve_phase(pool, phase.id).await?;

        /* Desbloquear siguiente fase o completar la orden */
        let all_phases = OrderRepository::list_order_phases(pool, order_id).await?;
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let total = all_phases.len() as i32;
        let all_approved = all_phases.iter().all(|p| p.status == PhaseStatus::Approved || p.id == phase.id);

        if all_approved {
            OrderRepository::complete_order(pool, order_id).await?;
        } else {
            /* Desbloquear siguiente fase */
            let next_number = phase_number + 1;
            if let Some(next_phase) = all_phases.iter().find(|p| p.phase_number == next_number) {
                if next_phase.status == PhaseStatus::Locked {
                    let unlock_status = Self::initial_phase_status(order.payment_mode);
                    OrderRepository::update_phase_status(pool, next_phase.id, unlock_status).await?;
                }
            }
            if next_number <= total {
                OrderRepository::update_current_phase(pool, order_id, next_number).await?;
            }
        }

        Ok(approved)
    }

    /* [044A-38 Fase 2] Cliente solicita revisión.
     * Máquina de estados: Delivered → RevisionRequested.
     * Valida que no se excedan las revisiones máximas. */
    pub async fn request_revision(
        pool: &PgPool,
        order_id: Uuid,
        phase_number: i32,
        client_id: Uuid,
    ) -> Result<crate::models::OrderPhase, AppError> {
        let order = OrderRepository::find_order_by_id(pool, order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Orden no encontrada".into()))?;

        if order.client_id != client_id {
            return Err(AppError::Forbidden("Solo el cliente puede solicitar revisión".into()));
        }

        let phase = OrderRepository::find_phase_by_number(pool, order_id, phase_number)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Fase {phase_number} no encontrada")))?;

        if phase.status != PhaseStatus::Delivered {
            return Err(AppError::BadRequest(
                format!("Solo se puede pedir revisión en fases entregadas (estado actual: {:?})", phase.status),
            ));
        }

        if phase.revisions_used >= phase.max_revisions {
            return Err(AppError::BadRequest(
                format!("Se alcanzó el límite de revisiones ({}/{})", phase.revisions_used, phase.max_revisions),
            ));
        }

        let revised = OrderRepository::request_revision(pool, phase.id).await?;
        Ok(revised)
    }

    /* ============================================================
       HELPERS PRIVADOS
       ============================================================ */

    /// Descuento por modo de pago: full=20%, `half_half`=10%, phased=0%
    fn discount_for_mode(mode: PaymentMode) -> i32 {
        match mode {
            PaymentMode::Full => 20,
            PaymentMode::HalfHalf => 10,
            PaymentMode::Phased => 0,
        }
    }

    /// Estado inicial de la primera fase según modo de pago
    fn initial_phase_status(mode: PaymentMode) -> PhaseStatus {
        match mode {
            PaymentMode::Full => PhaseStatus::Paid,
            PaymentMode::HalfHalf | PaymentMode::Phased => PhaseStatus::PendingPayment,
        }
    }

    fn resolve_phase_definition(
        payment_mode: PaymentMode,
        phase_number: i32,
        title: &str,
        description: Option<&str>,
        project_description: Option<&str>,
    ) -> (String, Option<String>) {
        if payment_mode != PaymentMode::Phased {
            return (title.to_string(), description.map(ToOwned::to_owned));
        }

        if phase_number == 1 {
            let detail = project_description
                .filter(|value| !value.trim().is_empty())
                .map_or_else(|| {
                    "Revisión del brief del cliente y definición del alcance inicial del proyecto.".to_string()
                }, |value| {
                    format!(
                        "Brief inicial del cliente: {value}. Esta fase sirve para revisar alcance, prioridades y la estructura del proyecto."
                    )
                });
            return ("Definición del proyecto".to_string(), Some(detail));
        }

        (
            format!("Fase {phase_number} por definir"),
            Some(
                "Esta fase será definida por el especialista cuando revise el brief y la conversación con el cliente.".to_string(),
            ),
        )
    }
}
