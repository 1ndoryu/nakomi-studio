use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct HostingRuntimeBackupEntry {
    pub backup_id: String,
    pub tier: String,
    pub file_id: String,
    pub file_name: String,
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