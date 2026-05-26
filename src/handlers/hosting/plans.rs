use axum::extract::{Path, State};
use axum::Json;
use validator::Validate;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{HostingPlanConfig, PublicHostingPlan, UpdatePlanConfigRequest, UserRole};
use crate::repositories::HostingRepository;
use crate::AppState;

struct HostingPlanMarketing {
    label: &'static str,
    description: &'static str,
    features: &'static [&'static str],
    recommended: bool,
}

fn normal_hosting_base_plan(plan: &str) -> (&str, bool) {
    plan.strip_prefix("normal-")
        .map_or((plan, false), |base| (base, true))
}

fn hosting_plan_marketing(plan: &str) -> HostingPlanMarketing {
    let (base_plan, is_normal) = normal_hosting_base_plan(plan);
    if is_normal {
        /* [155A-20] El catálogo público evita "hosting normal" y describe el caso de uso.
         * Mantiene los slugs `normal-*` por compatibilidad, pero el copy visible habla de
         * hosting administrado para sitios a medida, landings y frontends. */
        return match base_plan {
            "pro" => HostingPlanMarketing {
                label: "Hosting Profesional",
                description: "Hosting administrado para sitios con más tráfico, frontends personalizados y despliegues con mayor exigencia operativa.",
                features: &[
                    "Nginx administrado",
                    "20 GB almacenamiento SSD",
                    "Tráfico ilimitado",
                    "Free temporary domain",
                    "Certificado SSL incluido",
                    "Backups diarios",
                    "SFTP seguro",
                    "Recursos aislados",
                ],
                recommended: true,
            },
            "ecommerce" => HostingPlanMarketing {
                label: "Hosting Avanzado",
                description: "Hosting administrado de mayor capacidad para catálogos amplios, assets pesados y operaciones con más demanda.",
                features: &[
                    "Nginx administrado",
                    "50 GB almacenamiento SSD",
                    "Tráfico ilimitado",
                    "Free temporary domain",
                    "Certificado SSL incluido",
                    "Backups diarios + semanales",
                    "SFTP seguro",
                    "Recursos ampliados",
                ],
                recommended: false,
            },
            _ => HostingPlanMarketing {
                label: "Hosting Básico",
                description: "Hosting administrado con Nginx, SSL y SFTP para landings, sitios corporativos y proyectos sin WordPress.",
                features: &[
                    "Nginx administrado",
                    "5 GB almacenamiento SSD",
                    "Tráfico ilimitado",
                    "Free temporary domain",
                    "Certificado SSL incluido",
                    "Backups semanales",
                    "SFTP seguro",
                ],
                recommended: false,
            },
        };
    }

    match base_plan {
        "pro" => HostingPlanMarketing {
            label: "WordPress Profesional",
            description: "WordPress para negocios que necesitan más recursos, backups diarios y staging listo.",
            features: &[
                "WordPress pre-instalado",
                "20 GB almacenamiento SSD",
                "Tráfico ilimitado",
                "Free temporary domain",
                "Certificado SSL incluido",
                "Free CDN",
                "Backups diarios",
                "WP-CLI + SSH",
                "Staging environment",
            ],
            recommended: true,
        },
        "ecommerce" => HostingPlanMarketing {
            label: "WordPress Avanzado",
            description: "WordPress administrado de mayor capacidad para sitios con más contenido, tráfico y caché avanzada.",
            features: &[
                "WordPress pre-instalado",
                "50 GB almacenamiento SSD",
                "Tráfico ilimitado",
                "Free temporary domain",
                "Certificado SSL incluido",
                "Free CDN",
                "Backups diarios + semanales",
                "WP-CLI + SSH",
                "Caché avanzada WordPress",
            ],
            recommended: false,
        },
        _ => HostingPlanMarketing {
            label: "WordPress Básico",
            description: "WordPress administrado para sitios livianos, landings y contenido institucional con costo controlado.",
            features: &[
                "WordPress pre-instalado",
                "5 GB almacenamiento SSD",
                "Tráfico ilimitado",
                "Free temporary domain",
                "Certificado SSL incluido",
                "Free CDN",
                "Backups semanales",
                "WP-CLI + SSH",
            ],
            recommended: false,
        },
    }
}

pub(super) fn public_plan_from_config(config: HostingPlanConfig) -> PublicHostingPlan {
    let marketing = hosting_plan_marketing(&config.plan_name);
    PublicHostingPlan {
        plan_name: config.plan_name,
        label: marketing.label.to_string(),
        description: marketing.description.to_string(),
        monthly_price_cents: config.monthly_price_cents,
        wp_cpu_millicores: config.wp_cpu_millicores,
        wp_memory_mb: config.wp_memory_mb,
        db_cpu_millicores: config.db_cpu_millicores,
        db_memory_mb: config.db_memory_mb,
        ssh_cpu_millicores: config.ssh_cpu_millicores,
        ssh_memory_mb: config.ssh_memory_mb,
        storage_limit_mb: config.storage_limit_mb,
        bandwidth_limit_gb: config.bandwidth_limit_gb,
        features: marketing
            .features
            .iter()
            .map(|feature| (*feature).to_string())
            .collect(),
        recommended: marketing.recommended,
    }
}

/// Listar todas las configuraciones de planes de hosting (admin)
#[utoipa::path(
    get,
    path = "/api/hosting/plan-configs",
    responses(
        (status = 200, description = "Configuraciones de planes", body = Vec<HostingPlanConfig>),
        (status = 403, description = "Sin permisos"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub(super) async fn list_plan_configs(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<HostingPlanConfig>>, AppError> {
    auth.require_role(&[UserRole::Admin])?;
    let configs = HostingRepository::list_plan_configs(&state.pool).await?;
    Ok(Json(configs))
}

#[utoipa::path(
    get,
    path = "/api/hosting/public-plans",
    responses(
        (status = 200, description = "Catálogo público de hosting", body = Vec<PublicHostingPlan>),
    ),
    tag = "hosting"
)]
pub(super) async fn list_public_plans(
    State(state): State<AppState>,
) -> Result<Json<Vec<PublicHostingPlan>>, AppError> {
    let configs = HostingRepository::list_plan_configs(&state.pool).await?;
    Ok(Json(
        configs.into_iter().map(public_plan_from_config).collect(),
    ))
}

/// Actualizar configuración de un plan (admin). Campos opcionales: solo se actualizan los enviados.
#[utoipa::path(
    put,
    path = "/api/hosting/plan-configs/{plan}",
    params(("plan" = String, Path, description = "Nombre del plan (basico, pro, ecommerce)")),
    request_body = UpdatePlanConfigRequest,
    responses(
        (status = 200, description = "Config actualizada", body = HostingPlanConfig),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "Plan no encontrado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub(super) async fn update_plan_config(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(plan): Path<String>,
    Json(req): Json<UpdatePlanConfigRequest>,
) -> Result<Json<HostingPlanConfig>, AppError> {
    auth.require_role(&[UserRole::Admin])?;
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let updated = HostingRepository::update_plan_config(&state.pool, &plan, &req).await?;
    Ok(Json(updated))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn plan_config(plan_name: &str, monthly_price_cents: i32) -> HostingPlanConfig {
        HostingPlanConfig {
            id: Uuid::new_v4(),
            plan_name: plan_name.to_string(),
            monthly_price_cents,
            wp_cpu_millicores: 1000,
            wp_memory_mb: 512,
            db_cpu_millicores: 500,
            db_memory_mb: 512,
            ssh_cpu_millicores: 500,
            ssh_memory_mb: 256,
            storage_limit_mb: 20_480,
            bandwidth_limit_gb: 200,
            cpu_scaling_policy: "contention_throttle".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn public_plan_from_config_uses_marketing_metadata() {
        let public_plan = public_plan_from_config(plan_config("pro", 413));
        assert_eq!(public_plan.label, "WordPress Profesional");
        assert!(public_plan.recommended);
        assert!(public_plan
            .features
            .iter()
            .any(|feature| feature.contains("Staging")));
    }

    #[test]
    fn public_plan_from_config_distinguishes_normal_hosting() {
        let public_plan = public_plan_from_config(plan_config("normal-pro", 537));
        assert_eq!(public_plan.label, "Hosting Profesional");
        assert!(public_plan.recommended);
        assert!(public_plan
            .features
            .iter()
            .any(|feature| feature.contains("Nginx")));
    }
}
