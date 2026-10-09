use dark_pc_core::disk;

use crate::dto::{ScanRootSuggestionDto, VolumeStatsDto};

#[tauri::command]
pub fn get_volume_stats(mount_path: Option<String>) -> Result<VolumeStatsDto, String> {
    disk::volume_stats(mount_path.as_deref())
        .map(VolumeStatsDto::from)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_mounted_volumes() -> Vec<VolumeStatsDto> {
    disk::list_mounted_volumes()
        .into_iter()
        .map(VolumeStatsDto::from)
        .collect()
}

#[tauri::command]
pub fn get_scan_root_suggestions() -> Vec<ScanRootSuggestionDto> {
    disk::scan_root_suggestions()
        .into_iter()
        .map(ScanRootSuggestionDto::from)
        .collect()
}
