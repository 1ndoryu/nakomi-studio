use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use std::collections::HashMap;

use super::stats::resolve_ssh_key;
use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::{CoolifyDeploymentResponse, UserRole};
use crate::repositories::HostingRepository;
use crate::services::coolify::CoolifyServiceSummary;
use crate::services::docker_stats::ContainerStats;
use crate::services::{CoolifyConfig, CoolifyService};
use crate::AppState;

fn map_coolify_services(
    services: Vec<CoolifyServiceSummary>,
    label: &str,
    subscriptions_by_uuid: &HashMap<&str, &crate::models::HostingSubscription>,
    subscriptions_by_name: &HashMap<&str, &crate::models::HostingSubscription>,
) -> Vec<CoolifyDeploymentResponse> {
    services
        .into_iter()
        .map(|service| {
            let linked_subscription = subscriptions_by_uuid
                .get(service.uuid.as_str())
                .copied()
                .or_else(|| subscriptions_by_name.get(service.name.as_str()).copied());

            CoolifyDeploymentResponse {
                uuid: service.uuid,
                name: service.name,
                status: service.status,
                fqdn: service.fqdn,
                server_uuid: service.server_uuid,
                server_name: service.server_name,
                project_uuid: service.project_uuid,
                environment_name: service.environment_name,
                linked_subscription_id: linked_subscription.map(|subscription| subscription.id),
                linked_subscription_domain: linked_subscription
                    .and_then(|subscription| subscription.domain.clone()),
                linked_subscription_status: linked_subscription
                    .map(|subscription| subscription.status.clone()),
                linked_subscription_plan: linked_subscription
                    .map(|subscription| subscription.plan.clone()),
                server_label: label.to_string(),
                linked_subscription_client: linked_subscription
                    .map(|subscription| subscription.client_name.clone()),
                storage_limit_mb: linked_subscription
                    .map(|subscription| subscription.storage_limit_mb),
                cpu_percent: None,
                ram_used_mb: None,
                ram_limit_mb: None,
                storage_used_mb: None,
            }
        })
        .collect()
}

/* [225A-1] Obtiene stats de TODOS los contenedores y el uso de disco de TODAS las apps/servicios
 * en una sola SSH call. Retorna (Vec<ContainerStats>, HashMap<String, i64>) con {uuid: mb}. 
 * Utiliza un caché local de 30 segundos para evitar llamadas SSH excesivas. */
async fn fetch_all_container_stats(server_ip: &str, ssh_key: &str) -> (Vec<ContainerStats>, HashMap<String, i64>) {
    struct CacheEntry {
        fetched_at: std::time::Instant,
        containers: Vec<ContainerStats>,
        storage: HashMap<String, i64>,
    }

    static SERVER_STATS_CACHE: std::sync::OnceLock<tokio::sync::RwLock<HashMap<String, CacheEntry>>> = std::sync::OnceLock::new();

    let cache = SERVER_STATS_CACHE.get_or_init(|| tokio::sync::RwLock::new(HashMap::new()));
    
    // 1. Intentar leer del caché
    {
        let cache_guard = cache.read().await;
        if let Some(entry) = cache_guard.get(server_ip) {
            if entry.fetched_at.elapsed() < std::time::Duration::from_secs(30) {
                return (entry.containers.clone(), entry.storage.clone());
            }
        }
    }

    let docker_cmd = "docker stats --no-stream --format '{{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}'; echo '===STORAGE==='; du -sm /var/lib/docker/volumes/*/_data 2>/dev/null || true";

    let mut command = tokio::process::Command::new("ssh");
    command
        .args([
            "-i",
            ssh_key,
            "-o",
            "StrictHostKeyChecking=accept-new",
            "-o",
            "ConnectTimeout=8",
            "-o",
            "BatchMode=yes",
            &format!("root@{server_ip}"),
            docker_cmd,
        ])
        .stdin(std::process::Stdio::null());

    let output_result = tokio::time::timeout(std::time::Duration::from_secs(15), command.output()).await;

    let output = match output_result {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => {
            tracing::warn!("[deployments] SSH a {server_ip} falló al ejecutar: {e}");
            return (vec![], HashMap::new());
        }
        Err(_) => {
            tracing::warn!("[deployments] Timeout: SSH a {server_ip} tomó más de 15 segundos y fue cancelado");
            return (vec![], HashMap::new());
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut parts = stdout.split("===STORAGE===");
    
    let stats_part = parts.next().unwrap_or("").trim();
    let storage_part = parts.next().unwrap_or("").trim();

    let containers = crate::services::docker_stats::parse_docker_stats_public(stats_part);
    
    let mut storage_map = HashMap::new();
    for line in storage_part.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() >= 2 {
            if let Ok(mb) = cols[0].parse::<i64>() {
                // cols[1] es como "/var/lib/docker/volumes/uuid_data/_data"
                if let Some(folder_path) = cols[1].strip_suffix("/_data") {
                    if let Some(volume_name) = folder_path.split('/').last() {
                        let uuid = volume_name.split('_').next().unwrap_or(volume_name);
                        *storage_map.entry(uuid.to_string()).or_insert(0) += mb;
                    }
                }
            }
        }
    }

    // 2. Guardar en el caché
    {
        let mut cache_guard = cache.write().await;
        cache_guard.insert(server_ip.to_string(), CacheEntry {
            fetched_at: std::time::Instant::now(),
            containers: containers.clone(),
            storage: storage_map.clone(),
        });
    }

    (containers, storage_map)
}

/* [225A-1] Enriquece despliegues con CPU/RAM de TODOS los contenedores (2 SSH calls máximo,
 * en paralelo via tokio::join!). Storage se omite aquí — se consulta en stats individuales. */
async fn enrich_deployment_resources(
    state: &AppState,
    deployments: &mut [CoolifyDeploymentResponse],
) {
    /* Fase 1: obtener docker stats de cada VPS en paralelo */
    let vps1_fut = async {
        if let Some(cfg) = state.coolify_config_vps1.as_ref() {
            if let Some(ssh_key) = resolve_ssh_key(state, &cfg.server_ip) {
                let (stats, storage) = fetch_all_container_stats(&cfg.server_ip, ssh_key).await;
                return Some((cfg.server_ip.clone(), stats, storage));
            }
        }
        None
    };
    let vps2_fut = async {
        if let Some(cfg) = state.coolify_config.as_ref() {
            if let Some(ssh_key) = resolve_ssh_key(state, &cfg.server_ip) {
                let (stats, storage) = fetch_all_container_stats(&cfg.server_ip, ssh_key).await;
                return Some((cfg.server_ip.clone(), stats, storage));
            }
        }
        None
    };

    let (res1, res2) = tokio::join!(vps1_fut, vps2_fut);
    let mut stats_by_server: HashMap<String, (Vec<ContainerStats>, HashMap<String, i64>)> = HashMap::new();
    if let Some((ip, s, disk)) = res1 {
        tracing::info!("[deployments] VPS1 ({ip}): {} contenedores, {} carpetas disco", s.len(), disk.len());
        stats_by_server.insert(ip, (s, disk));
    }
    if let Some((ip, s, disk)) = res2 {
        tracing::info!("[deployments] VPS2 ({ip}): {} contenedores, {} carpetas disco", s.len(), disk.len());
        stats_by_server.insert(ip, (s, disk));
    }

    /* Fase 2: cruzar CPU/RAM y Disco en memoria */
    for deployment in deployments {
        let server_ip = if deployment.server_label == "VPS Principal" {
            state
                .coolify_config_vps1
                .as_ref()
                .map(|c| c.server_ip.as_str())
        } else {
            state
                .coolify_config
                .as_ref()
                .map(|c| c.server_ip.as_str())
        };

        if let Some(server_ip) = server_ip {
            if let Some((containers, storage_map)) = stats_by_server.get(server_ip) {
                let mut cpu = 0.0_f64;
                let mut ram_used = 0.0_f64;
                let mut ram_limit = 0.0_f64;
                let mut found = false;

                if deployment.server_label == "VPS2" {
                    tracing::info!("[deployments/debug] Checking VPS2 deployment: name='{}', uuid='{}'", deployment.name, deployment.uuid);
                }

                for c in containers {
                    // Coolify nombra los contenedores con el UUID del recurso (ej: app-do8k... o wordpress-u00g...)
                    if c.name.contains(&deployment.uuid) || c.name.contains(&deployment.name) {
                        cpu += c.cpu_percent;
                        ram_used += c.mem_used_mb;
                        ram_limit += c.mem_limit_mb;
                        found = true;
                    }
                }

                if found {
                    deployment.cpu_percent = Some(cpu);
                    deployment.ram_used_mb = Some(ram_used);
                    deployment.ram_limit_mb = Some(ram_limit);
                }
                
                // Si la carpeta del despliegue existe en el storage map, asignarlo
                if let Some(&mb) = storage_map.get(&deployment.uuid) {
                    deployment.storage_used_mb = Some(mb);
                }
            }
        }
    }
}

struct DeploymentsCacheEntry {
    fetched_at: std::time::Instant,
    deployments: Vec<CoolifyDeploymentResponse>,
}

static DEPLOYMENTS_CACHE: std::sync::OnceLock<tokio::sync::RwLock<DeploymentsCacheEntry>> = std::sync::OnceLock::new();

/// Listar despliegues reales de todas las VPS configuradas en Coolify (admin only)
#[utoipa::path(
    get,
    path = "/api/hosting/deployments",
    responses(
        (status = 200, description = "Lista de despliegues reales en Coolify (todas las VPS)", body = Vec<CoolifyDeploymentResponse>),
        (status = 403, description = "Sin permisos"),
        (status = 503, description = "Coolify no configurado"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
#[allow(clippy::too_many_lines)]
pub(super) async fn list_vps2_deployments(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<CoolifyDeploymentResponse>>, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    if state.coolify_config.is_none() && state.coolify_config_vps1.is_none() {
        return Err(AppError::ServiceUnavailable(
            "Coolify no está configurado para listar despliegues".into(),
        ));
    }

    let cache = DEPLOYMENTS_CACHE.get_or_init(|| {
        tokio::sync::RwLock::new(DeploymentsCacheEntry {
            // Empezar con una fecha antigua para forzar el primer refresh
            fetched_at: std::time::Instant::now() - std::time::Duration::from_secs(9999),
            deployments: Vec::new(),
        })
    });

    let needs_refresh;
    let mut current_deployments = Vec::new();
    {
        let cache_guard = cache.read().await;
        if !cache_guard.deployments.is_empty() {
            current_deployments = cache_guard.deployments.clone();
            // Stale-while-revalidate: cache dura 30 segundos, pero devolvemos viejo mientras carga el nuevo
            needs_refresh = cache_guard.fetched_at.elapsed() > std::time::Duration::from_secs(30);
        } else {
            needs_refresh = true;
        }
    }

    if needs_refresh {
        if current_deployments.is_empty() {
            tracing::info!("[deployments] Primer carga, esperando datos...");
            current_deployments = build_deployments(state.clone()).await?;
            let mut cache_guard = cache.write().await;
            cache_guard.deployments = current_deployments.clone();
            cache_guard.fetched_at = std::time::Instant::now();
        } else {
            tracing::info!("[deployments] Devolviendo de caché (stale), refrescando en background...");
            let state_clone = state.clone();
            tokio::spawn(async move {
                if let Ok(new_deployments) = build_deployments(state_clone).await {
                    let cache = DEPLOYMENTS_CACHE.get().unwrap();
                    let mut cache_guard = cache.write().await;
                    cache_guard.deployments = new_deployments;
                    cache_guard.fetched_at = std::time::Instant::now();
                    tracing::info!("[deployments] Caché refrescado en background");
                } else {
                    tracing::warn!("[deployments] Falló el refresco en background");
                }
            });
        }
    } else {
        tracing::info!("[deployments] Devolviendo de caché (fresco)");
    }

    Ok(Json(current_deployments))
}

async fn build_deployments(state: AppState) -> Result<Vec<CoolifyDeploymentResponse>, AppError> {
    tracing::info!("[deployments] -> Inicio build_deployments");

    tracing::info!("[deployments] Consultando repositorios...");
    let subscriptions = HostingRepository::list_all(&state.pool).await?;
    let subscriptions_by_uuid: HashMap<&str, _> = subscriptions
        .iter()
        .filter_map(|subscription| {
            subscription
                .server_uuid
                .as_deref()
                .map(|server_uuid| (server_uuid, subscription))
        })
        .collect();
    let subscriptions_by_name: HashMap<&str, _> = subscriptions
        .iter()
        .filter_map(|subscription| {
            subscription
                .coolify_site_name
                .as_deref()
                .map(|site_name| (site_name, subscription))
        })
        .collect();

    let mut deployments: Vec<CoolifyDeploymentResponse> = Vec::new();

    tracing::info!("[deployments] Consultando VPS1 en Coolify...");
    if let Some(cfg) = state.coolify_config_vps1.as_ref() {
        match CoolifyService::list_services(&state.http_client, cfg).await {
            Ok(services) => {
                tracing::info!("[deployments] VPS1 devolvió {} servicios", services.len());
                deployments.extend(map_coolify_services(
                    services,
                    "VPS Principal",
                    &subscriptions_by_uuid,
                    &subscriptions_by_name,
                ))
            },
            Err(error) => tracing::warn!("[deployments] Error listando VPS1: {error}"),
        }
    }

    tracing::info!("[deployments] Consultando VPS2 en Coolify...");
    if let Some(cfg) = state.coolify_config.as_ref() {
        match CoolifyService::list_services(&state.http_client, cfg).await {
            Ok(services) => {
                tracing::info!("[deployments] VPS2 devolvió {} servicios", services.len());
                deployments.extend(map_coolify_services(
                    services,
                    "VPS2",
                    &subscriptions_by_uuid,
                    &subscriptions_by_name,
                ))
            },
            Err(error) => tracing::warn!("[deployments] Error listando VPS2: {error}"),
        }
    }

    tracing::info!("[deployments] Iniciando enrich_deployment_resources...");
    enrich_deployment_resources(&state, &mut deployments).await;
    tracing::info!("[deployments] Finalizó enrich_deployment_resources");

    deployments.sort_by(|left, right| {
        right
            .linked_subscription_id
            .is_some()
            .cmp(&left.linked_subscription_id.is_some())
            .then(left.name.cmp(&right.name))
            .then(left.uuid.cmp(&right.uuid))
    });

    tracing::info!("[deployments] -> Fin build_deployments");
    Ok(deployments)
}

/* [165A-4] Permite limpiar despliegues huérfanos desde el panel admin.
 * Solo borra stacks sin vínculo en BD para evitar desalinear suscripciones reales. */
#[utoipa::path(
    delete,
    path = "/api/hosting/deployments/{deployment_uuid}",
    params(("deployment_uuid" = String, Path, description = "UUID del despliegue en Coolify")),
    responses(
        (status = 204, description = "Despliegue eliminado"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "Despliegue no encontrado"),
        (status = 409, description = "El despliegue ya está vinculado a una suscripción"),
        (status = 503, description = "Coolify no configurado o no disponible"),
    ),
    security(("bearer_auth" = [])),
    tag = "hosting"
)]
pub(super) async fn delete_deployment(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(deployment_uuid): Path<String>,
) -> Result<StatusCode, AppError> {
    auth.require_role(&[UserRole::Admin])?;

    if state.coolify_config.is_none() && state.coolify_config_vps1.is_none() {
        return Err(AppError::ServiceUnavailable(
            "Coolify no está configurado para eliminar despliegues".into(),
        ));
    }

    let subscriptions = HostingRepository::list_all(&state.pool).await?;
    let mut lookup_failed = false;
    let mut target_config: Option<&CoolifyConfig> = None;
    let mut target_name: Option<String> = None;

    if let Some(cfg) = state.coolify_config_vps1.as_ref() {
        match CoolifyService::list_services(&state.http_client, cfg).await {
            Ok(services) => {
                if let Some(service) = services
                    .into_iter()
                    .find(|service| service.uuid == deployment_uuid)
                {
                    target_name = Some(service.name);
                    target_config = Some(cfg);
                }
            }
            Err(error) => {
                lookup_failed = true;
                tracing::warn!(
                    "[deployments] Error buscando despliegue {} en VPS Principal: {}",
                    deployment_uuid,
                    error
                );
            }
        }
    }

    if target_config.is_none() {
        if let Some(cfg) = state.coolify_config.as_ref() {
            match CoolifyService::list_services(&state.http_client, cfg).await {
                Ok(services) => {
                    if let Some(service) = services
                        .into_iter()
                        .find(|service| service.uuid == deployment_uuid)
                    {
                        target_name = Some(service.name);
                        target_config = Some(cfg);
                    }
                }
                Err(error) => {
                    lookup_failed = true;
                    tracing::warn!(
                        "[deployments] Error buscando despliegue {} en VPS2: {}",
                        deployment_uuid,
                        error
                    );
                }
            }
        }
    }

    let target_config = match target_config {
        Some(config) => config,
        None if lookup_failed => {
            return Err(AppError::ServiceUnavailable(
                "No se pudo consultar Coolify para ubicar el despliegue".into(),
            ));
        }
        None => {
            return Err(AppError::NotFound(
                "Despliegue no encontrado en Coolify".into(),
            ));
        }
    };

    let target_name = target_name.expect("deployment name must exist when config is found");
    let linked_subscription = subscriptions.iter().find(|subscription| {
        subscription.server_uuid.as_deref() == Some(deployment_uuid.as_str())
            || subscription.coolify_site_name.as_deref() == Some(target_name.as_str())
    });

    if let Some(subscription) = linked_subscription {
        return Err(AppError::Conflict(format!(
            "El despliegue {} ya está vinculado a la suscripción {}. Elimínalo desde la suscripción para no dejar datos huérfanos.",
            target_name,
            subscription.id
        )));
    }

    CoolifyService::delete_service(&state.http_client, target_config, &deployment_uuid, true)
        .await?;
    tracing::info!(
        "[deployments] Despliegue huérfano {} ({}) eliminado desde el panel admin.",
        target_name,
        deployment_uuid
    );

    Ok(StatusCode::NO_CONTENT)
}
