use axum::extract::{Path, State};
use axum::Json;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::UserRole;
use crate::AppState;

pub(super) fn map_contabo_error(message: &str) -> AppError {
    let lower = message.to_ascii_lowercase();
    tracing::warn!("Contabo request failed: {message}");

    if lower.contains("invalid_grant")
        || lower.contains("auth failed: 400")
        || lower.contains("auth failed: 401")
        || lower.contains("unauthorized")
    {
        return AppError::ServiceUnavailable(
            "Contabo rechazó la autenticación. Revisa CONTABO_API_PASSWORD y las credenciales OAuth2 configuradas.".into(),
        );
    }

    if lower.contains("parse error") {
        return AppError::ServiceUnavailable(
            "Contabo respondió con un formato inesperado. Revisa la integración antes de usar el panel VPS.".into(),
        );
    }

    if lower.contains("api error: 5")
        || lower.contains("timed out")
        || lower.contains("dns")
        || lower.contains("temporarily unavailable")
    {
        return AppError::ServiceUnavailable(
            "Contabo no está disponible temporalmente. Intenta de nuevo en unos minutos.".into(),
        );
    }

    AppError::ServiceUnavailable(
        "No se pudo consultar Contabo. Revisa la configuración y el estado del proveedor.".into(),
    )
}

/// Listar instancias VPS (admin only — proxy Contabo API)
#[utoipa::path(
    get,
    path = "/api/hosting/vps",
    responses(
        (status = 200, description = "Lista de VPS"),
        (status = 403, description = "Sin permisos"),
        (status = 503, description = "Contabo no configurado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub(super) async fn list_vps(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let service = state
        .contabo_service
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Contabo API no configurada".into()))?;

    let instances = service
        .list_instances()
        .await
        .map_err(|error| map_contabo_error(&error))?;

    Ok(Json(serde_json::json!({ "data": instances })))
}

/// Obtener instancia VPS por ID (admin only)
#[utoipa::path(
    get,
    path = "/api/hosting/vps/{instance_id}",
    params(("instance_id" = i64, Path, description = "ID de instancia Contabo")),
    responses(
        (status = 200, description = "Detalles de la VPS"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "Instancia no encontrada"),
        (status = 503, description = "Contabo no configurado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub(super) async fn get_vps(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(instance_id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    let service = state
        .contabo_service
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Contabo API no configurada".into()))?;

    let instance = service.get_instance(instance_id).await.map_err(|error| {
        if error.to_ascii_lowercase().contains("not found") {
            AppError::NotFound(format!("VPS {instance_id} no encontrada"))
        } else {
            map_contabo_error(&error)
        }
    })?;

    Ok(Json(serde_json::json!({ "data": instance })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_contabo_error_invalid_grant_is_service_unavailable() {
        let error = map_contabo_error(
            "Contabo auth failed: 400 Bad Request — {\"error_description\":\"invalid_grant\"}",
        );

        match error {
            AppError::ServiceUnavailable(message) => {
                assert!(message.contains("CONTABO_API_PASSWORD"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn map_contabo_error_parse_issue_is_service_unavailable() {
        let error = map_contabo_error("Contabo parse error: missing field data");

        match error {
            AppError::ServiceUnavailable(message) => {
                assert!(message.contains("formato inesperado"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }
}
