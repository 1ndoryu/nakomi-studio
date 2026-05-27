use serde::{Deserialize, Serialize};

/* [265A-6] Backup entry enriquecido con tamaño y fecha para la UI.
 * file_size_bytes y created_at son Option para compat con Lightweight existente. */
#[derive(Debug, Clone, Serialize)]
pub struct HostingRuntimeBackupEntry {
    pub backup_id: String,
    pub tier: String,
    pub file_id: String,
    pub file_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HostingRuntimeBackupReport {
    pub backup_id: String,
    pub tier: String,
    pub status: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HostingRuntimeRestoreReport {
    pub backup_id: String,
    pub status: String,
    pub fqdn: Option<String>,
    pub access_user: Option<String>,
    pub access_password: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LightweightManagerBackupListReport {
    pub(crate) entries: Vec<LightweightManagerBackupEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LightweightManagerBackupEntry {
    pub(crate) backup_id: String,
    pub(crate) tier: String,
    pub(crate) file_id: String,
    pub(crate) file_name: String,
}

/* [265A-6] Entrada de backup parseada desde la salida de `ls -la /backups/`
 * en el volumen backup-data de un hosting Coolify vía SSH.
 * Formato esperado: "PERM LINKS OWNER GROUP SIZE MON DAY HH:MM FILENAME" */
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CoolifyBackupFileEntry {
    pub(crate) file_name: String,
    pub(crate) file_size_bytes: u64,
    pub(crate) tier: String,
    pub(crate) created_at: String,
}

/* [265A-6] Parsea la salida de `ls -la --time-style=long-iso /backups/` en entradas
 * de backup. Extrae nombre, tamaño, tier (daily/weekly/manual) y fecha.
 * Filtra solo archivos .sql y .tar.gz. */
pub(crate) fn parse_coolify_backup_listing(ls_output: &str) -> Vec<CoolifyBackupFileEntry> {
    ls_output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with("total") {
                return None;
            }
            /* Formato: "-rw-r--r-- 1 root root 12345 2026-05-26 03:00 daily_20260526_030000.sql" */
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 9 { return None; }
            let file_name = parts[8..].join(" ");
            if !std::path::Path::new(&file_name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("sql"))
                && !file_name.ends_with(".tar.gz")
            {
                return None;
            }
            let file_size_bytes = parts[4].parse::<u64>().ok()?;
            let date_part = parts[5];
            let time_part = parts[6];
            let created_at = format!("{date_part}T{time_part}:00Z");
            /* Determinar tier por prefijo del nombre */
            let tier = if file_name.starts_with("daily_") {
                "daily"
            } else if file_name.starts_with("weekly_") {
                "weekly"
            } else if file_name.starts_with("manual_") {
                "manual"
            } else {
                "unknown"
            };
            Some(CoolifyBackupFileEntry {
                file_name: file_name.clone(),
                file_size_bytes,
                tier: tier.to_string(),
                created_at,
            })
        })
        .collect()
}

#[derive(Debug, Deserialize)]
pub(crate) struct LightweightManagerBackupReport {
    pub(crate) backup_id: String,
    pub(crate) tier: String,
    pub(crate) status: String,
    pub(crate) notes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LightweightManagerRestoreReport {
    pub(crate) backup_id: String,
    pub(crate) status: String,
    pub(crate) fqdn: Option<String>,
    pub(crate) access_user: Option<String>,
    pub(crate) access_password: Option<String>,
    pub(crate) notes: Vec<String>,
}