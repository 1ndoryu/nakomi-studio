use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct InfrastructureServerMetricsResponse {
    pub id: Uuid,
    pub label: String,
    pub provider: String,
    pub server_ip: String,
    pub status: String,
    pub is_active: bool,
    pub coolify_server_uuid: Option<String>,
    pub cpu_cores: Option<f64>,
    pub ram_mb: Option<i32>,
    pub disk_mb: Option<i32>,
    pub cpu_avg_1h: Option<f64>,
    pub ram_used_mb: Option<f64>,
    pub ram_limit_mb: Option<f64>,
    pub disk_used_mb: Option<f64>,
    pub disk_limit_mb: Option<f64>,
    pub sampled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct ResourceMetricPoint {
    pub sampled_at: DateTime<Utc>,
    pub cpu_percent: Option<f64>,
    pub ram_used_mb: Option<f64>,
    pub ram_limit_mb: Option<f64>,
    pub disk_used_mb: Option<f64>,
    pub disk_limit_mb: Option<f64>,
    pub site_cpu_limit_cores: Option<f64>,
    pub site_ram_limit_mb: Option<f64>,
    pub db_cpu_limit_cores: Option<f64>,
    pub db_ram_limit_mb: Option<f64>,
    pub ssh_cpu_limit_cores: Option<f64>,
    pub ssh_ram_limit_mb: Option<f64>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DeploymentMetricsResponse {
    pub deployment_uuid: String,
    pub range: String,
    pub points: Vec<ResourceMetricPoint>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct ResourceUsageReportItem {
    pub subscription_id: Uuid,
    pub client_name: String,
    pub domain: Option<String>,
    pub plan: String,
    pub status: String,
    pub storage_limit_mb: i32,
    pub storage_used_mb: Option<f64>,
    pub bandwidth_limit_gb: i32,
    pub bandwidth_used_gb: Option<f64>,
    pub storage_usage_pct: Option<f64>,
    pub bandwidth_usage_pct: Option<f64>,
}
