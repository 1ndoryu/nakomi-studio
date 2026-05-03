/* [044A-38 Fase 6] Modelo de entregables (phase_deliverables).
 * Archivos subidos por el empleado al entregar una fase.
 * Incluye revision_number para historial de entregas. */

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/* ============================================================
ENTREGABLE
============================================================ */

#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct PhaseDeliverable {
    pub id: Uuid,
    pub phase_id: Uuid,
    pub uploaded_by: Uuid,
    pub file_name: String,
    pub file_url: String,
    pub file_size_bytes: Option<i64>,
    pub mime_type: Option<String>,
    pub revision_number: i32,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/* ============================================================
REQUESTS / RESPONSES
============================================================ */

/// Request para entregar fase con notas (archivos van por multipart separado)
#[derive(Debug, Deserialize, ToSchema)]
pub struct DeliverPhaseRequest {
    pub notes: Option<String>,
}

/// Respuesta completa de entrega con archivos
#[derive(Debug, Serialize, ToSchema)]
pub struct DeliverPhaseResponse {
    pub phase_id: Uuid,
    pub revision_number: i32,
    pub deliverables: Vec<PhaseDeliverable>,
}

/// Respuesta de listar entregables de una fase
#[derive(Debug, Serialize, ToSchema)]
pub struct PhaseDeliverablesResponse {
    pub deliverables: Vec<PhaseDeliverable>,
    pub approval_status: String,
    pub revisions_used: i32,
    pub max_revisions: i32,
}

/* MIME whitelist para upload de archivos */
pub const ALLOWED_MIME_TYPES: &[&str] = &[
    "application/pdf",
    "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/svg+xml",
    "image/webp",
    "video/mp4",
    "application/zip",
    "application/x-rar-compressed",
    "application/x-7z-compressed",
    "application/octet-stream",
];

pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; /* 10 MB */
pub const MAX_FILES_PER_DELIVERY: usize = 5;
