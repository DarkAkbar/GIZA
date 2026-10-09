use dark_pc_core::disk::VolumeStats;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeStatsDto {
    pub mount_path: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRootSuggestionDto {
    pub mount_path: String,
    pub label: String,
    pub path: String,
    pub scope: String,
}

impl From<dark_pc_core::disk::ScanRootSuggestion> for ScanRootSuggestionDto {
    fn from(value: dark_pc_core::disk::ScanRootSuggestion) -> Self {
        Self {
            mount_path: value.mount_path,
            label: value.label,
            path: value.path,
            scope: value.scope,
        }
    }
}

impl From<VolumeStats> for VolumeStatsDto {
    fn from(value: VolumeStats) -> Self {
        Self {
            mount_path: value.mount_path,
            total_bytes: value.total_bytes,
            available_bytes: value.available_bytes,
            used_bytes: value.used_bytes,
            usage_percent: value.usage_percent,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfoDto {
    pub name: String,
    pub version: String,
    pub db_path: String,
    pub migration_version: i64,
    pub platform: String,
}
